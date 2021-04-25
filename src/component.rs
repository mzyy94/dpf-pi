#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use std::mem::size_of;
use std::os::raw::c_void;

use crate::error::OMXError;
use crate::ilclient;
use crate::ilclient::*;

pub struct Component {
    pub component: *mut COMPONENT_T,
    pub handle: OMX_HANDLETYPE,
    pub in_port: u32,
    pub out_port: u32,
}

impl Default for Component {
    fn default() -> Self {
        Self {
            component: &mut Default::default(),
            handle: 0 as *mut c_void,
            in_port: 0,
            out_port: 0,
        }
    }
}

#[derive(Debug)]
pub enum Direction {
    In,
    Out,
}

#[derive(Debug)]
pub enum State {
    Invalid,
    Loaded,
    Idle,
    Executing,
    Pause,
    WaitForResources,
}

fn set_image_defs(image: &mut OMX_IMAGE_PORTDEFINITIONTYPE, width: u32, height: u32) {
    image.nFrameWidth = width;
    image.nFrameHeight = height;
    image.nStride = 0;
    image.nSliceHeight = 0;
    image.bFlagErrorConcealment = OMX_BOOL_OMX_FALSE;
    image.eCompressionFormat = OMX_IMAGE_CODINGTYPE_OMX_IMAGE_CodingUnused;
    image.eColorFormat = OMX_COLOR_FORMATTYPE_OMX_COLOR_Format32bitABGR8888;
}

impl Component {
    pub fn create(
        &mut self,
        client: *mut ILCLIENT_T,
        name: String,
        flags: ILCLIENT_CREATE_FLAGS_T,
    ) -> Result<(), OMXError> {
        ilclient::create_component(client, &mut self.component, name, flags)?;

        self.handle = ilclient::get_handle(self.component);
        Ok(())
    }

    pub fn get_parameter<T>(&self, index: OMX_INDEXTYPE, param: &mut T) -> Result<(), OMXError> {
        let param = param as *mut _ as *mut c_void;
        omx::get_parameter(self.handle, index, param)
    }

    pub fn set_parameter<T>(&self, index: OMX_INDEXTYPE, param: &mut T) -> Result<(), OMXError> {
        let param = param as *mut _ as *mut c_void;
        omx::set_parameter(self.handle, index, param)
    }

    pub fn set_config<T>(&self, index: OMX_INDEXTYPE, config: &mut T) -> Result<(), OMXError> {
        let config = config as *mut _ as *mut c_void;
        omx::set_config(self.handle, index, config)
    }

    pub fn set_display_region(
        &mut self,
        direction: Direction,
        display_rect: Option<OMX_DISPLAYRECTTYPE>,
    ) -> Result<(), OMXError> {
        let port = match direction {
            Direction::In => self.in_port,
            Direction::Out => self.out_port,
        };

        let mut disp = OMX_CONFIG_DISPLAYREGIONTYPE {
            nSize: size_of::<OMX_CONFIG_DISPLAYREGIONTYPE>() as u32,
            nVersion: OMX_VERSIONTYPE {
                nVersion: OMX_VERSION,
            },
            nPortIndex: port,
            set: OMX_DISPLAYSETTYPE_OMX_DISPLAY_SET_NUM
                | OMX_DISPLAYSETTYPE_OMX_DISPLAY_SET_MODE
                | OMX_DISPLAYSETTYPE_OMX_DISPLAY_SET_NOASPECT
                | OMX_DISPLAYSETTYPE_OMX_DISPLAY_SET_FULLSCREEN
                | OMX_DISPLAYSETTYPE_OMX_DISPLAY_SET_DEST_RECT
                | OMX_DISPLAYSETTYPE_OMX_DISPLAY_SET_TRANSFORM,
            num: 0,
            mode: OMX_DISPLAYMODETYPE_OMX_DISPLAY_MODE_LETTERBOX,
            noaspect: OMX_BOOL_OMX_TRUE,
            fullscreen: match display_rect {
                None => OMX_BOOL_OMX_TRUE,
                _ => OMX_BOOL_OMX_FALSE,
            },
            dest_rect: match display_rect {
                Some(rect) => rect,
                None => Default::default(),
            },
            transform: OMX_DISPLAYTRANSFORMTYPE_OMX_DISPLAY_ROT0,
            ..Default::default()
        };
        self.set_config(OMX_INDEXTYPE_OMX_IndexConfigDisplayRegion, &mut disp)
    }

    pub fn send_command(&self, cmd: OMX_COMMANDTYPE, direction: Direction) -> Result<(), OMXError> {
        let port = match direction {
            Direction::In => self.in_port,
            Direction::Out => self.out_port,
        };
        omx::send_command(self.handle, cmd, port, std::ptr::null_mut())
    }

    pub fn enable_port(&self, direction: Direction) -> Result<(), OMXError> {
        self.send_command(OMX_COMMANDTYPE_OMX_CommandPortEnable, direction)
    }

    pub fn set_image_size(
        &mut self,
        direction: Direction,
        width: u32,
        height: u32,
        buffer_size: Option<u32>,
    ) -> Result<(), OMXError> {
        let port = match direction {
            Direction::In => self.in_port,
            Direction::Out => self.out_port,
        };

        let mut port = OMX_PARAM_PORTDEFINITIONTYPE {
            nSize: size_of::<OMX_PARAM_PORTDEFINITIONTYPE>() as u32,
            nVersion: OMX_VERSIONTYPE {
                nVersion: OMX_VERSION,
            },
            nPortIndex: port,
            ..Default::default()
        };

        self.get_parameter(OMX_INDEXTYPE_OMX_IndexParamPortDefinition, &mut port)?;

        unsafe {
            set_image_defs(&mut port.format.image, width, height);
        }
        if let Some(size) = buffer_size {
            port.nBufferSize = size;
        }

        self.set_parameter(OMX_INDEXTYPE_OMX_IndexParamPortDefinition, &mut port)
    }

    pub fn set_state(&mut self, state: State) {
        let state = match state {
            State::Invalid => OMX_STATETYPE_OMX_StateInvalid,
            State::Loaded => OMX_STATETYPE_OMX_StateLoaded,
            State::Idle => OMX_STATETYPE_OMX_StateIdle,
            State::Executing => OMX_STATETYPE_OMX_StateExecuting,
            State::Pause => OMX_STATETYPE_OMX_StatePause,
            State::WaitForResources => OMX_STATETYPE_OMX_StateWaitForResources,
        };
        ilclient::change_component_state(self.component, state).ok();
    }
}
