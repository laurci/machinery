use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn service(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as syn::ItemFn);
    let block = item.block;
    let vis = item.vis;
    let mut sig = item.sig;
    sig.inputs.insert(
        0,
        syn::parse_quote!(
            #[allow(unused_variables)]
            ctx: &machinery::context::Context
        ),
    );

    let item = quote::quote! {
        #vis #sig #block
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

#[proc_macro_attribute]
pub fn injectable(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as syn::ItemImpl);

    let item = quote::quote! {
        #[machinery::__internal_async::async_trait]
        #item
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
        machinery::Machinery::new(|ctx, fn_name, json_input| Box::pin(crate::__machinery::handle(ctx, fn_name, json_input)))
    };

    item.into()
}

#[proc_macro]
pub fn inject_async(item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as syn::Path);

    let item = quote::quote!(
        machinery::inject::__internal::inject_async::<#item>(ctx)
    );
    item.into()
}

#[proc_macro]
pub fn inject(item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as syn::Path);

    let item = quote::quote!(
        machinery::inject::__internal::inject::<#item>(ctx)
    );
    item.into()
}
