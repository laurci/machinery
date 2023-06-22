use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn service(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as syn::ItemFn);

    // #[machinery::async_trait]

    let item = quote::quote! {
        #item
    };

    item.into()
}

#[proc_macro_attribute]
pub fn message(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as syn::Item);

    let name = match &item {
        syn::Item::Struct(item) => &item.ident,
        syn::Item::Enum(item) => &item.ident,
        _ => panic!("Only structs and enums can be messages"),
    };

    let item = quote::quote! {
        #[derive(machinery::Serialize, machinery::Deserialize, Debug)]
        #item
        impl From<#name> for machinery::Result<#name> {
            fn from(src: #name) -> machinery::Result<#name> {
                Ok(src)
            }
        }

    };

    item.into()
}

#[proc_macro]
pub fn load_services(_item: TokenStream) -> TokenStream {
    let item = quote::quote! {
        include!(concat!(env!("OUT_DIR"), "/machinery.rs"));
    };

    item.into()
}

#[proc_macro]
pub fn machinery(_item: TokenStream) -> TokenStream {
    let item = quote::quote! {
        machinery::Machinery::new(|fn_name, json_input| Box::pin(crate::__machinery::handle(fn_name, json_input)))
    };

    item.into()
}
