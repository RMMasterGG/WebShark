mod utils;

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, ItemImpl, ItemStruct};
use crate::utils::controller::controller_impl;
use crate::utils::main::main_impl;
use crate::utils::route::method_impl;
use crate::utils::socket::ws_impl;

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
pub fn websocket(attr: TokenStream, item: TokenStream) -> TokenStream {
    ws_impl(attr, item)
}

#[proc_macro_attribute]
pub fn webtransport(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn controller(attr: TokenStream, item: TokenStream) -> TokenStream {
    controller_impl(attr, item)
}

#[proc_macro_attribute]
pub fn main(attr: TokenStream, item: TokenStream) -> TokenStream {
    main_impl(attr, item)
}

#[proc_macro_attribute]
pub fn config(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = syn::parse_macro_input!(item as ItemStruct);

    let expended = quote!(
        #item
    );

    TokenStream::from(expended)
}

#[proc_macro_attribute]
pub fn config_file(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = syn::parse_macro_input!(item as ItemStruct);

    let expended = quote!(
        #item
    );

    TokenStream::from(expended)
}

#[proc_macro_attribute]
pub fn provider(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = syn::parse_macro_input!(item as ItemImpl);

    let expended = quote!(
        #item
    );

    TokenStream::from(expended)
}

#[proc_macro_attribute]
pub fn bean(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = syn::parse_macro_input!(item as ItemFn);

    let expended = quote!(
        #item
    );

    TokenStream::from(expended)
}

#[proc_macro_attribute]
pub fn registry_controller(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}