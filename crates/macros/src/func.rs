use crate::util::closed;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    Ident, Lifetime, Token, Type,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

struct FuncInput {
    args: Vec<Type>,
    output: Option<TokenStream2>,
    lifetime: Option<Lifetime>,
    mode: Option<Mode>,
    is_async: bool,
}

enum Mode {
    Once,
    Mut,
}

struct ValueInput {
    is_async: bool,
    params: Vec<(Ident, Type)>,
    output: Option<Type>,
    body: TokenStream2,
}

impl Parse for ValueInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let span = input.span();
        let _closed = closed(span)?;

        let mut is_async = false;

        // Parse the `async` and `move` modifiers.
        // We use `Token!` directly so that reserved keywords like `async` and
        // `move` are accepted even though they cannot be ordinary identifiers.
        if input.peek(Token![async]) {
            input.parse::<Token![async]>()?;
            is_async = true;

            // `async` requires `move` — without it the generated future would
            // be unable to outlive the closure's call, which contradicts the
            // `'static` guarantee of `PromisePin`.
            if input.peek(Token![move]) {
                input.parse::<Token![move]>()?;
            } else if input.peek(Token![|]) {
                return Err(syn::Error::new(
                    span,
                    "async closures must use the `move` keyword, e.g. \
                     `func!(async move |x: i32| x + 1)`",
                ));
            } else {
                return Err(syn::Error::new(
                    span,
                    "expected `move` or `|` after `async`",
                ));
            }
        } else if input.peek(Token![move]) {
            input.parse::<Token![move]>()?;
        } else if input.is_empty() {
            return Err(syn::Error::new(
                span,
                "unexpected end of input; expected `async`, `move` or `|`",
            ));
        }

        if !input.peek(Token![|]) {
            return Err(syn::Error::new(
                span,
                "expected `|` to start the closure parameter list",
            ));
        }
        input.parse::<Token![|]>()?;

        let mut params = Vec::new();
        while !input.is_empty()
            && !input.peek(Token![|])
            && !input.peek(Token![->])
        {
            let name: Ident = input.parse()?;
            if !input.peek(Token![:]) {
                return Err(syn::Error::new(
                    name.span(),
                    format!(
                        "expected `:` after parameter `{}` (parameter types \
                         are required in `func!`)",
                        name
                    ),
                ));
            }
            input.parse::<Token![:]>()?;
            let ty: Type = input.parse().map_err(|e| {
                syn::Error::new(
                    e.span(),
                    format!(
                        "failed to parse type for parameter `{}`: {}",
                        name, e
                    ),
                )
            })?;
            params.push((name, ty));

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            } else {
                break;
            }
        }

        let output = if input.peek(Token![->]) {
            input.parse::<Token![->]>()?;
            if input.peek(Token![!]) {
                input.parse::<Token![!]>()?;
                let c = closed(span)?;
                Some(syn::parse_quote!(::#c::Never))
            } else if input.peek(Token![|]) || input.is_empty() {
                return Err(syn::Error::new(
                    span,
                    "expected a return type after `->`",
                ));
            } else {
                Some(input.parse::<Type>().map_err(|e| {
                    syn::Error::new(
                        e.span(),
                        format!("failed to parse return type: {}", e),
                    )
                })?)
            }
        } else {
            None
        };

        if !input.peek(Token![|]) {
            return Err(syn::Error::new(
                span,
                "expected closing `|` to end the closure parameter list",
            ));
        }
        input.parse::<Token![|]>()?;

        if input.is_empty() {
            return Err(syn::Error::new(
                span,
                "expected a closure body after `| ... |`",
            ));
        }
        let body: TokenStream2 = input.parse()?;

        Ok(ValueInput {
            is_async,
            params,
            output,
            body,
        })
    }
}

