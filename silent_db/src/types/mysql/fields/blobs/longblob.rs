use crate::core::fields::{Field, FieldType};
#[derive(Clone)]
pub struct LongBlob {
    pub name: String,
    pub default: Option<String>,
    pub nullable: bool,
    pub primary_key: bool,
    pub unique: bool,
    pub comment: Option<String>,
}
impl Default for LongBlob {
    fn default() -> Self {
        LongBlob {
            name: "longblob".to_string(),
            default: None,
            nullable: true,
            primary_key: false,
            unique: false,
            comment: None,
        }
    }
}
struct LongBlobType;

impl FieldType for LongBlobType {
    fn get_type_str(&self) -> String {
        "LONGBLOB".to_string()
    }
}

impl Field for LongBlob {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_type(&self) -> Box<dyn FieldType> {
        Box::new(LongBlobType)
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
    fn get_comment(&self) -> Option<String> {
        self.comment.clone()
    }
}
