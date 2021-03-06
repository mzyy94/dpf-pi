/*
Copyright (c) 2021, Yuki MIZUNO
SPDX-License-Identifier: BSD-3-Clause
*/
use serde::Deserialize;

use futures::prelude::*;
use gotham::handler::*;
use gotham::helpers::http::response::create_empty_response;
use gotham::hyper::{self, body::Bytes, Body, Response, StatusCode};
use gotham::middleware::logger::RequestLogger;
use gotham::middleware::{state::StateMiddleware, Middleware};
use gotham::pipeline::{new_pipeline, single::single_pipeline};
use gotham::router::{builder::*, Router};
use gotham::state::{FromState, State};
use gotham_derive::*;
use std::pin::Pin;

use crate::display::{image::*, power::*, result::*};
use crate::error::*;
use crate::pipeline::*;

#[derive(Deserialize, StateData, StaticResponseExtender)]
struct ImageDisplayOption {
    format: Option<String>,
    mode: Option<String>,
}

fn load_image(body: Bytes, format: Option<&str>) -> Result<DisplayImage, ImageError> {
    use image::io::Reader as ImageReader;
    use image::ImageFormat::{Bmp, Jpeg, Png};

    let size = body.len();
    let cur = std::io::Cursor::new(body);
    let mut image = ImageReader::new(cur);
    match format {
        Some("image/png") | Some("png") => image.set_format(Png),
        Some("image/jpeg") | Some("jpeg") | Some("jpg") => image.set_format(Jpeg),
        Some("image/bmp") | Some("bmp") => image.set_format(Bmp),
        _ => image = image.with_guessed_format().unwrap(),
    };

    let format = image.format().unwrap();
    let image = match image.decode() {
        Ok(image) => image,
        Err(image_error) => return Err(ImageError { image_error }),
    };
    let image = image::DynamicImage::to_rgba8(&image);
    Ok(DisplayImage::new(image, size, format))
}

async fn show_image(state: &mut State) -> Result<impl IntoResponse, HandlerError> {
    let body = Body::take_from(state);
    let query = ImageDisplayOption::take_from(state);
    let headers = hyper::HeaderMap::borrow_from(state);

    let whole_body = hyper::body::to_bytes(body).await?;
    let format = query.format.or(headers
        .get(hyper::header::CONTENT_TYPE)
        .and_then(|f| f.to_str().ok().and_then(|s| Some(String::from(s)))));

    let image = match load_image(whole_body, format.as_deref()) {
        Ok(image) => image,
        Err(err) => {
            return Ok(DisplayResult {
                status: StatusCode::BAD_REQUEST,
                error: Some(err),
                ..Default::default()
            })
        }
    };

    let content_mode = ContentMode::from_str(query.mode.as_deref().unwrap_or_default());

    let pipeline = Pipeline::borrow_mut_from(state);
    pipeline.render_image(&image, content_mode, 2000).unwrap();

    Ok(DisplayResult {
        image: Some(image),
        content_mode: Some(content_mode),
        ..Default::default()
    })
}

#[derive(Clone, NewMiddleware, Debug, PartialEq, Default)]
struct CORSMiddleware {}

impl Middleware for CORSMiddleware {
    fn call<Chain>(self, state: State, chain: Chain) -> Pin<Box<HandlerFuture>>
    where
        Chain: FnOnce(State) -> Pin<Box<HandlerFuture>>,
    {
        chain(state)
            .and_then(|(state, mut response)| {
                use hyper::header::*;
                let headers = HeaderMap::borrow_from(&state);
                for (key, value) in headers {
                    let key = match key {
                        &ACCESS_CONTROL_REQUEST_HEADERS => ACCESS_CONTROL_ALLOW_HEADERS,
                        &ACCESS_CONTROL_REQUEST_METHOD => ACCESS_CONTROL_ALLOW_METHODS,
                        &ORIGIN => ACCESS_CONTROL_ALLOW_ORIGIN,
                        _ => continue,
                    };
                    response.headers_mut().insert(key, value.clone());
                }
                future::ok((state, response))
            })
            .boxed()
    }
}

fn empty(state: State) -> (State, Response<Body>) {
    let resp = create_empty_response(&state, StatusCode::NO_CONTENT);

    (state, resp)
}

fn display_off(state: State) -> (State, impl IntoResponse) {
    crate::vc::tv::power_off();
    let resp = DisplayPower {
        status: StatusCode::OK,
        power: Some(false),
    };

    (state, resp)
}

fn display_on(state: State) -> (State, impl IntoResponse) {
    crate::vc::tv::hdmi_power_on_preferred();
    let resp = DisplayPower {
        status: StatusCode::OK,
        power: Some(true),
    };

    (state, resp)
}

pub fn router(pipeline: Pipeline) -> Router {
    let middleware = StateMiddleware::new(pipeline);
    let pipeline = new_pipeline()
        .add(RequestLogger::new(log::Level::Info))
        .add(middleware)
        .add(CORSMiddleware::default())
        .build();
    let (chain, pipelines) = single_pipeline(pipeline);

    build_router(chain, pipelines, |route| {
        route.options("/image/show").to(empty);

        route
            .post("/image/show")
            .with_query_string_extractor::<ImageDisplayOption>()
            .to_async_borrowing(show_image);

        route.post("/display/power/on").to(display_on);
        route.post("/display/power/off").to(display_off);
    })
}
