#[allow(dead_code)]
pub enum FieldType {
    String,
    Integer,
    Float,
    Boolean,
    Date,
    Time,
    DateTime,
    Timestamp,
    Binary,
    Json,
    Jsonb,
    Array,
    Enum,
    Custom,
}

pub trait Field {
    fn get_name(&self) -> &str;
    fn get_type(&self) -> &FieldType;
    fn get_default(&self) -> &str;
    fn get_comment(&self) -> &str;
}
