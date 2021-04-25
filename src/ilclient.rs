#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use crate::error::OMXError;

use std::ffi::CString;

impl Default for COMPONENT_T {
    fn default() -> Self {
        Self { _unused: [] }
    }
}

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
) -> Result<(), OMXError> {
    let name = CString::new(name).unwrap();
    unsafe {
        if ilclient_create_component(handle, comp, name.into_raw(), flags)
            != OMX_ERRORTYPE_OMX_ErrorNone
        {
            return Err(OMXError::CreateComponentFailed);
        }
        Ok(())
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
) -> Result<(), OMXError> {
    unsafe {
        if ilclient_wait_for_event(
            comp,
            event,
            nData1,
            ignore1,
            nData2,
            ignore2,
            event_flag as i32,
            timeout,
        ) != 0
        {
            return Err(OMXError::EventTimeout);
        }
        Ok(())
    }
}

pub fn change_component_state(
    comp: *mut COMPONENT_T,
    state: OMX_STATETYPE,
) -> Result<(), OMXError> {
    unsafe {
        if ilclient_change_component_state(comp, state) != 0 {
            return Err(OMXError::EventTimeout);
        }
        Ok(())
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
}
