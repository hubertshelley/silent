use silent_db::mysql::base::TableManager;
use silent_db::mysql::fields::{Int, VarChar};
use silent_db::Query;
use silent_db::Table;
use silent_db::TableManage;
use silent_db_macros::Table;
use std::rc::Rc;

#[derive(Table)]
#[table(name = "test_name", comment = "test_comment")]
struct TestTable {
    #[field(field_type = "Int", primary_key, auto_increment, comment = "ID")]
    id: u32,
    #[field(field_type = "VarChar", comment = "姓名", max_length = 36)]
    name: String,
    #[field(field_type = "Int", comment = "年龄")]
    age: u32,
}

fn main() {
    println!("{}", TestTable::manager().get_create_sql());
    println!("{:?}", Id::query_eq(1));
    println!("{:?}", Id::query_eq(1).get_sql());
    let query = Id::query_eq("1") & Name::query_eq("zhangsan");
    println!("{:?}", query);
    println!("{:?}", query.get_query());
}
