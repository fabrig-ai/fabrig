//! The lowest real-directory proof of compatibility before recovery or mutation.
use crate::disk;
#[cfg(test)]
mod tests {
    use super::disk;
    use disk::{Disk, Kind, snapshot};
    use smith_local_domain::{self as local, Older, StoreFamily};
    use smith_local_protocol::{History, StoreRequest, format, local_store_v1 as wire};
    use std::{fs, os::unix::fs::MetadataExt};

    fn state(activation: u64) -> local::ChatState {
        local::ChatState {
            identity: smith_domain::run::Identity::new([42; 16]),
            activation,
            next_message: 17,
            read: Some(skein_lib::Token::new(11)),
        }
    }
    fn empty(event: local::Event) {
        let local::Event::Loaded { state: None, transcript: None, deliveries } = event else {
            panic!("empty Load: {event:?}");
        };
        assert!(deliveries.is_empty());
    }
    fn marker(store: bool, transcript: bool, position: usize) -> Box<[u8]> {
        let mut parts = format::layouts().into_parts();
        if store {
            parts.store[position] ^= 1;
        }
        if transcript {
            parts.transcript[position] ^= 1;
        }
        let value = wire::Layouts::new(&wire::CEILINGS, parts).unwrap();
        let mut writer = skein_lib::Writer::new(usize::try_from(value.measure()).expect("measured bytes fit usize"));
        value.encode(&mut writer).unwrap();
        writer.finish()
    }
    fn refused(disk: &mut Disk, why: Option<Older>) {
        let before = snapshot(disk.path());
        disk.clear_requests();
        let event = disk.drive(StoreRequest::Load);
        let local::Event::StoreFailed { turn: None, reason } = event else {
            panic!("refusal: {event:?}");
        };
        let expected = match why {
            Some(why) => local::StoreFailure::OlderBuild { why },
            None => local::StoreFailure::Read,
        };
        assert_eq!(reason, expected);
        let requests: &[Kind] = match why {
            Some(Older::Unrecorded) => &[Kind::Load, Kind::Scan],
            Some(Older::Layout(_)) | None => &[Kind::Load],
        };
        assert_eq!(disk.requests(), requests);
        assert_eq!(before, snapshot(disk.path()));
        assert!(
            disk.requests().iter().all(|kind| matches!(kind, Kind::Load | Kind::Scan | Kind::Close)),
            "{:?}",
            disk.requests()
        );
        disk.close();
        assert_eq!(before, snapshot(disk.path()));
    }
    fn compatible(path: &std::path::Path) {
        fs::write(path.join("format"), format::file().bytes).unwrap();
    }

    fn write_state(path: &std::path::Path, value: local::ChatState) {
        let file = smith_local_protocol::save_state(value);
        fs::write(path.join("state"), file.bytes).unwrap();
    }

    fn old_records(disk: &Disk) {
        for name in ["state", "fresh"] {
            fs::write(disk.path().join(name), smith_local_protocol::save_state(state(9)).bytes).unwrap();
        }
        fs::write(disk.path().join("0000000001.turn"), b"old transcript bytes").unwrap();
        fs::write(disk.path().join("00000000000000000001.0000000001.0000000000"), b"old delivery bytes").unwrap();
    }

    #[test]
    fn empty_create_is_durable_and_a_new_store_reopens_without_mutation() {
        let mut disk = Disk::new();
        empty(disk.drive(StoreRequest::Load));
        assert_eq!(disk.requests(), [Kind::Load, Kind::Scan, Kind::Store]);
        assert_eq!(fs::read(disk.path().join("format")).unwrap(), format::file().bytes.as_ref());
        disk.reopen();
        let before = snapshot(disk.path());
        empty(disk.drive(StoreRequest::Load));
        assert!(disk.requests().iter().all(|kind| matches!(kind, Kind::Load | Kind::Scan)));
        assert_eq!(before, snapshot(disk.path()));
        disk.clear_requests();
        empty(disk.drive(StoreRequest::Load));
        // The same owned root skips only the marker load.
        assert_eq!(disk.requests(), [Kind::Load, Kind::Load, Kind::Scan]);
        disk.close();
    }

    #[test]
    fn every_unrecorded_old_record_is_refused_before_any_mutation() {
        for name in [
            "state",
            "fresh",
            "0000000001.turn",
            "00000000000000000001.0000000001.0000000000",
            "foreign",
            "turn-00000001",
            "delivery",
            "unrelated",
        ] {
            let mut disk = Disk::new();
            fs::write(disk.path().join(name), b"retained bytes").unwrap();
            refused(&mut disk, Some(Older::Unrecorded));
        }
    }

