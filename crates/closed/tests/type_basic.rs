//! Type Mode Tests - Basic

use closed::func;

#[test]
fn test_type_mode_no_args() {
    let _: func!() = (|| {}).into();
    let _: func!(..) = (|| {}).into();
}

#[test]
fn test_type_mode_no_args_with_return() {
    let _: func!(-> i32) = (|| 42).into();
    let _: func!(.. -> String) = (|| String::from("hello")).into();
}

#[test]
fn test_type_mode_single_arg() {
    let _: func!(i32 -> i32) = (|x| x + 1).into();
    let _: func!(String -> usize) = (|s: String| s.len()).into();
}

#[test]
fn test_type_mode_multiple_args() {
    let _: func!(i32, i32 -> i32) = (|a, b| a + b).into();
    let _: func!(i32, i32, i32 -> i32) = (|a, b, c| a + b + c).into();
    let _: func!(String, usize -> String) =
        (|s: String, n| s[..n].to_string()).into();
}

#[test]
fn test_type_mode_no_return() {
    let _: func!(i32) = (|_| {}).into();
    let _: func!(i32, i32) = (|_, _| {}).into();
}
