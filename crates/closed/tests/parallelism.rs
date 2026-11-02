//! Thread Safety Tests

use closed::{Func, func};

#[test]
fn test_func_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<Func<(), i32>>();
    assert_send_sync::<Func<(i32, String), bool>>();
}

#[test]
fn test_func_across_threads() {
    use std::sync::{Arc, Mutex};
    use std::thread;

    let state = Arc::new(Mutex::new(0));
    let state_clone = state.clone();

    let f: func!(i32 -> ()) = (move |x| {
        let mut guard = state_clone.lock().unwrap();
        *guard += x;
    })
    .into();

    let f_clone = f.clone();

    let h1 = thread::spawn(move || f.invoke((10,)));
    let h2 = thread::spawn(move || f_clone.invoke((20,)));

    h1.join().unwrap();
    h2.join().unwrap();

    assert_eq!(*state.lock().unwrap(), 30);
}