    #[test]
    fn mismatches_precede_valid_fresh_journal_filename_deletion() {
        for (store, transcript, family) in [
            (true, false, StoreFamily::Store),
            (false, true, StoreFamily::Transcript),
            (true, true, StoreFamily::Store),
        ] {
            for position in 0..32 {
                let mut disk = Disk::new();
                old_records(&disk);
                fs::write(disk.path().join("format"), marker(store, transcript, position)).unwrap();
                refused(&mut disk, Some(Older::Layout(family)));
            }
        }
    }

    #[test]
    fn malformed_truncated_wrong_version_and_oversized_markers_refuse_unchanged() {
        let valid = format::file().bytes;
        let mut wrong = valid.to_vec();
        wrong[1] = 2;
        let mut trailing = valid.to_vec();
        trailing.extend_from_slice(b"trailing record");
        let mut markers = vec![b"not a marker".to_vec(), wrong, vec![0; 67], trailing];
        for prefix in 0..valid.len() {
            markers.push(valid[..prefix].to_vec());
        }
        for bytes in markers {
            let oversized = bytes.len() > usize::try_from(format::max_bytes()).unwrap();
            let mut disk = Disk::new();
            old_records(&disk);
            fs::write(disk.path().join("format"), bytes).unwrap();
            refused(&mut disk, if oversized { None } else { Some(Older::Layout(StoreFamily::Store)) });
        }
    }

    #[test]
    fn marker_symlinks_and_directories_are_io_refusals_and_stay_untouched() {
        for symlink in [false, true] {
            let mut disk = Disk::new();
            old_records(&disk);
            if symlink {
                fs::write(disk.path().join("valid-target"), format::file().bytes).unwrap();
                std::os::unix::fs::symlink("valid-target", disk.path().join("format")).unwrap();
            } else {
                fs::create_dir(disk.path().join("format")).unwrap();
            }
            refused(&mut disk, None);
        }
    }

    #[test]
    fn near_miss_and_nonregular_reserved_residue_refuses_unchanged() {
        let residue = "format.skein-0123456789abcdef";
        for name in [
            "format.skein-0123456789abcdeF",
            "format.skein-0123456789abcde",
            "format.skein-0123456789abcdef0",
            "format.skein-0123456789abcdeg",
            "format.skein-0123456789abcdef.extra",
            "state.skein-0123456789abcdef",
            "xformat.skein-0123456789abcdef",
            ".format.skein-0123456789abcdef",
        ] {
            let mut disk = Disk::new();
            fs::write(disk.path().join(name), b"unknown").unwrap();
            refused(&mut disk, Some(Older::Unrecorded));
        }
        for mode in 0_u8..3_u8 {
            let mut disk = Disk::new();
            match mode {
                0 => std::os::unix::fs::symlink("missing", disk.path().join(residue)).unwrap(),
                1 => fs::create_dir(disk.path().join(residue)).unwrap(),
                2 => {
                    fs::write(disk.path().join(residue), b"partial").unwrap();
                    fs::write(disk.path().join("format.skein-fedcba9876543210"), b"partial").unwrap();
                    fs::write(disk.path().join("fresh"), b"old pending recovery").unwrap();
                }
                _ => unreachable!(),
            }
            refused(&mut disk, Some(Older::Unrecorded));
        }
    }

    #[test]
    fn a_matching_marker_permits_real_fresh_journal_recovery_and_keeps_the_marker() {
        let mut disk = Disk::new();
        old_records(&disk);
        fs::write(disk.path().join("format"), format::file().bytes).unwrap();
        let marker_before = fs::metadata(disk.path().join("format")).unwrap();
        let local::Event::Loaded { state: Some(loaded), transcript: None, deliveries } = disk.drive(StoreRequest::Load)
        else {
            panic!("fresh recovered");
        };
        assert_eq!(loaded, state(9));
        assert!(deliveries.is_empty());
        assert!(!disk.path().join("fresh").exists());
        assert!(!disk.path().join("0000000001.turn").exists());
        assert_eq!(fs::read(disk.path().join("format")).unwrap(), format::file().bytes.as_ref());
        assert_eq!(marker_before.ino(), fs::metadata(disk.path().join("format")).unwrap().ino());
        disk.close();
    }

