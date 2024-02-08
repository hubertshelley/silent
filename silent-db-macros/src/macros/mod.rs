use syn::{Attribute, LitInt, LitStr};

// #[cfg(feature = "mysql")]
// mod mysql;
pub(crate) struct TableAttr {
    pub(crate) name: Option<String>,
    pub(crate) comment: Option<String>,
}
pub(crate) fn get_table_attr(args: &Attribute) -> TableAttr {
    let mut table_attr = TableAttr {
        name: None,
        comment: None,
    };
    let attr_parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("name") {
            table_attr.name = Some(meta.value()?.parse::<LitStr>()?.value());
            Ok(())
        } else if meta.path.is_ident("comment") {
            table_attr.comment = Some(meta.value()?.parse::<LitStr>()?.value());
            Ok(())
        } else {
            Ok(())
        }
    });
    args.parse_args_with(attr_parser).unwrap();
    table_attr
}

#[derive(Debug)]
pub(crate) struct FieldAttr {
    pub(crate) field_type: String,
    pub(crate) name: Option<String>,
    pub(crate) default: Option<String>,
    pub(crate) nullable: Option<bool>,
    pub(crate) primary_key: Option<bool>,
    pub(crate) auto_increment: Option<bool>,
    pub(crate) unique: Option<bool>,
    pub(crate) comment: Option<String>,
    pub(crate) max_digits: Option<u8>,
    pub(crate) decimal_places: Option<u8>,
    pub(crate) length: Option<u16>,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn get_field_attr(args: &Attribute) -> FieldAttr {
    let mut field_attr = FieldAttr {
        field_type: "".to_string(),
        name: None,
        default: None,
        nullable: None,
        primary_key: None,
        auto_increment: None,
        unique: None,
        comment: None,
        max_digits: None,
        decimal_places: None,
        length: None,
    };
    let attr_parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("field_type") {
            field_attr.field_type = meta.value()?.parse::<LitStr>()?.value();
            Ok(())
        } else if meta.path.is_ident("name") {
            field_attr.name = Some(meta.value()?.parse::<LitStr>()?.value());
            Ok(())
        } else if meta.path.is_ident("default") {
            field_attr.default = Some(meta.value()?.parse::<LitStr>()?.value());
            Ok(())
        } else if meta.path.is_ident("nullable") {
            field_attr.nullable = Some(true);
            Ok(())
        } else if meta.path.is_ident("primary_key") {
            field_attr.primary_key = Some(true);
            Ok(())
        } else if meta.path.is_ident("auto_increment") {
            field_attr.auto_increment = Some(true);
            Ok(())
        } else if meta.path.is_ident("unique") {
            field_attr.unique = Some(true);
            Ok(())
        } else if meta.path.is_ident("comment") {
            field_attr.comment = Some(meta.value()?.parse::<LitStr>()?.value());
            Ok(())
        } else if meta.path.is_ident("max_digits") {
            field_attr.max_digits = Some(meta.value()?.parse::<LitInt>()?.base10_parse::<u8>()?);
            Ok(())
        } else if meta.path.is_ident("decimal_places") {
            field_attr.decimal_places =
                Some(meta.value()?.parse::<LitInt>()?.base10_parse::<u8>()?);
            Ok(())
        } else if meta.path.is_ident("max_length") {
            field_attr.length = Some(meta.value()?.parse::<LitInt>()?.base10_parse::<u16>()?);
            Ok(())
        } else {
            println!("meta.path: {:?}", meta.path);
            println!("meta.value: {:?}", meta.value()?);
            Ok(())
        }
    });
    args.parse_args_with(attr_parser)
        .map_err(|e| {
            println!("parse_args_with error: {:?}", e);
            e
        })
        .unwrap();
    field_attr
}
