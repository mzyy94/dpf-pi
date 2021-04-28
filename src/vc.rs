#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use crate::error::{Operation, PipelineError};

use std::ffi::CString;

impl Default for COMPONENT_T {
    fn default() -> Self {
        Self { _unused: [] }
    }
}

impl Default for OMX_PARAM_PORTDEFINITIONTYPE {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

impl Default for OMX_CONFIG_DISPLAYREGIONTYPE {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

impl Default for OMX_DISPLAYRECTTYPE {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

impl Default for OMX_BUFFERHEADERTYPE {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

pub mod ilclient {
    use super::*;

    pub fn init() -> *mut ILCLIENT_T {
        unsafe { ilclient_init() }
    }

    pub fn destroy(handle: *mut ILCLIENT_T) {
        unsafe { ilclient_destroy(handle) }
    }

    pub fn create_component(
        handle: *mut ILCLIENT_T,
        comp: *mut *mut COMPONENT_T,
        name: String,
        flags: ILCLIENT_CREATE_FLAGS_T,
    ) -> Result<(), PipelineError> {
        let name = CString::new(name).unwrap();
        unsafe {
            match ilclient_create_component(handle, comp, name.into_raw(), flags) {
                OMX_ERRORTYPE_OMX_ErrorNone => Ok(()),
                state => Err(PipelineError::ILClientError(
                    Operation::CreateComponentFailed,
                    state,
                )),
            }
        }
    }

    pub fn get_handle(comp: *mut COMPONENT_T) -> OMX_HANDLETYPE {
        unsafe { ilclient_get_handle(comp) }
    }

    pub fn wait_for_event(
        comp: *mut COMPONENT_T,
        event: OMX_EVENTTYPE,
        nData1: OMX_U32,
        ignore1: ::std::os::raw::c_int,
        nData2: OMX_U32,
        ignore2: ::std::os::raw::c_int,
        event_flag: u32,
        timeout: ::std::os::raw::c_int,
    ) -> Result<(), PipelineError> {
        unsafe {
            match ilclient_wait_for_event(
                comp,
                event,
                nData1,
                ignore1,
                nData2,
                ignore2,
                event_flag as i32,
                timeout,
            ) {
                OMX_ERRORTYPE_OMX_ErrorNone => Ok(()),
                state => Err(PipelineError::ILClientError(Operation::EventTimeout, state)),
            }
        }
    }

    pub fn change_component_state(
        comp: *mut COMPONENT_T,
        state: OMX_STATETYPE,
    ) -> Result<(), PipelineError> {
        unsafe {
            match ilclient_change_component_state(comp, state) {
                OMX_ERRORTYPE_OMX_ErrorNone => Ok(()),
                state => Err(PipelineError::ILClientError(Operation::EventTimeout, state)),
            }
        }
    }

    pub fn cleanup_components(list: *mut *mut COMPONENT_T) {
        unsafe { ilclient_cleanup_components(list) }
    }
}

pub mod omx {
    use super::*;

    pub fn init() {
        unsafe {
            bcm_host_init();
            OMX_Init();
        }
    }

    pub fn deinit() {
        unsafe {
            OMX_Deinit();
            bcm_host_deinit();
        }
    }

    pub fn get_display_size(display_number: u16) -> (u32, u32) {
        unsafe {
            let mut width: u32 = 0;
            let mut height: u32 = 0;
            graphics_get_display_size(display_number, &mut width, &mut height);
            (width, height)
        }
    }

    pub fn send_command(
        hComponent: OMX_HANDLETYPE,
        Cmd: OMX_COMMANDTYPE,
        nParam1: OMX_U32,
        pCmdData: *mut ::std::os::raw::c_void,
    ) -> Result<(), PipelineError> {
        unsafe {
            match wOMX_SendCommand(hComponent, Cmd, nParam1, pCmdData) {
                OMX_ERRORTYPE_OMX_ErrorNone => Ok(()),
                state => Err(PipelineError::OMXError(Operation::SendCommandFailed, state)),
            }
        }
    }

