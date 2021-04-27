use hyper::{Body, Method, Request, Response, StatusCode};
use std::sync::{Arc, Mutex};

use crate::picture::*;
use crate::pipeline::*;

pub async fn handler(
    req: Request<Body>,
    pipeline: Arc<Mutex<Pipeline>>,
) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/image/show") => {
            use image::io::Reader as ImageReader;

            let whole_body = hyper::body::to_bytes(req.into_body()).await?;
            let cur = std::io::Cursor::new(whole_body);

            let image = ImageReader::new(cur)
                .with_guessed_format()
                .unwrap()
                .decode()
                .unwrap();
            let image = image::DynamicImage::to_rgba8(&image);
            let image = DisplayImage::new(image);

            let text = format!("render {}x{}", image.width(), image.height());

            pipeline.lock().unwrap().init().unwrap();
            pipeline
                .lock()
                .unwrap()
                .render_image(&image, ContentMode::Aspect(AspectMode::Fit), 2000)
                .unwrap();

            Ok(Response::new(Body::from(text)))
        }

        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}
