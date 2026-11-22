use skein_scratch::Scratch as Files;
use smith_local_shell::{credentials::Directory, testing::read_or_draw};
use std::os::unix::fs::PermissionsExt;

#[test]
fn host_id_is_drawn_once() {
    let files = Files::new("smith-credentials");
    let directory = Directory::from_base(files.path().join("installation")).unwrap();
    let first = read_or_draw(&directory, &[3; 32]).unwrap();
    let second = read_or_draw(&directory, &[4; 32]).unwrap();
    assert_eq!(first, second);
    assert_eq!(first, "urn:uuid:03030303-0303-4303-8303-030303030303");
    assert_eq!(std::fs::metadata(directory.host_id()).unwrap().permissions().mode() & 0o777, 0o600);
}
