/*
Copyright (c) 2021, Yuki MIZUNO
SPDX-License-Identifier: BSD-3-Clause
*/
use hyper::{Body, Method, Request, Response, StatusCode};
use serde::Serialize;
use std::sync::{Arc, Mutex};

use crate::display::*;
use crate::error::*;
use crate::pipeline::*;

struct Query<'a>(Vec<Vec<&'a str>>);

impl Query<'_> {
    pub fn new<'a>(query: &'a str) -> Query<'a> {
        Query(query.split('&').map(|x| x.split('=').collect()).collect())
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        for param in &self.0 {
            if param[0] == key {
                return if param.len() == 2 {
                    Some(param[1])
                } else {
                    Some("")
                };
            }
        }
        None
    }
}

fn response_json(
    status_code: StatusCode,
    value: serde_json::Value,
) -> Result<Response<Body>, hyper::Error> {
    let text = serde_json::to_string(&value).unwrap();

    let mut response = Response::new(Body::from(text));
    *response.status_mut() = status_code;
    response.headers_mut().append(
        hyper::header::CONTENT_TYPE,
        hyper::header::HeaderValue::from_static("application/json"),
    );
    Ok(response)
}

fn error_handle<T>(
    status_code: StatusCode,
    details: Option<T>,
) -> Result<Response<Body>, hyper::Error>
where
    T: Serialize,
{
    let error_response = serde_json::json!(HttpError {
        status: status_code.as_u16(),
        reason: status_code.canonical_reason(),
        details,
    });
    response_json(status_code, error_response)
}

pub async fn handler(
    req: Request<Body>,
    pipeline: Arc<Mutex<Pipeline>>,
) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/image/show") => {
            use image::io::Reader as ImageReader;
            use image::ImageFormat::{Bmp, Jpeg, Png};

            let (parts, body) = req.into_parts();
            let query = Query::new(parts.uri.query().unwrap_or_default());
            let whole_body = hyper::body::to_bytes(body).await?;
            let size = whole_body.len();
            let cur = std::io::Cursor::new(whole_body);

            let mut image = ImageReader::new(cur);
            let format = query.get("format").or(parts
                .headers
                .get(hyper::header::CONTENT_TYPE)
                .and_then(|f| f.to_str().ok()));

            match format {
                Some("image/png") | Some("png") => image.set_format(Png),
                Some("image/jpeg") | Some("jpeg") | Some("jpg") => image.set_format(Jpeg),
                Some("image/bmp") | Some("bmp") => image.set_format(Bmp),
                _ => image = image.with_guessed_format().unwrap(),
            };

            let format = image.format().unwrap();
            let image = image.decode();
            if let Err(image_error) = image {
                return error_handle(
                    StatusCode::BAD_REQUEST,
                    Some(crate::error::ImageError { image_error }),
                );
            }
            let image = image.unwrap();
            let image = image::DynamicImage::to_rgba8(&image);
            let image = DisplayImage::new(image, size, format);

            let mode = query.get("mode").or(parts
                .headers
                .get("x-rendering-mode")
                .and_then(|f| f.to_str().ok()));

            let content_mode = match mode {
                Some("AspectFit") | Some("aspect_fit") | None => {
                    ContentMode::Aspect(AspectMode::Fit)
                }
                Some("AspectFill") | Some("aspect_fill") => ContentMode::Aspect(AspectMode::Fill),
                Some("Fill") | Some("fill") => ContentMode::ScaleToFill,
                Some(_) => ContentMode::None,
            };

            {
                let mut pipeline = pipeline.lock().unwrap();
                pipeline.render_image(&image, content_mode, 2000).unwrap();
            }

            let render_config = serde_json::json!({
                "status": StatusCode::OK.as_u16(),
                "image": image,
                "content_mode": content_mode,
            });
            response_json(StatusCode::OK, render_config)
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

        _ => error_handle::<crate::error::NoError>(StatusCode::NOT_FOUND, None),
    }
}
