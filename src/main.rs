#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

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

struct Pipeline {
    client: *mut ILCLIENT_T,
    buffer_header: *mut OMX_BUFFERHEADERTYPE,
    render: Element,
    resize: Element,
}

fn main() {
    println!("Hello, world!");
}
