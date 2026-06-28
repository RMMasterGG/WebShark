mod utils;

use proc_macro::TokenStream;
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
pub fn registry_controller(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}