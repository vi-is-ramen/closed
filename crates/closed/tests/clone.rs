//! Cloning Tests

use closed::func;

#[test]
fn test_func_clone() {
    let f1: func!(i32 -> i32) = (|x| x + 1).into();
    let f2 = f1.clone();

    assert_eq!(f1.invoke((5,)), 6);
    assert_eq!(f2.invoke((5,)), 6);
}

#[test]
fn test_func_clone_with_state() {
    let data = std::sync::Arc::new(std::sync::Mutex::new(0));
    let data_clone = data.clone();

    let f1: func!(i32 -> i32) = (move |x| {
        let mut guard = data_clone.lock().unwrap();
        *guard += x;
        *guard
    })
    .into();

    let f2 = f1.clone();

    assert_eq!(f1.invoke((5,)), 5);
    assert_eq!(f2.invoke((3,)), 8);
    assert_eq!(*data.lock().unwrap(), 8);
}
