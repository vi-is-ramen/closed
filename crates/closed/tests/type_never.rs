//! Type Mode Tests - Never Type

use closed::func;

#[test]
fn test_type_mode_never() {
    let _: func!(-> !) = (|| panic!("never returns")).into();
}

#[test]
fn test_type_mode_never_with_args() {
    let _: func!(bool -> !) = (|b| {
        if b {
            panic!("panic on true");
        } else {
            loop {
                std::hint::spin_loop();
            }
        }
    })
    .into();
}
