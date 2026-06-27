use proc_macro::TokenStream;
use crate::utils::method_impl;

mod utils;

#[proc_macro_attribute]
pub fn get(attr: TokenStream, item: TokenStream) -> TokenStream {
    method_impl(attr, item, syn::parse_quote!(Method::GET))
}

#[proc_macro_attribute]
pub fn post(attr: TokenStream, item: TokenStream) -> TokenStream {
    method_impl(attr, item, syn::parse_quote!(Method::POST))
}

#[proc_macro_attribute]
pub fn put(attr: TokenStream, item: TokenStream) -> TokenStream {
    method_impl(attr, item, syn::parse_quote!(Method::PUT))
}

#[proc_macro_attribute]
pub fn delete(attr: TokenStream, item: TokenStream) -> TokenStream {
    method_impl(attr, item, syn::parse_quote!(Method::DELETE))
}

#[proc_macro_attribute]
pub fn websocket(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn webtransport(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn controller(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn service(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn registry_controller(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}