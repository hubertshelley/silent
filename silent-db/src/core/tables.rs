use crate::core::dsl::SqlStatement;
use crate::core::fields::Field;
use crate::core::indices::IndexTrait;
use anyhow::Result;
use std::path::Path;
use std::rc::Rc;

pub trait TableUtil {
    fn get_name(&self) -> String;
    fn get_all_tables(&self) -> String;
    fn get_table(&self, table: &str) -> String;
    fn transform(&self, table: &SqlStatement) -> Result<Box<dyn Table>>;
    fn generate_models(&self, tables: Vec<SqlStatement>, models_path: &Path) -> Result<()>;

    /// 从字段字符串中检测字段类型和长度
    fn detect_fields(&self, field_str: &str) -> DetectField {
        let field_str = field_str.to_lowercase();
        // 利用正则取出字段后的包含括号的长度数值
        // 如 int(11) -> 11
        // 如 varchar(255) -> 255
        // 如 decimal(10, 2) -> 10, 2
        let re = regex::Regex::new(r"\((\d+)(?:, (\d+))?\)").unwrap();
        let length = if let Some(caps) = re.captures(&field_str) {
            if caps.len() == 3 {
                match (caps.get(1), caps.get(2)) {
                    (Some(max_digits), Some(decimal_places)) => Some(
                        (
                            max_digits.as_str().parse::<u8>().unwrap_or(0),
                            decimal_places.as_str().parse::<u8>().unwrap_or(0),
                        )
                            .into(),
                    ),
                    (Some(max_length), None) => {
                        Some(max_length.as_str().parse::<u16>().unwrap_or(0).into())
                    }
                    (_, _) => None,
                }
            } else {
                None
            }
        } else {
            None
        };
        // 利用正则表达式取出首个左括号之前的字符串作为字段类型
        // 如 int(11) -> int
        // 如 varchar(255) -> varchar
        // 如 decimal(10, 2) -> decimal
        let re = regex::Regex::new(r"(\w+)(?:\(\d+(?:, \d+)?\))?$").unwrap();
        let field_type = re
            .captures(&field_str)
            .unwrap()
            .get(1)
            .unwrap()
            .as_str()
            .to_string();
        DetectField { field_type, length }
    }

    /// 从DetectField中获取字段类型和结构体类型
    fn get_field_type(&self, detect_field: &DetectField) -> (&str, &str);
}

#[derive(Debug, Eq, PartialEq)]
pub struct DetectField {
    pub field_type: String,
    pub length: Option<DetectFieldLength>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum DetectFieldLength {
    MaxLength(u16),
    MaxDigits(u8, u8),
}

impl From<u16> for DetectFieldLength {
    fn from(length: u16) -> Self {
        DetectFieldLength::MaxLength(length)
    }
}

impl From<(u8, u8)> for DetectFieldLength {
    fn from(digits: (u8, u8)) -> Self {
        DetectFieldLength::MaxDigits(digits.0, digits.1)
    }
}

pub trait Table {
    fn get_name(&self) -> String;
    fn get_fields(&self) -> Vec<Rc<dyn Field>>;
    fn get_indices(&self) -> Vec<Rc<dyn IndexTrait>> {
        vec![]
    }
    fn get_comment(&self) -> Option<String> {
        None
    }
    fn get_create_sql(&self) -> String {
        let mut sql = format!("CREATE TABLE `{}` (", self.get_name());
        let fields: Vec<String> = self
            .get_fields()
            .iter()
            .map(|field| field.get_create_sql())
            .collect();
        sql.push_str(&fields.join(", "));
        if !self.get_indices().is_empty() {
            sql.push_str(", ");
            let indices: Vec<String> = self
                .get_indices()
                .iter()
                .map(|index| index.get_create_sql())
                .collect();
            sql.push_str(&indices.join(", "));
        }
        sql.push(')');
        if let Some(comment) = self.get_comment() {
            sql.push_str(&format!(" COMMENT='{}'", comment));
        }
        sql.push(';');
        sql
    }
    fn get_drop_sql(&self) -> String {
        format!("DROP TABLE `{}`;", self.get_name())
    }
}

pub trait TableManage {
    fn get_manager(&self) -> Box<dyn Table> {
        Self::manager()
    }
    fn manager() -> Box<dyn Table>;
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

    impl Table for TestTable {
        fn get_name(&self) -> String {
            "test_table".to_string()
        }
        fn get_fields(&self) -> Vec<Rc<dyn Field>> {
            let int = IntField {
                name: "id".to_string(),
                default: None,
                nullable: false,
                primary_key: true,
                unique: false,
                auto_increment: true,
                comment: None,
            };
            vec![Rc::new(int)]
        }
        fn get_comment(&self) -> Option<String> {
            Some("Test Table".to_string())
        }
    }

    #[test]
    fn test_get_create_sql() {
        let table = TestTable;
        assert_eq!(
            table.get_create_sql(),
            "CREATE TABLE `test_table` (`id` INT NOT NULL PRIMARY KEY AUTO_INCREMENT) COMMENT='Test Table';"
        );
    }

    #[test]
    fn test_get_drop_sql() {
        let table = TestTable;
        assert_eq!(table.get_drop_sql(), "DROP TABLE `test_table`;");
    }
}
