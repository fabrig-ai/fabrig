use super::{Directory, Expect, read_or_draw_with};
use skein_lib::Secret;
use skein_shell::Unread;
use std::{cell::RefCell, path::Path};

#[test]
fn host_id_is_drawn_once() {
    let directory = Directory::from_base("/fixture/installation".into()).expect("absolute credentials base");
    let path = Path::new("/fixture/installation/host-id");
    let stored: RefCell<Option<Box<[u8]>>> = RefCell::new(None);
    let read = |file: &Path| {
        assert_eq!(file, path);
        stored.borrow().clone().map(Secret::new).ok_or(Unread::Absent)
    };
    let first = read_or_draw_with(&directory, &[3; 32], read, |file, bytes, expected, entropy| {
        assert_eq!(file, path);
        assert_eq!(expected, Expect::Absent);
        assert_eq!(entropy, &[3; 32]);
        assert!(stored.replace(Some(bytes)).is_none(), "only the absent ID is published");
        Ok(true)
    })
    .expect("first host ID");
    let second = read_or_draw_with(&directory, &[4; 32], read, |_, _, _, _| panic!("existing ID was published again"))
        .expect("saved host ID");
    assert_eq!(first, "urn:uuid:03030303-0303-4303-8303-030303030303");
    assert_eq!(first, second);
}

#[test]
fn a_malformed_host_id_is_kept() {
    let stored = RefCell::new(b"not-a-uuid".to_vec().into_boxed_slice());
    assert!(
        read_or_draw_with(
            &Directory::from_base("/fixture/installation".into()).expect("absolute credentials base"),
            &[4; 32],
            |_| Ok(Secret::new(stored.borrow().clone())),
            |_, bytes, _, _| {
                drop(stored.replace(bytes));
                Ok(true)
            },
        )
        .is_err()
    );
    assert_eq!(stored.into_inner().as_ref(), b"not-a-uuid");
}
