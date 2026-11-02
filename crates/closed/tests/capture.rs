//! State Capture Tests

use closed::func;

#[test]
fn test_func_mut_state_capture() {
    let mut counter = 0;
    let mut f: func!(-> i32; mut) = (move || {
        counter += 1;
        counter
    })
    .into();

    assert_eq!(f.invoke_mut(()), 1);
    assert_eq!(f.invoke_mut(()), 2);
    assert_eq!(f.invoke_mut(()), 3);
}

#[test]
fn test_func_once_consumption() {
    let data = String::from("consumed");
    let len = data.len();

    let f: func!(-> String; once) = (move || data).into();
    let result = f.invoke_once(());

    assert_eq!(result, "consumed");
    assert_eq!(result.len(), len);
}