    pub fn get_parameter(
        hComponent: OMX_HANDLETYPE,
        nParamIndex: OMX_INDEXTYPE,
        pComponentParameterStructure: *mut ::std::os::raw::c_void,
    ) -> Result<(), PipelineError> {
        unsafe {
            match wOMX_GetParameter(hComponent, nParamIndex, pComponentParameterStructure) {
                OMX_ERRORTYPE_OMX_ErrorNone => Ok(()),
                state => Err(PipelineError::OMXError(
                    Operation::UnableToGetParameter,
                    state,
                )),
            }
        }
    }

    pub fn set_parameter(
        hComponent: OMX_HANDLETYPE,
        nParamIndex: OMX_INDEXTYPE,
        pComponentParameterStructure: *mut ::std::os::raw::c_void,
    ) -> Result<(), PipelineError> {
        unsafe {
            match wOMX_SetParameter(hComponent, nParamIndex, pComponentParameterStructure) {
                OMX_ERRORTYPE_OMX_ErrorNone => Ok(()),
                state => Err(PipelineError::OMXError(
                    Operation::UnableToSetParameter,
                    state,
                )),
            }
        }
    }

    pub fn set_config(
        hComponent: OMX_HANDLETYPE,
        nConfigIndex: OMX_INDEXTYPE,
        pComponentConfigStructure: *mut ::std::os::raw::c_void,
    ) -> Result<(), PipelineError> {
        unsafe {
            match wOMX_SetConfig(hComponent, nConfigIndex, pComponentConfigStructure) {
                OMX_ERRORTYPE_OMX_ErrorNone => Ok(()),
                state => Err(PipelineError::OMXError(Operation::UnableToSetConfig, state)),
            }
        }
    }

    pub fn use_buffer(
        hComponent: OMX_HANDLETYPE,
        ppBufferHdr: *mut *mut OMX_BUFFERHEADERTYPE,
        nPortIndex: OMX_U32,
        pAppPrivate: OMX_PTR,
        nSizeBytes: OMX_U32,
        pBuffer: *mut OMX_U8,
    ) -> Result<(), PipelineError> {
        unsafe {
            match wOMX_UseBuffer(
                hComponent,
                ppBufferHdr,
                nPortIndex,
                pAppPrivate,
                nSizeBytes,
                pBuffer,
            ) {
                OMX_ERRORTYPE_OMX_ErrorNone => Ok(()),
                state => Err(PipelineError::OMXError(Operation::UseBufferFailed, state)),
            }
        }
    }

    pub fn empty_this_buffer(
        hComponent: OMX_HANDLETYPE,
        pBuffer: *mut OMX_BUFFERHEADERTYPE,
    ) -> Result<(), PipelineError> {
        unsafe {
            match wOMX_EmptyThisBuffer(hComponent, pBuffer) {
                OMX_ERRORTYPE_OMX_ErrorNone => Ok(()),
                state => Err(PipelineError::OMXError(Operation::EmptyBufferFailed, state)),
            }
        }
    }

    pub fn free_buffer(
        hComponent: OMX_HANDLETYPE,
        nPortIndex: OMX_U32,
        pBuffer: *mut OMX_BUFFERHEADERTYPE,
    ) -> Result<(), PipelineError> {
        unsafe {
            match wOMX_FreeBuffer(hComponent, nPortIndex, pBuffer) {
                OMX_ERRORTYPE_OMX_ErrorNone => Ok(()),
                state => Err(PipelineError::OMXError(Operation::FreeBufferFailed, state)),
            }
        }
    }

    pub fn setup_tunnel(
        hOutput: OMX_HANDLETYPE,
        nPortOutput: OMX_U32,
        hInput: OMX_HANDLETYPE,
        nPortInput: OMX_U32,
    ) -> Result<(), PipelineError> {
        unsafe {
            match OMX_SetupTunnel(hOutput, nPortOutput, hInput, nPortInput) {
                OMX_ERRORTYPE_OMX_ErrorNone => Ok(()),
                state => Err(PipelineError::OMXError(Operation::SetupTunnelFailed, state)),
            }
        }
    }
}