    #[test]
    fn selecting_a_different_root_resets_the_check() {
        for mismatched in [false, true] {
            let mut disk = Disk::new();
            fs::create_dir(disk.path().join("current")).unwrap();
            fs::write(disk.path().join("current/format"), format::file().bytes).unwrap();
            fs::create_dir(disk.path().join("older")).unwrap();
            fs::write(disk.path().join("older/fresh"), smith_local_protocol::save_state(state(9)).bytes).unwrap();
            if mismatched {
                fs::write(disk.path().join("older/format"), marker(false, true, 0)).unwrap();
            }
            assert!(matches!(
                disk.drive(StoreRequest::SelectChat { name: b"current".as_slice().into(), create: false }),
                local::Event::ChatSelected { result: Ok(()), .. }
            ));
            empty(disk.drive(StoreRequest::Load));
            assert!(matches!(
                disk.drive(StoreRequest::SaveState { state: state(1), history: History::Kept }),
                local::Event::StateSaved
            ));
            assert!(matches!(
                disk.drive(StoreRequest::SelectChat { name: b"older".as_slice().into(), create: false }),
                local::Event::ChatSelected { result: Ok(()), .. }
            ));
            let before = snapshot(&disk.path().join("older"));
            disk.clear_requests();
            let expected = if mismatched { Older::Layout(StoreFamily::Transcript) } else { Older::Unrecorded };
            refused(&mut disk, Some(expected));
            assert_eq!(before, snapshot(&disk.path().join("older")));
        }
    }

    #[test]
    fn a_checked_current_store_saves_and_reopens_a_real_turn_with_its_read_fence() {
        let mut disk = Disk::new();
        empty(disk.drive(StoreRequest::Load));
        assert!(matches!(
            disk.drive(StoreRequest::SaveState { state: state(1), history: History::Kept }),
            local::Event::StateSaved
        ));
        let turn = smith_domain::Turn {
            endpoint: Some(smith_domain::session::llm::Endpoint(2)),
            ..smith_local_domain_fixtures::turn()
        };
        assert!(matches!(
            disk.drive(StoreRequest::SaveTurn {
                number: 1,
                read: Some(skein_lib::Token::new(1)),
                opens_window: true,
                turn: turn.clone()
            }),
            local::Event::TurnSaved { number: 1 }
        ));
        disk.reopen();
        let before = snapshot(disk.path());
        let local::Event::Loaded { state: Some(loaded), transcript: Some(transcript), deliveries } =
            disk.drive(StoreRequest::Load)
        else {
            panic!("current marked turn reload");
        };
        assert_eq!(loaded.read, Some(skein_lib::Token::new(1)));
        assert_eq!(transcript.value().turns.as_ref(), &[turn]);
        assert!(deliveries.is_empty());
        assert_eq!(before, snapshot(disk.path()));
        assert!(disk.requests().iter().all(|kind| matches!(kind, Kind::Load | Kind::Scan)));
        disk.close();
    }
    #[test]
    fn marker_at_file_bound_is_adopted() {
        let mut bounds = disk::limits();
        bounds.file_bytes = format::max_bytes();
        let mut disk = Disk::bounded(bounds);
        compatible(disk.path());
        let before = snapshot(disk.path());
        let terminal = disk.drive(StoreRequest::Load);
        disk.close();
        empty(terminal);
        assert_eq!(snapshot(disk.path()), before);
    }

    #[test]
    fn compatible_marker_loads_the_exact_public_state() {
        let mut disk = Disk::new();
        compatible(disk.path());
        let expected = state(23);
        write_state(disk.path(), expected);
        let before = snapshot(disk.path());
        let terminal = disk.drive(StoreRequest::Load);
        disk.close();
        assert!(
            matches!(&terminal, local::Event::Loaded { state: Some(value), transcript: None, deliveries } if *value == expected && deliveries.is_empty()),
            "{terminal:?}"
        );
        assert_eq!(snapshot(disk.path()), before);
    }

    #[test]
    fn repeated_load_refreshes_state_on_the_checked_root() {
        let mut disk = Disk::new();
        compatible(disk.path());
        let _loaded = disk.drive(StoreRequest::Load);
        let expected = state(29);
        write_state(disk.path(), expected);
        let terminal = disk.drive(StoreRequest::Load);
        disk.close();
        assert!(
            matches!(&terminal, local::Event::Loaded { state: Some(value), transcript: None, deliveries } if *value == expected && deliveries.is_empty()),
            "{terminal:?}"
        );
    }

