use std::fmt;
use std::collections::HashMap;


#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Erd {
    pub entities: Vec<Entity>,
    pub relationships: Vec<Relation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ast {
    Entity(Entity),
    Attribute(Attribute),
    Relation(Relation),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Entity {
    pub name: String,
    pub attribs: Vec<Attribute>,
    pub options: HashMap<String, String>,
}

impl Entity {
    pub fn new<S: Into<String>>(name: S, options: HashMap<String, String>) -> Self {
        Self {
            name: name.into(),
            attribs: Vec::new(),
            options,
        }
    }
    pub fn with_name<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            attribs: Vec::new(),
            options: HashMap::new(),
        }
    }
    
    pub fn add_attribute(&mut self, attr: Attribute) {
        self.attribs.push(attr)
    }
}

#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct Attribute {
    pub field: String,
    pub pk: bool,
    pub fk: bool,
    pub options: HashMap<String, String>,
}

impl Attribute {
    pub fn with_field<S: Into<String>>(field: S) -> Self {
        Self {
            field: field.into(),
            pk: false,
            fk: false,
            options: HashMap::new(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Relation {
    pub entity1: String,
    pub entity2: String,
    pub card1: Cardinality,
    pub card2: Cardinality,
    pub options: HashMap<String, String>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Cardinality {
    ZeroOne,
    One,
    ZeroPlus,
    OnePlus,
}

impl fmt::Display for Cardinality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Cardinality::ZeroOne => write!(f, "{{0,1}}"),
            Cardinality::One => write!(f, "1"),
            Cardinality::ZeroPlus => write!(f, "0..N"),
            Cardinality::OnePlus => write!(f, "1..N"),
        }
    }
}