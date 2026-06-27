use proc_macro::TokenStream;
use quote::quote;
use syn::{Expr, ExprArray, LitStr, Token};
use syn::parse::{Parse, ParseStream};

pub(crate) struct MethodArgs {
    pub path: String,
    pub filters: Vec<Expr>,
}

impl Parse for MethodArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut path: String = String::new();
        let mut filters: Vec<Expr> = Vec::new();
        if input.peek(LitStr) {
            let lit: LitStr = input.parse()?;
            path = lit.value();

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        while !input.is_empty() {
            if input.peek(LitStr) {
                let lit: LitStr = input.parse()?;
                path = lit.value();
            } else {
                // Иначе мы ожидаем имя аргумента (идентификатор)
                let ident: syn::Ident = input.parse()?;
                input.parse::<Token![=]>()?; // "Съедаем" знак равенства

                if ident == "path" {
                    let lit: LitStr = input.parse()?;
                    path = lit.value();
                } else if ident == "filters" {
                    let array: ExprArray = input.parse()?;
                    filters = array.elems.into_iter().collect();
                } else {
                    return Err(syn::Error::new(ident.span(), "Разрешены только 'path' и 'filters'"));
                }
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        if path.is_empty() {
            return Err(syn::Error::new(input.span(), "Пропущен обязательный аргумент пути"));
        }

        Ok(MethodArgs { path, filters })
    }
}

pub(crate) fn method_impl(attr: TokenStream, item: TokenStream, method_path: syn::Path) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);

    let args = syn::parse_macro_input!(attr as MethodArgs);

    let fn_name = &input.sig.ident; // Sig - означает взять название функции

    let new_fn_name = quote::format_ident!("new_route_{}", fn_name);

    println!("{:}", new_fn_name);

    let path_str = args.path;
    let filters = args.filters;

    let method_name = &method_path.segments.last().unwrap().ident;


    let expanded = quote! {
        #input

        pub fn #new_fn_name() -> Route {
            println!("Регистрируем путь: {}", #path_str);

            Route::new(Method::#method_name, #path_str, Self::#fn_name)
        }
    };

    TokenStream::from(expanded)
}