    #[test]
    fn reopening_a_checked_root_rechecks_its_marker() {
        let mut disk = Disk::new();
        compatible(disk.path());
        let _loaded = disk.drive(StoreRequest::Load);
        fs::remove_file(disk.path().join("format")).unwrap();
        write_state(disk.path(), state(31));
        disk.reopen();
        refused(&mut disk, Some(Older::Unrecorded));
    }

    #[test]
    fn fresh_history_keeps_the_original_layout_marker() {
        let mut disk = Disk::new();
        compatible(disk.path());
        write_state(disk.path(), state(37));
        let marker_before: Vec<_> = snapshot(disk.path()).into_iter().filter(|entry| entry.name == "format").collect();
        let _loaded = disk.drive(StoreRequest::Load);
        let expected = state(41);
        let saved = disk.drive(StoreRequest::SaveState { state: expected, history: History::Fresh });
        disk.reopen();
        let terminal = disk.drive(StoreRequest::Load);
        let marker_after: Vec<_> = snapshot(disk.path()).into_iter().filter(|entry| entry.name == "format").collect();
        disk.close();
        assert!(matches!(saved, local::Event::StateSaved), "{saved:?}");
        assert!(
            matches!(&terminal, local::Event::Loaded { state: Some(value), transcript: None, deliveries } if *value == expected && deliveries.is_empty()),
            "{terminal:?}"
        );
        assert_eq!(marker_before, marker_after);
    }
}

#[cfg(test)]
mod scan_reservation_tests {
    use super::disk::{Disk, limits};
    use smith_local_domain::{self as local, Older};
    use smith_local_protocol::StoreRequest;

    #[test]
    fn interrupted_create_reserves_the_marker_entry_at_both_scan_bounds() {
        for (count, admitted) in [(7_u32, true), (8_u32, false)] {
            let mut disk = Disk::new();
            for index in 0..count {
                std::fs::write(disk.path().join(format!("format.skein-{index:016x}")), b"partial").unwrap();
            }
            let before = super::disk::snapshot(disk.path());
            let result = disk.drive(StoreRequest::Load);
            if admitted {
                assert!(matches!(result, local::Event::Loaded { state: None, .. }));
                let after_create = super::disk::snapshot(disk.path());
                let retained: Vec<_> = after_create.iter().filter(|entry| entry.name != "format").collect();
                assert_eq!(retained, before.iter().collect::<Vec<_>>());
                disk.reopen();
                assert!(matches!(disk.drive(StoreRequest::Load), local::Event::Loaded { state: None, .. }));
                assert_eq!(super::disk::snapshot(disk.path()), after_create);
            } else {
                assert!(matches!(
                    result,
                    local::Event::StoreFailed {
                        reason: local::StoreFailure::OlderBuild { why: Older::Unrecorded },
                        ..
                    }
                ));
                assert!(!disk.path().join("format").exists());
            }
            disk.close();
        }
        let entry = u64::try_from(size_of::<skein_io::file::Entry>()).unwrap();
        for (bytes, admitted) in [
            (entry.checked_mul(2).unwrap().checked_add(35).unwrap(), true),
            (entry.checked_mul(2).unwrap().checked_add(34).unwrap(), false),
        ] {
            let mut bounds = limits();
            bounds.scan_bytes = bytes;
            let mut disk = Disk::bounded(bounds);
            std::fs::write(disk.path().join("format.skein-0123456789abcdef"), b"partial").unwrap();
            let before = super::disk::snapshot(disk.path());
            let result = disk.drive(StoreRequest::Load);
            if admitted {
                assert!(matches!(result, local::Event::Loaded { state: None, .. }));
                let after_create = super::disk::snapshot(disk.path());
                let retained: Vec<_> = after_create.iter().filter(|entry| entry.name != "format").collect();
                assert_eq!(retained, before.iter().collect::<Vec<_>>());
                disk.reopen();
                assert!(matches!(disk.drive(StoreRequest::Load), local::Event::Loaded { state: None, .. }));
                assert_eq!(super::disk::snapshot(disk.path()), after_create);
            } else {
                assert!(matches!(
                    result,
                    local::Event::StoreFailed {
                        reason: local::StoreFailure::OlderBuild { why: Older::Unrecorded },
                        ..
                    }
                ));
                assert!(!disk.path().join("format").exists());
            }
            disk.close();
        }
    }
}
