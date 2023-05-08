use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Eq, PartialEq)]
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

pub trait Field<'de>: Deserialize<'de> + Serialize {
    const NAME: &'static str;
    const TYPE: FieldType;
    const COMMENT: &'static str;
    type Value: Deserialize<'de> + Serialize;
    fn get_field_name(&self) -> &str {
        Self::NAME
    }
    fn get_type(&self) -> &FieldType {
        &Self::TYPE
    }
    fn get_comment(&self) -> &str {
        Self::COMMENT
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    struct Table {
        name: StringField,
    }

    type StringField = String;

    impl<'de> Field<'de> for StringField {
        const NAME: &'static str = "name";
        const TYPE: FieldType = FieldType::String;
        const COMMENT: &'static str = "姓名";
        type Value = String;
    }

    #[test]
    fn test_field() {
        let table = serde_json::from_str::<Table>(r#"{"name":"张三"}"#).unwrap();
        // let table = Table {
        //     name: "张三".to_string(),
        // };
        assert_eq!(table.name.get_field_name(), "name");
        assert_eq!(table.name.get_type(), &FieldType::String);
        assert_eq!(table.name.get_comment(), "姓名");
        println!("{:?}", table);
        println!("{}", serde_json::to_string(&table).unwrap());
    }
}
