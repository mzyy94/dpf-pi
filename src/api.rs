/*
Copyright (c) 2021, Yuki MIZUNO
SPDX-License-Identifier: BSD-3-Clause
*/
use serde::{Deserialize, Serialize};

use gotham::handler::*;
use gotham::hyper::{self, Body, Response, StatusCode};
use gotham::middleware::state::StateMiddleware;
use gotham::pipeline::single::single_pipeline;
use gotham::pipeline::single_middleware;
use gotham::router::builder::*;
use gotham::router::Router;
use gotham::state::{FromState, State};
use gotham_derive::*;

use crate::display::*;
use crate::error::*;
use crate::pipeline::*;

fn response_json(
    status_code: StatusCode,
    value: serde_json::Value,
) -> Result<Response<Body>, HandlerError> {
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
) -> Result<Response<Body>, HandlerError>
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

#[derive(Deserialize, StateData, StaticResponseExtender)]
struct ImageDisplayOption {
    format: Option<String>,
    mode: Option<String>,
}

async fn show_image(state: &mut State) -> Result<Response<Body>, HandlerError> {
    use image::io::Reader as ImageReader;
    use image::ImageFormat::{Bmp, Jpeg, Png};

    let body = Body::take_from(state);
    let query = ImageDisplayOption::take_from(state);
    let headers = hyper::HeaderMap::take_from(state);

    let whole_body = hyper::body::to_bytes(body).await?;
    let size = whole_body.len();
    let cur = std::io::Cursor::new(whole_body);

    let mut image = ImageReader::new(cur);
    let format = query.format.or(headers
        .get(hyper::header::CONTENT_TYPE)
        .and_then(|f| f.to_str().ok().and_then(|s| Some(String::from(s)))));

    match format.as_deref() {
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

    let mode = query.mode.or(headers
        .get("x-rendering-mode")
        .and_then(|f| f.to_str().ok().and_then(|s| Some(String::from(s)))));

    let content_mode = match mode.as_deref() {
        Some("AspectFit") | Some("aspect_fit") | None => ContentMode::Aspect(AspectMode::Fit),
        Some("AspectFill") | Some("aspect_fill") => ContentMode::Aspect(AspectMode::Fill),
        Some("Fill") | Some("fill") => ContentMode::ScaleToFill,
        Some(_) => ContentMode::None,
    };

    {
        let pipeline = Pipeline::borrow_mut_from(state);
        pipeline.render_image(&image, content_mode, 2000).unwrap();
    }

    let render_config = serde_json::json!({
        "status": StatusCode::OK.as_u16(),
        "image": image,
        "content_mode": content_mode,
    });
    response_json(StatusCode::OK, render_config)
}

async fn cors(state: &mut State) -> Result<Response<Body>, HandlerError> {
    let headers = hyper::HeaderMap::take_from(state);
    let mut cors = Response::default();
    *cors.status_mut() = StatusCode::OK;
    for (key, value) in headers {
        use hyper::header::*;
        let key = match key {
            Some(ACCESS_CONTROL_REQUEST_HEADERS) => ACCESS_CONTROL_ALLOW_HEADERS,
            Some(ACCESS_CONTROL_REQUEST_METHOD) => ACCESS_CONTROL_ALLOW_METHODS,
            Some(ORIGIN) => ACCESS_CONTROL_ALLOW_ORIGIN,
            _ => continue,
        };
        cors.headers_mut().insert(key, value.clone());
    }

    Ok(cors)
}

pub fn router(pipeline: Pipeline) -> Router {
    let middleware = StateMiddleware::new(pipeline);
    let pipeline = single_middleware(middleware);
    let (chain, pipelines) = single_pipeline(pipeline);

    build_router(chain, pipelines, |route| {
        route
            .post("/image/show")
            .with_query_string_extractor::<crate::api::ImageDisplayOption>()
            .to_async_borrowing(crate::api::show_image);

        route.options("*").to_async_borrowing(crate::api::cors);
    })
}
