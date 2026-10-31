//! Deleted project uses (jig/docs/design/domain/accounts.md, 4.2;
//! jig/docs/design/domain/accounts.md, 4.2 and 4.3).
use jig_accounts_world::{World, account, provider, providers, source};
use jig_domain_accounts::{Coverage, Event, Holder, Kind, Request, Selection, Stored, Use};

const PROJECT: u32 = 42;
const OTHER_PROJECT: u32 = 43;
const PROVIDER: u32 = 1;

fn row(number: u32, uses: Option<&[Use]>) -> Stored {
    let mut row = account(number, PROVIDER, Holder::Party(u64::from(number)), b"");
    if let Stored::Account { uses: kept, display, .. } = &mut row {
        *kept = uses.map(Box::from);
        *display = b"Kept name".as_slice().into();
    }
    row
}

fn fixture(rows: Vec<Stored>) -> World {
    let mut world = World::with(providers(vec![provider(PROVIDER, false, true)]), &[]);
    for row in rows {
        let Stored::Account { number, .. } = &row else { unreachable!("account fixtures") };
        let source = source(*number, number + 100, 0, Kind::Key);
        world.restore(vec![row, source]);
    }
    assert_eq!(world.domain.unrestored(), 0);
    world.step(Event::Want);
    world
}

#[test]
fn a_deleted_projects_use_leaves_an_account_with_others() {
    for uses in [
        vec![Use::Project(PROJECT), Use::Party(7), Use::Project(OTHER_PROJECT)],
        vec![Use::Party(7), Use::Project(PROJECT), Use::Project(OTHER_PROJECT)],
        vec![Use::Party(7), Use::Project(OTHER_PROJECT), Use::Project(PROJECT)],
    ] {
        let mut world = fixture(vec![row(1, Some(&uses)), row(2, Some(&[Use::Project(PROJECT), Use::Party(8)]))]);
        let first = [Use::Party(7), Use::Project(OTHER_PROJECT)];
        let second = [Use::Party(8)];
        assert_eq!(
            world.step(Event::ForgetProject { project: PROJECT }),
            vec![Request::Save { record: row(1, Some(&first)) }, Request::Save { record: row(2, Some(&second)) }]
        );
        assert_eq!(world.domain.account(1).expect("kept account").uses(), Some(first.as_slice()));
        assert_eq!(world.domain.account(2).expect("kept account").uses(), Some(second.as_slice()));
        let coverage = [1, 2].map(|number| Coverage {
            account: number,
            serves: world
                .domain
                .account(number)
                .expect("kept account")
                .uses()
                .expect("explicit uses")
                .contains(&Use::Project(PROJECT)),
        });
        for preferred in [None, Some(1)] {
            assert_eq!(
                world.domain.select(Use::Project(PROJECT), PROVIDER, None, preferred, None, &coverage, &[], world.now),
                Selection::NoAccount
            );
        }
    }
}

#[test]
fn an_account_whose_only_use_is_the_project_keeps_it() {
    let uses = [Use::Project(PROJECT)];
    let mut world = fixture(vec![row(1, Some(&uses))]);
    assert!(world.step(Event::ForgetProject { project: PROJECT }).is_empty());
    assert_eq!(world.domain.account(1).expect("kept account").uses(), Some(uses.as_slice()));
    // The core's current coverage excludes a project that cannot be a payer again.
    let coverage = [Coverage { account: 1, serves: false }];
    assert_eq!(
        world.domain.select(Use::Project(PROJECT), PROVIDER, None, Some(1), None, &coverage, &[], world.now),
        Selection::NoAccount
    );
}

#[test]
fn forgetting_twice_or_an_unused_project_saves_nothing() {
    let mut world =
        fixture(vec![row(1, Some(&[Use::Project(PROJECT), Use::Party(7)])), row(2, None), row(3, Some(&[]))]);
    assert!(world.step(Event::ForgetProject { project: OTHER_PROJECT }).is_empty());
    assert_eq!(
        world.step(Event::ForgetProject { project: PROJECT }),
        vec![Request::Save { record: row(1, Some(&[Use::Party(7)])) }]
    );
    assert!(world.step(Event::ForgetProject { project: PROJECT }).is_empty());
    assert!(world.step(Event::ForgetProject { project: OTHER_PROJECT }).is_empty());
    assert_eq!(world.domain.account(2).expect("default uses").uses(), None);
    assert_eq!(world.domain.account(3).expect("empty explicit uses").uses(), Some([].as_slice()));
}
