//! Accounts' longest variants at the ceilings (skein's data/codec/golden.md, 1).
use jig_protocol_store::accounts_v2::{self, Record};
use skein_heap::Meter;
use skein_lib::Reader;

#[test]
fn the_longest_accounts_rows_decode_within_the_rows_decoded_bound() {
    let ceilings = accounts_v2::CEILINGS;
    let bound = accounts_v2::Row::decoded_bound(&ceilings).expect("accounts heap bound");
    let bytes = |length: u32| vec![0xa5; skein_lib::length::usize_of(length)].into_boxed_slice();
    let party = accounts_v2::Party { party: u64::MAX };
    let account = accounts_v2::Account {
        list: None,
        place: u32::MAX,
        uses: Some({
            let mut uses = skein_lib::List::with_capacity(ceilings.account_uses_some);
            for n in 0..ceilings.account_uses_some {
                uses.push(accounts_v2::Use::Party(accounts_v2::Party { party: u64::from(n) })).expect("uses fit");
            }
            uses
        }),
        number: u32::MAX,
        provider: u32::MAX,
        holder: accounts_v2::Holder::Party(party),
        subject: bytes(ceilings.account_subject),
        display: bytes(ceilings.account_display),
    };
    let limited = accounts_v2::Limited { reset: Some(u64::MAX) };
    let source = accounts_v2::Source {
        account: u32::MAX,
        number: u32::MAX,
        place: u32::MAX,
        kind: accounts_v2::Kind::Program,
        name: bytes(ceilings.source_name),
        held: Some(accounts_v2::Held::Limited(limited)),
    };
    let mut facts = skein_lib::List::with_capacity(ceilings.facts_facts);
    for kind in 0..ceilings.facts_facts {
        let text = accounts_v2::FactText { text: bytes(ceilings.fact_text_text) };
        facts
            .push(accounts_v2::Fact {
                kind: u16::try_from(kind).expect("few facts"),
                label: bytes(ceilings.fact_label),
                value: accounts_v2::FactValue::Text(text),
                seen: u64::MAX,
                source: Some(u32::MAX),
                origin: u16::MAX,
            })
            .expect("facts fit");
    }
    let facts = accounts_v2::Facts { account: u32::MAX, facts };
    eprintln!(
        "accounts decoded bounds: account={} row={bound} bytes",
        accounts_v2::Account::decoded_bound(&ceilings).expect("account bound")
    );
    for value in
        [accounts_v2::Value::Account(account), accounts_v2::Value::Source(source), accounts_v2::Value::Facts(facts)]
    {
        let row = accounts_v2::Row { value };
        let encoded = row.encode(&ceilings).expect("within limits");
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
