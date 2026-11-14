//! Deployment census reads and rendering (client/ctl/results.md, 1–3).
use crate::tests::fixtures as f;
use jig_client::contract as c;
use skein_lib::List;

fn census() -> c::StoreCensus {
    let mut keyspaces = List::with_capacity(2);
    let mut heads = List::with_capacity(1);
    heads.push(c::CensusHead { head: None, rows: 2 }).expect("aggregate head");
    keyspaces
        .push(c::CensusKeyspace {
            prefix: Box::default(),
            name: Box::from(""),
            rows: 2,
            keys: 4,
            values: 6,
            heads,
            puts: 1,
            erases: 2,
            written: 10,
        })
        .expect("aggregate keyspace");
    let mut heads = List::with_capacity(1);
    heads.push(c::CensusHead { head: Some(3), rows: 993_441 }).expect("milestones head");
    keyspaces
        .push(c::CensusKeyspace {
            prefix: Box::from([1, 1, 10]),
            name: Box::from("tasks.milestones"),
            rows: 993_441,
            keys: 9_000_000,
            values: 3_300_000,
            heads,
            puts: 1000,
            erases: 204,
            written: 85_000,
        })
        .expect("milestones keyspace");
    c::StoreCensus { commit: 812, since: 640, keyspaces }
}
fn answer(census: Option<c::StoreCensus>, json: bool) -> crate::jig::Printed {
    // No deployment role in me: the tool must ask jig, not filter locally.
    let (mut root, call) =
        f::read_call(if json { &["deployment", "census", "--json"] } else { &["deployment", "census"] });
    assert_eq!(root.reads.get(&call).expect("deployment read").1, c::Read::Deployment);
    let mut deployment = f::deployment();
    deployment.census = match census {
        Some(census) => Some(Box::new(census)),
        None => None,
    };
    root.serve(call, c::ReadResult::Value(c::ReadValue::Deployment(Box::new(deployment))));
    let answer = root.answer(call).expect("deployment read answered");
    assert_eq!(answer.status, 0);
    assert!(answer.stderr.is_empty());
    answer
}

#[test]
fn census_text_keeps_counts_prefix_order_and_head_identity() {
    let answer = answer(Some(census()), false);
    assert_eq!(answer.stdout.as_ref(), b"census at commit 812, writes since commit 640\n- - rows 2 keys 4 values 6 puts 1 erases 2 written 10 heads -:2\n01010a tasks.milestones rows 993441 keys 9000000 values 3300000 puts 1000 erases 204 written 85000 heads 3:993441\n");
}

#[test]
fn census_json_is_the_whole_census_with_hex_prefix_and_null_head() {
    let answer = answer(Some(census()), true);
    assert_eq!(answer.stdout.as_ref(), br#"{"census":{"commit":812,"since":640,"keyspaces":[{"prefix":"","name":"","rows":2,"keys":4,"values":6,"heads":[{"head":null,"rows":2}],"puts":1,"erases":2,"written":10},{"prefix":"01010a","name":"tasks.milestones","rows":993441,"keys":9000000,"values":3300000,"heads":[{"head":3,"rows":993441}],"puts":1000,"erases":204,"written":85000}]}}"#);
}

#[test]
fn before_the_first_census_is_a_successful_empty_read() {
    assert_eq!(answer(None, false).stdout.as_ref(), b"no census yet\n");
    assert_eq!(answer(None, true).stdout.as_ref(), br#"{"census":null}"#);
}

#[test]
fn census_heads_distinguish_no_heads_from_unnumbered_and_several() {
    let mut census = census();
    census.keyspaces.get_mut(0).expect("aggregate").heads = List::with_capacity(0);
    let heads = &mut census.keyspaces.get_mut(1).expect("milestones").heads;
    *heads = List::with_capacity(2);
    heads.push(c::CensusHead { head: None, rows: 2 }).expect("unnumbered");
    heads.push(c::CensusHead { head: Some(3), rows: 993_439 }).expect("numbered");
    let text = answer(Some(census.clone()), false);
    assert_eq!(text.stdout.as_ref(), b"census at commit 812, writes since commit 640\n- - rows 2 keys 4 values 6 puts 1 erases 2 written 10 heads -\n01010a tasks.milestones rows 993441 keys 9000000 values 3300000 puts 1000 erases 204 written 85000 heads -:2,3:993439\n");
    let json = answer(Some(census), true);
    assert!(skein_lib::bytes::find(&json.stdout, br#""heads":[]"#).is_some());
    assert!(
        skein_lib::bytes::find(&json.stdout, br#""heads":[{"head":null,"rows":2},{"head":3,"rows":993439}]"#).is_some()
    );
}

#[test]
fn census_keeps_every_keyspace_and_head_at_the_contract_bounds() {
    let mut heads = List::with_capacity(u32::try_from(c::CENSUS_HEADS).expect("head bound fits"));
    for head in 0..c::CENSUS_HEADS {
        heads
            .push(c::CensusHead { head: Some(u16::try_from(head).expect("head fits")), rows: u64::MAX })
            .expect("head room");
    }
    let mut keyspaces = List::with_capacity(u32::try_from(c::CENSUS_ROWS).expect("row bound fits"));
    for row in 0..c::CENSUS_ROWS {
        keyspaces
            .push(c::CensusKeyspace {
                prefix: Box::from(u64::try_from(row).expect("prefix fits").to_be_bytes()),
                name: Box::from("tasks.milestones"),
                rows: u64::MAX,
                keys: u64::MAX,
                values: u64::MAX,
                heads: heads.clone(),
                puts: u64::MAX,
                erases: u64::MAX,
                written: u64::MAX,
            })
            .expect("keyspace room");
    }
    let census = c::StoreCensus { commit: u64::MAX, since: u64::MAX, keyspaces };
    let text = answer(Some(census.clone()), false);
    assert!(
        skein_lib::bytes::find(&text.stdout, b"0000000000000080 tasks.milestones rows 18446744073709551615").is_some()
    );
    assert!(skein_lib::bytes::find(&text.stdout, b",7:18446744073709551615\n").is_some());
    let json = answer(Some(census), true);
    assert!(skein_lib::bytes::find(&json.stdout, br#""prefix":"0000000000000080""#).is_some());
    assert!(skein_lib::bytes::find(&json.stdout, br#"{"head":7,"rows":18446744073709551615}"#).is_some());
    assert!(skein_lib::bytes::find(&json.stdout, br#""written":18446744073709551615"#).is_some());
}

#[test]
fn census_without_a_deployment_role_keeps_jigs_refusal() {
    for json in [false, true] {
        let words = if json { &["deployment", "census", "--json"][..] } else { &["deployment", "census"][..] };
        let (mut root, call) = f::read_call(words);
        assert_eq!(root.reads.get(&call).expect("deployment read").1, c::Read::Deployment);
        root.serve(call, c::ReadResult::None(c::ReadNone::Refused));
        let answer = root.answer(call).expect("jig refused");
        assert_eq!(answer.status, 2);
        assert!(answer.stdout.is_empty());
        assert_eq!(answer.stderr.as_ref(), if json { br#"{"error":"refused"}"#.as_slice() } else { b"refused\n" });
    }
}
