extern crate proc_macro;

mod macros;
mod utils;

use crate::macros::{get_field_attr, get_table_attr, TableAttr};
use crate::utils::{to_camel_case, to_snake_case};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Fields};

#[proc_macro_derive(Table, attributes(table, field))]
pub fn derive_table(item: TokenStream) -> TokenStream {
    let item_copy = item.clone();
    let input = parse_macro_input!(item as DeriveInput);
    let fields = match input.data {
        Data::Struct(ref data_struct) => {
            if let Fields::Named(ref fields_named) = data_struct.fields {
                fields_named.named.iter()
            } else {
                panic!("Only named fields are supported");
            }
        }
        _ => panic!("Only structs are supported"),
    };
    let mut fields_data: Vec<FieldAttr> = vec![];
    for field in fields {
        let field_name = field.ident.as_ref().unwrap().to_string();
        if let Some(attr) = field
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("field"))
        {
            fields_data.push(derive_field_attribute(attr, field_name));
        } else {
            let attr = FieldAttr {
                name: to_snake_case(&field_name),
                token_stream: Default::default(),
            };
            fields_data.push(attr);
        }
    }
    let table_token = match input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("table"))
    {
        Some(attr) => get_table_attr(attr),
        None => TableAttr {
            name: Some(to_snake_case(&input.ident.to_string())),
            comment: None,
        },
    };
    let table_token = derive_table_attribute(table_token, item_copy, &fields_data);
    let mut fields = TokenStream::from(quote! {});
    for field in fields_data {
        fields.extend(field.token_stream);
    }
    fields.extend(table_token);
    fields
}

fn derive_table_attribute(
    table_attr: TableAttr,
    input: TokenStream,
    field_data: &[FieldAttr],
) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    let comment = match table_attr.comment {
        Some(c) => quote! { Some(#c.to_string()) },
        None => {
            quote! { None }
        }
    };

    let name = match table_attr.name {
        Some(c) => quote! { #c.to_string() },
        None => {
            let table_name = to_snake_case(&struct_name.to_string());
            quote! { #table_name.to_string() }
        }
    };

    let fields_code = format!(
        "vec![{}]",
        field_data
            .iter()
            .map(|field| format!("{}::new().rc()", to_camel_case(&field.name)))
            .collect::<Vec<String>>()
            .join(", ")
    );
    let fields_token: proc_macro2::TokenStream = fields_code.parse().unwrap();

    // Generate the code for implementing the trait
    let expanded = quote! {
        impl TableManage for #struct_name {
            fn manager() -> Box<dyn Table> {
                Box::new(TableManager {
                    name: #name,
                    fields: #fields_token,
                    comment: #comment,
                })
            }
        }
    };

    // Return the generated implementation
    TokenStream::from(expanded)
}

#[derive(Debug)]
struct FieldAttr {
    name: String,
    token_stream: TokenStream,
}

fn derive_field_attribute(args: &Attribute, field_name: String) -> FieldAttr {
    let field_attr = get_field_attr(args);

    let snake_field_name = to_snake_case(&field_name.to_string());
    let camel_field_name = to_camel_case(&field_name.to_string());

    let args = quote! {
        name: Self::get_field(),
    };
    // 设置字段名称
    let args = match field_attr.name.clone() {
        Some(c) => quote! { name: #c.to_string(), },
        None => quote! { #args },
    };
    // 设置字段默认值
    let args = match field_attr.default {
        Some(c) => quote! { #args
        default: Some(#c.to_string()), },
        None => quote! { #args },
    };
    // 设置字段是否为空
    let args = match field_attr.nullable {
        Some(c) => quote! { #args
        nullable: #c, },
        None => quote! { #args },
    };
    // 设置字段是否为主键
    let args = match field_attr.primary_key {
        Some(c) => quote! { #args
        primary_key: #c, },
        None => quote! { #args },
    };
    // 设置字段是否唯一
    let args = match field_attr.unique {
        Some(c) => quote! { #args
        unique: #c, },
        None => quote! { #args },
    };
    // 设置字段注释
    let args = match field_attr.comment {
        Some(c) => quote! { #args
        comment: Some(#c.to_string()), },
        None => quote! { #args },
    };
    // 设置字段是否自增
    let args = match field_attr.auto_increment {
        Some(c) => quote! { #args
        auto_increment: #c, },
        None => quote! { #args },
    };
    // 设置字段最大位数
    let args = match field_attr.max_digits {
        Some(c) => quote! { #args
        max_digits: #c, },
        None => quote! { #args },
    };
    // 设置字段小数位数
    let args = match field_attr.decimal_places {
        Some(c) => quote! { #args
        decimal_places: #c, },
        None => quote! { #args },
    };
    // 设置字段长度
    let args = match field_attr.length {
        Some(c) => quote! { #args
        length: #c, },
        None => quote! { #args },
    };

    let code = format!(
        r#"
    pub struct {camel_field_name}({field_type});
    
    impl Query for {camel_field_name} {{
        fn get_field() -> String {{
            "{snake_field_name}".to_string()
        }}
    }}
    
    impl {camel_field_name} {{
        pub fn new() -> Self {{
            {camel_field_name}({field_type} {{
                {args}
                ..Default::default()
            }})
        }}
        pub fn rc(&self) -> Rc<{field_type}> {{
            Rc::new(self.0.clone())
        }}
    }}
    "#,
        camel_field_name = camel_field_name,
        field_type = field_attr.field_type,
        snake_field_name = snake_field_name,
        args = args
    );
    FieldAttr {
        name: field_attr.name.unwrap_or(snake_field_name),
        token_stream: code.parse().unwrap(),
    }
}
