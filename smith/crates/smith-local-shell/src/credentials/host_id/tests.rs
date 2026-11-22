use super::read_or_draw_with;
use skein_lib::Secret;
use std::cell::RefCell;

#[test]
fn a_malformed_host_id_is_kept() {
    let stored = RefCell::new(b"not-a-uuid".to_vec().into_boxed_slice());
    assert!(
        read_or_draw_with(
            &[4; 32],
            || Ok(Secret::new(stored.borrow().clone())),
            |bytes| {
                drop(stored.replace(bytes));
                Ok(())
            },
        )
        .is_err()
    );
    assert_eq!(stored.into_inner().as_ref(), b"not-a-uuid");
}
