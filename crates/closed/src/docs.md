# `closed`: user guide

This is the conceptual guide. For installation, quick start, and a feature
overview, see the [crate page](https://crates.io/crates/closed). For the
API reference, see the module docs.

The guide assumes you know what `Fn`, `FnMut`, and `FnOnce` are, and that you
have written at least one `Box<dyn Fn(...)>` in your life.

## 1. The problem: viral generics

If you want to store a closure in a struct, Rust gives you three options,
all bad:

```rust,ignore
// Option A: generic parameter.
struct Handler<F: Fn(i32) -> i32> {
    cb: F,
}
// Now Handler is generic. Every function that touches Handler becomes
// generic. Every struct that contains Handler becomes generic. The
// parameter spreads through your entire API.

// Option B: boxed trait object.
struct Handler {
    cb: Box<dyn Fn(i32) -> i32>,
}
// Not Clone. If you need Clone, you need Arc<dyn Fn>, which is another
// layer of indirection, and now you have to think about thread-safety
// attributes at every call site.

// Option C: boxed + arc + send + sync.
struct Handler {
    cb: Arc<dyn Fn(i32) -> i32 + Send + Sync>,
}
// Now it's Clone. But the type is 60 characters, the Send + Sync
// bounds leak into every signature that mentions the field, and async
// is still impossible: an async closure returns an anonymous future,
// which you cannot name in a Box<dyn ...> type without Pin<Box<dyn
// Future + Send>> everywhere.
```

And when your closure takes references (`&Event`, `&mut World`), the type
becomes genuinely hostile:

```rust,ignore
let cb: Arc<dyn for<'a, 'b> Fn(&'a Event, &'b mut World) + Send + Sync>
    = Arc::new(|e, w| { /* ... */ });
```

The `for<'a, 'b>` is a *higher-ranked trait bound*. It says "this closure
must be callable with any lifetimes the caller picks." It's easy to write
once and then easy to forget when you refactor. And it doesn't work with
`async` at all — see [5. Async deep dive](#5--Async-deep-dive).

## 2. The solution: one concrete type

`closed` gives you a single concrete type per signature:

```rust
# use closed::Func;
struct Handler {
    cb: Func<(i32,), i32>,
}
```

`Func<(i32,), i32>` is a *newtype* for `Arc<dyn Fn(i32) -> i32 + Send + Sync>`.
Not a new type with runtime overhead: the same `Arc`, the same `dyn Fn`, the
same call path. You can see this by looking at the definition:

```rust,ignore
pub struct FuncRef<'a, Args: FnArgs + 'a, Out: 'a> {
    pub(crate) inner: Arc<Args::DynFn<'a, Out>>,
}
```

`Args::DynFn<'a, Out>` for `Args = (i32,)` and `Out = i32` resolves to
`dyn Fn(i32) -> i32 + Send + Sync + 'a`. So `FuncRef<'static, (i32,), i32>` is
`Arc<dyn Fn(i32) -> i32 + Send + Sync + 'static>`.

The `Args` and `Out` parameters exist only so that the *type* is parameterized;
at the value level, there is no extra indirection. `Func` is zero-cost.

`Func` is `Clone` because `Arc` is `Clone`. It is `Send` and `Sync` because the
`dyn Fn` bound includes `Send + Sync`. The bounds are baked into the type, not
into every signature that uses it.

## 3. What `func!` actually generates

You don't write `Func<(i32,), i32>` by hand; you write `func!(i32 -> i32)`:

```rust
# use closed::func;
# fn example() {
let f: func!(i32, i32 -> i32) = (|a, b| a + b).into();
# } // fn example
```

`func!(i32, i32 -> i32)` expands to `Func<(i32, i32), i32>`. The macro is pure
sugar: you could write the type by hand. But for reference types, the expansion
is more involved:

```rust,ignore
func!(&str -> usize)  // expands to  Func<Lend<(Ref<str>,)>, usize>
```

`Lend<Ref<str>>` is the mechanism that encodes "this argument is `&'x str` for
any `'x`." It's ugly, but it's contained inside the macro. You will see it in
error messages and in `cargo expand` output, but you never write it by hand.

The next section explains why `Lend<Ref<str>>` has to exist.

## 4. HRTB deep dive

This is the section that actually matters if you plan to use `closed` for
callbacks that take references. Skip it if all your callbacks take owned
arguments.

### 4.1 What is `for<'a>`?

Rust's `Fn` traits are parameterized by argument types. `Fn(A) -> R` is the
trait for "callable with one argument of type `A`." When `A` is a reference,
`A` includes a lifetime: `&'a T` is `A` for some specific `'a`.

If you write `Box<dyn Fn(&'a T) -> R>`, the `'a` is *fixed* by the scope in
which the type appears. The closure can only be called with `&'a T` for that
specific `'a`. That's rarely what you want — a callback should accept a
reference with any lifetime the caller chooses.

The fix is a *higher-ranked* trait bound:

```rust
# type T = ();
# type R = ();
# type Example =
Box<dyn for<'a> Fn(&'a T) -> R>
# ;
```

This says "this box contains a closure callable with `&'a T` for *any* `'a`."
The caller picks `'a` at each call site. This is what makes `Box<dyn Fn(&T)>`
work in ordinary Rust without you writing `for<'a>` — the compiler desugars it.

### 4.2 Why doesn't `Func<(&'a T,), R>` work?

`Func` is parameterized by a *tuple* of argument types, not by a callable
signature:

```rust,ignore
pub struct FuncRef<'a, Args: FnArgs + 'a, Out: 'a> {
    inner: Arc<Args::DynFn<'a, Out>>,
}
```

For owned arguments, `Args = (i32, i32)` works: the tuple elements are concrete
types. For references, `Args = (&'a str,)` compiles, but it means "a tuple
containing a reference with lifetime `'a`": the `'a` is a single fixed
lifetime, and `Func` can only be called with references that live exactly that
long. That's the opposite of what a callback needs.

You cannot write `Args = (for<'a> &'a str,)`. There is no such thing as a
tuple element that is universally quantified over a lifetime. Tuples are
*monomorphic*: every element has a single concrete type.

The HRTB lives on the *trait object*, not on the *tuple*. The trick is to
introduce a wrapper type that "remembers" it can be rebound per call:

```rust,ignore
pub struct Ref<T: ?Sized>(PhantomData<fn() -> *const T>);
```

`Ref<T>` is not a reference; it's a *marker* that says "this argument position
is `&'x T` for whatever `'x` the caller picks." A tuple of markers
`(Ref<str>, Ref<i32>)` is a *lending tuple* — it can be instantiated with fresh
lifetimes on each call.

### 4.3 `Lend`: rebinding per call

`Lend<(A, B)>` wraps a tuple of markers and instructs `FnArgs` to produce a
higher-ranked `dyn Fn`:

```rust,ignore
impl<A: Arg, B: Arg> FnArgs for Lend<(A, B)> {
    type DynFn<'f, Out: 'f> =
        dyn for<'x0, 'x1> Fn(A::Rebind<'x0>, B::Rebind<'x1>) -> Out
            + Send + Sync + 'f;
    //  ^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^
    //  universal over each lifetime
}
```

`Arg::Rebind<'x>` is the type-level function that turns a marker into the actual
type at lifetime `'x`:

| Marker | `Rebind<'x>` |
|---|---|
| `Ref<T>` | `&'x T` |
| `Mut<T>` | `&'x mut T` |
| `Own<T>` | `T` |

So `Lend<(Ref<str>, Mut<Vec<u8>>, Own<bool>)>` becomes
`for<'x0, 'x1, 'x2> Fn(&'x0 str, &'x1 mut Vec<u8>, bool) -> Out`. Three separate
universal lifetimes: each argument can be borrowed for a different duration.
This is exactly the `for<'a, 'b, 'c>` you'd write by hand, generated
automatically from the `func!` input.

### 4.4 Why `func!` is the right entry point

You can write `Func<Lend<(Ref<str>,)>, usize>` by hand, but it's not obvious
that `Ref<str>` means "reference with any lifetime," or that `Lend` is what
triggers the HRTB. `func!(&str -> usize)` is:

- Shorter.
- Obvious: `&str` in a function signature always means "reference with any
  lifetime," and `func!` follows the same convention.
- Forward-compatible: if Rust gets variadic generics and `closed` can drop
  `Lend`, `func!` will keep working with the same input.

### 4.5 The error messages

If you write `func!(&'a str -> usize)` with an explicit lifetime, the macro
rejects it:

```text
error: explicit lifetime `'a` is not supported in argument position;
       `func!` always generates HRTB (`for<'a>`) bounds for references.
       Use `; 'a` syntax after `->` to constrain the callable's lifetime.
```

The `; 'a` suffix means "the `FuncRef` itself has lifetime `'a`" — i.e., its
*captures* live for `'a`, but its *arguments* are still HRTB. This is what you
want if the callback borrows from a local variable:

```rust
# use closed::func;
# fn example() {
let data = vec![1, 2, 3];
let get: func!(usize -> i32; '_) = (|i| data[i]).into();
//       ^^^^^^^^^^^^^^^^^^^^^^^^^^
//       HRTB argument, capture borrows `data`
# } // fn example
```

The two lifetimes (capture and argument) are independent, and that's the point.

### 4.6 If you don't want HRTB

Sometimes you genuinely want a fixed argument lifetime. For example, if the
closure will only ever be called with references from a specific arena. In that
case, use `Func<(T,), U>` and pass `&'a T` as an owned type:

```rust,ignore
// The closure captures `&'a Arena` and is called with `&'a T`.
let f: Func<(&'a T,), U> = ...;
```

But this is rare. Most callbacks need HRTB, and `func!(&T -> U)` gives it.

## 5. Async deep dive

Async callbacks have an additional structural challenge: the return type of an
`async fn` (or `async` block) is an anonymous future that you cannot name in
`Box<dyn Future>`. The usual workaround is `Pin<Box<dyn Future + Send>>`:

```rust,ignore
pub type Promise<T> = Pin<Box<dyn Future<Output = T> + Send + 'static>>;
```

`closed` calls this `Promise<T>`. An async callable is just a callable whose
output is `Promise<T>`:

```rust,ignore
let f: func!(u32 -> String; async) = ...;
// f.invoke((42,)) : Promise<String>
```

Inside the closure, `promise!` wraps the block:

```rust
# use closed::{func, promise};
# fn example() {
let f: func!(u32 -> String; async) =
    (|id| promise! { format!("data-{id}") }).into();
# } // fn example
```

`promise!` expands to `Box::pin(async move { ... })`. It exists because writing
`Box::pin` by hand is easy to forget; the macro also pins to a helper function
that forces the returned future to unify with `Promise<T>`, which improves type
inference at call sites.

### 5.1 The `'static` requirement

Notice that `Promise<T>` is `'static`. This is not an arbitrary restriction: it
means the future cannot borrow from anything with a shorter lifetime. The
closure may return a future that owns its data, but not one that references
the closure's arguments — because the arguments may have shorter lifetimes
than the future.

Concretely:

```rust,ignore
// ✗ does not compile
let f: func!(&str -> usize; async) = (|s: &str| promise! { s.len() }).into();
```

The closure receives `s: &'x str` for arbitrary `'x`, but returns a future that
must be `'static`. The future captures `s`, so it can only be `'x`-bounded, not
`'static`. Contradiction.

The fix is to extract what you need *before* entering the async block:

```rust
# use closed::{func, promise};
# fn example() {
// compiles
let f: func!(&str -> usize; async) = (|s: &str| {
    let len = s.len();
    promise! { len }
}).into();
# } // fn example
```

`len` is a `usize`; `'static`, no borrow. The future captures `len` by value,
so it's `'static`.

This is a general rule for async closures: **the async block captures only
`'static` data**. If you need borrowed data inside the future, copy or clone it
into an owned form before `async move`/`promise!`.

#### Async closure pattern for `closed`

```rust
let closure: func(/* args */ -> /* rtype */; async)
    = func!(|args| /* sync part */ promise! { /* async part */ });
```

Note: `/* sync part */` executes at the moment of closure invocation, while
`/* async part */` executes at the moment of polling (`.await`). This means that
it is a bad idea to put something heavy in synchronous part of closure. The main
goal of it must be data transmission for the asynchronous part.

### 5.2 Why this is a fundamental limit, not a bug

The alternative would be `Promise<'a, T>` with a lifetime parameter tied to the
argument lifetime. But HRTB gives *each argument position its own lifetime*,
and the future's lifetime would have to depend on all of them simultaneously —
`Promise<'x0 + 'x1, T>`. That's expressible in type theory but not ergonomic in
Rust's syntax, and it would infect every `Func` type with a union of argument
lifetimes. The `'static` restriction is a pragmatic tradeoff.

For async callbacks with references, the pattern is: borrow before the future,
own inside the future. This works for 99% of use cases.

### 5.3 `FuncMut` and `FuncOnce` with async

`func!(-> i32; async mut)` and `func!(-> String; async once)` work the same way.
The mutable or consuming state mutation happens **before** the future is
created, in the synchronous part of the closure:

```rust
# use closed::{func, promise};
# fn example() {
let mut counter = 0;
let mut f: func!(-> i32; async mut) = (move || {
    counter += 1;
    let current = counter;
    promise! { current }
}).into();

// f.invoke_mut(()).await == 1
// f.invoke_mut(()).await == 2
# } // fn example
```

Mutating after `.await` doesn't work — the future is `'static` and cannot
capture `&mut counter`. This is a general pattern, not a `closed` quirk.

## 6. When to use `*Ref`

`FuncRef<'a, Args, Out>` is `Func<Args, Out>` with a lifetime parameter for the
*captures*. The `*Ref` family is used when the closure borrows from a local:

```rust
# use closed::func;
# fn example() {
let data = vec![1, 2, 3];
let get: func!(usize -> i32; '_) = (|i| data[i]).into();
// `get` borrows `data`. If `get` is dropped, `data` is unaffected; if
// `data` is dropped first, `get` cannot be used.
# } // fn example
```

For 95% of callbacks, captures are `'static` (they own their data), and `Func`
is the right type. `*Ref` exists to close the remaining 5% — callbacks that
borrow from an arena, a configuration, or a per-request context.

The `*Ref` variants are `FuncRef`, `FuncMutRef`, `FuncOnceRef`. `Func` is
defined as `FuncRef<'static, ...>`, and similarly for the others.

## 7. Design decisions and gotchas

**`!Send` closures are not supported.** All three types store `dyn Fn(...) +
Send`. If your closure captures `Rc` or a `!Send` future, you cannot store it
in `closed`. Use `Box<dyn Fn(...)>` directly.

**`Debug` is opaque.** `Func` and friends render as `"fn ref (..) -> .."` — the
closure's captures are not inspected. `dyn Fn` doesn't implement `Debug`, so
there is no way to inspect the inner closure. If you need to identify a
callback, wrap it in a struct with an explicit name field.

**Arity ≤ 8.** A function with more than 8 arguments is almost always a design
smell; the macro generates code for each arity up to 8, and higher arities
multiply the generated code. If you need more, reconsider the signature.

## 8. Migrating from `Arc<dyn Fn>`

Side-by-side translation:

| Before | After |
|---|---|
| `Arc<dyn Fn(A) -> R + Send + Sync>` | `Func<(A,), R>` or `func!(A -> R)` |
| `Box<dyn FnMut(A) -> R + Send>` | `FuncMut<(A,), R>` or `func!(A -> R; mut)` |
| `Box<dyn FnOnce(A) -> R + Send>` | `FuncOnce<(A,), R>` or `func!(A -> R; once)` |
| `Arc<dyn for<'a> Fn(&'a T) -> R + Send + Sync>` | `func!(&T -> R)` |
| `Arc<dyn Fn(A) -> Pin<Box<dyn Future<Output = R> + Send>> + Send + Sync>` | `func!(A -> R; async)` |

Common pitfalls during migration:

1. **`Clone` semantics are shared, not copied.** `Func` clones share the
   underlying closure. If the closure captures `Mutex<i32>`, both clones mutate
   the same `Mutex`. This is usually what you want, but if you need independent
   state, clone the closure's captures before wrapping.

2. **`.into()` instead of `::new(...)`.** `Func::new(f)` works for owned
   closures, but for `Lend` types the inference fails without a turbofish.
   `.into()` uses the annotation type to resolve the `From<F>` impl, which
   works for both. Prefer `(closure).into()`.

## 9. Further reading

- [Rust reference: higher-ranked trait bounds](https://doc.rust-lang.org/nomicon/hrtb.html)
- [Rust reference: `Fn` traits](https://doc.rust-lang.org/std/ops/trait.Fn.html)
- [`closed` API reference](https://docs.rs/closed)
- [Test suite](https://github.com/vi-is-ramen/closed/tree/main/crates/closed/tests)
