use proc_macro::TokenStream;

mod func;
mod is_mut;
mod is_ref;
mod is_static;
mod position;
mod promise;
mod util;

/// Constructs a `Func`, `FuncMut`, or `FuncOnce` (or their `Ref` variants)
/// from an ergonomic function-like signature, or generates a closure value
/// directly if the input starts with `|`, `async`, or `move`.
#[proc_macro]
#[doc(hidden)]
pub fn func(input: TokenStream) -> TokenStream {
    func::func(input)
}

/// Wraps any expression or block into a `closed::PromisePin<'_, T>`.
///
/// # Examples
///
/// ```rust,ignore
/// use closed::{promise, PromisePin};
///
/// // From a block:
/// let p: PromisePin<'static, i32> = promise! {
///     let x = 42;
///     x
/// };
///
/// // From an async function call:
/// async fn compute() -> String { "ok".to_string() }
/// let p: PromisePin<'static, String> = promise! { compute().await };
/// ```
#[proc_macro]
#[doc(hidden)]
pub fn promise(input: TokenStream) -> TokenStream {
    promise::promise(input)
}

#[proc_macro]
#[doc(hidden)]
pub fn is_ref(input: TokenStream) -> TokenStream {
    is_ref::is_ref(input)
}

#[proc_macro]
#[doc(hidden)]
pub fn is_mut(input: TokenStream) -> TokenStream {
    is_mut::is_mut(input)
}

#[proc_macro]
#[doc(hidden)]
pub fn position(_: TokenStream) -> TokenStream {
    let pos = position::position();
    quote::quote! { #pos }.into()
}

#[proc_macro]
#[doc(hidden)]
pub fn is_static(input: TokenStream) -> TokenStream {
    is_static::is_static(input)
}
