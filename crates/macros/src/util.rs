use proc_macro_crate::FoundCrate;
use proc_macro2::Span;

/// Resolves the path to the `closed` crate as an identifier.
///
/// Returns a `syn::Error` instead of panicking when:
/// - The `closed` crate cannot be found in the dependency graph.
/// - The proc-macro crate resolution fails for any other reason.
pub fn closed(span: Span) -> syn::Result<syn::Ident> {
    let name = match proc_macro_crate::crate_name("closed") {
        // When the macro is invoked from inside the `closed` crate itself
        // (e.g. from its own tests or doc-tests), the `extern crate self as
        // closed;` declaration makes `closed::...` a valid path.
        Ok(FoundCrate::Itself) => "closed",
        Ok(FoundCrate::Name(name)) => &*Box::leak(name.into_boxed_str()),
        Err(e) => {
            return Err(syn::Error::new(
                span,
                format!(
                    "failed to resolve the `closed` crate dependency: {e}. \
                     Make sure `closed` is listed in your Cargo.toml."
                ),
            ));
        }
    };
    Ok(syn::Ident::new(name, span))
}

// pub fn random() -> usize {
//     use std::hash::Hasher;
//     let mut hasher = std::hash::DefaultHasher::new();
//     std::hash::Hash::hash(&std::time::SystemTime::now(), &mut hasher);
//     hasher.finish() as _
// }
