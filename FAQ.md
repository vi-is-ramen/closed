# Why MSRV is 1.88?

*   proc_macro_span_location (1.88.0): `Span::line()` and `Span::column()` used in
    `crates/macros/src/`;
*   proc_macro_span_file (1.88.0): `Span::file()` used in `crates/macros/src/`;
*   let_chains (1.88.0): `if let ... && let ...` used in `crates/closed/src/`.

# Why `std` feature if `closed` doesn't use `std` at all?

Because it can change in future, and users who need `std` integration and have
outdated `Cargo.toml` with `features = ["macros"]` would be surprised. Now, if
you write `features = ["std", "macros"]`, you allow `closed` to depend on `std`,
even if it isn't required now. In other words: removing `std` feature you
guarantee that `closed` will not depend on `std` crate. Enabling  `std` feature
you allow `closed` to depend on `std` crate.

# Why `closed-proc-macros` contains weird and unused macros and doesn't gate 'em?

Because `closed-proc-macro` designed as implementation detail of `closed`.
Nobody shouldn't depend on it directly. If you need `closed` macros, depend on
`closed` with `macros` feature (`on` by default).
