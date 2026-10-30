# closed-proc-macros

**Internal procedural macros for the
[`closed`](https://crates.io/crates/closed) crate.**

This crate is an implementation detail of `closed`. It is **not
intended for direct use**, and it will **never** have a stable
public API. Macros may be renamed, re-signed, or removed at any
time, without a semver bump or changelog entry.

If you are looking for type-erased callable objects for Rust,
use [`closed`](https://crates.io/crates/closed) directly. If you
have found this crate through a dependency tree and are considering
calling into it, please don't. Nothing here is designed to be
consumed from outside the `closed` crate.

## License

Dual-licensed under
[Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0) or
[MIT](https://opensource.org/licenses/MIT), at your option.
