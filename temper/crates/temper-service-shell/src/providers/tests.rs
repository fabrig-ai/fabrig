use super::*;
use crate::testing::provider_fixture as local;

#[test]
fn absent_and_unreadable_copies_fall_back_and_bad_copies_explain_why() {
    let scratch = skein_scratch::Scratch::new("providers-fallback");
    let path = scratch.path().join("providers.setting");
    let mut log = Vec::new();
    assert_eq!(read(&path, Box::new([local()]), &mut log).as_ref(), &[local()]);
    assert_eq!(log, [] as [u8; 0]);
    for bad in [false, true] {
        if bad {
            std::fs::write(&path, b"not JSON").unwrap();
        } else {
            std::fs::create_dir(&path).unwrap();
        }
        assert_eq!(read(&path, Box::new([local()]), &mut log).as_ref(), &[local()]);
        let line = String::from_utf8(core::mem::take(&mut log)).unwrap();
        assert!(line.starts_with("providers.setting: "), "{line}");
        assert!(line.ends_with("; the stock's providers run until the store's are kept again\n"), "{line}");
        if !bad {
            std::fs::remove_dir(&path).unwrap();
        }
    }
}
