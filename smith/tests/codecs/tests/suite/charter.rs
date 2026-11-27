//! Charter and result wire boundaries (protocol/charter.md, sections 3-6).

use crate::support::fixed_values::FixedValues;
use smith_charter::Record;

use skein_codec::Reason;
use skein_lib::Reader;
use smith_charter::v5::{CEILINGS, Charter, RunResult, paths};

#[test]
fn charter_and_result_fixed_value_records_decode() {
    let fixed_values = FixedValues::new("charter", 5);
    fixed_values.assert_measured("charter", |bytes| {
        Charter::decode(&CEILINGS, &mut Reader::new(bytes))
            .expect("charter decodes")
            .measure(&CEILINGS)
            .expect("bounded record")
    });
    fixed_values.assert_measured("run_result", |bytes| {
        RunResult::decode(&CEILINGS, &mut Reader::new(bytes))
            .expect("result decodes")
            .measure(&CEILINGS)
            .expect("bounded record")
    });
}

#[test]
fn charter_rejects_an_unsupported_version_before_fields() {
    let problem = Charter::decode(&CEILINGS, &mut Reader::new(&[0, 1])).expect_err("version one");
    assert_eq!((problem.path, problem.reason), (paths::CHARTER_VERSION, Reason::Version));
}

#[test]
fn llm_fixed_values_keep_the_window_and_output_as_separate_quantities() {
    use smith_charter::v5::Llm;
    let full = &crate::support::fixed_values::fixed_value("charter", 5, "record_llm_full.bin");
    let value = Llm::decode(&CEILINGS, &mut Reader::new(full)).expect("full LLM entry");
    assert_eq!((value.window, value.output), (u32::MAX, u32::MAX));
    let smallest = &crate::support::fixed_values::fixed_value("charter", 5, "record_llm_smallest.bin");
    let value = Llm::decode(&CEILINGS, &mut Reader::new(smallest)).expect("smallest LLM entry");
    assert_eq!((value.window, value.output), (0, 0));
}

#[test]
fn prices_fixed_values_keep_an_optional_separate_cache_write_rate() {
    use smith_charter::v5::Prices;
    let full = &crate::support::fixed_values::fixed_value("charter", 5, "record_prices_full.bin");
    let decoded = Prices::decode(&CEILINGS, &mut Reader::new(full)).expect("full prices");
    assert_eq!(decoded.cache_write, Some(u64::MAX));
    let smallest = &crate::support::fixed_values::fixed_value("charter", 5, "record_prices_smallest.bin");
    let decoded = Prices::decode(&CEILINGS, &mut Reader::new(smallest)).expect("smallest prices");
    assert_eq!(decoded.cache_write, None, "absent write rate inherits input in the domain");
}

#[test]
fn llm_fixed_values_carry_an_account_or_none() {
    for (bytes, expected) in [
        (crate::support::fixed_values::fixed_value("charter", 5, "record_llm_smallest.bin").as_slice(), None),
        (crate::support::fixed_values::fixed_value("charter", 5, "record_llm_full.bin").as_slice(), Some(u32::MAX)),
    ] {
        let value = smith_charter::Llm::decode(&CEILINGS, &mut Reader::new(bytes)).expect("LLM fixed-rule value");
        assert_eq!(value.account, expected);
    }
}

#[test]
fn earlier_charter_and_result_versions_are_refused() {
    for version in [1, 2, 3, 4] {
        let problem = Charter::decode(&CEILINGS, &mut Reader::new(&[0, version])).expect_err("older charter");
        assert_eq!((problem.path, problem.reason), (paths::CHARTER_VERSION, Reason::Version));
        let problem = RunResult::decode(&CEILINGS, &mut Reader::new(&[0, version])).expect_err("older result");
        assert_eq!((problem.path, problem.reason), (paths::RUN_RESULT_VERSION, Reason::Version));
    }
}

