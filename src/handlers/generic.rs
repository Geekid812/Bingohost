use serde::Serialize;

use super::Response;

#[derive(Serialize, Debug)]
pub struct Ok;

#[typetag::serialize]
impl Response for Ok {}

#[derive(Serialize, Debug)]
pub struct Error(String);

#[typetag::serialize]
impl Response for Error {}
