use crate::core::fields::{Field, FieldType};
#[derive(Clone)]
pub struct TinyInt {
    pub name: String,
    pub default: Option<String>,
    pub nullable: bool,
    pub primary_key: bool,
    pub unique: bool,
    pub auto_increment: bool,
    pub comment: Option<String>,
    pub length: u16,
}

impl Default for TinyInt {
    fn default() -> Self {
        TinyInt {
            name: "tinyint".to_string(),
            default: None,
            nullable: true,
            primary_key: false,
            unique: false,
            auto_increment: false,
            comment: None,
            length: 8,
        }
    }
}
struct TinyIntType(u16);

impl FieldType for TinyIntType {
    fn get_type_str(&self) -> String {
        format!("TINYINT({})", self.0)
    }
}

impl Field for TinyInt {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_type(&self) -> Box<dyn FieldType> {
        Box::new(TinyIntType(self.length))
    }
    fn get_default(&self) -> Option<String> {
        self.default.clone()
    }
    fn get_nullable(&self) -> bool {
        match self.primary_key {
            true => false,
            false => self.nullable,
        }
    }
    fn get_primary_key(&self) -> bool {
        self.primary_key
    }
    fn get_unique(&self) -> bool {
        match self.primary_key {
            true => true,
            false => self.unique,
        }
    }
    fn get_auto_increment(&self) -> bool {
        self.auto_increment
    }
    fn get_comment(&self) -> Option<String> {
        self.comment.clone()
    }
}
