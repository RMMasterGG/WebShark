use proc_macro::TokenStream;
use quote::quote;
use syn::ImplItem;
use crate::utils::parse::MethodArgs;

pub(crate) fn controller_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemImpl);

    // let args = syn::parse_macro_input!(attr as MethodArgs);
    //
    // let path_str = args.path;
    // let filters = args.filters;

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
            pub fn configure(scope_ref: &mut webshark::routing::scope::Scope) {
                let mut scope = std::mem::take(scope_ref);

                scope = scope
                    #( #all_function_names )*;

                *scope_ref = scope;
            }
        }
    };

    TokenStream::from(expended)
}