use proc_macro::TokenStream;
use quote::quote;
use syn::parse_quote;

pub(crate) fn main_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = syn::parse_macro_input!(item as syn::ItemFn);

    let tokio_attr: syn::Attribute = parse_quote! {
        #[::webshark::tokio::main(crate = "::webshark::tokio")]
    };

    input.attrs.insert(0, tokio_attr);

    let expanded = quote! {
        #input
    };

    TokenStream::from(expanded)
}