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
    let image = image::DynamicImage::to_rgba8(&image);
    let mut image = Image {
        width: image.width(),
        height: image.height(),
        data: image.into_raw(),
    };

    pipeline.prepare_image(&mut image).unwrap();
    pipeline
        .set_image_config(Some(OMX_DISPLAYRECTTYPE {
            x_offset: (width - image.width) as i16 / 2,
            y_offset: (height - image.height) as i16 / 2,
            width: image.width as i16,
            height: image.height as i16,
        }))
        .unwrap();

    pipeline.render_image(&image, width, height, 2000).unwrap();

    pipeline.destroy();
    destroy_bcm_omx();

    println!("Hello, world!");
}
