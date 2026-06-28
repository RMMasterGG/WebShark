use syn::parse::{Parse, ParseStream};
use syn::{Expr, ExprArray, LitStr, Token};

pub struct MethodArgs {
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
                    return Err(syn::Error::new(
                        ident.span(),
                        "Разрешены только 'path' и 'filters'",
                    ));
                }
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        if path.is_empty() {
            return Err(syn::Error::new(
                input.span(),
                "Пропущен обязательный аргумент пути",
            ));
        }

        Ok(MethodArgs { path, filters })
    }
}