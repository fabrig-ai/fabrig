//! Deleted projects disappear from explicit account uses (accounts.md, 4.2).
use jig_accounts_world::{World, account, provider, providers};
use jig_domain_accounts::{Event, Holder, Request, Stored, Use};

#[test]
fn each_accounts_use_of_a_gone_project_is_dropped() {
    let mut world = World::with_limits(
        jig_domain_accounts::Limits { accounts: 4, sources: 4, ..jig_accounts_world::LIMITS },
        providers(vec![provider(1, true, true)]),
        &[],
    );
    let cases = [
        (10, Some(vec![Use::Project(3), Use::Party(7), Use::Project(4)]), Some(vec![Use::Party(7), Use::Project(4)])),
        (20, Some(vec![Use::Project(3)]), Some(vec![])),
        (30, None, None),
        (40, Some(vec![Use::Project(4)]), Some(vec![Use::Project(4)])),
    ];
    let rows = cases
        .iter()
        .map(|(number, uses, _)| {
            let mut row = account(*number, 1, Holder::Party(u64::from(*number)), b"");
            if let Stored::Account { uses: payers, .. } = &mut row {
                *payers = uses.clone().map(Vec::into_boxed_slice);
            }
            row
        })
        .collect();
    world.restore(rows);
    world.restore(
        [10, 20, 30, 40]
            .into_iter()
            .map(|number| jig_accounts_world::source(number, number + 1, 0, jig_domain_accounts::Kind::Key))
            .collect(),
    );
    world.step(Event::Want);
    let out = world.step(Event::ProjectGone { project: 3 });
    let mut saved = Vec::new();
    for request in out {
        if let Request::Save { record: Stored::Account { number, uses, .. } } = request {
            assert!(!uses.as_ref().expect("explicit uses retained").contains(&Use::Project(3)));
            saved.push(number);
        }
    }
    saved.sort_unstable();
    assert_eq!(saved, vec![10, 20]);
    assert_eq!(world.domain.accounts(), 4);
    for (number, _, expected) in cases {
        assert_eq!(world.domain.account(number).expect("account retained").uses(), expected.as_deref());
    }
}
