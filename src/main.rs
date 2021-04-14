#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::mem::zeroed;
use std::os::raw::c_void;

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
