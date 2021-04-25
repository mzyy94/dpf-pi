#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use std::mem::{size_of, zeroed};

use crate::component::*;
use crate::error::OMXError;
use crate::picture::*;
use crate::vc::*;

pub struct Pipeline {
    client: *mut ILCLIENT_T,
    buffer_header: *mut OMX_BUFFERHEADERTYPE,
    render: Component,
    resize: Component,
    viewport: (u32, u32),
}

impl Pipeline {
    pub fn new() -> Pipeline {
        let client = ilclient::init();

        unsafe {
            Pipeline {
                client: client,
                buffer_header: zeroed(),
                render: Default::default(),
                resize: Default::default(),
                viewport: (0, 0),
            }
        }
    }

    pub fn init(&mut self, width: u32, height: u32) -> Result<(), OMXError> {
        self.viewport = (width, height);

        let mut port = OMX_PORT_PARAM_TYPE {
            nSize: size_of::<OMX_PORT_PARAM_TYPE>() as u32,
            nVersion: OMX_VERSIONTYPE {
                nVersion: OMX_VERSION,
            },
            nPorts: 0,
            nStartPortNumber: 0,
        };

        self.render.create(
            self.client,
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
            self.client,
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

    pub fn destroy(&mut self) {
        ilclient::destroy(self.client)
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

        omx::use_buffer(
            self.resize.handle,
            &mut self.buffer_header,
            self.resize.in_port,
            std::ptr::null_mut(),
            image.len(),
            image.as_raw().as_ptr(),
        )?;

        self.resize.set_state(State::Executing);

        unsafe {
            (*self.buffer_header).nFilledLen = image.len();
            (*self.buffer_header).nFlags = OMX_BUFFERFLAG_EOS;
        }
        Ok(())
    }

    pub fn set_image_config(
        &mut self,
        display_rect: Option<OMX_DISPLAYRECTTYPE>,
    ) -> Result<(), OMXError> {
        self.render.set_display_region(Direction::In, display_rect)
    }

    pub fn render_image(&mut self, timeout: i32) -> Result<(), OMXError> {
        omx::empty_this_buffer(self.resize.handle, self.buffer_header)?;

        ilclient::wait_for_event(
            self.resize.component,
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

        omx::setup_tunnel(
            self.resize.handle,
            self.resize.out_port,
            self.render.handle,
            self.render.in_port,
        )
        .ok();

        self.resize.enable_port(Direction::Out)?;
        self.render.enable_port(Direction::In)?;

        ilclient::wait_for_event(
            self.render.component,
            OMX_EVENTTYPE_OMX_EventBufferFlag,
            self.render.in_port,
            0,
            OMX_BUFFERFLAG_EOS,
            0,
            ILEVENT_MASK_T_ILCLIENT_BUFFER_FLAG_EOS,
            timeout,
        )
        .ok();

        Ok(())
    }
}
