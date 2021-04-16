mod component;
mod error;

use component::*;
use std::path::Path;

fn main() {
    init_bcm_omx();

    let mut pipeline = Pipeline::new();
    pipeline.init().unwrap();

    let (width, height) = get_display_size();

    let image = image::open(&Path::new("./rust-logo-512x512.png")).unwrap();
    let image = image::DynamicImage::as_rgba8(&image).unwrap();

    pipeline.prepare_image(image).unwrap();
    pipeline
        .set_image_config(Some(OMX_DISPLAYRECTTYPE {
            x_offset: (width - image.width()) as i16 / 2,
            y_offset: (height - image.height()) as i16 / 2,
            width: image.width() as i16,
            height: image.height() as i16,
        }))
        .unwrap();

    pipeline
        .render_image(image.len() as u32, width, height, 2000)
        .unwrap();

    pipeline.destroy();
    destroy_bcm_omx();

    println!("Hello, world!");
}
