use crate::core::dsl::SqlStatement;
use crate::core::tables::{DetectField, DetectFieldLength, TableUtil};
use crate::utils::{to_camel_case, to_snake_case};
use crate::{Field, IndexTrait, Table};
use anyhow::Result;
use sqlparser::ast::Statement;
use std::fmt::Debug;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::rc::Rc;

#[derive(Default)]
pub struct TableUtils;

impl TableUtils {
    pub fn new() -> Self {
        TableUtils
    }
}

impl TableUtil for TableUtils {
    fn get_name(&self) -> String {
        "MySQL".to_string()
    }
    fn get_all_tables(&self) -> String {
        "SHOW TABLES;".to_string()
    }

    fn get_table(&self, table: &str) -> String {
        format!("SHOW CREATE TABLE `{}`;", table)
    }

    fn transform(&self, table: &SqlStatement) -> Result<Box<dyn Table>> {
        let SqlStatement(statement, _) = table;
        if let Statement::CreateTable { name, comment, .. } = statement.clone() {
            let name = name.0.first().unwrap().value.clone();
            Ok(Box::new(TableManager {
                name,
                fields: vec![],
                indices: vec![],
                comment,
            }))
        } else {
            Err(anyhow::anyhow!("Not a CreateTable statement"))
        }
    }
    fn generate_models(&self, tables: Vec<SqlStatement>, models_path: &Path) -> Result<()> {
        if !models_path.exists() || !models_path.is_dir() {
            fs::create_dir_all(models_path)?;
        } else {
            fs::remove_dir_all(models_path)?;
            fs::create_dir_all(models_path)?;
        }
        // todo!("生成模型文件")
        let _ = tables;
        let mut mod_files = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(models_path.join("mod.rs"))?;
        let mut models = vec![];
        for table in tables {
            if let Statement::CreateTable {
                name,
                columns,
                comment,
                ..
            } = table.0
            {
                let name = name.0.first().unwrap().value.clone();

                models.push(name.clone());
                let mut field_types = Vec::new();
                let mut file = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(models_path.join(format!("{}.rs", to_snake_case(&name))))?;
                let table_derive = format!(
                    "#[derive(Table, Clone, Debug, Deserialize, Serialize)]\n#[table(name = \"{}\"",
                    name
                );
                let table_derive = if let Some(comment) = comment {
                    format!("{}, comment = \"{}\")]\n", table_derive, comment)
                } else {
                    format!("{})]\n", table_derive)
                };
                let content = columns
                    .iter()
                    .map(|column| {
                        let name = column.name.value.clone();
                        let field_type = column.data_type.to_string();
                        let detect_field = self.detect_fields(&field_type);
                        let (field_type, struct_type) = self.get_field_type(&detect_field);
                        if !field_types.contains(&field_type.to_string()) {
                            field_types.push(field_type.to_string());
                        }
                        let mut field_derive = format!(
                            "    #[field(field_type = \"{}\", name = \"{}\"",
                            field_type, name
                        );
                        let mut is_optional = false;
                        for options in column.options.iter() {
                            match &options.option {
                                sqlparser::ast::ColumnOption::Null => {
                                    field_derive.push_str(", nullable");
                                    is_optional = true;
                                }
                                sqlparser::ast::ColumnOption::Unique { .. } => {
                                    field_derive.push_str(", unique");
                                }
                                sqlparser::ast::ColumnOption::Default(default) => {
                                    field_derive.push_str(&format!(", default = \"{}\"", default));
                                    is_optional = true;
                                }
                                sqlparser::ast::ColumnOption::Generated { .. } => {
                                    field_derive.push_str(", auto_increment");
                                    is_optional = true;
                                }
                                sqlparser::ast::ColumnOption::Comment(comment) => {
                                    field_derive.push_str(&format!(", comment = \"{}\"", comment));
                                }
                                _ => {}
                            }
                        }
                        let struct_type = if is_optional {
                            format!("Option<{}>", struct_type)
                        } else {
                            struct_type.to_string()
                        };
                        match detect_field.length {
                            Some(DetectFieldLength::MaxLength(max_length)) => {
                                field_derive.push_str(&format!(", max_length = {}", max_length));
                            }
                            Some(DetectFieldLength::MaxDigits(max_digits, decimal_places)) => {
                                field_derive.push_str(&format!(
                                    ", max_digits = {:?}, decimal_places = {:?}",
                                    max_digits, decimal_places
                                ));
                            }
                            _ => {}
                        }
                        let field = format!("    {}: {}", to_snake_case(&name), struct_type);
                        format!("{})]\n{}", field_derive, field)
                    })
                    .collect::<Vec<String>>()
                    .join(",\n");
                let table_derive = format!(
                    "{}{}{}\n\n\n{}",
                    r#"use serde::{Deserialize, Serialize};
use silent_db::mysql::base::TableManager;
use silent_db::mysql::fields::{"#,
                    field_types.join(", "),
                    r#"};
use silent_db::*;
use std::rc::Rc;"#,
                    table_derive
                );
                file.write_all(table_derive.as_bytes())?;
                file.write_all(format!("pub struct {}{{\n", to_camel_case(&name)).as_bytes())?;
                file.write_all(content.as_bytes())?;
                file.write_all("\n}\n".to_string().as_bytes())?;
                mod_files.write_all(format!("pub mod {};\n", to_snake_case(&name)).as_bytes())?;
            }
        }
        Ok(())
    }
    fn get_field_type(&self, detect_field: &DetectField) -> (&str, &str) {
        match detect_field.field_type.as_str() {
            // blob
            "blob" => {
                let field_type = "Vec<u8>";
                ("Blob", field_type)
            }
            "longblob" => {
                let field_type = "Vec<u8>";
                ("LongBlob", field_type)
            }
            "mediumblob" => {
                let field_type = "Vec<u8>";
                ("MediumBlob", field_type)
            }
            "tinyblob" => {
                let field_type = "Vec<u8>";
                ("TinyBlob", field_type)
            }
            // datetime
            "date" => {
                let field_type = "DateTime<Utc>";
                ("Date", field_type)
            }
            "datetime" => {
                let field_type = "DateTime<Utc>";
                ("Datetime", field_type)
            }
            "time" => {
                let field_type = "DateTime<Utc>";
                ("Time", field_type)
            }
            "timestamp" => {
                let field_type = "DateTime<Utc>";
                ("TimeStamp", field_type)
            }
            "year" => {
                let field_type = "i16";
                ("Year", field_type)
            }
            // number
            "bigint" => {
                let field_type = "i64";
                ("BigInt", field_type)
            }
            "decimal" => {
                let field_type = "f64";
                ("Decimal", field_type)
            }
            "double" => {
                let field_type = "f64";
                ("Double", field_type)
            }
            "float" => {
                let field_type = "f32";
                ("Float", field_type)
            }
            "int" => {
                let field_type = match detect_field.length {
                    Some(DetectFieldLength::MaxLength(max_length)) => {
                        if max_length == 1 {
                            "bool"
                        } else if max_length <= 16 {
                            "i16"
                        } else if max_length <= 32 {
                            "i32"
                        } else if max_length <= 64 {
                            "i64"
                        } else {
                            "u64"
                        }
                    }
                    _ => "u64",
                };
                ("Int", field_type)
            }
            "mediumint" => {
                let field_type = "i32";
                ("MediumInt", field_type)
            }
            "smallint" => {
                let field_type = "i16";
                ("SmallInt", field_type)
            }
            "tinyint" => {
                let field_type = match detect_field.length {
                    Some(DetectFieldLength::MaxLength(max_length)) => {
                        if max_length == 1 {
                            "bool"
                        } else {
                            "i8"
                        }
                    }
                    _ => "i8",
                };
                ("TinyInt", field_type)
            }
            // string
            "char" => {
                let field_type = "String";
                ("Char", field_type)
            }
            "longtext" => {
                let field_type = "String";
                ("LongText", field_type)
            }
            "mediumtext" => {
                let field_type = "String";
                ("MediumText", field_type)
            }
            "text" => {
                let field_type = "String";
                ("Text", field_type)
            }
            "tinytext" => {
                let field_type = "String";
                ("TinyText", field_type)
            }
            "varchar" => {
                let field_type = "String";
                ("VarChar", field_type)
            }
            // string
            "json" => {
                let field_type = "Value";
                ("Json", field_type)
            }
            _ => ("VarChar", "String"),
        }
    }
}

#[derive(Clone)]
pub struct TableManager {
    pub name: String,
    pub fields: Vec<Rc<dyn Field>>,
    pub indices: Vec<Rc<dyn IndexTrait>>,
    pub comment: Option<String>,
}

impl Default for TableManager {
    fn default() -> Self {
        TableManager {
            name: "".to_string(),
            fields: vec![],
            indices: vec![],
            comment: None,
        }
    }
}

impl Table for TableManager {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_fields(&self) -> Vec<Rc<dyn Field>> {
        self.fields.clone()
    }
    fn get_indices(&self) -> Vec<Rc<dyn IndexTrait>> {
        self.indices.clone()
    }

