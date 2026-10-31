use jig_accounts_world::{LIMITS, World, account, provider, providers, source};
use jig_domain_accounts::{Coverage, Event, Holder, Kind, Limits, Request, Selection, Stored, Use};

const PROVIDER: u32 = 1;
const ACCOUNTS: [u32; 4] = [10, 20, 30, 40];
const LIMITS_WITH_FOUR_ACCOUNTS: Limits = Limits { accounts: 4, holder_accounts: 4, ..LIMITS };

fn rows() -> Vec<Stored> {
    let uses =
        [Some(vec![Use::Project(1), Use::Party(4)]), Some(vec![Use::Project(1)]), Some(vec![Use::Project(2)]), None];
    let mut rows = Vec::new();
    for (number, uses) in ACCOUNTS.into_iter().zip(uses) {
        let mut row = account(number, PROVIDER, Holder::Party(4), b"");
        if let Stored::Account { uses: stored, .. } = &mut row {
            *stored = uses.map(Vec::into_boxed_slice);
        }
        rows.extend([row, source(number, number + 1, 0, Kind::Key)]);
    }
    rows
}

fn restored(rows: &[Stored]) -> World {
    World::with_start(LIMITS_WITH_FOUR_ACCOUNTS, providers(vec![provider(PROVIDER, false, true)]), rows)
}

fn assert_uses(world: &World) {
    for (number, expected) in
        ACCOUNTS.into_iter().zip([Some(vec![Use::Party(4)]), Some(vec![]), Some(vec![Use::Project(2)]), None])
    {
        assert_eq!(world.domain.account(number).expect("restored account").uses(), expected.as_deref());
    }
}

#[test]
fn dropping_a_project_removes_it_from_every_accounts_uses() {
    for phase in [0, 1, 2] {
        let mut world = restored(&rows());
        if phase > 0 {
            world.step(Event::Want);
        }
        if phase == 2 {
            world.step(Event::Close);
        }
        let out = world.step(Event::DropProject { project: 1 });
        assert_uses(&world);
        let expected: Vec<_> = rows()
            .into_iter()
            .filter_map(|mut record| {
                if let Stored::Account { number, uses, .. } = &mut record {
                    match *number {
                        10 => *uses = Some(Box::new([Use::Party(4)])),
                        20 => *uses = Some(Box::new([])),
                        _ => return None,
                    }
                    Some(Request::Save { record })
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(out, expected, "only changed accounts saved, in number order, phase {phase}");
        assert!(world.step(Event::DropProject { project: 1 }).is_empty(), "a repeated drop changes nothing");
    }
}

#[test]
fn an_account_with_no_use_left_serves_no_project() {
    let mut world = restored(&rows());
    world.step(Event::Want);
    world.step(Event::DropProject { project: 1 });
    let account = world.domain.account(20).expect("account with no uses left");
    assert_eq!(account.uses(), Some([].as_slice()));
    for payer in [Use::Project(1), Use::Project(2), Use::Party(4)] {
        let serves = account.uses().expect("explicit uses").contains(&payer);
        assert!(!serves);
        assert_eq!(
            world.domain.select(
                payer,
                PROVIDER,
                None,
                Some(20),
                None,
                &[Coverage { account: 20, serves }],
                &[],
                world.now
            ),
            Selection::NoAccount
        );
    }
}

#[test]
fn the_drop_survives_a_restart() {
    let mut rows = rows();
    let mut world = restored(&rows);
    world.step(Event::Want);
    for request in world.step(Event::DropProject { project: 1 }) {
        let Request::Save { record } = request else { panic!("a drop only saves accounts") };
        let row = rows.iter_mut().find(|row| row.key() == record.key()).expect("existing account record");
        *row = record;
    }
    let mut restarted = restored(&rows);
    restarted.step(Event::Want);
    assert_uses(&restarted);
}
