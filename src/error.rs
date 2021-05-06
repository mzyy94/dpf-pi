/*
Copyright (c) 2021, Yuki MIZUNO
SPDX-License-Identifier: BSD-3-Clause
*/
use serde::Serialize;

#[derive(Debug)]
pub enum PipelineError {
    ILClientError(Operation, i32),
    OMXError(Operation, i32),
    Assertion(Operation),
}

#[derive(Debug)]
pub enum Operation {
    CreateComponentFailed,
    UnableToGetParameter,
    UnableToSetParameter,
    UnableToSetConfig,
    InvalidNumberOfPorts,
    SendCommandFailed,
    UseBufferFailed,
    EmptyBufferFailed,
    FreeBufferFailed,
    EventTimeout,
    SetupTunnelFailed,
}

#[derive(Debug, Serialize)]
pub struct ImageError {
    #[serde(serialize_with = "image_error_serde")]
    pub image_error: image::ImageError,
}

fn image_error_serde<S>(image_error: &image::ImageError, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_str(&format!("{:?}", image_error))
}

#[derive(Debug, Serialize)]
pub struct NoError {}

#[derive(Debug, Serialize)]
pub struct HttpError<T: Serialize> {
    pub status: u16,
    pub reason: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<T>,
}
