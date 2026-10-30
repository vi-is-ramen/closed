//! Procedural macros for the [`closed`] crate.
//!
//! This crate is an implementation detail and is not intended for direct
//! use by downstream code.

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{LitInt, Token, Type, parse_macro_input};

/// Parsed form of `A, B, C -> O`.
struct CallableInput {
    args: Vec<Type>,
    output: Type,
}

impl syn::parse::Parse for CallableInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut args = Vec::new();
        while !input.peek(Token![->]) && !input.is_empty() {
            args.push(input.parse::<Type>()?);
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            } else if !input.peek(Token![->]) {
                return Err(input.error("expected `,` or `->`"));
            }
        }
        input.parse::<Token![->]>()?;
        let output = input.parse::<Type>()?;
        Ok(CallableInput { args, output })
    }
}

/// Construct a `Callable<...>` bound with a pinned `Output` from an
/// ergonomic signature.
///
/// # Syntax
///
/// ```text
/// Callable!(Arg1, Arg2, ... -> Output)
/// Callable!(Arg1 -> Output)
/// Callable!(-> Output)          // nullary
/// ```
///
/// # Expansion
///
/// - `Callable!(A -> B)` → `Callable<(A,), Output = B>`
/// - `Callable!(A, C -> B)` → `Callable<(A, C), Output = B>`
/// - `Callable!(-> B)` → `Callable<(), Output = B>`
#[proc_macro]
pub fn callable(input: TokenStream) -> TokenStream {
    let CallableInput { args, output } =
        parse_macro_input!(input as CallableInput);
    let expanded = quote! {
        ::closed::Callable<( #(#args,)* ), Output = #output>
    };
    expanded.into()
}

/// Construct a `Function<...>` type from an ergonomic signature.
///
/// # Syntax
///
/// ```text
/// Function!(Arg1, Arg2, ... -> Output)
/// Function!(Arg1 -> Output)
/// Function!(-> Output)          // nullary
/// ```
///
/// # Expansion
///
/// - `Function!(A -> B)` → `Function<(A,), B>`
/// - `Function!(A, C -> B)` → `Function<(A, C), B>`
/// - `Function!(-> B)` → `Function<(), B>`
///
/// # Example
///
/// ```rust,ignore
/// use closed::{function, Function};
///
/// let f: Function!(i32, i32 -> i32) = Function::new(|a: i32, b: i32| a + b);
/// assert_eq!(f.invoke((3, 4)), 7);
/// ```
#[proc_macro]
pub fn function(input: TokenStream) -> TokenStream {
    let CallableInput { args, output } =
        parse_macro_input!(input as CallableInput);
    let expanded = quote! {
        ::closed::Function<( #(#args,)* ), #output>
    };
    expanded.into()
}

/// Generate blanket implementations of `Callable` for `Fn` closures and
/// function pointers of arities `0..=N`.
///
/// The input must be an integer literal specifying the maximum arity. The
/// generated implementations are suitable for inclusion in a module where
/// `Callable` is in scope (i.e. `use crate::traits::Callable;`).
///
/// # Example
///
/// ```rust,ignore
/// closed_proc_macros::gen_impl_callable!(12);
/// ```
///
/// For each arity `N`, the generated impl takes the form:
///
/// ```rust,ignore
/// impl<F, A0, A1, ..., AN, O> Callable<(A0, A1, ..., AN)> for F
/// where
///     F: Fn(A0, A1, ..., AN) -> O,
/// {
///     type Output = O;
///     fn invoke(&self, (A0, A1, ..., AN): (A0, A1, ..., AN)) -> O {
///         (self)(A0, A1, ..., AN)
///     }
/// }
/// ```
///
/// The arity-0 case is emitted separately because its argument tuple is
/// `()` and the closure takes no parameters.
#[proc_macro]
pub fn gen_impl_callable(input: TokenStream) -> TokenStream {
    let lit = parse_macro_input!(input as LitInt);
    let max_arity: usize =
        lit.base10_parse().expect("expected integer literal");

    let impls = (0..=max_arity).map(|arity| {
        let params: Vec<_> = (0..arity)
            .map(|i| syn::Ident::new(&format!("A{i}"), Span::call_site()))
            .collect();

        if arity == 0 {
            quote! {
                impl<F, O> Callable<()> for F
                where F: Fn() -> O,
                {
                    type Output = O;
                    fn invoke(&self, _: ()) -> O { (self)() }
                }
            }
        } else {
            quote! {
                impl<F, #(#params,)* O> Callable<(#(#params,)*)> for F
                where F: Fn(#(#params),*) -> O,
                {
                    type Output = O;
                    #[allow(non_snake_case)]
                    fn invoke(&self, (#(#params,)*): (#(#params,)*)) -> O {
                        (self)(#(#params),*)
                    }
                }
            }
        }
    });

    let expanded = quote! { #(#impls)* };
    expanded.into()
}
