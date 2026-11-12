//! Full golden rows decode within their decoded bounds (skein's data/codec/README.md, 6).
use jig_protocol_store::{
    accounts_v2, component_v3, core_v6, notes_v1, people_v16, secrets_v1, settings_v2, tasks_v11,
};
use skein_heap::Meter;
use skein_lib::Reader;

fn check<T>(family: &str, record: &str, bound: u64, decode: impl FnOnce(&[u8]) -> T) {
    let golden = std::fs::read(
        skein_scratch::paths::package()
            .join(format!("../../crates/jig-protocol-store/golden/{family}/record_{record}_full.bin")),
    )
    .expect("full golden row");
    let meter = Meter::new();
    meter.start();
    let decoded = decode(&golden);
    let measured = meter.end();
    assert!(measured.peak() <= bound, "{family}/{record} decoding peaks within the generated bound");
    drop(decoded);
    assert_eq!(meter.held(), 0, "the decoded row's heap is released");
}

#[test]
fn a_full_notes_row_decodes_within_its_decoded_bound() {
    let bound = notes_v1::Row::decoded_bound(&notes_v1::CEILINGS).expect("notes heap bound");
    check("notes", "row", bound, |golden| {
        notes_v1::Row::decode(&notes_v1::CEILINGS, &mut Reader::new(golden)).expect("full row")
    });
}
#[test]
fn a_full_tasks_row_decodes_within_its_decoded_bound() {
    let bound = tasks_v11::Row::decoded_bound(&tasks_v11::CEILINGS).expect("tasks heap bound");
    check("tasks", "row", bound, |golden| {
        tasks_v11::Row::decode(&tasks_v11::CEILINGS, &mut Reader::new(golden)).expect("full row")
    });
}
#[test]
fn a_full_core_row_decodes_within_its_decoded_bound() {
    let bound = core_v6::Row::decoded_bound(&core_v6::CEILINGS).expect("core heap bound");
    check("core", "row", bound, |golden| {
        core_v6::Row::decode(&core_v6::CEILINGS, &mut Reader::new(golden)).expect("full row")
    });
}
#[test]
fn a_full_people_row_decodes_within_its_decoded_bound() {
    let bound = people_v16::Row::decoded_bound(&people_v16::CEILINGS).expect("people heap bound");
    check("people", "row", bound, |golden| {
        people_v16::Row::decode(&people_v16::CEILINGS, &mut Reader::new(golden)).expect("full row")
    });
}
#[test]
fn each_full_component_row_decodes_within_its_decoded_bound() {
    check(
        "component",
        "layouts",
        component_v3::Layouts::decoded_bound(&component_v3::CEILINGS).expect("layouts bound"),
        |golden| {
            component_v3::Layouts::decode(&component_v3::CEILINGS, &mut Reader::new(golden)).expect("full layouts")
        },
    );
    check(
        "component",
        "format",
        component_v3::Format::decoded_bound(&component_v3::CEILINGS).expect("format bound"),
        |golden| component_v3::Format::decode(&component_v3::CEILINGS, &mut Reader::new(golden)).expect("full format"),
    );
    check(
        "component",
        "transcript",
        component_v3::Transcript::decoded_bound(&component_v3::CEILINGS).expect("transcript bound"),
        |golden| {
            component_v3::Transcript::decode(&component_v3::CEILINGS, &mut Reader::new(golden))
                .expect("full transcript")
        },
    );
    check(
        "component",
        "attempt",
        component_v3::Attempt::decoded_bound(&component_v3::CEILINGS).expect("attempt bound"),
        |golden| {
            component_v3::Attempt::decode(&component_v3::CEILINGS, &mut Reader::new(golden)).expect("full attempt")
        },
    );
}
#[test]
fn a_full_installation_row_decodes_within_its_decoded_bound() {
    let bound = accounts_v2::Installation::decoded_bound(&accounts_v2::CEILINGS).expect("installation bound");
    check("accounts", "installation", bound, |golden| {
        accounts_v2::Installation::decode(&accounts_v2::CEILINGS, &mut Reader::new(golden)).expect("full installation")
    });
}
#[test]
fn a_full_accounts_row_decodes_within_its_decoded_bound() {
    let bound = accounts_v2::Row::decoded_bound(&accounts_v2::CEILINGS).expect("accounts heap bound");
    check("accounts", "row", bound, |golden| {
        accounts_v2::Row::decode(&accounts_v2::CEILINGS, &mut Reader::new(golden)).expect("full row")
    });
}
#[test]
fn the_longest_accounts_rows_decode_within_the_rows_decoded_bound() {
    let ceilings = accounts_v2::CEILINGS;
    let bound = accounts_v2::Row::decoded_bound(&ceilings).expect("accounts heap bound");
    let bytes = |length: u32| vec![0xa5; skein_lib::length::usize_of(length)].into_boxed_slice();
    let party = accounts_v2::Party::new(&ceilings, accounts_v2::PartyParts { party: u64::MAX }).expect("a party");
    let account = accounts_v2::Account::new(
        &ceilings,
        accounts_v2::AccountParts {
            list: None,
            place: u32::MAX,
            uses: Some({
                let mut uses = skein_lib::List::with_capacity(ceilings.account_uses_some);
                for n in 0..ceilings.account_uses_some {
                    uses.push(accounts_v2::Use::Party(
                        accounts_v2::Party::new(&ceilings, accounts_v2::PartyParts { party: u64::from(n) })
                            .expect("party"),
                    ))
                    .expect("uses fit");
                }
                uses
            }),
            number: u32::MAX,
            provider: u32::MAX,
            holder: accounts_v2::Holder::Party(party),
            subject: bytes(ceilings.account_subject),
            display: bytes(ceilings.account_display),
        },
    )
    .expect("an account at its ceilings");
    let limited =
        accounts_v2::Limited::new(&ceilings, accounts_v2::LimitedParts { reset: Some(u64::MAX) }).expect("a limit");
    let source = accounts_v2::Source::new(
        &ceilings,
        accounts_v2::SourceParts {
            account: u32::MAX,
            number: u32::MAX,
            place: u32::MAX,
            kind: accounts_v2::Kind::Program,
            name: bytes(ceilings.source_name),
            held: Some(accounts_v2::Held::Limited(limited)),
        },
    )
    .expect("a source at its ceiling");
    let mut facts = skein_lib::List::with_capacity(ceilings.facts_facts);
    for kind in 0..ceilings.facts_facts {
        let text =
            accounts_v2::FactText::new(&ceilings, accounts_v2::FactTextParts { text: bytes(ceilings.fact_text_text) })
                .expect("text at its ceiling");
        facts
            .push(
                accounts_v2::Fact::new(
                    &ceilings,
                    accounts_v2::FactParts {
                        kind: u16::try_from(kind).expect("few facts"),
                        label: bytes(ceilings.fact_label),
                        value: accounts_v2::FactValue::Text(text),
                        seen: u64::MAX,
                        source: Some(u32::MAX),
                        origin: u16::MAX,
                    },
                )
                .expect("fact at its ceilings"),
            )
            .expect("facts fit");
    }
    let facts = accounts_v2::Facts::new(&ceilings, accounts_v2::FactsParts { account: u32::MAX, facts })
        .expect("facts at their ceiling");
    eprintln!(
        "accounts decoded bounds: account={} row={bound} bytes",
        accounts_v2::Account::decoded_bound(&ceilings).expect("account bound")
    );
    for value in
        [accounts_v2::Value::Account(account), accounts_v2::Value::Source(source), accounts_v2::Value::Facts(facts)]
    {
        let row = accounts_v2::Row::new(&ceilings, accounts_v2::RowParts { value }).expect("a row");
        let mut writer = skein_lib::Writer::new(skein_lib::length::usize_of(row.measure()));
        row.encode(&mut writer).expect("measured room");
        let encoded = writer.finish();
        let meter = Meter::new();
        meter.start();
        let decoded = accounts_v2::Row::decode(&ceilings, &mut Reader::new(&encoded)).expect("the longest row");
        let measured = meter.end();
        assert!(measured.peak() <= bound, "the longest accounts row decodes within the generated bound");
        assert_eq!(decoded, row);
        drop(decoded);
        assert_eq!(meter.held(), 0, "the decoded row's heap is released");
    }
}

