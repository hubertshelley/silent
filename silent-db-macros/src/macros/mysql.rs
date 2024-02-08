#[cfg(test)]
mod tests {
    // use super::*;
    use quote::quote;

    // // 使用自定义宏重写 Test 结构体
    // #[table(name = "test", comment = "Test Table")]
    // pub(crate) struct TestTable {
    //     // #[silent_db::field(Int, primary_key = true, auto_increment = true, comment = "ID")]
    //     pub id: u32,
    //     // #[silent_db::field(VarChar, comment = "姓名", length = 36)]
    //     pub name: String,
    //     // #[silent_db::field(Int, comment = "年龄")]
    //     pub age: u32,
    // }

    #[test]
    fn test_table() {
        let input = quote! {
            pub(crate) struct TestTable {
                pub id: u32,
                pub name: String,
                pub age: u32,
            }
        };

        let expected = quote! {
            pub(crate) struct TestTable {
                pub id: u32,
                pub name: String,
                pub age: u32,
            }
        };

        // let result = table(input.into());
        // assert_eq!(result.to_string(), expected.to_string());
    }
}