    fn get_comment(&self) -> Option<String> {
        self.comment.clone()
    }
}

impl Debug for TableManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Table {{ name: {}\n    fields: {}\n}}",
            self.name,
            self.fields
                .iter()
                .map(|field| format!("{}: {}", field.get_name(), field.get_type().get_type_str()))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

impl PartialEq<Self> for TableManager {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for TableManager {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::tables::{DetectFieldLength, TableManage};
    use crate::mysql::fields::Int;
    use crate::mysql::indices::{Index, IndexType};
    use crate::IndexSort;

    struct TestTable;

    impl TableManage for TestTable {
        fn manager() -> Box<dyn Table> {
            Box::new(TableManager {
                name: "test_table".to_string(),
                fields: vec![Rc::new(Int {
                    name: "id".to_string(),
                    primary_key: true,
                    auto_increment: true,
                    comment: None,
                    ..Default::default()
                })],
                indices: vec![],
                comment: Some("Test Table".to_string()),
            })
        }
    }

    #[test]
    fn test_get_create_sql() {
        let table = TestTable;
        assert_eq!(
            table.get_manager().get_create_sql(),
            "CREATE TABLE `test_table` (`id` INT NOT NULL PRIMARY KEY UNIQUE AUTO_INCREMENT) COMMENT='Test Table';"
        );
    }
    struct TestTableWithIndex;

