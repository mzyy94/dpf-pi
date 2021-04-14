#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::ffi::CString;
use std::mem::zeroed;
use std::os::raw::c_void;

#[derive(Debug)]
pub enum OMXError {
    CreateComponentFailed,
    UnableToGetParameter,
    UnableToSetParameter,
}

struct Image {
    data: Vec<u8>,
    width: u32,
    height: u32,
}

struct Element {
    component: *mut COMPONENT_T,
    handle: OMX_HANDLETYPE,
    in_port: u32,
    out_port: u32,
}

impl Default for Element {
    fn default() -> Self {
        Self {
            component: &mut COMPONENT_T { _unused: [] },
            handle: 0 as *mut c_void,
            in_port: 0,
            out_port: 0,
        }
    }
}

impl Element {
    pub fn create(
        &mut self,
        client: *mut ILCLIENT_T,
        name: String,
        flags: ILCLIENT_CREATE_FLAGS_T,
    ) -> Result<(), OMXError> {
        unsafe {
            let name = CString::new(name).unwrap();

            if ilclient_create_component(client, &mut self.component, name.into_raw(), flags)
                != OMX_ERRORTYPE_OMX_ErrorNone
            {
                return Err(OMXError::CreateComponentFailed);
            }

            self.handle = ilclient_get_handle(self.component);
            Ok(())
        }
    }

    pub fn get_parameter<T>(&self, index: OMX_INDEXTYPE, param: &mut T) -> Result<(), OMXError> {
        let param = param as *mut _ as *mut c_void;
        unsafe {
            if wOMX_GetParameter(self.handle, index, param) != OMX_ERRORTYPE_OMX_ErrorNone {
                return Err(OMXError::UnableToGetParameter);
            }
            Ok(())
        }
    }

    pub fn set_parameter<T>(&self, index: OMX_INDEXTYPE, param: &mut T) -> Result<(), OMXError> {
        let param = param as *mut _ as *mut c_void;
        unsafe {
            if wOMX_SetParameter(self.handle, index, param) != OMX_ERRORTYPE_OMX_ErrorNone {
                return Err(OMXError::UnableToSetParameter);
            }
            Ok(())
        }
    }
}

struct Pipeline {
    client: *mut ILCLIENT_T,
    buffer_header: *mut OMX_BUFFERHEADERTYPE,
    render: Element,
    resize: Element,
}

impl Pipeline {
    fn new() -> Pipeline {
        unsafe {
            let client = ilclient_init();

            Pipeline {
                client: client,
                buffer_header: zeroed(),
                render: Default::default(),
                resize: Default::default(),
            }
        }
    }
}

fn main() {
    println!("Hello, world!");
}
