use crate::core::tables::TableUtil;

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
}
