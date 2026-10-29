//! Checked wire and heap worst cases (codec.md, section 4).

#[cfg(test)]
mod tests {
    use skein_codec_tests::{scalars, v1};
    use skein_lib::List;

    #[test]
    fn sample_wire_and_heap_ceilings_include_nested_lists() {
        assert_eq!(v1::Field::encoded_bound(&v1::CEILINGS), Some(16));
        assert_eq!(v1::Field::decoded_bound(&v1::CEILINGS), Some(8));
        assert_eq!(v1::Report::encoded_bound(&v1::CEILINGS), Some(63));
        assert_eq!(v1::Choice::encoded_bound(&v1::CEILINGS), Some(64));
        assert_eq!(v1::encoded_bound(&v1::CEILINGS), Some(64));

        let list_heap = List::<v1::Field>::price(2).expect("list heap fits");
        let report_heap = 24_u64.checked_add(list_heap).expect("sum fits");
        assert_eq!(v1::Report::decoded_bound(&v1::CEILINGS), Some(report_heap));
        assert_eq!(v1::Choice::decoded_bound(&v1::CEILINGS), Some(report_heap));
        assert_eq!(v1::decoded_bound(&v1::CEILINGS), Some(report_heap));

        let mut invalid = v1::CEILINGS;
        invalid.report_fields = 3;
        assert_eq!(v1::encoded_bound(&invalid), None);
        assert_eq!(v1::decoded_bound(&invalid), None);
    }

    #[test]
    fn scalar_family_has_no_heap_owned_by_decoded_values() {
        assert_eq!(scalars::Scalars::encoded_bound(&scalars::CEILINGS), Some(27));
        assert_eq!(scalars::encoded_bound(&scalars::CEILINGS), Some(27));
        assert_eq!(scalars::decoded_bound(&scalars::CEILINGS), Some(0));
    }

    #[test]
    fn a_bound_past_a_ceiling_is_none() {
        let mut limits = v1::CEILINGS;
        limits.report_fields = 3;
        assert_eq!(v1::Report::encoded_bound(&limits), None);
        assert_eq!(v1::Report::decoded_bound(&limits), None);
        assert_eq!(v1::Choice::encoded_bound(&limits), None);
        assert_eq!(v1::Choice::decoded_bound(&limits), None);
        assert_eq!(v1::Report::encoded_bound(&v1::CEILINGS), Some(63));
        assert_eq!(v1::Report::decoded_bound(&v1::CEILINGS), Some(88));
        assert_eq!(v1::Choice::encoded_bound(&v1::CEILINGS), Some(64));
        assert_eq!(v1::Choice::decoded_bound(&v1::CEILINGS), Some(88));
    }
}
