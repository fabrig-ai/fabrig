use jig_protocol_config as config;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use temper_service_shell::{
    providers::{read, write},
    testing::provider_fixture as local,
};

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
