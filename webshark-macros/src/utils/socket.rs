use proc_macro::TokenStream;
use quote::quote;
use crate::utils::parse::MethodArgs;

pub(crate) fn ws_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);

    let args = syn::parse_macro_input!(attr as MethodArgs);

    let fn_name = &input.sig.ident;

    let new_fn_name = quote::format_ident!("new_socket_{}", fn_name);

    let path_str = args.path;
    let filters = args.filters;

    let expended = quote! {
        #input

        pub fn #new_fn_name() -> webshark::routing::socket::Socket {
            println!("Регистрируем путь: {}", #path_str);

            webshark::routing::socket::Socket::new(#path_str, Self::#fn_name)
        }
    };

    TokenStream::from(expended)
}