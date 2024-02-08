use crate::core::fields::{Field, FieldType};
#[derive(Clone)]
pub struct Decimal {
    pub name: String,
    pub default: Option<String>,
    pub nullable: bool,
    pub primary_key: bool,
    pub unique: bool,
    pub comment: Option<String>,
    pub max_digits: u8,
    pub decimal_places: u8,
}
impl Default for Decimal {
    fn default() -> Self {
        Decimal {
            name: "decimal".to_string(),
            default: None,
            nullable: true,
            primary_key: false,
            unique: false,
            comment: None,
            max_digits: 10,
            decimal_places: 0,
        }
    }
}
struct DecimalType(u8, u8);

impl FieldType for DecimalType {
    fn get_type_str(&self) -> String {
        format!("DECIMAL({}, {})", self.0, self.1)
    }
}

impl Field for Decimal {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_type(&self) -> Box<dyn FieldType> {
        Box::new(DecimalType(self.max_digits, self.decimal_places))
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
