/*
Copyright (c) 2021, Yuki MIZUNO
SPDX-License-Identifier: BSD-3-Clause
*/
use hyper::{Body, Method, Request, Response, StatusCode};
use std::sync::{Arc, Mutex};

use crate::display::*;
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
            let size = whole_body.len();
            let cur = std::io::Cursor::new(whole_body);

            let image = ImageReader::new(cur).with_guessed_format().unwrap();
            let format = image.format().unwrap();
            let image = image.decode().unwrap();
            let image = image::DynamicImage::to_rgba8(&image);
            let image = DisplayImage::new(image, size, format);

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

            let render_config = serde_json::json!({
                "image": serde_json::to_value(image).unwrap(),
                "content_mode": serde_json::to_value(content_mode).unwrap(),
            });
            let text = serde_json::to_string(&render_config).unwrap();

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
