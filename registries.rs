#[allow(unused_imports)]
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter, Result};
use std::hash::{Hash, Hasher};
use std::sync::Mutex;

use once_cell::sync::Lazy;

pub static REGISTRY: Lazy<Mutex<Registry>> = Lazy::new(|| Mutex::new(Registry::new()));

// --
// ID
// --

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct ID {
    pub namespace: String,
    pub name: String,
}

impl ID {
    pub fn new(namespace: &str, name: &str) -> Self {
        if Self::is_valid_identifier(namespace) && Self::is_valid_identifier(name) {
            Self {
                namespace: namespace.to_string(),
                name: name.to_string(),
            }
        } else {
            panic!("Invalid ID: namespace '{}' or name '{}' contains invalid characters", namespace, name);
        }
    }

    pub fn is_valid_identifier(s: &str) -> bool {
        !s.is_empty() && s.chars().all(|c| matches!(c, 'a'..='z' | '_'))
    }
}

impl Display for ID {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}:{}", self.namespace, self.name)
    }
}

// -----
// ITEMS
// -----

#[derive(Clone, Debug)]
pub struct Item {
    pub id: ID,
    pub tags: Vec<ID>,
}

impl Item {
    pub fn new(id: ID, tags: Vec<ID>) -> Self {
        Item { id, tags }
    }

    pub fn tags(&self) -> &[ID] {
        &self.tags
    }
}

impl Registrable for Item {
    fn id(&self) -> &ID {
        &self.id
    }
}

impl Display for Item {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}:{}", self.id.namespace, self.id.name)
    }
}

// ------
// BLOCKS
// ------

#[derive(Clone, Debug)]
pub struct Block {
    pub id: ID,
    pub tags: Vec<ID>,
    pub hardness: f32,
}

impl Block {
    pub fn new(id: ID, tags: Vec<ID>, hardness: f32) -> Self {
        Block { id, tags, hardness }
    }

    pub fn hardness(&self) -> f32 {
        self.hardness
    }

    fn tags(&self) -> &[ID] {
        &self.tags
    }
}

impl Registrable for Block {
    fn id(&self) -> &ID {
        &self.id
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}:{}", self.id.namespace, self.id.name)
    }
}

// ----
// TAGS
// ----

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TagType {
    Item,
    Block,
    Tool,
    Recipe,
}

impl Display for TagType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            TagType::Item => write!(f, "Item"),
            TagType::Block => write!(f, "Block"),
            TagType::Tool => write!(f, "Tool"),
            TagType::Recipe => write!(f, "Recipe"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Tag {
    pub id: ID,
    pub entries: HashSet<(TagType, ID)>, // (Typ, ID) z.B. ("Item", ID), ("Block", ID)
}

impl Tag {
    pub fn new(id: ID) -> Self {
        Tag {
            id,
            entries: HashSet::new(),
        }
    }

    pub fn add(&mut self, typ: &TagType, entity_id: &ID) {
        self.entries.insert((typ.clone(), entity_id.clone()));
    }

    pub fn id(&self) -> &ID {
        &self.id
    }
}

impl Registrable for Tag {
    fn id(&self) -> &ID {
        &self.id
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}:#{}", self.id.namespace, self.id.name)
    }
}

// -----
// TOOLS
// -----

#[derive(Clone, Debug)]
pub struct Tool {
    pub id: ID,
    pub tags: Vec<ID>,
    pub durability: u32,
    pub speed: f32,
}

impl Tool {
    pub fn new(id: ID, tags: Vec<ID>, durability: u32, speed: f32) -> Self {
        Tool { id, tags, durability, speed }
    }

    pub fn durability(&self) -> u32 {
        self.durability
    }

    pub fn speed(&self) -> f32 {
        self.speed
    }
}

impl Registrable for Tool {
    fn id(&self) -> &ID {
        &self.id
    }
}

// -------
// RECIPES
// -------

#[derive(Clone, Debug)]
pub struct RecipeComponent {
    pub id: ID, // ID of the item or block
    pub count: u32, // Number of items or blocks needed
}

impl RecipeComponent {
    pub fn new(id: ID, count: u32) -> Self {
        RecipeComponent { id, count }
    }
}

#[derive(Clone, Debug)]
pub struct Recipe {
    pub id: ID,
    pub ingredients: Vec<RecipeComponent>, // IDs of items or blocks
    pub results: Vec<RecipeComponent>,     // ID of the resulting item or block
}

impl Recipe {
    pub fn new(id: ID, ingredients: Vec<ID>, results: Vec<ID>) -> Self {
        Recipe { id, ingredients, results }
    }

    pub fn ingredients(&self) -> &[ID] {
        &self.ingredients
    }

    pub fn results(&self) -> &[ID] {
        &self.results
    }
}

impl Registrable for Recipe {
    fn id(&self) -> &ID {
        &self.id
    }
}

impl Display for Recipe {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}:{}", self.id.namespace, self.id.name)
    }
}

// --------
// REGISTRY
// --------

pub trait Registrable {
    fn id(&self) -> &ID;
}

