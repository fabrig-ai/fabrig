use skein_fake_machine::{How, Is};
use temper_one_process_world::{testing::USER, tools};

#[test]
fn ctl_sign_in_creates_private_files_for_its_user() {
    let mut disk = tools::signed_in_disk();
    let root = disk.reopen_root(2);
    for (path, kind, mode) in [
        (b"private".as_slice(), Is::Directory, 0o700),
        (b"private/token", Is::File, 0o600),
        (b"private/address", Is::File, 0o600),
    ] {
        let opened = disk.open(root, path, How::PathNoFollow).expect("signed-in entry");
        let facts = disk.stat(opened);
        assert_eq!(
            (facts.is, facts.mode & 0o777, facts.owner),
            (kind, mode, USER),
            "{}",
            String::from_utf8_lossy(path)
        );
        disk.close(opened);
    }
    disk.close(root);
}

#[test]
fn two_ctl_end_waits_stay_open_while_me_answers() {
    tools::waits();
}
