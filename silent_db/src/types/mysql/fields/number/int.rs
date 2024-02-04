use crate::core::fields::{Field, FieldType};
pub struct IntField {
    name: String,
    default: Option<String>,
    nullable: bool,
    primary_key: bool,
    unique: bool,
    auto_increment: bool,
    comment: Option<String>,
}

struct IntType;

impl FieldType for IntType {
    fn get_type_str(&self) -> String {
        "INT".to_string()
    }
}

impl Field for IntField {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_type(&self) -> &dyn FieldType {
        &IntType
    }
    fn get_default(&self) -> Option<String> {
        self.default.clone()
    }
    fn get_nullable(&self) -> bool {
        match self.primary_key {
            true => true,
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
