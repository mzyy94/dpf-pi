mod api;
mod component;
mod error;
mod picture;
mod pipeline;
mod vc;

use pipeline::*;
use vc::*;

use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

    pipeline.lock().unwrap().destroy();
    omx::deinit();
    println!("See you!");
    Ok(())
}
