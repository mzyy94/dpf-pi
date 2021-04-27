mod api;
mod component;
mod error;
mod picture;
mod pipeline;
mod vc;

use pipeline::*;
use vc::*;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;

use hyper::service::{make_service_fn, service_fn};
use hyper::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .unwrap();

    omx::init();

    let (width, height) = omx::get_display_size(0);
    let pipeline = Arc::new(Mutex::new(Pipeline::new(width, height)));

    let addr = ([127, 0, 0, 1], 3000).into();
    let service_pipeline = pipeline.clone();
    let service = make_service_fn(move |_| {
        let pipeline = service_pipeline.clone();
        async { Ok::<_, hyper::Error>(service_fn(move |req| api::handler(req, pipeline.clone()))) }
    });

    let server = Server::bind(&addr).serve(service);
    println!("Listening on http://{}", addr);
    server.await?;

    while running.load(Ordering::SeqCst) {
        thread::sleep(time::Duration::from_millis(10));
    }

    pipeline.lock().unwrap().destroy();
    omx::deinit();
    Ok(())
}
