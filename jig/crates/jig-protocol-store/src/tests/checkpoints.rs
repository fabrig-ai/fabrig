//! Names and kept checkpoint bytes (protocol/store/checkpoints.md, 2).
use crate::{
    CheckpointName,
    checkpoints::FILES,
    codec::component::{Checkpoint, CheckpointFile, Manifest},
};
use skein_lib::{List, Reader, Writer};

#[test]
fn checkpoint_names_and_draw_halves_have_one_form() {
    let (name, temporary) = CheckpointName::from_draw(1234, [0, 0, 0, 0, 0, 0, 0, 0xab, 1, 2, 3, 4, 5, 6, 7, 8]);
    assert_eq!(name, CheckpointName { store: 1234, suffix: 0xab });
    assert_eq!(temporary, 0x0102_0304_0506_0708);
    assert_eq!(&name.bytes(), b"00000000000000001234-00000000000000ab");
    assert_eq!(&name.temporary(temporary), b"00000000000000001234-00000000000000ab.skein-0102030405060708");
    for name in [name, CheckpointName { store: 0, suffix: 0 }, CheckpointName { store: u64::MAX, suffix: u64::MAX }] {
        assert_eq!(CheckpointName::parse(&name.bytes()), Some(name));
        assert_eq!(CheckpointName::parse_temporary(&name.temporary(temporary)), Some((name, temporary)));
        assert_eq!(CheckpointName::parse(&name.temporary(temporary)), None);
        assert_eq!(CheckpointName::parse_temporary(&name.bytes()), None);
    }
    for bytes in [
        b"0000000000000001234-00000000000000ab".as_slice(),
        b"000000000000000001234-00000000000000ab",
        b"00000000000000001234-00000000000000aB",
        b"00000000000000001234-00000000000000abx",
        b"00000000000000001234_00000000000000ab",
        b"0000000000000000123x-00000000000000ab",
        b"18446744073709551616-00000000000000ab",
        b"00000000000000001234-00000000000000ag",
    ] {
        assert_eq!(CheckpointName::parse(bytes), None, "{bytes:?}");
    }
    for bytes in [
        b"00000000000000001234-00000000000000ab.skein-010203040506070".as_slice(),
        b"00000000000000001234-00000000000000ab.skein-01020304050607080",
        b"00000000000000001234-00000000000000ab.skein-010203040506070F",
        b"00000000000000001234-00000000000000ab.skein-010203040506070g",
        b"00000000000000001234-00000000000000aB.skein-0102030405060708",
        b"00000000000000001234-00000000000000ab.Skein-0102030405060708",
    ] {
        assert_eq!(CheckpointName::parse_temporary(bytes), None, "{bytes:?}");
    }
}

#[test]
#[expect(clippy::disallowed_macros, reason = "checkpoint and manifest bytes are pinned to generated goldens")]
fn checkpoint_and_manifest_goldens_round_trip_at_both_bounds() {
    for bytes in [
        include_bytes!("../../golden/component/record_checkpoint_smallest.bin").as_slice(),
        include_bytes!("../../golden/component/record_checkpoint_full.bin").as_slice(),
    ] {
        let record = Checkpoint::decode(bytes).expect("checkpoint golden");
        assert_eq!(record.clone().encode().expect("bounded record").as_ref(), bytes);
        assert_eq!(Checkpoint::decode(&record.encode().expect("bounded record")), Checkpoint::decode(bytes));
    }
    let mut full = Checkpoint::decode(include_bytes!("../../golden/component/record_checkpoint_full.bin"))
        .expect("full checkpoint");
    full.build = Box::from([b'a'; 128]);
    assert_eq!(crate::component_v3::CEILINGS.checkpoint_files, FILES);
    let file = full.files.get(0).expect("full golden has a file").clone();
    let mut files = List::with_capacity(FILES);
    for _ in 0..FILES {
        files.push(file.clone()).expect("file list ceiling");
    }
    full.files = files;
    let bytes = full.clone().encode().expect("FILES entries admitted");
    assert_eq!(Checkpoint::decode(&bytes), Ok(full));
    for bytes in [
        include_bytes!("../../golden/component/record_checkpoint_file_smallest.bin").as_slice(),
        include_bytes!("../../golden/component/record_checkpoint_file_full.bin").as_slice(),
    ] {
        let record =
            CheckpointFile::decode(&crate::component_v3::CEILINGS, &mut Reader::new(bytes)).expect("file golden");
        let mut writer = Writer::new(usize::try_from(record.measure()).expect("file size"));
        record.encode(&mut writer).expect("measured file");
        assert_eq!(writer.finish().as_ref(), bytes);
    }
    for bytes in [
        include_bytes!("../../golden/component/record_manifest_smallest.bin").as_slice(),
        include_bytes!("../../golden/component/record_manifest_full.bin").as_slice(),
    ] {
        let record = Manifest::decode(bytes).expect("manifest golden");
        assert_eq!(record.encode().as_ref(), bytes);
    }
}
