use serde::{Deserialize, Serialize};
use silent_db::mysql::base::TableManager;
use silent_db::mysql::fields::{Int, VarChar};
use silent_db::{Query, Table, TableManage};
use std::rc::Rc;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub(crate) struct Test {
    pub id: u32,
    pub name: String,
    pub age: u32,
}
pub(crate) struct Id(Int);

impl Query for Id {
    fn get_field() -> String {
        "id".to_string()
    }
}

impl Id {
    pub fn new() -> Self {
        Id(Int {
            name: Self::get_field(),
            primary_key: true,
            auto_increment: true,
            comment: Some("ID".to_string()),
            ..Default::default()
        })
    }
    pub fn rc(&self) -> Rc<Int> {
        Rc::new(self.0.clone())
    }
}

pub(crate) struct Name(VarChar);

impl Query for Name {
    fn get_field() -> String {
        "name".to_string()
    }
}

impl Name {
    pub fn new() -> Self {
        Name(VarChar {
            name: Self::get_field(),
            comment: Some("姓名".to_string()),
            length: 36,
            ..Default::default()
        })
    }
    pub fn rc(&self) -> Rc<VarChar> {
        Rc::new(self.0.clone())
    }
}

pub(crate) struct Age(Int);

impl Query for Age {
    fn get_field() -> String {
        "age".to_string()
    }
}

impl Age {
    pub fn new() -> Self {
        Age(Int {
            name: Self::get_field(),
            comment: Some("年龄".to_string()),
            ..Default::default()
        })
    }
    pub fn rc(&self) -> Rc<Int> {
        Rc::new(self.0.clone())
    }
}

impl TableManage for Test {
    fn manager() -> Box<dyn Table> {
        Box::new(TableManager {
            name: "test".to_string(),
            fields: vec![Id::new().rc(), Name::new().rc(), Age::new().rc()],
            comment: Some("Test Table".to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_table() {
        let table = Test::manager();
        assert_eq!(table.get_name(), "test");
        let fields = table.get_fields();
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].get_name(), "id");
        assert_eq!(fields[1].get_name(), "name");
        assert_eq!(fields[2].get_name(), "age");
        assert_eq!(table.get_create_sql(), "CREATE TABLE `test` (`id` INT NOT NULL PRIMARY KEY UNIQUE AUTO_INCREMENT COMMENT 'ID', `name` VARCHAR(36) COMMENT '姓名', `age` INT COMMENT '年龄');");
        assert_eq!(table.get_drop_sql(), "DROP TABLE `test`;");
    }
}
