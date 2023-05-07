use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

#[proc_macro_attribute]
pub fn my_attribute(_: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let fields = match input.data {
        Data::Struct(s) => s.fields,
        _ => panic!("my_attribute can only be used with structs"),
    };

    let mut field_attributes = Vec::new();
    for field in fields.iter() {
        if let Some(ident) = &field.ident {
            let field_name = ident.to_string();
            let attrs = &field.attrs;

            let field_attribute = quote! {
                pub fn #field_name(&self) -> &'static str {
                    stringify!(#(#attrs)*)
                }
            };

            field_attributes.push(field_attribute);
        }
    }

    let output = quote! {
        impl #name {
            #( #field_attributes )*
        }
    };

    output.into()
}

#[proc_macro_attribute]
pub fn my_field_attribute(_: TokenStream, input: TokenStream) -> TokenStream {
    input
}
