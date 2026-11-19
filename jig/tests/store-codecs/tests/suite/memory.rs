//! Accounts' longest variants at the ceilings (skein's data/codec/README.md, 6).
use jig_protocol_store::accounts_v2;
use skein_heap::Meter;
use skein_lib::Reader;

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
