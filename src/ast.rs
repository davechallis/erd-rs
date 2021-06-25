use std::fmt;
use std::collections::HashMap;

pub const OPT_COLOR: &str = "color";
pub const OPT_LABEL: &str = "label";
pub const OPT_SIZE: &str = "size";
pub const OPT_FONT: &str = "font";
pub const OPT_BACKGROUND_COLOR: &str = "bgcolor";
pub const OPT_BORDER_COLOR: &str = "border-color";
pub const OPT_BORDER: &str = "border";

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Erd {
    pub entities: Vec<Entity>,
    pub relationships: Vec<Relation>,
    pub title_options: TitleOptions,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ast {
    Entity(Entity),
    Attribute(Attribute),
    Relation(Relation),
    GlobalOption(GlobalOption),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Entity {
    pub name: String,
    pub attribs: Vec<Attribute>,
    pub options: EntityOptions,
    pub header_options: HeaderOptions,
}

impl Entity {
    pub fn add_attribute(&mut self, attr: Attribute) {
        self.attribs.push(attr)
    }
}

#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct Attribute {
    pub field: String,
    pub pk: bool,
    pub fk: bool,
    pub options: AttributeOptions,
}

impl Attribute {
    pub fn with_field<S: Into<String>>(field: S) -> Self {
        Self {
            field: field.into(),
            pk: false,
            fk: false,
            options: AttributeOptions::default(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Relation {
    pub entity1: String,
    pub entity2: String,
    pub card1: Cardinality,
    pub card2: Cardinality,
    pub options: RelationshipOptions,
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum GlobalOptionType {
    Title,
    Header,
    Entity,
    Relationship,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlobalOption {
    pub option_type: GlobalOptionType,
    pub options: HashMap<String, String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TitleOptions {
    pub size: u8,
    pub label: Option<String>,
    pub color: Option<String>,
    pub font: Option<String>,
}

impl TitleOptions {
    pub fn merge_hashmap(&mut self, m: &HashMap<String, String>) -> Result<(), String> {
         for (k, v) in m {
            match k.as_str() {
                OPT_LABEL => self.label = Some(v.clone()),
                OPT_COLOR => self.color = Some(v.clone()),
                OPT_FONT => self.font = Some(v.clone()),
                OPT_SIZE => self.size = match v.parse() {
                    Ok(v) => v,
                    Err(_) => return Err(format!("could not parse size as integer: {}", v)),
                },
                _ => return Err(format!("invalid header option: {}", v))
            }
        }

        Ok(())
    }
}

impl Default for TitleOptions {
    fn default() -> Self {
        Self {
            size: 30,
            label: None,
            color: None,
            font: None,
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HeaderOptions {
    pub size: u8,
    pub font: String,
    pub border: u8,
    pub cell_border: u8,
    pub cell_spacing: u8,
    pub cell_padding: u8,

    pub background_color: Option<String>,
    pub label: Option<String>,
    pub color: Option<String>,
    pub border_color: Option<String>,
}


impl HeaderOptions {
    pub fn from_hashmap(m: &HashMap<String, String>) -> Result<Self, String> {
        let mut opts = Self::default();
        opts.merge_hashmap(m)?;
        Ok(opts)
    }

    pub fn merge_hashmap(&mut self, m: &HashMap<String, String>) -> Result<(), String> {
         for (k, v) in m {
            match k.as_str() {
                OPT_SIZE => self.size = match v.parse() {
                    Ok(v) => v,
                    Err(_) => return Err(format!("could not parse size as integer: {}", v)),
                },
                OPT_LABEL => self.label = Some(v.clone()),
                OPT_COLOR => self.color = Some(v.clone()),
                OPT_BACKGROUND_COLOR => self.background_color = Some(v.clone()),
                OPT_FONT => self.font = v.clone(),
                OPT_BORDER_COLOR => self.border_color = Some(v.clone()),
                OPT_BORDER => self.border = match v.parse() {
                    Ok(v) => v,
                    Err(_) => return Err(format!("could not parse border as integer: {}", v)),
                },
                _ => return Err(format!("invalid header option: {}", v))
            }
        }

        Ok(())
    }
}

impl Default for HeaderOptions {
    fn default() -> Self {
        Self {
            size: 16,
            font: "Helvetica".to_owned(),
            border: 0,
            cell_border: 1,
            cell_padding: 4,
            cell_spacing: 0,
            background_color: None,
            label: None,
            color: None,
            border_color: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EntityOptions {
    pub border: u8,
    pub cell_border: u8,
    pub cell_spacing: u8,
    pub cell_padding: u8,
    pub font: String,

    pub background_color: Option<String>,
    pub label: Option<String>,
    pub color: Option<String>,
    pub size: Option<u8>,
    pub border_color: Option<String>,
}

impl EntityOptions {
    pub fn from_hashmap(m: &HashMap<String, String>) -> Result<Self, String> {
        let mut opts = Self::default();
        opts.merge_hashmap(m)?;
        Ok(opts)
    }

    pub fn merge_hashmap(&mut self, m: &HashMap<String, String>) -> Result<(), String> {
        for (k, v) in m {
            match k.as_str() {
                OPT_BACKGROUND_COLOR => self.background_color = Some(v.clone()),
                OPT_LABEL => self.label = Some(v.clone()),
                OPT_COLOR => self.color = Some(v.clone()),
                OPT_SIZE => self.size = Some(match v.parse() {
                    Ok(v) => v,
                    Err(_) => return Err(format!("could not parse size as integer: {}", v)),
                }),
                OPT_FONT => self.font = v.clone(),
                OPT_BORDER_COLOR => self.border_color = Some(v.clone()),
                OPT_BORDER => self.border = match v.parse() {
                    Ok(v) => v,
                    Err(_) => return Err(format!("could not parse border as integer: {}", v)),
                },
                _ => return Err(format!("invalid entity option: {}", v))
            }
        }

        Ok(())
    }
}

impl Default for EntityOptions {
    fn default() -> Self {
        Self {
            border: 0,
            cell_border: 1,
            cell_spacing: 0,
            cell_padding: 4,
            font: "Helvetica".to_owned(),
            background_color: None,
            label: None,
            color: None,
            size: None,
            border_color: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttributeOptions {
    pub text_alignment: String,
    pub label: Option<String>,
    pub color: Option<String>,
    pub background_color: Option<String>,
    pub font: Option<String>,
    pub border: Option<u8>,
    pub border_color: Option<String>,
}

impl AttributeOptions {
    pub fn from_hashmap(m: &HashMap<String, String>) -> Result<Self, String> {
        let mut opts = Self::default();
        opts.merge_hashmap(m)?;
        Ok(opts)
    }

    pub fn merge_hashmap(&mut self, m: &HashMap<String, String>) -> Result<(), String> {
        for (k, v) in m {
            match k.as_str() {
                OPT_LABEL => self.label = Some(v.clone()),
                OPT_COLOR => self.color = Some(v.clone()),
                OPT_BACKGROUND_COLOR => self.background_color = Some(v.clone()),
                OPT_FONT => self.font = Some(v.clone()),
                OPT_BORDER_COLOR => self.border_color = Some(v.clone()),
                OPT_BORDER => self.border = Some(match v.parse() {
                    Ok(v) => v,
                    Err(_) => return Err(format!("could not parse border as integer: {}", v)),
                }),
                _ => return Err(format!("invalid attribute option: {}", v))
            }
        }

        Ok(())
    }
}

impl Default for AttributeOptions {
    fn default() -> Self {
        Self {
            text_alignment: "LEFT".to_owned(),
            label: None,
            color: None,
            background_color: None,
            font: None,
            border: None,
            border_color: None,
        }
    }
}

#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct RelationshipOptions {
    label: Option<String>,
    color: Option<String>,
    size: Option<u8>,
    font: Option<String>,
}

impl RelationshipOptions {
    pub fn from_hashmap(m: &HashMap<String, String>) -> Result<Self, String> {
        let mut opts = Self::default();
        opts.merge_hashmap(m)?;
        Ok(opts)
    }

    pub fn merge_hashmap(&mut self, m: &HashMap<String, String>) -> Result<(), String> {
        for (k, v) in m {
            match k.as_str() {
                OPT_LABEL => self.label = Some(v.clone()),
                OPT_COLOR => self.color = Some(v.clone()),
                OPT_SIZE => self.size = Some(match v.parse() {
                    Ok(v) => v,
                    Err(_) => return Err(format!("could not parse size as integer: {}", v)),
                }),
                OPT_FONT => self.font = Some(v.clone()),
                _ => return Err(format!("invalid relationship option: {}", v))
            }
        }

        Ok(())
    }
}
