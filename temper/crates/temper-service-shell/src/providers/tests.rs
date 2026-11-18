use super::*;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
fn local() -> ProviderSetting {
    let document = config::document(br#"{"number":4,"name":"local","endpoint":"local","signed_in_plan":false,"sources":{},"endpoints":[{"name":"local","number":5,"dialect":3,"address":{"socket":"127.0.0.1:8080"},"transport":"plaintext","headers":[],"connect_ms":10000,"handshake_ms":10000}]}"#, &config::DocumentLimits { bytes: 4096, tokens: 256, depth: 16 }).unwrap();
    let mut problems = config::Problems::empty();
    config::llm::provider_setting(&document, 0, &config::Path::root(), &mut problems).unwrap()
}
#[test]
fn restore_and_changes_are_whole_private_and_unchanged_bytes_are_not_written() {
    let scratch = skein_scratch::Scratch::new("kept-providers");
    let path = scratch.path().join("providers.setting");
    let values = [local()];
    assert!(write(&path, &values).unwrap(), "restore writes an absent copy");
    assert_eq!(std::fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
    assert_eq!(read(&path, Box::new([]), &mut Vec::new()).as_ref(), &values);
    let before = std::fs::metadata(&path).unwrap();
    assert!(!write(&path, &values).unwrap());
    assert_eq!(std::fs::metadata(&path).unwrap().ino(), before.ino(), "no rename of equal bytes");
    let mut changed = values[0].clone();
    changed.endpoints[0].address = config::EndpointAddress::Socket("127.0.0.1:8081".parse().unwrap());
    assert!(write(&path, &[changed.clone()]).unwrap());
    assert_eq!(read(&path, Box::new([]), &mut Vec::new()).as_ref(), &[changed]);
    assert!(!path.with_extension("setting.new").exists());
}
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
