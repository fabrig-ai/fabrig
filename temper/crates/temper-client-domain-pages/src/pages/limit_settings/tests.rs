use super::*;

#[test]
fn raises_choose_the_least_round_figure_above_both_floors() {
    for (configured, needed, expected) in [
        (8, 9, Some(16)),
        (160, 161, Some(384)),
        (768, 2000, Some(2000)),
        (0, 0, Some(1)),
        (0, 9, Some(9)),
        (1_u64 << 63_u32, 0, None),
        (1, u64::MAX, None),
    ] {
        assert_eq!(raise_figure(configured, needed), expected, "configured {configured}, needed {needed}");
    }
}
#[test]
fn a_full_read_pool_defers_the_limit_reads_instead_of_losing_them() {
    let handle = Handle { asker: Owner::Pages.asker(), number: 10 };
    let limits = crate::tests::fixtures::limits();
    for kind in 0_u32..3 {
        let mut pages = Pages::new();
        pages.limit_settings.inbox = Some(handle);
        match kind {
            0 => {
                let mut page = LimitsPage::new();
                page.pending = Some(handle);
                pages.limit_settings.page = Some(page);
            }
            1 => pages.limit_settings.holds_pending = Some(handle),
            2 => {
                pages.limit_settings.value_pending = Some((handle, 1));
                pages.limit_settings.revision = Some(1);
            }
            _ => unreachable!(),
        }
        assert!(told(&mut pages, &Told::Page { handle, page: Readout::Limit }, Time::ZERO, &limits));
        assert_eq!(pages.limit_settings.next_deadline(), Some(Time::ZERO.saturating_add(Duration::from_secs(1))));
        assert!(!pages.limit_settings.holds_done);
        assert!(!pages.limit_settings.value_read);
        if let Some(page) = pages.limit_settings.page() {
            assert!(!page.refused);
        }
    }
}
#[test]
fn shared_settings_tolds_fall_through_and_only_the_limits_send_is_claimed() {
    let mut pages = Pages::new();
    let limits = crate::tests::fixtures::limits();
    let changed = Told::Changed { handle: forge::ways::handle() };
    assert!(!told(&mut pages, &changed, Time::ZERO, &limits));
    let key = Key { asker: Owner::Pages.asker(), number: 17 };
    pages.limit_settings.sending = Some((42, key));
    let other = Key { asker: Owner::Pages.asker(), number: 18 };
    let outcome = jig_client::Outcome::Limit;
    assert!(!told(&mut pages, &Told::Answered { key: other, outcome: outcome.clone() }, Time::ZERO, &limits));
    assert!(told(&mut pages, &Told::Answered { key, outcome: outcome.clone() }, Time::ZERO, &limits));
    assert!(pages.limit_settings.sending.is_none());
    assert_eq!(pages.limit_settings.refused(42), Some(&outcome));
}
fn item(name: &[u8], value: u64) -> w::LimitOverride {
    w::LimitOverride { name: name.into(), value, reason: b"kept reason".as_slice().into() }
}
#[test]
fn a_raise_sets_one_override_and_keeps_the_rest_of_the_list() {
    let held = w::LimitHold {
            task: 42,
            limit: b"forge.entries".as_slice().into(),
            configured: 8,
            needed: 9,
            revision: 3,
        };
    for previous in [None, Some(item(b"forge.entries", 10))] {
        let mut overrides = List::with_capacity(3);
        overrides.push(item(b"forge.client.entries", 12)).unwrap();
        if let Some(previous) = previous {
            overrides.push(previous).unwrap();
        }
        overrides.push(item(b"transcript_memory", 2_000_000_000)).unwrap();
        let value = w::LimitOverrides { overrides };
        let changed = raised_item(&value, &held, &w::CEILINGS).unwrap();
        assert_eq!(changed.setting, c::Setting::Application { kind: 22, scope: 0, number: 1 });
        let c::SettingEdit::Set(c::SettingSet { value: c::SettingValue::Application(encoded) }) = changed.edit else {
            panic!("one setting set")
        };
        let value = w::LimitOverrides::decode(&w::CEILINGS, &mut Reader::new(&encoded)).unwrap();
        let rows = &value.overrides;
        assert_eq!(rows.len(), 3);
        assert_eq!(rows.get(0).unwrap(), &item(b"forge.client.entries", 12));
        assert_eq!(rows.get(1).unwrap().name.as_ref(), b"forge.entries");
        assert_eq!(rows.get(1).unwrap().value, 16);
        assert_eq!(rows.get(1).unwrap().reason.as_ref(), b"raised for task 42, which needed 9");
        assert_eq!(rows.get(2).unwrap(), &item(b"transcript_memory", 2_000_000_000));
    }
}
