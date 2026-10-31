use jig_client_domain::contract::{Funder, OwnerBudget, OwnerNumbers, PoolLedger};
use skein_lib::Wall;

fn sources() -> [Funder; 4] {
    [
        Funder::Task(u64::MAX),
        Funder::Pool { project: u32::MAX, person: u64::MAX - 1, period: u64::MAX - 2 },
        Funder::Recurring { project: u32::MAX - 1, task: u64::MAX - 3, period: u64::MAX - 4 },
        Funder::Period { project: u32::MAX - 2, period: u64::MAX - 5 },
    ]
}
fn numbers() -> OwnerNumbers {
    OwnerNumbers { budget: u64::MAX, spent: u64::MAX - 1, spent_below: u64::MAX - 2, reserved: u64::MAX - 3 }
}

#[test]
fn maximal_budget_and_ledger_render_all_accounting_values() {
    use jig_client_view::owner_budget as view;
    use jig_client_view::words::Words;
    use skein_ui::Builder;
    use skein_ui::Limits;
    use skein_ui::phrases::PhraseLimits;

    let language_limits = PhraseLimits {
        languages: 1,
        phrases: jig_client_view::owner_words::phrase_count(),
        holes: 4,
        segments: 20_000,
        text_bytes: 200_000,
        hole_bytes: 4096,
    };
    let mut english = jig_client_view::phrases::english::language();
    english.wall = skein_ui::phrases::WallFormat::Nanos;
    let words = Words::new(
        skein_ui::phrases::Phrasebook::load(
            jig_client_view::phrases::jig::shapes(4096),
            Box::new([english]),
            &language_limits,
        )
        .expect("checked rendering fixture"),
        0,
        0,
    );
    let receiving = jig_client_world::test::limits().jig;
    let limits = Limits {
        nodes: 16_384,
        text_bytes: 16_777_216,
        ..jig_client_view::limits::envelope(
            &receiving,
            &jig_client_world::test::limits().client.domain,
            words.jig_capacity().expect("checked rendering fixture"),
            &jig_client_view::page::Arrangement::none(),
            0,
            0,
            0,
        )
        .expect("checked rendering fixture")
    };

    let budget = OwnerBudget {
        numbers: numbers(),
        left: u64::MAX - 4,
        funder: sources()[1],
        deadline: Some(Wall::from_nanos(u64::MAX)),
    };
    let ledger = PoolLedger {
        source: sources()[2],
        parent: Some(sources()[1]),
        numbers: numbers(),
        left: u64::MAX - 4,
        closed: true,
        began: Some(Wall::from_nanos(u64::MAX - 1)),
        next: Some(Wall::from_nanos(u64::MAX)),
    };
    for is_ledger in [false, true] {
        let mut builder = Builder::new(&limits);
        if is_ledger {
            view::ledger(&mut builder, &ledger, &words);
        } else {
            view::render(&mut builder, &budget, &words);
        }
        let tree = builder.finish();
        assert_eq!(tree.len(), if is_ledger { 13 } else { 9 });
        for exact in [
            &b"Budget: 18446744073709551615"[..],
            &b"Spent directly: 18446744073709551614"[..],
            &b"Settled spend below: 18446744073709551613"[..],
            &b"Reserved for funded work: 18446744073709551612"[..],
            &b"Left: 18446744073709551611"[..],
        ] {
            assert!(
                tree.nodes().iter().any(|node| node.text.iter().any(|run| run.text.as_ref() == exact)),
                "all owner accounting values retain their full width in both fragments"
            );
        }
    }
}
