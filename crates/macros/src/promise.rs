use crate::util::closed;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;

pub fn promise(input: TokenStream) -> TokenStream {
    let span = Span::call_site();
    let closed_ident = match closed(span) {
        Ok(id) => id,
        Err(e) => return e.to_compile_error().into(),
    };

    let expr: TokenStream2 = input.into();

    let expanded = quote! {
        {
            #[inline(always)]
            fn __cast_promise<'__a, __T, __Fut>(
                f: __Fut,
            ) -> ::#closed_ident::PromisePin<'__a, __T>
            where
                __Fut: ::core::future::Future<Output = __T>
                    + ::core::marker::Send
                    + '__a,
            {
                ::#closed_ident::alloc::boxed::Box::pin(f)
            }
            __cast_promise(async move { #expr })
        }
    };

    expanded.into()
}