#[test]
fn added_dependencies_kept_lines_and_circle_payloads_fit_their_heap_bounds() {
    let limits = tasks_v11::CEILINGS;
    check(
        "tasks",
        "task_definition",
        tasks_v11::TaskDefinition::decoded_bound(&limits).expect("definition bound"),
        |golden| {
            tasks_v11::TaskDefinition::decode(&limits, &mut Reader::new(golden))
                .expect("full definition with additions")
        },
    );
    check("tasks", "kept_line", tasks_v11::KeptLine::decoded_bound(&limits).expect("kept bound"), |golden| {
        tasks_v11::KeptLine::decode(&limits, &mut Reader::new(golden)).expect("full pending list")
    });
    check("tasks", "circle", tasks_v11::Circle::decoded_bound(&limits).expect("circle bound"), |golden| {
        tasks_v11::Circle::decode(&limits, &mut Reader::new(golden)).expect("full circle")
    });
}

#[test]
fn each_full_secrets_row_decodes_within_its_heap_bound() {
    let limits = &secrets_v1::CEILINGS;
    check("secrets", "digest", secrets_v1::Digest::decoded_bound(limits).expect("digest heap"), |golden| {
        secrets_v1::Digest::decode(limits, &mut Reader::new(golden)).expect("full digest")
    });
    check("secrets", "source", secrets_v1::Source::decoded_bound(limits).expect("source heap"), |golden| {
        secrets_v1::Source::decode(limits, &mut Reader::new(golden)).expect("full source")
    });
    check("secrets", "carried", secrets_v1::Carried::decoded_bound(limits).expect("carried heap"), |golden| {
        secrets_v1::Carried::decode(limits, &mut Reader::new(golden)).expect("full carried")
    });
    check("secrets", "next_carried", secrets_v1::NextCarried::decoded_bound(limits).expect("counter heap"), |golden| {
        secrets_v1::NextCarried::decode(limits, &mut Reader::new(golden)).expect("full counter")
    });
}