impl Parse for FuncInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let span = input.span();
        let closed_ident = closed(span)?;

        let mut args = Vec::new();

        // `..` is shorthand for "no arguments".
        if input.peek(Token![..]) {
            input.parse::<Token![..]>()?;
            if !input.peek(Token![->])
                && !input.peek(Token![;])
                && !input.is_empty()
            {
                return Err(syn::Error::new(
                    span,
                    "expected `->`, `;` or end of input after `..`",
                ));
            }
        } else if !input.peek(Token![->])
            && !input.peek(Token![;])
            && !input.is_empty()
        {
            loop {
                let arg: Type = input.parse().map_err(|e| {
                    syn::Error::new(
                        e.span(),
                        format!("failed to parse argument type: {}", e),
                    )
                })?;
                args.push(arg);

                if input.peek(Token![,]) {
                    input.parse::<Token![,]>()?;
                    if input.peek(Token![->])
                        || input.peek(Token![;])
                        || input.is_empty()
                    {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        // Explicit lifetimes on arguments are not supported — the macro
        // always generates HRTB (`for<'a>`) bounds for references. Users who
        // need a specific lifetime should use the `; 'a` suffix to constrain
        // the `FuncRef` itself.
        for arg in &args {
            if let Type::Reference(r) = arg
                && let Some(lt) = &r.lifetime
                && lt.ident != "static"
            {
                return Err(syn::Error::new(
                    lt.span(),
                    format!(
                        "explicit lifetime `'{}' is not supported in \
                                 argument position; `func!` always generates \
                                 HRTB (`for<'a>`) bounds for references. \
                                 Use `; 'a` syntax after `->` to constrain \
                                 the callable's lifetime instead.",
                        lt.ident
                    ),
                ));
            }
        }

        let output = if input.peek(Token![->]) {
            input.parse::<Token![->]>()?;
            if input.peek(Token![!]) {
                input.parse::<Token![!]>()?;
                Some(quote!(::#closed_ident::Never))
            } else if input.peek(Token![;]) || input.is_empty() {
                return Err(syn::Error::new(
                    span,
                    "expected a return type after `->`",
                ));
            } else {
                let ty: Type = input.parse().map_err(|e| {
                    syn::Error::new(
                        e.span(),
                        format!("failed to parse return type: {}", e),
                    )
                })?;
                Some(quote!(#ty))
            }
        } else {
            None
        };

        let mut lifetime = None;
        let mut is_async = false;
        let mut is_once = false;
        let mut is_mut = false;

        if input.peek(Token![;]) {
            input.parse::<Token![;]>()?;

            if input.peek(Lifetime) {
                lifetime = Some(input.parse::<Lifetime>()?);
            }

            // Parse any combination of `once`, `mut`, `async` modifiers.
            // `once` is an ordinary identifier, but `mut` and `async` are
            // reserved keywords and must be matched as `Token!` directly.
            while !input.is_empty() {
                if input.peek(Token![mut]) {
                    input.parse::<Token![mut]>()?;
                    is_mut = true;
                } else if input.peek(Token![async]) {
                    input.parse::<Token![async]>()?;
                    is_async = true;
                } else if input.peek(Ident) {
                    let ident: Ident = input.parse()?;
                    if ident == "once" {
                        is_once = true;
                    } else {
                        return Err(syn::Error::new(
                            ident.span(),
                            format!(
                                "unexpected identifier `{}`; expected one of \
                                 `once`, `mut`, or `async`",
                                ident
                            ),
                        ));
                    }
                } else {
                    return Err(syn::Error::new(
                        span,
                        "expected a modifier (`once`, `mut`, or `async`) or \
                         end of input after `;`",
                    ));
                }
            }
        }

        if is_once && is_mut {
            return Err(syn::Error::new(
                span,
                "cannot combine `once` and `mut` — a callable cannot be both \
                 `FnOnce` and `FnMut`",
            ));
        }

        let mode = match (is_once, is_mut) {
            (true, false) => Some(Mode::Once),
            (false, true) => Some(Mode::Mut),
            _ => None,
        };

        Ok(FuncInput {
            args,
            output,
            lifetime,
            mode,
            is_async,
        })
    }
}

#[allow(clippy::large_enum_variant)]
enum FuncMacroInput {
    Type(FuncInput),
    Value(ValueInput),
}

impl Parse for FuncMacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            // Empty input → zero-arity sync type, equivalent to `func!()`.
            return Ok(FuncMacroInput::Type(FuncInput {
                args: Vec::new(),
                output: None,
                lifetime: None,
                mode: None,
                is_async: false,
            }));
        }

        let is_value = input.peek(Token![|])
            || input.peek(Token![async])
            || input.peek(Token![move]);

        if is_value {
            Ok(FuncMacroInput::Value(input.parse().map_err(|e| {
                syn::Error::new(
                    e.span(),
                    format!("failed to parse `func!` value expression: {}", e),
                )
            })?))
        } else {
            Ok(FuncMacroInput::Type(input.parse().map_err(|e| {
                syn::Error::new(
                    e.span(),
                    format!("failed to parse `func!` type signature: {}", e),
                )
            })?))
        }
    }
}

pub fn func(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as FuncMacroInput);

    let span = Span::call_site();
    let closed_ident = match closed(span) {
        Ok(id) => id,
        Err(e) => return e.to_compile_error().into(),
    };

    let result: syn::Result<TokenStream2> = match parsed {
        FuncMacroInput::Type(FuncInput {
            args,
            output,
            lifetime,
            mode,
            is_async,
        }) => {
            let base_output = output.unwrap_or_else(|| quote!(()));

            let output_ty = if is_async {
                let lifetime_arg = match lifetime {
                    Some(ref lt) => quote!(#lt),
                    None => quote!('static),
                };
                quote!(::#closed_ident::PromisePin<#lifetime_arg, #base_output>)
            } else {
                base_output
            };

            let type_name = match (lifetime.is_some(), mode) {
                (true, None) => quote!(FuncRef),
                (true, Some(Mode::Once)) => quote!(FuncOnceRef),
                (true, Some(Mode::Mut)) => quote!(FuncMutRef),
                (false, None) => quote!(Func),
                (false, Some(Mode::Once)) => quote!(FuncOnce),
                (false, Some(Mode::Mut)) => quote!(FuncMut),
            };

            let lifetime_arg = match lifetime {
                Some(lt) => quote!(#lt,),
                None => quote!(),
            };

            let has_ref =
                args.iter().any(|ty| matches!(ty, Type::Reference(_)));

            let args_ty = if args.is_empty() {
                quote!(())
            } else if has_ref {
                let markers: Vec<_> = args
                    .iter()
                    .map(|ty| marker_for_ty(ty, &closed_ident))
                    .collect();
                quote!(::#closed_ident::Lend<( #(#markers,)* )>)
            } else {
                quote!(( #(#args,)* ))
            };

            Ok(quote! {
                ::#closed_ident::#type_name<#lifetime_arg #args_ty, #output_ty>
            })
        }
        FuncMacroInput::Value(ValueInput {
            is_async,
            params,
            output,
            body,
        }) => {
            let params_pat = params.iter().map(|(name, ty)| quote!(#name: #ty));

            let closure_expr = if is_async {
                // Для async генерируем хелпер, который принудительно коэрцирует
                // конкретный async-блок в `PromisePin` (трейт-объект).
                // Это позволяет `.into()` успешно найти нужный `From` имплемент.
                let inner_body = match output {
                    Some(ref ty) => quote!({ let __res: #ty = #body; __res }),
                    None => quote!({ #body }),
                };
                quote! {
                    {
                        fn __cast_promise<'__a, __T, __Fut>(
                            f: __Fut
                        ) -> ::#closed_ident::PromisePin<'__a, __T>
                        where
                            __Fut: ::core::future::Future<Output = __T> + ::core::marker::Send + '__a,
                        {
                            ::#closed_ident::alloc::boxed::Box::pin(f)
                        }
                        move |#(#params_pat),*| __cast_promise(async move #inner_body)
                    }
                }
            } else {
                let move_kw = quote!(move);
                let body_expr = match output {
                    Some(ref ty) => quote!({ let __res: #ty = #body; __res }),
                    None => quote!({ #body }),
                };
                quote! {
                    #move_kw |#(#params_pat),*| {
                        #body_expr
                    }
                }
            };

            Ok(quote! {
                (#closure_expr).into()
            })
        }
    };

    match result {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn marker_for_ty(ty: &Type, closed: &Ident) -> TokenStream2 {
    match ty {
        Type::Reference(r) if r.lifetime.is_some() => {
            // Explicit lifetime: treat as `Own<&'a T>`. The explicit
            // lifetime has already been rejected for non-'static lifetimes
            // earlier in the parser.
            quote!(::#closed::Own<#ty>)
        }
        Type::Reference(r) => {
            let inner = &r.elem;
            if r.mutability.is_some() {
                quote!(::#closed::Mut<#inner>)
            } else {
                quote!(::#closed::Ref<#inner>)
            }
        }
        _ => quote!(::#closed::Own<#ty>),
    }
}
