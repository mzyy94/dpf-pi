/*
Copyright (c) 2021, Yuki MIZUNO
SPDX-License-Identifier: BSD-3-Clause
*/
use gotham::handler::IntoResponse;
use gotham::helpers::http::response::create_response;
use gotham::hyper::{Body, Response, StatusCode};
use gotham::state::State;
use serde::Serialize;

#[derive(Debug, Serialize, Default)]
pub struct DisplayPower {
    #[serde(serialize_with = "status_serde")]
    pub status: StatusCode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub power: Option<bool>,
}

fn status_serde<S>(status: &StatusCode, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_str(&format!("{}", status.as_u16()))
}

impl IntoResponse for DisplayPower {
    fn into_response(self, state: &State) -> Response<Body> {
        create_response(
            state,
            self.status,
            mime::APPLICATION_JSON,
            serde_json::to_string(&self).expect("serialize JSON"),
        )
    }
}
