use crate::{context::GameContext, rest::auth::PlayerIdentity};

pub mod create_room;
pub mod generic;

#[typetag::deserialize(tag = "req")]
pub trait Request {
    fn handle(&self, ctx: &mut Option<GameContext>) -> Box<dyn Response>;
}

#[typetag::serialize(tag = "res")]
pub trait Response {}
