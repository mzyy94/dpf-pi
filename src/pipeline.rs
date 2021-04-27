use std::mem::size_of;

use crate::component::*;
use crate::error::OMXError;
use crate::picture::*;
use crate::vc::*;

pub struct Pipeline {
    client: i32,
    buffer_header: i32,
    render: Component,
    resize: Component,
    viewport: (u32, u32),
}

impl Pipeline {
    pub fn new(width: u32, height: u32) -> Pipeline {
        let client = ilclient::init();

        Pipeline {
            client: client as i32,
            buffer_header: 0,
            render: Default::default(),
            resize: Default::default(),
            viewport: (width, height),
        }
    }

    pub fn init(&mut self) -> Result<(), OMXError> {
        let mut port = OMX_PORT_PARAM_TYPE {
            nSize: size_of::<OMX_PORT_PARAM_TYPE>() as u32,
            nVersion: OMX_VERSIONTYPE {
                nVersion: OMX_VERSION,
            },
            nPorts: 0,
            nStartPortNumber: 0,
        };

        self.render.create(
            self.client as *mut _,
            "video_render".to_string(),
            ILCLIENT_CREATE_FLAGS_T_ILCLIENT_DISABLE_ALL_PORTS
                | ILCLIENT_CREATE_FLAGS_T_ILCLIENT_ENABLE_INPUT_BUFFERS,
        )?;

        self.render
            .get_parameter(OMX_INDEXTYPE_OMX_IndexParamVideoInit, &mut port)?;

        if port.nPorts != 1 {
            return Err(OMXError::InvalidNumberOfPorts);
        }
        self.render.in_port = port.nStartPortNumber;

        self.resize.create(
            self.client as *mut _,
            "resize".to_string(),
            ILCLIENT_CREATE_FLAGS_T_ILCLIENT_DISABLE_ALL_PORTS
                | ILCLIENT_CREATE_FLAGS_T_ILCLIENT_ENABLE_INPUT_BUFFERS
                | ILCLIENT_CREATE_FLAGS_T_ILCLIENT_ENABLE_OUTPUT_BUFFERS,
        )?;

        self.resize
            .get_parameter(OMX_INDEXTYPE_OMX_IndexParamImageInit, &mut port)?;

        if port.nPorts != 2 {
            return Err(OMXError::InvalidNumberOfPorts);
        }
        self.resize.in_port = port.nStartPortNumber;
        self.resize.out_port = port.nStartPortNumber + 1;

        Ok(())
    }

    pub fn deinit(&mut self) -> Result<(), OMXError> {
        if self.render.component == 0i32 || self.resize.component == 0i32 {
            // Already de-initialized
            return Ok(());
        }
        let timeout = 1000i32;

        omx::free_buffer(
            self.resize.handle(),
            self.resize.in_port,
            self.buffer_header as *mut _,
        )?;

        self.resize.disable_port(Direction::In)?;

        let _ = ilclient::wait_for_event(
            self.resize.component as *mut _,
            OMX_EVENTTYPE_OMX_EventCmdComplete,
            OMX_COMMANDTYPE_OMX_CommandPortDisable,
            0,
            self.resize.in_port,
            0,
            ILEVENT_MASK_T_ILCLIENT_PORT_DISABLED,
            timeout,
        );

        self.resize
            .send_command(OMX_COMMANDTYPE_OMX_CommandFlush, Direction::Out)?;
        self.render
            .send_command(OMX_COMMANDTYPE_OMX_CommandFlush, Direction::In)?;

        let _ = ilclient::wait_for_event(
            self.resize.component as *mut _,
            OMX_EVENTTYPE_OMX_EventCmdComplete,
            OMX_COMMANDTYPE_OMX_CommandFlush,
            0,
            self.resize.out_port,
            0,
            ILEVENT_MASK_T_ILCLIENT_PORT_FLUSH,
            timeout,
        );

        let _ = ilclient::wait_for_event(
            self.render.component as *mut _,
            OMX_EVENTTYPE_OMX_EventCmdComplete,
            OMX_COMMANDTYPE_OMX_CommandFlush,
            0,
            self.render.in_port,
            0,
            ILEVENT_MASK_T_ILCLIENT_PORT_FLUSH,
            timeout,
        );

        self.resize.disable_port(Direction::Out)?;
        self.render.disable_port(Direction::In)?;

        self.resize.set_state(State::Idle);
        self.resize.set_state(State::Loaded);

        self.render.set_state(State::Idle);
        self.render.set_state(State::Loaded);

        let mut list = vec![
            self.render.component as *mut COMPONENT_T,
            self.resize.component as *mut COMPONENT_T,
            0i32 as *mut COMPONENT_T,
        ];
        ilclient::cleanup_components(list.as_mut_ptr());

        self.render.component = 0i32;
        self.resize.component = 0i32;

        Ok(())
    }

