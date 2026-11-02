//! Type Mode Tests - Modifiers (mut, once)

use closed::func;

#[test]
fn test_type_mode_mut() {
    let mut counter = 0;
    let _: func!(-> i32; mut) = (move || {
        counter += 1;
        counter
    })
    .into();
}

#[test]
fn test_type_mode_once() {
    let data = String::from("consumed");
    let _: func!(-> String; once) = (move || data).into();
}

#[test]
fn test_type_mode_mut_with_args() {
    let mut state = 0;
    let _: func!(i32 -> i32; mut) = (move |x| {
        state += x;
        state
    })
    .into();
}

#[test]
fn test_type_mode_once_with_args() {
    let data = [1, 2, 3];
    let _: func!(usize -> i32; once) = (move |idx| data[idx]).into();
}
