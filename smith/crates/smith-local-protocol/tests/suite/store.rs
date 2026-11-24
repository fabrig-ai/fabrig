//! Real file-adapter proof of fresh-journal recovery and marker inode preservation.
//! smith/docs/design/testing.md, 2.5.

#[cfg(test)]
mod tests {
    use crate::disk::Disk;
    use smith_local_domain as local;
    use smith_local_protocol::{StoreRequest, format};
    use std::{fs, os::unix::fs::MetadataExt};

    fn state(activation: u64) -> local::ChatState {
        local::ChatState {
            identity: smith_domain::run::Identity::new([42; 16]),
            activation,
            next_message: 17,
            read: Some(skein_lib::Token::new(11)),
        }
    }

    fn old_records(disk: &Disk) {
        for name in ["state", "fresh"] {
            fs::write(disk.path().join(name), smith_local_protocol::save_state(state(9)).bytes).unwrap();
        }
        fs::write(disk.path().join("0000000001.turn"), b"old transcript bytes").unwrap();
        fs::write(disk.path().join("00000000000000000001.0000000001.0000000000"), b"old delivery bytes").unwrap();
    }

    #[test]
    fn a_matching_marker_permits_real_fresh_journal_recovery_and_keeps_the_marker() {
        let mut disk = Disk::new();
        old_records(&disk);
        fs::write(disk.path().join("format"), format::file(&format::layouts()).bytes).unwrap();
        let marker_before = fs::metadata(disk.path().join("format")).unwrap();
        let local::Event::Loaded { state: Some(loaded), transcript: None, deliveries } = disk.drive(StoreRequest::Load)
        else {
            panic!("fresh recovered");
        };
        assert_eq!(loaded, state(9));
        assert!(deliveries.is_empty());
        assert!(!disk.path().join("fresh").exists());
        assert!(!disk.path().join("0000000001.turn").exists());
        assert_eq!(fs::read(disk.path().join("format")).unwrap(), format::file(&format::layouts()).bytes.as_ref());
        assert_eq!(marker_before.ino(), fs::metadata(disk.path().join("format")).unwrap().ino());
        disk.close();
    }
}