#[test]
fn both_modes_round_trip_in_the_version_five_charter() {
    use smith_charter::Mode;
    let smallest = &crate::support::fixed_values::fixed_value("charter", 5, "record_charter_smallest.bin");
    for (bytes, mode) in [
        (crate::support::fixed_values::fixed_value("charter", 5, "enum_mode_finite_full.bin").as_slice(), Mode::Finite),
        (crate::support::fixed_values::fixed_value("charter", 5, "enum_mode_open_full.bin").as_slice(), Mode::Open),
    ] {
        assert_eq!(Mode::decode(&CEILINGS, &mut Reader::new(bytes)).expect("mode fixed-rule value"), mode);
        assert_eq!(mode.encode(&CEILINGS).expect("bounded mode").as_ref(), bytes);
        let mut parts = Charter::decode(&CEILINGS, &mut Reader::new(smallest)).expect("smallest charter");
        parts.mode = mode;
        let charter = parts;
        let encoded = charter.encode(&CEILINGS).expect("bounded charter");
        assert_eq!(Charter::decode(&CEILINGS, &mut Reader::new(&encoded)), Ok(charter));
    }
}

#[test]
fn charter_round_trips_main_and_child_speeds_whole() {
    use smith_charter::{Prices, Speed};
    let smallest = &crate::support::fixed_values::fixed_value("charter", 5, "record_charter_smallest.bin");
    let mut charter = Charter::decode(&CEILINGS, &mut Reader::new(smallest)).expect("charter without speed");
    assert_eq!(charter.main.speed, None);
    let mut child = charter.main.clone();
    child.model = b"child".as_slice().into();
    let mut also = skein_lib::List::with_capacity(1);
    also.push(b"priority".as_slice().into()).expect("one alias");
    child.speed = Some(Speed {
        name: b"fast".as_slice().into(),
        also,
        prices: Prices { input: 3, cached: 2, cache_write: Some(4), output: 9, unit: 100 },
    });
    charter.main.speed = Some(Speed {
        name: b"ultrafast".as_slice().into(),
        also: skein_lib::List::with_capacity(0),
        prices: Prices { input: 7, cached: 5, cache_write: None, output: 11, unit: 1000 },
    });
    charter.models = skein_lib::List::with_capacity(1);
    charter.models.push(child).expect("one sub-agent model");
    let encoded = charter.encode(&CEILINGS).expect("bounded charter");
    assert_eq!(Charter::decode(&CEILINGS, &mut Reader::new(&encoded)), Ok(charter));
}

#[test]
fn conventions_keep_guide_and_checks_across_the_hooks_layout() {
    use skein_codec::layout::{Mode, Plan, Workspace};
    use skein_codec::schema::parse;
    use skein_lib::Buffer;
    use smith_charter::Conventions;

    let text = smith_charter::SCHEMA;
    let older = text.replace(" droppable hooks: option text 1024", "").replace(" hooks: option text 1024", "");
    assert_ne!(older, text, "derive the neighboring layout from the current grammar");
    let new = parse(text.as_bytes()).expect("new schema");
    let old = parse(older.as_bytes()).expect("schema without hooks");
    let new_root = new.root(b"Charter", None).expect("new charter root");
    let old_root = old.root(b"Charter", None).expect("old charter root");
    assert_ne!(new.layout(new_root, &[]), old.layout(old_root, &[]));

    let mut charter = Charter::decode(
        &CEILINGS,
        &mut Reader::new(&crate::support::fixed_values::fixed_value("charter", 5, "record_charter_smallest.bin")),
    )
    .expect("smallest charter");
    let guide = b"AGENTS.md";
    let checks = b".temper/pre-pr";
    charter.conventions = Some(Conventions {
        guide: guide.as_slice().into(),
        checks: checks.as_slice().into(),
        hooks: Some(b".temper/hooks".as_slice().into()),
    });
    let original = charter.encode(&CEILINGS).expect("new charter with hooks set");
    let drop_hooks = Plan::between(Mode::Keep, &new, new_root, &[], &old, old_root, &[]).expect("Keep plan");
    let add_default = Plan::between(Mode::Keep, &old, old_root, &[], &new, new_root, &[]).expect("Keep plan back");
    let mut work = Workspace::new();
    let mut reduced = Buffer::with_capacity(4096);
    assert!(work.fit(&drop_hooks));
    drop_hooks.convert(&original, &mut reduced, &mut work).expect("Keep drops a populated hooks field");
    let mut restored = Buffer::with_capacity(4096);
    assert!(work.fit(&add_default));
    add_default.convert(reduced.as_slice(), &mut restored, &mut work).expect("old charter reads with defaults");
    let read = Charter::decode(&CEILINGS, &mut Reader::new(restored.as_slice())).expect("converted charter");
    charter.conventions.as_mut().expect("conventions set above").hooks = None;
    assert_eq!(read, charter, "the older layout keeps guide and checks, with no hooks");
}
