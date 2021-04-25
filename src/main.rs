mod component;
mod error;
mod picture;

use component::*;
use picture::align_image;

use std::env;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time;

fn main() {
    let file = if env::args().count() == 2 {
        env::args().nth(1).unwrap()
    } else {
        panic!("Usage: {} image", env::args().nth(0).unwrap())
    };

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .unwrap();

    init_bcm_omx();

    let mut pipeline = Pipeline::new();
    pipeline.init().unwrap();

    let (width, height) = get_display_size();

    let image = image::open(&Path::new(&file)).unwrap();
    let image = image::DynamicImage::to_rgba8(&image);
    let image = align_image(image);

    pipeline.prepare_image(&image).unwrap();
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

    while running.load(Ordering::SeqCst) {
        thread::sleep(time::Duration::from_millis(10));
    }

    pipeline.destroy();
    destroy_bcm_omx();
}
