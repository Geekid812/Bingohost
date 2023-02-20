use crate::client::GameClient;

pub mod create_room;
pub mod generic;

#[typetag::deserialize(tag = "req")]
pub trait Request {
    fn handle(&self, client: &mut GameClient) -> Box<dyn Response>;
}

#[typetag::serialize(tag = "res")]
pub trait Response {}
