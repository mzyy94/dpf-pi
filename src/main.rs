#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::ffi::CString;
use std::mem::{size_of, zeroed};
use std::os::raw::c_void;

#[derive(Debug)]
pub enum OMXError {
    CreateComponentFailed,
    UnableToGetParameter,
    UnableToSetParameter,
    InvalidNumberOfPorts,
    SendCommandFailed,
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

#[derive(Debug)]
enum Direction {
    In,
    Out,
}

#[derive(Debug)]
enum State {
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

    pub fn send_command(&self, cmd: OMX_COMMANDTYPE, direction: Direction) -> Result<(), OMXError> {
        let port = match direction {
            Direction::In => self.in_port,
            Direction::Out => self.out_port,
        };

        unsafe {
            if wOMX_SendCommand(self.handle, cmd, port, std::ptr::null_mut())
                != OMX_ERRORTYPE_OMX_ErrorNone
            {
                return Err(OMXError::SendCommandFailed);
            }
        }

        Ok(())
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

        unsafe {
            let mut port = OMX_PARAM_PORTDEFINITIONTYPE {
                nSize: size_of::<OMX_PARAM_PORTDEFINITIONTYPE>() as u32,
                nVersion: OMX_VERSIONTYPE {
                    nVersion: OMX_VERSION,
                },
                nPortIndex: port,
                ..zeroed()
            };

            self.get_parameter(OMX_INDEXTYPE_OMX_IndexParamPortDefinition, &mut port)?;

            set_image_defs(&mut port.format.image, width, height);
            if let Some(size) = buffer_size {
                port.nBufferSize = size;
            }

            self.set_parameter(OMX_INDEXTYPE_OMX_IndexParamPortDefinition, &mut port)
        }
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
        unsafe {
            ilclient_change_component_state(self.component, state);
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
}

fn main() {
    println!("Hello, world!");
}
