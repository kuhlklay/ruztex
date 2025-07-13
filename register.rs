use crate::registries::{ID, Item, Block, Tag, REGISTRY, RegistrableEntity};

pub fn register() {
    // Initialize the registry
    let mut registry = REGISTRY.lock().unwrap();

    registry.register(RegistrableEntity::Tag(Tag::new(ID::new("ruz", "fuel"))));

    registry.register(RegistrableEntity::Item(Item::new(
        ID::new("ruztex", "coal"), vec![ID::new("ruz", "fuel")], 64,
    )));

    registry.register(RegistrableEntity::Block(Block::new(
        ID::new("ruztex", "coal"), vec![ID::new("ruz", "fuel")], 5.0,
    )));
}