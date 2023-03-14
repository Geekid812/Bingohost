use std::fmt::Debug;

use crate::context::ClientContext;

pub mod create_room;
pub mod generic;
pub mod ping;

#[typetag::deserialize(tag = "req")]
pub trait Request: Debug {
    fn handle(&self, ctx: &mut ClientContext) -> Box<dyn Response>;
}

#[typetag::serialize(tag = "res")]
pub trait Response: Debug {}
