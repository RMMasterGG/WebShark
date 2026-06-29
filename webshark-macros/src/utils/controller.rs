use proc_macro::TokenStream;
use quote::quote;
use syn::ImplItem;
use crate::utils::parse::MethodArgs;

pub(crate) fn controller_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemImpl);

    let args = syn::parse_macro_input!(attr as MethodArgs);

    let path_str = args.path;
    let filters = args.filters;

    let impl_name = &input.self_ty;

    let mut all_function_names = Vec::new();

    input.items.iter().for_each(|item| {
        if let ImplItem::Fn(item_fn) = item {
            let fn_name = &item_fn.sig.ident;

            for attr in &item_fn.attrs {
                if let Some(ident) = attr.path().get_ident() {
                    let attr_str = ident.to_string();
                    match attr_str.as_str() {
                        "get" | "post" | "put" | "delete" | "patch" => {
                            let new_fn_name = quote::format_ident!("new_route_{}", fn_name);

                            all_function_names.push(quote! {
                                .add_route(Self::#new_fn_name())
                            });
                        }
                        "websocket" => {
                            let new_fn_name = quote::format_ident!("new_socket_{}", fn_name);

                            all_function_names.push(quote! {
                                .add_websocket(Self::#new_fn_name())
                            });
                        }
                        "webtransport" => {
                            let new_fn_name = quote::format_ident!("new_socket_{}", fn_name);

                            all_function_names.push(quote! {
                                .add_webtransport(Self::#new_fn_name())
                            });
                        }
                        _ => {}
                    }
                }
            }
        }
    });

    let expended = quote! {
        #input

        impl #impl_name {
            pub fn scope() -> webshark::routing::scope::Scope {
                let mut scope = webshark::routing::scope::Scope::new(#path_str);

                #[inline(always)]
                fn __assert_is_filter<T: webshark::auth::authentication::Filter>() {}

               #(
                    __assert_is_filter::<#filters>();
                )*

                #(
                    scope = scope.with_filter(#filters {});
                )*

                scope = scope
                    #( #all_function_names )*;

                scope
            }
        }
    };

    TokenStream::from(expended)
}