/*
Copyright (c) 2021, Yuki MIZUNO
SPDX-License-Identifier: BSD-3-Clause
*/
use std::mem::size_of;
use std::os::raw::c_void;

use crate::error::PipelineError;
use crate::vc::*;

#[derive(Debug, Default, Copy, Clone)]
pub struct Component {
    pub component: i32,
    pub in_port: u32,
    pub out_port: u32,
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
    ) -> Result<(), PipelineError> {
        let mut component: *mut COMPONENT_T = &mut Default::default();

        ilclient::create_component(client, &mut component, name, flags)?;

        self.component = component as i32;
        Ok(())
    }

    pub fn component(&self) -> *mut COMPONENT_T {
        self.component as *mut _
    }

    pub fn handle(&self) -> OMX_HANDLETYPE {
        ilclient::get_handle(self.component())
    }

    pub fn get_parameter<T>(
        &self,
        index: OMX_INDEXTYPE,
        param: &mut T,
    ) -> Result<(), PipelineError> {
        let param = param as *mut _ as *mut c_void;
        omx::get_parameter(self.handle(), index, param)
    }

    pub fn set_parameter<T>(
        &self,
        index: OMX_INDEXTYPE,
        param: &mut T,
    ) -> Result<(), PipelineError> {
        let param = param as *mut _ as *mut c_void;
        omx::set_parameter(self.handle(), index, param)
    }

    pub fn set_config<T>(&self, index: OMX_INDEXTYPE, config: &mut T) -> Result<(), PipelineError> {
        let config = config as *mut _ as *mut c_void;
        omx::set_config(self.handle(), index, config)
    }

    pub fn set_display_region(
        &mut self,
        direction: Direction,
        display_rect: Option<OMX_DISPLAYRECTTYPE>,
    ) -> Result<(), PipelineError> {
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
            dest_rect: display_rect.unwrap_or_default(),
            transform: OMX_DISPLAYTRANSFORMTYPE_OMX_DISPLAY_ROT0,
            ..Default::default()
        };
        self.set_config(OMX_INDEXTYPE_OMX_IndexConfigDisplayRegion, &mut disp)
    }

    pub fn send_command(
        &self,
        cmd: OMX_COMMANDTYPE,
        direction: Direction,
    ) -> Result<(), PipelineError> {
        let port = match direction {
            Direction::In => self.in_port,
            Direction::Out => self.out_port,
        };
        omx::send_command(self.handle(), cmd, port, std::ptr::null_mut())
    }

    pub fn enable_port(&self, direction: Direction) -> Result<(), PipelineError> {
        self.send_command(OMX_COMMANDTYPE_OMX_CommandPortEnable, direction)
    }

    pub fn disable_port(&self, direction: Direction) -> Result<(), PipelineError> {
        self.send_command(OMX_COMMANDTYPE_OMX_CommandPortDisable, direction)
    }

    pub fn set_image_size(
        &mut self,
        direction: Direction,
        width: u32,
        height: u32,
        buffer_size: Option<u32>,
    ) -> Result<(), PipelineError> {
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
        port.nBufferSize = buffer_size.unwrap_or_default();

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
        let _ = ilclient::change_component_state(self.component(), state);
    }
}
