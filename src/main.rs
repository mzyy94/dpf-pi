mod component;
mod error;
mod image;

use component::*;
use image::Image;
use std::fs::File;

fn main() {
    init_bcm_omx();

    let mut pipeline = Pipeline::new();
    pipeline.init().unwrap();

    let (width, height) = get_display_size();

    let decoder = png::Decoder::new(File::open("./rust-logo-512x512.png").unwrap());
    let (info, mut reader) = decoder.read_info().unwrap();
    let mut buf = vec![0; info.buffer_size()];
    reader.next_frame(&mut buf).unwrap();

    let mut image = Image {
        width: info.width,
        height: info.height,
        data: buf,
    };

    pipeline.prepare_image(&mut image).unwrap();
    pipeline
        .set_image_config(Some(OMX_DISPLAYRECTTYPE {
            x_offset: (width - info.width) as i16 / 2,
            y_offset: (height - info.height) as i16 / 2,
            width: info.width as i16,
            height: info.height as i16,
        }))
        .unwrap();

    pipeline.render_image(&image, width, height, 2000).unwrap();

    pipeline.destroy();
    destroy_bcm_omx();

    println!("Hello, world!");
}
