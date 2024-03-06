pub trait FieldType {
    fn get_type_str(&self) -> String;
}

pub trait Field {
    fn get_name(&self) -> String;
    fn get_type(&self) -> Box<dyn FieldType>;
    fn get_default(&self) -> Option<String> {
        None
    }
    fn get_nullable(&self) -> bool {
        true
    }
    fn get_primary_key(&self) -> bool {
        false
    }
    fn get_unique(&self) -> bool {
        false
    }
    fn get_auto_increment(&self) -> bool {
        false
    }
    fn get_comment(&self) -> Option<String> {
        None
    }
    fn get_create_sql(&self) -> String {
        let mut sql = format!("`{}` {}", self.get_name(), self.get_type().get_type_str());
        if let Some(default) = self.get_default() {
            sql.push_str(&format!(" DEFAULT {}", default));
        }
        if !self.get_nullable() {
            sql.push_str(" NOT NULL");
        }
        if self.get_primary_key() {
            sql.push_str(" PRIMARY KEY");
        }
        if self.get_unique() {
            sql.push_str(" UNIQUE");
        }
        if self.get_auto_increment() {
            sql.push_str(" AUTO_INCREMENT");
        }
        if let Some(comment) = self.get_comment() {
            sql.push_str(&format!(" COMMENT '{}'", comment));
        }
        sql
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
    struct TestIntField {
        name: String,
        default: Option<String>,
        nullable: bool,
        primary_key: bool,
        unique: bool,
        auto_increment: bool,
        comment: Option<String>,
    }

    impl Field for TestIntField {
        fn get_name(&self) -> String {
            self.name.clone()
        }
        fn get_type(&self) -> Box<dyn FieldType> {
            Box::new(IntType)
        }
        fn get_default(&self) -> Option<String> {
            self.default.clone()
        }
        fn get_nullable(&self) -> bool {
            self.nullable
        }
        fn get_primary_key(&self) -> bool {
            self.primary_key
        }
        fn get_unique(&self) -> bool {
            self.unique
        }
        fn get_auto_increment(&self) -> bool {
            self.auto_increment
        }
        fn get_comment(&self) -> Option<String> {
            self.comment.clone()
        }
    }

    struct IntType;

    impl FieldType for IntType {
        fn get_type_str(&self) -> String {
            "INT".to_string()
        }
    }

    #[test]
    fn test_int_field() {
        let field = TestIntField {
            name: "id".to_string(),
            default: None,
            nullable: false,
            primary_key: true,
            unique: true,
            auto_increment: true,
            comment: Some("ID".to_string()),
        };
        assert_eq!(field.get_name(), "id");
        assert_eq!(field.get_type().get_type_str(), "INT");
        assert_eq!(
            field.get_create_sql(),
            "`id` INT NOT NULL PRIMARY KEY UNIQUE AUTO_INCREMENT COMMENT 'ID'"
        );
    }
}
