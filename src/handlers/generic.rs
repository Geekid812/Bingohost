use serde::Serialize;

use super::Response;

#[derive(Serialize)]
pub struct Ok;

#[typetag::serialize]
impl Response for Ok {}

#[derive(Serialize)]
pub struct Error(String);

#[typetag::serialize]
impl Response for Error {}
