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
