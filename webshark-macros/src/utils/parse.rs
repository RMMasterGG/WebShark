use syn::parse::{Parse, ParseStream};
use syn::{Expr, ExprArray, LitStr, Path, Token};

pub struct MethodArgs {
    pub path: String,
    pub filters: Vec<Path>,
}

impl Parse for MethodArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut path: String = String::new();
        let mut filters: Vec<Path> = Vec::new();
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
                    for element in array.elems {
                        if let Expr::Path(expr_path) = element {
                            filters.push(expr_path.path);
                        } else {
                            return Err(syn::Error::new_spanned(
                                element,
                                "Ошибка WebShark: Внутри массива filters должны быть только имена структур-фильтров!",
                            ));
                        }
                    }

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