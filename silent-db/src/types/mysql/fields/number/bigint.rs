use crate::core::fields::{Field, FieldType};
#[derive(Clone)]
pub struct BigInt {
    pub name: String,
    pub default: Option<String>,
    pub nullable: bool,
    pub primary_key: bool,
    pub unique: bool,
    pub auto_increment: bool,
    pub comment: Option<String>,
}
impl Default for BigInt {
    fn default() -> Self {
        BigInt {
            name: "bigint".to_string(),
            default: None,
            nullable: true,
            primary_key: false,
            unique: false,
            auto_increment: false,
            comment: None,
        }
    }
}
struct BigIntType;

impl FieldType for BigIntType {
    fn get_type_str(&self) -> String {
        "BIGINT".to_string()
    }
}

impl Field for BigInt {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_type(&self) -> Box<dyn FieldType> {
        Box::new(BigIntType)
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
