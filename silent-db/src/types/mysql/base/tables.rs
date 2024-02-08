use crate::core::dsl::SqlStatement;
use crate::core::tables::TableUtil;
use crate::{Field, Table};
use anyhow::Result;
use sqlparser::ast::Statement;
use std::fmt::Debug;
use std::fs;
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
                comment,
            }))
        } else {
            Err(anyhow::anyhow!("Not a CreateTable statement"))
        }
    }
    fn generate_models(&self, tables: Vec<SqlStatement>, models_path: &Path) -> Result<()> {
        if !models_path.exists() || !models_path.is_dir() {
            fs::create_dir_all(models_path)?;
        }
        // todo!("生成模型文件")
        let _ = tables;
        // let mut mod_files = OpenOptions::new()
        //     .create(true)
        //     .write(true)
        //     .truncate(true)
        //     .open(models_path.join("mod.rs"))?;
        // let mut models = vec![];
        // for table in tables {
        //     if let Statement::CreateTable {
        //         name,
        //         columns,
        //         comment,
        //         ..
        //     } = table.0
        //     {
        //         let name = name.0.first().unwrap().value.clone();
        //         models.push(name.clone());
        //         let mut file = OpenOptions::new()
        //             .create(true)
        //             .write(true)
        //             .truncate(true)
        //             .open(models_path.join(format!("{}.rs", to_snake_case(&name))))?;
        //         file.write_all(
        //             format!(
        //                 "pub mod {};\n",
        //                 to_snake_case(&name)
        //             )
        //             .as_bytes(),
        //         )?;
        //     }
        // }
        Ok(())
    }
}

#[derive(Clone)]
pub struct TableManager {
    pub name: String,
    pub fields: Vec<Rc<dyn Field>>,
    pub comment: Option<String>,
}

impl Default for TableManager {
    fn default() -> Self {
        TableManager {
            name: "".to_string(),
            fields: vec![],
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
    use crate::core::tables::TableManage;
    use crate::mysql::fields::Int;

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
                comment: Some("Test Table".to_string()),
            })
        }
    }

    #[test]
    fn test_get_create_sql() {
        let table = TestTable;
        assert_eq!(
            table.get_manager().get_create_sql(),
            "CREATE TABLE test_table (`id` INT NOT NULL PRIMARY KEY AUTO_INCREMENT) COMMENT='Test Table';"
        );
    }

    #[test]
    fn test_get_drop_sql() {
        let table = TestTable;
        assert_eq!(table.get_manager().get_drop_sql(), "DROP TABLE test_table;");
    }
}
