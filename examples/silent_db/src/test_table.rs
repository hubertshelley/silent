use silent_db::mysql::fields::{Int, VarChar};
use silent_db::{Field, Query, Table};

pub(crate) struct Test {
    pub id: u32,
    pub name: String,
    pub age: u32,
}
pub(crate) struct Id(Int);

impl Query for Id {
    fn get_filed() -> String {
        "id".to_string()
    }
}

impl Id {
    pub fn new() -> Self {
        Id(Int {
            name: Self::get_filed(),
            primary_key: true,
            auto_increment: true,
            comment: Some("ID".to_string()),
            ..Default::default()
        })
    }
    pub fn boxed(&self) -> Box<Int> {
        Box::new(self.0.clone())
    }
}

pub(crate) struct Name(VarChar);

impl Query for Name {
    fn get_filed() -> String {
        "name".to_string()
    }
}

impl Name {
    pub fn new() -> Self {
        Name(VarChar {
            name: Self::get_filed(),
            comment: Some("姓名".to_string()),
            length: 36,
            ..Default::default()
        })
    }
    pub fn boxed(&self) -> Box<VarChar> {
        Box::new(self.0.clone())
    }
}

pub(crate) struct Age(Int);

impl Query for Age {
    fn get_filed() -> String {
        "age".to_string()
    }
}

impl Age {
    pub fn new() -> Self {
        Age(Int {
            name: Self::get_filed(),
            comment: Some("年龄".to_string()),
            ..Default::default()
        })
    }
    pub fn boxed(&self) -> Box<Int> {
        Box::new(self.0.clone())
    }
}

impl Table for Test {
    fn get_name() -> String {
        "test".to_string()
    }
    fn get_fields() -> Vec<Box<dyn Field>> {
        vec![Id::new().boxed(), Name::new().boxed(), Age::new().boxed()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_table() {
        assert_eq!(Test::get_name(), "test");
        let fields = Test::get_fields();
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].get_name(), "id");
        assert_eq!(fields[1].get_name(), "name");
        assert_eq!(fields[2].get_name(), "age");
        println!("Create: {:?}", Test::get_create_sql());
        println!("Drop: {:?}", Test::get_drop_sql());
    }
}
