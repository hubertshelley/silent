#![allow(dead_code)]

use std::fmt::Display;

use darling::{ast, FromDeriveInput, FromField, FromMeta};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse_macro_input;

use crate::utils::{to_camel_case, to_snake_case};

mod utils;

#[proc_macro_derive(Table, attributes(table, field))]
pub fn derive_table(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as syn::DeriveInput);
    let table_attr = TableAttr::from_derive_input(&input).unwrap();
    let mut tokens = TokenStream::new();
    table_attr.to_tokens(&mut tokens);
    tokens.into()
}

#[derive(Debug, FromDeriveInput)]
#[darling(
    attributes(table),
    supports(struct_any),
    forward_attrs(allow, doc, cfg)
)]
struct TableAttr {
    ident: syn::Ident,
    generics: syn::Generics,
    data: ast::Data<(), FieldAttr>,
    name: Option<String>,
    comment: Option<String>,
    #[darling(multiple)]
    index: Vec<IndexAttr>,
}
impl ToTokens for TableAttr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let TableAttr {
            ref ident,
            ref data,
            name,
            comment,
            index,
            ..
        } = self;
        let indices = index;
        let struct_name = ident;

        let comment = match comment {
            Some(c) => quote! { Some(#c.to_string()) },
            None => {
                quote! { None }
            }
        };

        let name = match name {
            Some(c) => quote! { #c.to_string() },
            None => {
                let table_name = to_snake_case(&struct_name.to_string());
                quote! { #table_name.to_string() }
            }
        };
        let fields = data
            .as_ref()
            .take_struct()
            .expect("Should never be enum")
            .fields;

        let mut fields_data: Vec<FieldToken> = vec![];
        for field in fields {
            fields_data.push(derive_field_attribute(
                field,
                field.ident.as_ref().unwrap().to_string(),
            ));
        }
        let field_name_list = fields_data
            .iter()
            .map(|f| f.name.clone())
            .collect::<Vec<String>>();
        for field in fields_data {
            tokens.extend(field.token_stream);
        }

        let fields_code = format!(
            "vec![{}]",
            field_name_list
                .iter()
                .map(|field| format!("{}::new().rc()", to_camel_case(field)))
                .collect::<Vec<String>>()
                .join(", ")
        );

        let indices_code = format!(
            "vec![{}]",
            indices
                .iter()
                .map(|index| {
                    if !index.check_fields(&field_name_list) {
                        panic!("Index fields is empty");
                    }
                    format!("Rc::new({})", index)
                })
                .collect::<Vec<String>>()
                .join(", ")
        );
        let fields_token: TokenStream = fields_code.parse().unwrap();
        let indices_token: TokenStream = indices_code.parse().unwrap();

        // Generate the code for implementing the trait
        let expanded = quote! {
            impl TableManage for #struct_name {
                fn manager() -> Box<dyn Table> {
                    Box::new(TableManager {
                        name: #name,
                        fields: #fields_token,
                        indices: #indices_token,
                        comment: #comment,
                    })
                }
            }
        };

        tokens.extend(expanded);
    }
}

#[derive(Debug, FromMeta)]
struct IndexAttr {
    alias: Option<String>,
    index_type: String,
    fields: String,
    sort: Option<String>,
}

impl IndexAttr {
    fn get_index_type(&self) -> String {
        // TODO: add more index type
        self.index_type.clone()
    }

    fn get_sort(&self) -> String {
        if self.sort == Some("desc".to_string()) {
            "IndexSort::DESC".to_string()
        } else {
            self.sort.clone().unwrap_or("IndexSort::ASC".to_string())
        }
    }
    pub(crate) fn check_fields(&self, fields: &[String]) -> bool {
        let index_fields = self
            .fields
            .clone()
            .split(',')
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        index_fields.iter().all(|f| fields.contains(f))
    }
}

impl Display for IndexAttr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let alias = match &self.alias {
            Some(alias) => format!("Some(\"{}\".to_string())", alias),
            None => "None".to_string(),
        };
        let fields = self
            .fields
            .split(',')
            .map(|f| format!("\"{}\".to_string()", f))
            .collect::<Vec<String>>()
            .join(",");

        write!(
            f,
            "Index {{alias:{}, index_type: {}, fields: vec![{}],sort: {}}}",
            alias,
            self.get_index_type(),
            fields,
            self.get_sort()
        )
    }
}

#[derive(Debug)]
struct FieldToken {
    name: String,
    token_stream: TokenStream,
}

#[derive(Debug, FromField)]
#[darling(attributes(field))]
struct FieldAttr {
    ident: Option<syn::Ident>,
    ty: syn::Type,
    field_type: String,
    name: Option<String>,
    default: Option<String>,
    nullable: Option<bool>,
    primary_key: Option<bool>,
    auto_increment: Option<bool>,
    unique: Option<bool>,
    comment: Option<String>,
    max_digits: Option<u8>,
    decimal_places: Option<u8>,
    max_length: Option<u16>,
}

fn derive_field_attribute(field_attr: &FieldAttr, field_name: String) -> FieldToken {
    let FieldAttr {
        field_type,
        name,
        default,
        nullable,
        primary_key,
        auto_increment,
        unique,
        comment,
        max_digits,
        decimal_places,
        max_length,
        ..
    } = field_attr;

    let snake_field_name = to_snake_case(&field_name.to_string());
    let camel_field_name = to_camel_case(&field_name.to_string());

    let args = quote! {
        name: Self::get_field(),
    };
    // 设置字段名称
    let args = match name.clone() {
        Some(c) => quote! { name: #c.to_string(), },
        None => quote! { #args },
    };
    // 设置字段默认值
    let args = match default {
        Some(c) => quote! { #args
        default: Some(#c.to_string()), },
        None => quote! { #args },
    };
    // 设置字段是否为空
    let args = match nullable {
        Some(c) => quote! { #args
        nullable: #c, },
        None => quote! { #args },
    };
    // 设置字段是否为主键
    let args = match primary_key {
        Some(c) => quote! { #args
        primary_key: #c, },
        None => quote! { #args },
    };
    // 设置字段是否唯一
    let args = match unique {
        Some(c) => quote! { #args
        unique: #c, },
        None => quote! { #args },
    };
    // 设置字段注释
    let args = match comment {
        Some(c) => quote! { #args
        comment: Some(#c.to_string()), },
        None => quote! { #args },
    };
    // 设置字段是否自增
    let args = match auto_increment {
        Some(c) => quote! { #args
        auto_increment: #c, },
        None => quote! { #args },
    };
    // 设置字段最大位数
    let args = match max_digits {
        Some(c) => quote! { #args
        max_digits: #c, },
        None => quote! { #args },
    };
    // 设置字段小数位数
    let args = match decimal_places {
        Some(c) => quote! { #args
        decimal_places: #c, },
        None => quote! { #args },
    };
    // 设置字段长度
    let args = match max_length {
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
        field_type = field_type,
        snake_field_name = snake_field_name,
        args = args
    );
    FieldToken {
        name: name.clone().unwrap_or(snake_field_name),
        token_stream: code.parse().unwrap(),
    }
}
