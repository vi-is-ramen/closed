//! Edge Cases

use closed::{Func, func};

#[test]
fn test_zero_arity() {
    let f: func!(-> _) = (|| 42).into();
    assert_eq!(f.invoke(()), 42);
}

#[test]
fn test_many_args() {
    let f: func!(i32, i32, i32, i32, i32, i32, i32, i32 -> i32) =
        (|a, b, c, d, e, f, g, h| a + b + c + d + e + f + g + h).into();

    assert_eq!(f.invoke((1, 2, 3, 4, 5, 6, 7, 8)), 36);
}

#[test]
fn test_complex_return_type() {
    let f: func!(i32 -> Result<String, String>) = (|x| {
        if x > 0 {
            Ok(format!("positive: {}", x))
        } else {
            Err("negative".to_string())
        }
    })
    .into();

    assert_eq!(f.invoke((5,)), Ok("positive: 5".to_string()));
    assert_eq!(f.invoke((-1,)), Err("negative".to_string()));
}

#[test]
fn test_nested_closures() {
    let outer: func!(i32 -> Func<(i32,), i32>) =
        (|x| (move |y| x + y).into()).into();

    let inner = outer.invoke((10,));
    assert_eq!(inner.invoke((5,)), 15);
}

#[test]
fn test_closure_captures_reference() {
    static DATA: [i32; 3] = [1, 2, 3];
    let f: func!(usize -> i32) = (|idx: usize| DATA[idx]).into();

    assert_eq!(f.invoke((1,)), 2);
}

#[test]
fn test_mixed_owned_and_ref() {
    let f: func!(String, &str -> String) =
        (|owned: String, borrowed: &str| format!("{} + {}", owned, borrowed))
            .into();

    let result = f.invoke(("hello".to_string(), "world"));
    assert_eq!(result, "hello + world");
}
