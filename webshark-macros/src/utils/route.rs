use proc_macro::TokenStream;
use quote::quote;
use crate::utils::parse::MethodArgs;

pub(crate) fn method_impl(
    attr: TokenStream,
    item: TokenStream,
    method_path: syn::Path,
) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);

    let args = syn::parse_macro_input!(attr as MethodArgs);

    let fn_name = &input.sig.ident; // Sig - означает взять название функции

    let new_fn_name = quote::format_ident!("new_route_{}", fn_name);

    let path_str = args.path;
    let filters = args.filters;

    let method_name = &method_path.segments.last().unwrap().ident;

    let expanded = quote! {
        #input

        pub fn #new_fn_name() -> webshark::routing::route::Route {
            println!("Регистрируем путь: {}", #path_str);

            webshark::routing::route::Route::new(webshark::http::Method::#method_name, #path_str, Self::#fn_name)
        }
    };

    TokenStream::from(expanded)
}