pub enum RegistrableEntity {
    Item(Item),
    Block(Block),
    Tag(Tag),
    Tool(Tool),
    Recipe(Recipe),
}

pub struct Registry {
    pub items: HashMap<ID, Item>,
    pub blocks: HashMap<ID, Block>,
    pub tags: HashMap<ID, Tag>,
    pub tools: HashMap<ID, Tool>,
    pub recipes: HashMap<ID, Recipe>,
}

impl Registry {
    pub fn new() -> Self {
        Registry {
            items: HashMap::new(),
            blocks: HashMap::new(),
            tags: HashMap::new(),
            tools: HashMap::new(),
            recipes: HashMap::new(),
        }
    }

    pub fn register(&mut self, entity: RegistrableEntity) {
        match entity {
            RegistrableEntity::Item(item) => {
                if self.items.contains_key(&item.id) {
                    panic!("Item with ID {} already exists", item.id);
                }
                self.items.insert(item.id.clone(), item.clone());

                for tag_id in &item.tags {
                    self.tags.get_mut(tag_id).expect(&format!("Tag with ID {} does not exist", tag_id)).add(&TagType::Item, &item.id);
                }
            },
            RegistrableEntity::Block(block) => {
                if self.blocks.contains_key(&block.id) {
                    panic!("Block with ID {} already exists", block.id);
                }
                self.blocks.insert(block.id.clone(), block.clone());

                for tag_id in &block.tags {
                    self.tags.get_mut(tag_id).expect(&format!("Tag with ID {} does not exist", tag_id)).add(&TagType::Block, &block.id);
                }
            },
            RegistrableEntity::Tag(tag) => {
                if self.tags.contains_key(&tag.id) {
                    panic!("Tag with ID {} already exists", tag.id);
                }
                self.tags.insert(tag.id.clone(), tag.clone());
            },
            RegistrableEntity::Tool(tool) => {
                if self.tools.contains_key(&tool.id) {
                    panic!("Tool with ID {} already exists", tool.id);
                }
                self.tools.insert(tool.id.clone(), ());
                // Tools don't have tags, so we don't need to do anything here
            },
            RegistrableEntity::Recipe(recipe) => {
                if self.recipes.contains_key(&recipe.id) {
                    panic!("Recipe with ID {} already exists", recipe.id);
                }
                self.recipes.insert(recipe.id.clone(), ());
                // Recipes don't have tags, so we don't need to do anything here
            },
            _ => {},
        }
    }

    pub fn remove(&mut self, entity: &RegistrableEntity) {
        match entity {
            RegistrableEntity::Item(item) => {
                self.items.remove(&item.id);
                for tag_id in &item.tags {
                    if let Some(tag) = self.tags.get_mut(tag_id) {
                        tag.entries.remove(&(TagType::Item, item.id.clone()));
                    }
                }
            },
            RegistrableEntity::Block(block) => {
                self.blocks.remove(&block.id);
                for tag_id in &block.tags {
                    if let Some(tag) = self.tags.get_mut(tag_id) {
                        tag.entries.remove(&(TagType::Block, block.id.clone()));
                    }
                }
            },
            RegistrableEntity::Tag(tag) => {
                if !self.tags.contains_key(&tag.id) {
                    panic!("Tag with ID {} does not exist", tag.id);
                }
                // remove the tag from all items and blocks
                for item in self.items.values_mut() {
                    if let Some(pos) = item.tags.iter().position(|t| t == &tag.id) {
                        item.tags.remove(pos);
                    }
                }
                for block in self.blocks.values_mut() {
                    if let Some(pos) = block.tags.iter().position(|t| t == &tag.id) {
                        block.tags.remove(pos);
                    }
                }

                for tool in self.tools.values_mut() {
                    if let Some(pos) = tool.tags.iter().position(|t| t == &tag.id) {
                        tool.tags.remove(pos);
                    }
                }

                for recipe in self.recipes.values_mut() {
                    if let Some(pos) = recipe.ingredients.iter().position(|c| c.id == tag.id) {
                        recipe.ingredients.remove(pos);
                    }
                    if let Some(pos) = recipe.results.iter().position(|c| c.id == tag.id) {
                        recipe.results.remove(pos);
                    }
                }
                // finally remove the tag itself
                self.tags.remove(&tag.id);
            },
            _ => {}
        }
    }

    // return the entity by its ID
    pub fn get(&self, entity: RegistrableEntity, id: &ID) -> Option<&dyn Registrable> {
        match entity {
            RegistrableEntity::Item(_) => self.items.get(id).map(|item| item as &dyn Registrable),
            RegistrableEntity::Block(_) => self.blocks.get(id).map(|block| block as &dyn Registrable),
            RegistrableEntity::Tag(_) => self.tags.get(id).map(|tag| tag as &dyn Registrable),
            RegistrableEntity::Tool(_) => self.tools.get(id).map(|tool| tool as &dyn Registrable),
            RegistrableEntity::Recipe(_) => self.recipes.get(id).map(|recipe| recipe as &dyn Registrable),
            _ => None,
        }
    }
}