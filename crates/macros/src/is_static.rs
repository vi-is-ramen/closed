use crate::position::position;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::Ident;

pub fn is_static(input: TokenStream) -> TokenStream {
    // Reject anything that isn't a type up-front, for nicer error messages.
    let ty: syn::Type = syn::parse_macro_input!(input as syn::Type);
    let suffix = position();
    let span = Span::call_site();

    let marker = Ident::new(&("Marker".to_owned() + &suffix), span);
    let probe = Ident::new(&("Probe".to_owned() + &suffix), span);
    let fall = Ident::new(&("Fall".to_owned() + &suffix), span);
    let spec = Ident::new(&("Spec".to_owned() + &suffix), span);

    let expanded = quote! {{
        trait #marker {}
        impl<'a, A: ?Sized> #marker for A where A: 'static, 'a: 'static, 'static: 'a {}

        struct #probe<A: ?Sized>(::core::marker::PhantomData<A>);
        trait #fall {
            fn probe(&self) -> bool { false }
        }
        impl<A: ?Sized> #fall for #probe<A> {}

        trait #spec {
            fn probe(&self) -> bool { true }
        }
        impl<A: ?Sized + #marker> #spec for &#probe<A> {}

        let p = #probe::<#ty>(::core::marker::PhantomData);
        (&&p).probe()
    }};

    expanded.into()
}
