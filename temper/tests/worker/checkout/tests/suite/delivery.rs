//! Delivery ancestry through git's sequence and the shared fake's object graph.
use skein_lib::Token;
use temper_fake_checkout::git::{self as local, Remote};
use temper_worker_checkout_world::{
    LIMITS,
    hosted::{self, Scenario},
    translate,
};
use temper_worker_domain_checkout::git;
use temper_worker_protocol_git::{
    invocation::Command,
    sequence::{self, Next},
};

fn deliver(scenario: &mut Scenario, unchecked: bool) -> git::Done {
    let mut op = git::Op::Commit {
        always: false,
        at: git::Place { workspace: Token::new(1), repository: b"specimen".as_slice().into(), git: None },
        parent: translate::commit(scenario.base),
        merging: None,
        title: if unchecked {
            b"Unchecked: work in progress, saved before a question".as_slice().into()
        } else {
            b"Delivery".as_slice().into()
        },
        body: Box::default(),
        identity: 1,
        snapshot: Some(git::Snapshot { remote: scenario.remote.clone().into(), account: None }),
    };
    let mut next = sequence::first(&op);
    for _ in 0..16 {
        next = match next {
            Next::Run(command) => {
                let answer = hosted::apply(scenario, &command);
                sequence::after(&op, &command, answer.exit, &answer.stdout, &answer.stderr, 0, &LIMITS)
            }
            Next::Snapshot { parent, merging } => {
                let git::Op::Commit { parent: target, merging: second, .. } = &mut op else { unreachable!() };
                *target = parent;
                *second = merging;
                Next::Run(if merging.is_some() { Command::Unmerged } else { Command::Add })
            }
            Next::End(done) => return done,
            Next::Make | Next::Scan { .. } => panic!("these resolutions are staged"),
        };
    }
    panic!("delivery settles within its sequence");
}

#[test]
fn a_staged_merge_delivers_both_parents_and_a_committed_merge_keeps_its_id() {
    for unchecked in [false, true] {
        for committed in [false, true] {
            let mut scenario = hosted::scenario();
            let base = scenario.base;
            let topic = scenario.topic;
            let outcome = hosted::apply(&mut scenario, &Command::Merge { theirs: translate::commit(topic) });
            assert_eq!(outcome.exit, 0);
            hosted::apply(&mut scenario, &Command::Add);
            let existing = if committed {
                Some(
                    local::commit_merging(
                        &mut scenario.transport,
                        &mut scenario.checkout,
                        &scenario.at,
                        base,
                        topic,
                        b"Producer's merge",
                    )
                    .expect("staged merge"),
                )
            } else {
                None
            };
            let git::Done::Committed { commit } = deliver(&mut scenario, unchecked) else { panic!("delivered merge") };
            let delivered = translate::fake(commit);
            assert_eq!(scenario.transport.parent(delivered), Some(base));
            assert_eq!(scenario.transport.merge_parent(delivered), Some(topic));
            if let Some(existing) = existing {
                assert_eq!(delivered, existing);
                assert_eq!(scenario.transport.message(delivered), b"Producer's merge");
            }
            assert_eq!(scenario.checkout.content(&[scenario.at.as_slice(), b"/.git/MERGE_HEAD"].concat()), None);
        }
    }
}

#[test]
fn a_producers_local_commit_is_kept_when_its_tree_is_delivered() {
    for unchecked in [false, true] {
        let mut scenario = hosted::scenario();
        scenario.checkout.write(&[scenario.at.as_slice(), b"/new"].concat(), b"checked work");
        let existing = local::commit(
            &mut scenario.transport,
            &mut scenario.checkout,
            &scenario.at,
            scenario.base,
            b"Producer's commit",
        )
        .expect("local parent")
        .expect("new file");
        assert_eq!(deliver(&mut scenario, unchecked), git::Done::Committed { commit: translate::commit(existing) });
        assert_eq!(scenario.transport.message(existing), b"Producer's commit");
    }
}

#[test]
fn an_unpushed_second_parent_refuses_a_staged_or_committed_merge() {
    for unchecked in [false, true] {
        for committed in [false, true] {
            let mut scenario = hosted::scenario();
            scenario.checkout.write(&[scenario.at.as_slice(), b"/local"].concat(), b"local only");
            let second = local::commit(
                &mut scenario.transport,
                &mut scenario.checkout,
                &scenario.at,
                scenario.base,
                b"Unpushed",
            )
            .expect("local parent")
            .expect("new file");
            let base = scenario.base;
            hosted::apply(&mut scenario, &Command::CheckOut { commit: translate::commit(base) });
            hosted::apply(&mut scenario, &Command::Merge { theirs: translate::commit(second) });
            hosted::apply(&mut scenario, &Command::Add);
            if committed {
                local::commit_merging(
                    &mut scenario.transport,
                    &mut scenario.checkout,
                    &scenario.at,
                    base,
                    second,
                    b"Local merge",
                )
                .expect("staged merge");
            }
            let git::Done::FailedWithOutput { fault, diagnostic } = deliver(&mut scenario, unchecked) else {
                panic!("refused merge")
            };
            assert_eq!(fault, git::Fault::Refused);
            assert_eq!(diagnostic.output(), b"delivery second parent is not reachable from the branch's remote");
        }
    }
}
