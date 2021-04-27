mod component;
mod error;
mod picture;
mod pipeline;
mod vc;

use picture::*;
use pipeline::*;
use vc::*;

use std::env;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
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

    omx::init();

    let (width, height) = omx::get_display_size(0);

    let pipeline = Arc::new(Mutex::new(Pipeline::new(width, height)));
    pipeline.lock().unwrap().init().unwrap();

    let image = image::open(&Path::new(&file)).unwrap();
    let image = image::DynamicImage::to_rgba8(&image);
    let image = DisplayImage::new(image);

    pipeline
        .lock()
        .unwrap()
        .render_image(&image, ContentMode::Aspect(AspectMode::Fill), 2000)
        .unwrap();

    while running.load(Ordering::SeqCst) {
        thread::sleep(time::Duration::from_millis(10));
    }

    pipeline.lock().unwrap().destroy();
    omx::deinit();
}
