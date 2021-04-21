#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use image::RgbaImage;
use std::ffi::CString;
use std::fs::File;
use std::io::prelude::*;
use std::mem::{size_of, zeroed};
use std::os::raw::c_void;

use crate::error::OMXError;

struct Component {
    component: *mut COMPONENT_T,
    handle: OMX_HANDLETYPE,
    in_port: u32,
    out_port: u32,
}

impl Default for Component {
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

impl Component {
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

    pub fn set_config<T>(&self, index: OMX_INDEXTYPE, config: &mut T) -> Result<(), OMXError> {
        let config = config as *mut _ as *mut c_void;
        unsafe {
            if wOMX_SetConfig(self.handle, index, config) != OMX_ERRORTYPE_OMX_ErrorNone {
                return Err(OMXError::UnableToSetConfig);
            }
            Ok(())
        }
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
        unsafe {
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
                    None => zeroed(),
                },
                transform: OMX_DISPLAYTRANSFORMTYPE_OMX_DISPLAY_ROT0,
                ..zeroed()
            };
            self.set_config(OMX_INDEXTYPE_OMX_IndexConfigDisplayRegion, &mut disp)
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

pub struct Pipeline {
    client: *mut ILCLIENT_T,
    buffer_header: *mut OMX_BUFFERHEADERTYPE,
    decode_buffer_header0: *mut OMX_BUFFERHEADERTYPE,
    decode_buffer_header1: *mut OMX_BUFFERHEADERTYPE,
    decode_output_header: *mut OMX_BUFFERHEADERTYPE,
    decode: Component,
    render: Component,
    resize: Component,
}

impl Pipeline {
    pub fn new() -> Pipeline {
        unsafe {
            let client = ilclient_init();

            Pipeline {
                client: client,
                buffer_header: zeroed(),
                decode_buffer_header0: zeroed(),
                decode_buffer_header1: zeroed(),
                decode_output_header: zeroed(),
                decode: Default::default(),
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

        self.decode.create(
            self.client,
            "image_decode".to_string(),
            ILCLIENT_CREATE_FLAGS_T_ILCLIENT_DISABLE_ALL_PORTS
                | ILCLIENT_CREATE_FLAGS_T_ILCLIENT_ENABLE_INPUT_BUFFERS
                | ILCLIENT_CREATE_FLAGS_T_ILCLIENT_ENABLE_OUTPUT_BUFFERS,
        )?;

        self.decode
            .get_parameter(OMX_INDEXTYPE_OMX_IndexParamImageInit, &mut port)?;

        if port.nPorts != 2 {
            return Err(OMXError::InvalidNumberOfPorts);
        }
        self.decode.in_port = port.nStartPortNumber;
        self.decode.out_port = port.nStartPortNumber + 1;

        Ok(())
    }

    pub fn destroy(&mut self) {
        unsafe {
            ilclient_destroy(self.client);
        }
    }

    pub fn prepare_decoder(&mut self) -> Result<(), OMXError> {
        unsafe {
            self.decode.set_state(State::Idle);

            let mut port = OMX_IMAGE_PARAM_PORTFORMATTYPE {
                nSize: size_of::<OMX_IMAGE_PARAM_PORTFORMATTYPE>() as u32,
                nVersion: OMX_VERSIONTYPE {
                    nVersion: OMX_VERSION,
                },
                nPortIndex: self.decode.in_port,
                nIndex: 0,
                eCompressionFormat: OMX_IMAGE_CODINGTYPE_OMX_IMAGE_CodingJPEG,
                eColorFormat: OMX_COLOR_FORMATTYPE_OMX_COLOR_FormatUnused,
            };
            self.decode
                .set_parameter(OMX_INDEXTYPE_OMX_IndexParamImagePortFormat, &mut port)?;

            let mut port = OMX_PARAM_PORTDEFINITIONTYPE {
                nSize: size_of::<OMX_PARAM_PORTDEFINITIONTYPE>() as u32,
                nVersion: OMX_VERSIONTYPE {
                    nVersion: OMX_VERSION,
                },
                nPortIndex: self.decode.in_port,
                ..zeroed()
            };

            self.decode
                .get_parameter(OMX_INDEXTYPE_OMX_IndexParamPortDefinition, &mut port)?;
            port.nBufferCountActual = 2;
            self.decode
                .set_parameter(OMX_INDEXTYPE_OMX_IndexParamPortDefinition, &mut port)?;

            self.decode
                .send_command(OMX_COMMANDTYPE_OMX_CommandPortEnable, Direction::In)?;

            if wOMX_AllocateBuffer(
                self.decode.handle,
                &mut self.decode_buffer_header0,
                self.decode.in_port,
                std::ptr::null_mut(),
                port.nBufferSize,
            ) != OMX_ERRORTYPE_OMX_ErrorNone
            {
                return Err(OMXError::AllocateBufferFailed);
            }

            if wOMX_AllocateBuffer(
                self.decode.handle,
                &mut self.decode_buffer_header1,
                self.decode.in_port,
                std::ptr::null_mut(),
                port.nBufferSize,
            ) != OMX_ERRORTYPE_OMX_ErrorNone
            {
                return Err(OMXError::AllocateBufferFailed);
            }

            if ilclient_wait_for_event(
                self.decode.component,
                OMX_EVENTTYPE_OMX_EventCmdComplete,
                OMX_COMMANDTYPE_OMX_CommandPortEnable,
                0,
                self.decode.in_port,
                0,
                0,
                2000,
            ) != 0
            {
                return Err(OMXError::EventTimeout);
            }
            self.decode.set_state(State::Executing);

            Ok(())
        }
    }

    fn port_changed(&mut self, result: &mut Vec<u8>) -> Result<(), OMXError> {
        unsafe {
            let mut port = OMX_PARAM_PORTDEFINITIONTYPE {
                nSize: size_of::<OMX_PARAM_PORTDEFINITIONTYPE>() as u32,
                nVersion: OMX_VERSIONTYPE {
                    nVersion: OMX_VERSION,
                },
                nPortIndex: self.decode.out_port,
                ..zeroed()
            };

            self.decode
                .get_parameter(OMX_INDEXTYPE_OMX_IndexParamPortDefinition, &mut port)?;

            println!(
                "{}x{} ({})",
                port.format.image.nFrameWidth, port.format.image.nFrameHeight, port.nBufferSize
            );

            self.decode
                .send_command(OMX_COMMANDTYPE_OMX_CommandPortEnable, Direction::Out)?;

            result.set_len(port.nBufferSize as usize);

            if wOMX_UseBuffer(
                self.decode.handle,
                &mut self.decode_output_header,
                self.decode.out_port,
                std::ptr::null_mut(),
                port.nBufferSize,
                result.as_mut_ptr(),
            ) != OMX_ERRORTYPE_OMX_ErrorNone
            {
                return Err(OMXError::UseBufferFailed);
            }

            if wOMX_FillThisBuffer(self.decode.handle, self.decode_output_header)
                != OMX_ERRORTYPE_OMX_ErrorNone
            {
                return Err(OMXError::FillBufferFailed);
            }
            Ok(())
        }
    }
    
    pub fn decode_image(&mut self, file: &mut File) -> Result<(), OMXError> {
        unsafe {
            let metadata = file.metadata().unwrap();
            let len = (*self.decode_buffer_header0).nAllocLen as usize;
            let mut vec = Vec::<u8>::with_capacity(len);
            vec.set_len(len);

            let mut result = Vec::<u8>::with_capacity(1024 * 1024 * 4);

            let mut offset: usize = 0;
            let mut eof = false;
            let mut port_changed = false;

            while !eof {
                let read_bytes = file.read(vec.as_mut_slice()).unwrap();
                eof = offset + read_bytes == metadata.len() as usize;
                println!("{} {} {} {}", metadata.len(), read_bytes, len, eof);

                (*self.decode_buffer_header0).nFilledLen = read_bytes as u32;
                (*self.decode_buffer_header0).pBuffer = vec.as_mut_ptr();
                (*self.decode_buffer_header0).nOffset = 0;
                (*self.decode_buffer_header0).nFlags = if eof { OMX_BUFFERFLAG_EOS } else { 0 };

                // if ilclient_remove_event(
                //     self.decode.component,
                //     OMX_EVENTTYPE_OMX_EventBufferFlag,
                //     self.decode.out_port,
                //     0,
                //     OMX_BUFFERFLAG_EOS,
                //     0,
                // ) == 0
                // {
                //     println!("Got EOS");
                //     break;
                // }

                // if ilclient_wait_for_event(
                //     self.decode.component,
                //     OMX_EVENTTYPE_OMX_EventBufferFlag,
                //     self.decode.out_port,
                //     1,
                //     OMX_BUFFERFLAG_EOS,
                //     1,
                //     0,
                //     2,
                // ) == 0
                // {
                //     println!("Got EOS2");
                //     break;
                // }

                if wOMX_EmptyThisBuffer(self.decode.handle, self.decode_buffer_header0)
                    != OMX_ERRORTYPE_OMX_ErrorNone
                {
                    return Err(OMXError::EmptyBufferFailed);
                }

                if true {
                    if !port_changed
                        && ilclient_remove_event(
                            self.decode.component,
                            OMX_EVENTTYPE_OMX_EventPortSettingsChanged,
                            self.decode.out_port,
                            0,
                            0,
                            1,
                        ) == 0
                    {
                        println!("remove event OK");
                        self.port_changed(&mut result)?;
                        port_changed = true
                    }

                    // if !port_changed
                    //     && ilclient_wait_for_event(
                    //         self.decode.component,
                    //         OMX_EVENTTYPE_OMX_EventPortSettingsChanged,
                    //         self.decode.out_port,
                    //         0,
                    //         0,
                    //         1,
                    //         (ILEVENT_MASK_T_ILCLIENT_EVENT_ERROR
                    //             | ILEVENT_MASK_T_ILCLIENT_PARAMETER_CHANGED)
                    //             as i32,
                    //         2000,
                    //     ) == 0
                    // {
                    //     println!("wait event OK");
                    //     self.port_changed(&mut result)?;
                    //     port_changed = true
                    // }
                }

                offset += read_bytes;
            }

            if !port_changed
                && ilclient_wait_for_event(
                    self.decode.component,
                    OMX_EVENTTYPE_OMX_EventPortSettingsChanged,
                    self.decode.out_port,
                    0,
                    0,
                    1,
                    (ILEVENT_MASK_T_ILCLIENT_EVENT_ERROR
                        | ILEVENT_MASK_T_ILCLIENT_PARAMETER_CHANGED) as i32,
                    2000,
                ) == 0
            {
                println!("wait event after loop OK");
                self.port_changed(&mut result)?;
                port_changed = true
            } else if !port_changed {
                println!("Failed to waiting error");
            }

            println!("sizeof buffer: {}, {:?}", result.len(), &result[0..10]);
            Ok(())
        }
    }

    pub fn prepare_image(&mut self, image: &RgbaImage) -> Result<(), OMXError> {
        self.resize.set_state(State::Idle);

        self.resize.set_image_size(
            Direction::In,
            image.width(),
            image.height(),
            Some(image.len() as u32),
        )?;
        self.resize
            .send_command(OMX_COMMANDTYPE_OMX_CommandPortEnable, Direction::In)?;

        unsafe {
            if wOMX_UseBuffer(
                self.resize.handle,
                &mut self.buffer_header,
                self.resize.in_port,
                std::ptr::null_mut(),
                image.len() as u32,
                image.as_raw().as_ptr(),
            ) != OMX_ERRORTYPE_OMX_ErrorNone
            {
                return Err(OMXError::UseBufferFailed);
            }
        }

        self.resize.set_state(State::Executing);

        Ok(())
    }

    pub fn set_image_config(
        &mut self,
        display_rect: Option<OMX_DISPLAYRECTTYPE>,
    ) -> Result<(), OMXError> {
        self.render.set_display_region(Direction::In, display_rect)
    }

    pub fn render_image(
        &mut self,
        size: u32,
        width: u32,
        height: u32,
        timeout: i32,
    ) -> Result<(), OMXError> {
        unsafe {
            (*self.buffer_header).nFilledLen = size;
            (*self.buffer_header).nFlags = OMX_BUFFERFLAG_EOS;

            if wOMX_EmptyThisBuffer(self.resize.handle, self.buffer_header)
                != OMX_ERRORTYPE_OMX_ErrorNone
            {
                return Err(OMXError::EmptyBufferFailed);
            }

            if ilclient_wait_for_event(
                self.resize.component,
                OMX_EVENTTYPE_OMX_EventPortSettingsChanged,
                self.resize.out_port,
                0,
                0,
                1,
                (ILEVENT_MASK_T_ILCLIENT_EVENT_ERROR | ILEVENT_MASK_T_ILCLIENT_PARAMETER_CHANGED)
                    as i32,
                timeout,
            ) != 0
            {
                return Err(OMXError::EventTimeout);
            }

            self.render.set_state(State::Idle);
            self.render.set_state(State::Executing);

            self.resize
                .set_image_size(Direction::Out, width, height, None)?;
            self.render
                .set_image_size(Direction::In, width, height, None)?;

            OMX_SetupTunnel(
                self.resize.handle,
                self.resize.out_port,
                self.render.handle,
                self.render.in_port,
            );

            self.resize
                .send_command(OMX_COMMANDTYPE_OMX_CommandPortEnable, Direction::Out)?;
            self.render
                .send_command(OMX_COMMANDTYPE_OMX_CommandPortEnable, Direction::In)?;

            ilclient_wait_for_event(
                self.render.component,
                OMX_EVENTTYPE_OMX_EventBufferFlag,
                self.render.in_port,
                0,
                OMX_BUFFERFLAG_EOS,
                0,
                ILEVENT_MASK_T_ILCLIENT_BUFFER_FLAG_EOS as i32,
                timeout,
            );

            Ok(())
        }
    }
}

pub fn init_bcm_omx() {
    unsafe {
        bcm_host_init();
        OMX_Init();
    }
}

pub fn destroy_bcm_omx() {
    unsafe {
        OMX_Deinit();
        bcm_host_deinit();
    }
}

pub fn get_display_size() -> (u32, u32) {
    unsafe {
        let mut width: u32 = 0;
        let mut height: u32 = 0;
        graphics_get_display_size(0, &mut width, &mut height);
        (width, height)
    }
}
