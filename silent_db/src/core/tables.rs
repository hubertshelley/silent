use crate::core::fields::Field;

pub trait TableUtil {
    fn get_all_tables(&self) -> String;
    fn get_table(&self, table: &str) -> String;
}
pub trait Table {
    fn get_name() -> String;
    fn get_fields() -> Vec<Box<dyn Field>>;
    fn get_comment() -> Option<String> {
        None
    }
    fn get_create_sql(&self) -> String {
        let mut sql = format!("CREATE TABLE `{}` (", Self::get_name());
        let fields: Vec<String> = Self::get_fields()
            .iter()
            .map(|field| field.get_create_sql())
            .collect();
        sql.push_str(&fields.join(", "));
        sql.push(')');
        if let Some(comment) = Self::get_comment() {
            sql.push_str(&format!(" COMMENT='{}'", comment));
        }
        sql.push(';');
        sql
    }
    fn get_drop_sql(&self) -> String {
        format!("DROP TABLE {};", Self::get_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::fields::{Field, FieldType};
    use serde::{Deserialize, Serialize};

    struct TestTable;

    #[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
    struct IntField {
        name: String,
        default: Option<String>,
        nullable: bool,
        primary_key: bool,
        unique: bool,
        auto_increment: bool,
        comment: Option<String>,
    }

    impl Field for IntField {
        fn get_name(&self) -> String {
            format!("`{}`", self.name)
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

    impl Table for TestTable {
        fn get_name() -> String {
            "test_table".to_string()
        }
        fn get_fields() -> Vec<Box<dyn Field>> {
            let int = IntField {
                name: "id".to_string(),
                default: None,
                nullable: false,
                primary_key: true,
                unique: false,
                auto_increment: true,
                comment: None,
            };
            vec![Box::new(int)]
        }
        fn get_comment() -> Option<String> {
            Some("Test Table".to_string())
        }
    }

    #[test]
    fn test_get_create_sql() {
        let table = TestTable;
        assert_eq!(
            table.get_create_sql(),
            "CREATE TABLE test_table (`id` INT NOT NULL PRIMARY KEY AUTO_INCREMENT) COMMENT='Test Table';"
        );
    }

    #[test]
    fn test_get_drop_sql() {
        let table = TestTable;
        assert_eq!(table.get_drop_sql(), "DROP TABLE test_table;");
    }
}
