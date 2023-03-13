use crate::context::ClientContext;

pub mod create_room;
pub mod generic;

#[typetag::deserialize(tag = "req")]
pub trait Request {
    fn handle(&self, ctx: &mut ClientContext) -> Box<dyn Response>;
}

#[typetag::serialize(tag = "res")]
pub trait Response {}
