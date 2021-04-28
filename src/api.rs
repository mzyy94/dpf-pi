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

            let (parts, body) = req.into_parts();
            let whole_body = hyper::body::to_bytes(body).await?;
            let cur = std::io::Cursor::new(whole_body);

            let image = ImageReader::new(cur)
                .with_guessed_format()
                .unwrap()
                .decode()
                .unwrap();
            let image = image::DynamicImage::to_rgba8(&image);
            let image = DisplayImage::new(image);

            let text = format!("render {}x{}", image.width(), image.height());

            let content_mode = if let Some(mode) = parts.headers.get("x-rendering-mode") {
                match mode.to_str() {
                    Ok("AspectFit") => ContentMode::Aspect(AspectMode::Fit),
                    Ok("AspectFill") => ContentMode::Aspect(AspectMode::Fill),
                    Ok("Fill") => ContentMode::ScaleToFill,
                    _ => ContentMode::None,
                }
            } else {
                ContentMode::Aspect(AspectMode::Fit)
            };

            {
                let mut pipeline = pipeline.lock().unwrap();
                pipeline.render_image(&image, content_mode, 2000).unwrap();
            }

            Ok(Response::new(Body::from(text)))
        }

        (&Method::OPTIONS, _) => {
            let mut cors = Response::default();
            *cors.status_mut() = StatusCode::OK;
            for (key, value) in req.headers() {
                use hyper::header::*;
                if let Some(key) = match key {
                    &ACCESS_CONTROL_REQUEST_HEADERS => Some(ACCESS_CONTROL_ALLOW_HEADERS),
                    &ACCESS_CONTROL_REQUEST_METHOD => Some(ACCESS_CONTROL_ALLOW_METHODS),
                    &ORIGIN => Some(ACCESS_CONTROL_ALLOW_ORIGIN),
                    _ => None,
                } {
                    cors.headers_mut().insert(key, value.clone());
                }
            }

            Ok(cors)
        }

        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}
