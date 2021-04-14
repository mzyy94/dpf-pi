mod component;
mod error;
mod image;

use component::*;
use image::Image;

fn main() {
    init_bcm_omx();

    let mut pipeline = Pipeline::new();
    pipeline.init().unwrap();

    let (width, height) = get_display_size();

    let mut image = Image {
        width: 1,
        height: 1,
        data: vec![0x80, 0x80, 0x80, 0xff],
    };

    pipeline.prepare_image(&mut image).unwrap();
    pipeline.render_image(&image, width, height, 2000).unwrap();

    pipeline.destroy();
    destroy_bcm_omx();

    println!("Hello, world!");
}
