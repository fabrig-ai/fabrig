//! A run reads as its person under the asking family (`jig/docs/design/domain/engine.md`, 7.3).
use crate::settings_support::{ask, run_configuration, settings_run_with_limits};
use jig_core_world::fixtures;
use jig_domain_authority as authority;
use jig_domain_core::{self as core, reads};
use jig_domain_people as people;

#[test]
fn a_run_reads_as_its_person_only_with_the_asking_family() {
    let asking = core::ToolFamilies::JIG.ask.0;
    for asking_family in [true, false] {
        let (mut world, signed, task) = settings_run_with_limits(true, None, run_configuration().1);
        if asking_family {
            let mut granted = jig_test_profile::fixtures::people_member(b"Read as the person", 40).authority;
            granted.tools = 2047 | asking;
            let amendment = people::Amendment {
                authority: Some(granted),
                ..jig_test_profile::fixtures::person_unchanged(b"Read as the person")
            };
            let revision = fixtures::live_row(&world, task).revisions.spec;
            let reply = ask(
                &mut world,
                signed,
                102,
                people::Ask::Amend { project: 1, task, revision, amendment, release: None },
            );
            assert!(matches!(reply, people::Reply::Outcome(people::Outcome::Amended { .. })), "{reply:?}");
        }
        let row = fixtures::live_row(&world, task);
        assert_eq!(row.requester, jig_domain_tasks::Party::Person(crate::settings_support::OWNER));
        assert_eq!(row.authority.tools.0 & asking != 0, asking_family);
        let project = row.project;
        let rows = world.store.rows.clone();
        let applied = world.store.applied;
        let mut first = None;
        for _ in 0..2 {
            let token = world.person_read(reads::Read::Project { project });
            let key = if asking_family {
                let (to, key, answer) = world.person_answers.pop().expect("person read answer");
                assert_eq!(to, token);
                assert_eq!(answer, reads::Answer::None(reads::Why::Unknown));
                assert!(world.answers.is_empty());
                key
            } else {
                let (to, key, part) = world.answers.pop().expect("asking family refusal");
                assert_eq!(to, token);
                assert_eq!(
                    part,
                    core::CallPart::ToolDenied {
                        answer: authority::Answer::Propose,
                        findings: Box::new([authority::Finding::Tool]),
                    }
                );
                assert!(world.person_answers.is_empty());
                key
            };
            assert_eq!(key.task, task);
            if let Some(first) = first {
                assert_eq!(key, first, "the same call name is answered again");
            } else {
                first = Some(key);
            }
            assert_eq!(world.store.rows, rows);
            assert_eq!(world.store.applied, applied, "reads commit nothing");
        }
        world.broken();
    }
}