#[test]
fn every_full_settings_record_decodes_within_its_heap_bound() {
    let limits = &settings_v2::CEILINGS;
    check("settings", "document", settings_v2::Document::decoded_bound(limits).expect("document bound"), |golden| {
        settings_v2::Document::decode(limits, &mut Reader::new(golden)).expect("full document")
    });
    check("settings", "charter", settings_v2::Charter::decoded_bound(limits).expect("charter bound"), |golden| {
        settings_v2::Charter::decode(limits, &mut Reader::new(golden)).expect("full charter")
    });
    check("settings", "default", settings_v2::Default::decoded_bound(limits).expect("default bound"), |golden| {
        settings_v2::Default::decode(limits, &mut Reader::new(golden)).expect("full default")
    });
    check("settings", "changed", settings_v2::Changed::decoded_bound(limits).expect("changed bound"), |golden| {
        settings_v2::Changed::decode(limits, &mut Reader::new(golden)).expect("full change")
    });
    check("settings", "revision", settings_v2::Revision::decoded_bound(limits).expect("revision bound"), |golden| {
        settings_v2::Revision::decode(limits, &mut Reader::new(golden)).expect("full revision")
    });
    check(
        "settings",
        "charter_version",
        settings_v2::CharterVersion::decoded_bound(limits).expect("version bound"),
        |golden| settings_v2::CharterVersion::decode(limits, &mut Reader::new(golden)).expect("full version"),
    );
    check("settings", "proposal", settings_v2::Proposal::decoded_bound(limits).expect("proposal bound"), |golden| {
        settings_v2::Proposal::decode(limits, &mut Reader::new(golden)).expect("full proposal")
    });
}