    impl TableManage for TestTableWithIndex {
        fn manager() -> Box<dyn Table> {
            Box::new(TableManager {
                name: "test_table".to_string(),
                fields: vec![Rc::new(Int {
                    name: "id".to_string(),
                    primary_key: true,
                    auto_increment: true,
                    comment: None,
                    ..Default::default()
                })],
                indices: vec![Rc::new(Index {
                    alias: Some("idx".to_string()),
                    index_type: IndexType::Unique,
                    fields: vec!["id".to_string()],
                    sort: IndexSort::ASC,
                })],
                comment: Some("Test Table".to_string()),
            })
        }
    }

    #[test]
    fn test_get_create_sql_with_index() {
        let table = TestTableWithIndex;
        assert_eq!(
            table.get_manager().get_create_sql(),
            "CREATE TABLE `test_table` (`id` INT NOT NULL PRIMARY KEY UNIQUE AUTO_INCREMENT, UNIQUE KEY `idx` (`id`) ASC) COMMENT='Test Table';"
        );
    }

    #[test]
    fn test_get_drop_sql() {
        let table = TestTable;
        assert_eq!(
            table.get_manager().get_drop_sql(),
            "DROP TABLE `test_table`;"
        );
    }

    #[test]
    fn test_utils_detect_field() {
        let utils = TableUtils::new();
        let detect_field = utils.detect_fields("int(11)");
        assert_eq!(detect_field.field_type, "int");
        assert_eq!(
            detect_field,
            DetectField {
                field_type: "int".to_string(),
                length: Some(DetectFieldLength::MaxLength(11)),
            }
        );
        let detect_field = utils.detect_fields("varchar(255)");
        assert_eq!(detect_field.field_type, "varchar");
        assert_eq!(detect_field.length, Some(DetectFieldLength::MaxLength(255)));
        let detect_field = utils.detect_fields("decimal(10, 2)");
        assert_eq!(detect_field.field_type, "decimal");
        assert_eq!(
            detect_field.length,
            Some(DetectFieldLength::MaxDigits(10, 2))
        );
    }
}
