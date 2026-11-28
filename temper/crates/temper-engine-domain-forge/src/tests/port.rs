//! Cases that need no application state at the connector's edge.
use super::*;

fn release_project(project: u32) -> Event {
    Event::Ask(contract::Ask::Lifecycle(contract::lifecycle::Ask::ReleaseProject { project }))
}

#[test]
fn the_port_releases_a_project_after_erasing_what_the_forge_kept() {
    let mut d = domain();
    let row = adopted();
    let project = row.project;
    let repository = row.provider;
    outputs(&mut d, Event::Restore { record: Stored::Repository(row) });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    let release = outputs(&mut d, release_project(project));
    assert!(release.contains(&Request::Erase { key: Key::Repository(repository) }));
    assert!(d.repository(repository).is_none());
    assert_eq!(release.last(), Some(&project_released(project)));
    assert_eq!(outputs(&mut d, release_project(project)).as_ref(), [project_released(project)]);
}

#[test]
fn the_port_makes_a_released_outbox_entry_as_committed_does() {
    let mut legacy = domain();
    let mut port = domain();
    let entry = 11;
    for d in [&mut legacy, &mut port] {
        outputs(d, Event::Restore { record: Stored::Repository(adopted()) });
        outputs(d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
        outputs(d, Event::CatchUp);
        outputs(d, Event::Enqueue { entry: status(entry, 7) });
        assert!(ready(d).is_empty(), "the enqueue is not a release");
    }
    assert_eq!(
        outputs(&mut port, Event::Released(contract::Released::Make { entry })),
        outputs(&mut legacy, Event::Committed { entry }),
    );
    let made = ready(&mut port);
    assert!(!made.is_empty(), "the released entry was offered");
    assert_eq!(made, ready(&mut legacy));
}

#[test]
fn the_port_judges_a_setting_in_the_asking_step() {
    use contract::settings::{Ask, Edit, Item, Judged, Up};
    let mut d = domain();
    let owner = Token::new(11);
    let items = Box::new([Item { kind: 4, scope: 1, number: 1, edit: Edit::Remove }]);
    let out =
        outputs(&mut d, Event::Ask(contract::Ask::Settings(Ask::Judge { owner, revision: 2, proposal: None, items })));
    assert_eq!(
        out.as_ref(),
        [Request::Up(contract::Up::Settings(Up::Judged {
            owner,
            judged: Box::new([Judged::Accept { size: 0, charters: Box::new([]) }]),
        }))]
    );
}
