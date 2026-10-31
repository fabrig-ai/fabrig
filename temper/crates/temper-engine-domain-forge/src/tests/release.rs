//! Project adoption release (temper/docs/design/draft/playground-reset.md, 5).
use super::*;

fn release(d: &mut Domain, project: u32) -> Box<[Request]> {
    outputs(d, Event::ReleaseProject { project, request: Token::new(u64::from(project)) })
}

#[test]
fn release_takes_the_projects_repositories_out_of_every_list() {
    let mut d = domain();
    let home = adopted();
    let mut context = home.clone();
    context.provider.forge = 2;
    context.provider.repository = 3;
    context.home = false;
    context.role = Role::Context;
    for row in [home, context] {
        outputs(&mut d, Event::Restore { record: Stored::Repository(row) });
    }
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    outputs(&mut d, Event::Restore { record: Stored::Names { task: 7, resources: Box::new([branch()]) } });
    outputs(&mut d, Event::Unnamed { task: 7 });
    assert!(!d.readable_repositories(5).is_empty());
    let sent = release(&mut d, 5);
    assert!(d.readable_repositories(5).is_empty());
    assert!(d.writable_repositories(5).is_empty());
    assert!(d.context_repositories(5, 2).unwrap().is_empty());
    assert!(d.adopted_ids(1).is_empty());
    assert!(d.adopted_ids(2).is_empty());
    assert_eq!(d.home(5), None);
    assert_eq!(d.repository(REPO), None);
    assert_eq!(d.names(7), None);
    for repository in [REPO, skein_forge::api::Repository { forge: 2, repository: 3 }] {
        assert!(sent.contains(&Request::Erase { key: Key::Repository(repository) }));
        assert!(sent.contains(&Request::Erase { key: Key::Client(skein_forge_client::Key::Repository(repository)) }));
    }
    assert_eq!(sent.last(), Some(&Request::ProjectReleased { request: Token::new(5) }));
}

#[test]
fn release_of_an_empty_project_answers_at_once() {
    let mut d = domain();
    let other = adopted();
    outputs(&mut d, Event::Restore { record: Stored::Repository(other.clone()) });
    assert_eq!(release(&mut d, 6).as_ref(), &[Request::ProjectReleased { request: Token::new(6) }]);
    assert_eq!(d.repository(REPO), Some(&other));
}

#[test]
fn a_resent_release_is_idempotent() {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    let mut other = adopted();
    other.project = 6;
    other.provider.repository = 3;
    outputs(&mut d, Event::Restore { record: Stored::Repository(other.clone()) });
    let sent = release(&mut d, 5);
    for committed in [false, true] {
        let mut restarted = domain();
        if !committed {
            outputs(&mut restarted, Event::Restore { record: Stored::Repository(adopted()) });
        }
        outputs(&mut restarted, Event::Restore { record: Stored::Repository(other.clone()) });
        let resent = release(&mut restarted, 5);
        if committed {
            assert_eq!(resent.as_ref(), &[Request::ProjectReleased { request: Token::new(5) }]);
        } else {
            assert_eq!(resent, sent, "an uncommitted release repeats the same erasures");
        }
        assert_eq!(restarted.repository(REPO), None);
        assert_eq!(restarted.repository(other.provider), Some(&other));
    }
    assert_eq!(release(&mut d, 5).as_ref(), &[Request::ProjectReleased { request: Token::new(5) }]);
    assert!(sent.contains(&Request::Erase { key: Key::Repository(REPO) }));
}