    pub fn destroy(&mut self) {
        ilclient::destroy(self.client as *mut _)
    }

    pub fn prepare_image(&mut self, image: &DisplayImage) -> Result<(), OMXError> {
        self.resize.set_state(State::Idle);

        self.resize.set_image_size(
            Direction::In,
            image.width(),
            image.height(),
            Some(image.len()),
        )?;
        self.resize.enable_port(Direction::In)?;

        let mut buffer_header: *mut OMX_BUFFERHEADERTYPE = &mut Default::default();

        omx::use_buffer(
            self.resize.handle(),
            &mut buffer_header,
            self.resize.in_port,
            std::ptr::null_mut(),
            image.len(),
            image.as_raw().as_ptr(),
        )?;

        self.resize.set_state(State::Executing);

        unsafe {
            (*buffer_header).nFilledLen = image.len();
            (*buffer_header).nFlags = OMX_BUFFERFLAG_EOS;
            self.buffer_header = buffer_header as i32;
        }
        Ok(())
    }

    pub fn set_image_config(
        &mut self,
        display_rect: Option<OMX_DISPLAYRECTTYPE>,
    ) -> Result<(), OMXError> {
        self.render.set_display_region(Direction::In, display_rect)
    }

    pub fn set_image_scale(
        &mut self,
        content_mode: ContentMode,
        image: &DisplayImage,
    ) -> Result<(), OMXError> {
        let DisplayRect { x, y, w, h } =
            DisplayRect::new_with_mode(content_mode, self.viewport, image.size());
        self.set_image_config(Some(OMX_DISPLAYRECTTYPE {
            x_offset: x,
            y_offset: y,
            width: w,
            height: h,
        }))
    }

    pub fn render_image(
        &mut self,
        image: &DisplayImage,
        content_mode: ContentMode,
        timeout: i32,
    ) -> Result<(), OMXError> {
        self.prepare_image(image)?;
        self.set_image_scale(content_mode, image)?;
        omx::empty_this_buffer(self.resize.handle(), self.buffer_header as *mut _)?;

        ilclient::wait_for_event(
            self.resize.component as *mut _,
            OMX_EVENTTYPE_OMX_EventPortSettingsChanged,
            self.resize.out_port,
            0,
            0,
            1,
            ILEVENT_MASK_T_ILCLIENT_EVENT_ERROR | ILEVENT_MASK_T_ILCLIENT_PARAMETER_CHANGED,
            timeout,
        )?;

        self.render.set_state(State::Idle);
        self.render.set_state(State::Executing);

        let (width, height) = self.viewport;
        self.resize
            .set_image_size(Direction::Out, width, height, None)?;
        self.render
            .set_image_size(Direction::In, width, height, None)?;

        let _ = omx::setup_tunnel(
            self.resize.handle(),
            self.resize.out_port,
            self.render.handle(),
            self.render.in_port,
        );

        self.resize.enable_port(Direction::Out)?;
        self.render.enable_port(Direction::In)?;

        let _ = ilclient::wait_for_event(
            self.render.component as *mut _,
            OMX_EVENTTYPE_OMX_EventBufferFlag,
            self.render.in_port,
            0,
            OMX_BUFFERFLAG_EOS,
            0,
            ILEVENT_MASK_T_ILCLIENT_BUFFER_FLAG_EOS,
            timeout,
        );

        Ok(())
    }
}
