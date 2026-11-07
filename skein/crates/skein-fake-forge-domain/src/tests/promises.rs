//! Hand-built decoded streams, judged through the public API alone.

use alloc::boxed::Box;
use skein_lib::{List, bytes::copy_of};

use crate::api::{Error, Kind, QueueRemoval, Verdict as ReviewVerdict};
use crate::promises::{Allowance, Bounds, Breach, Decision, Judge, Profile, Promise, Recorded, Verdict, Why};
use crate::{Observation, Operation};

const CLIENT: u64 = 11;
const PERSON: u64 = 22;
const REPOSITORY: &[u8] = b"owner/repo";
const OTHER: &[u8] = b"owner/other";
const HEAD: u64 = 30;
const COMMIT: u64 = 40;
// Room for every item, key, label and piece of evidence in a test stream.
const OBJECTS: u32 = 16;
const DECISIONS: u32 = 8;

fn profile() -> Profile {
    Profile {
        client: CLIENT,
        marker: (copy_of(b"<key>"), copy_of(b"</key>")),
        owned_branches: Box::new([copy_of(b"bot/")]),
        allowed: Box::new([]),
        bounds: Bounds { objects: OBJECTS, decisions: DECISIONS },
    }
}

fn judge() -> Judge {
    Judge::new(profile())
}

fn observe(judge: &mut Judge, observation: Observation) {
    judge.observe(&Recorded::Observed { observation, forward: None });
}

fn finish(judge: &mut Judge) -> Verdict {
    judge.observe(&Recorded::Lost(0));
    judge.decided();
    judge.verdict()
}

fn breach(promise: Promise, subject: &[u8]) -> Breach {
    Breach { promise, subject: copy_of(subject) }
}

fn fails(promise: Promise, subject: &[u8]) -> Verdict {
    Verdict::Fail(Box::new([breach(promise, subject)]))
}

fn opened(repository: &[u8], number: u64, kind: Kind, body: &[u8], by: u64) -> Observation {
    Observation::Opened {
        repository: copy_of(repository),
        number,
        kind,
        title: copy_of(b"title"),
        body: copy_of(body),
        labels: Box::new([]),
        branches: None,
        by,
    }
}

fn pull(number: u64, by: u64) -> Observation {
    opened(REPOSITORY, number, Kind::Pull, b"", by)
}

fn comment(number: u64, id: u64, body: &[u8], by: u64) -> Observation {
    Observation::Commented { repository: copy_of(REPOSITORY), number, id, body: copy_of(body), by }
}

fn revised(number: u64, by: u64) -> Observation {
    Observation::Revised { repository: copy_of(REPOSITORY), number, title: copy_of(b"revised"), body: copy_of(b""), by }
}

fn edited(id: u64, by: u64) -> Observation {
    Observation::Edited { repository: copy_of(REPOSITORY), number: 1, id, body: copy_of(b"edit"), by }
}

fn labelled(number: u64, labels: &[&[u8]], by: u64) -> Observation {
    let mut names = List::with_capacity(u32::try_from(labels.len()).unwrap());
    for label in labels {
        names.push(copy_of(label)).unwrap();
    }
    Observation::Labelled { repository: copy_of(REPOSITORY), number, labels: names.into_boxed(), by }
}

fn moved(judge: &mut Judge, branch: &[u8], from: Option<u64>, to: u64, by: u64, forward: Option<bool>) {
    judge.observe(&Recorded::Observed {
        observation: Observation::Moved { repository: copy_of(REPOSITORY), branch: copy_of(branch), from, to, by },
        forward,
    });
}

fn deleted(branch: &[u8], at: u64, by: u64) -> Observation {
    Observation::Deleted { repository: copy_of(REPOSITORY), branch: copy_of(branch), at, by }
}

fn merged(number: u64, head: u64, by: u64) -> Observation {
    Observation::Merged { repository: copy_of(REPOSITORY), number, base: copy_of(b"main"), head, commit: COMMIT, by }
}

fn enqueued(number: u64, head: u64) -> Observation {
    Observation::Enqueued { repository: copy_of(REPOSITORY), number, head, position: 1, by: CLIENT }
}

fn decision(number: u64, head: u64) -> Decision {
    Decision { repository: copy_of(REPOSITORY), number, head }
}

fn group(ended: bool, landed: bool) -> Observation {
    Observation::Group { repository: copy_of(REPOSITORY), number: 1, commit: COMMIT, ended, landed }
}

#[test]
fn duplicate_client_markers_breach_once_within_their_namespace() {
    skein_rows::check(
        [
            ("once_fails_on_two_pulls_with_one_key", {
                let mut row = PromiseRow::new(fails(Promise::Once, b"owner%2Frepo/pull/same"));
                for number in [1, 2] {
                    row.before
                        .push(recorded(opened(REPOSITORY, number, Kind::Pull, b"<key>same</key>", CLIENT)))
                        .expect("bounded stream fixture");
                }
                row
            }),
            ("once_fails_on_two_comments_with_one_key", {
                let mut row = PromiseRow::new(fails(Promise::Once, b"owner%2Frepo/comment/same"));
                row.before.push(recorded(pull(1, CLIENT))).expect("bounded stream fixture");
                for id in [1, 2] {
                    row.before
                        .push(recorded(comment(1, id, b"<key>same</key>", CLIENT)))
                        .expect("bounded stream fixture");
                }
                row
            }),
        ],
        promised,
    );
}

#[test]
fn once_passes_one_key_across_kinds_and_repositories() {
    let mut judge = judge();
    for entry in [
        opened(REPOSITORY, 1, Kind::Pull, b"<key>same</key>", CLIENT),
        opened(REPOSITORY, 2, Kind::Issue, b"<key>same</key>", CLIENT),
        opened(OTHER, 1, Kind::Pull, b"<key>same</key>", CLIENT),
        opened(REPOSITORY, 3, Kind::Pull, b"<key>same</key>", PERSON),
        opened(REPOSITORY, 4, Kind::Pull, b"<key>same", CLIENT),
    ] {
        observe(&mut judge, entry);
    }
    assert_eq!(finish(&mut judge), Verdict::Pass);
}

#[test]
fn once_uses_the_first_marker_and_next_close() {
    let mut judge = judge();
    for (number, body) in [(1, &b"<key>first</key><key>second</key>"[..]), (2, b"<key>first</key>")] {
        observe(&mut judge, opened(REPOSITORY, number, Kind::Issue, body, CLIENT));
    }
    assert_eq!(finish(&mut judge), fails(Promise::Once, b"owner%2Frepo/issue/first"));
}

#[test]
fn nonforward_client_pushes_breach_the_forced_promise() {
    skein_rows::check(
        [
            ("forced_fails_on_a_move_that_does_not_descend", {
                let mut row = PromiseRow::new(fails(Promise::Forced, b"owner%2Frepo/bot%2Fwork"));
                row.before
                    .push(movement(b"bot/work", Some(HEAD), COMMIT, CLIENT, Some(false)))
                    .expect("bounded stream fixture");
                row
            }),
            ("forced_fails_on_a_remake_that_does_not_descend", {
                let mut row = PromiseRow::new(fails(Promise::Forced, b"owner%2Frepo/work"));
                row.before.push(movement(b"work", None, HEAD, CLIENT, None)).expect("bounded stream fixture");
                row.before.push(recorded(deleted(b"work", HEAD, CLIENT))).expect("bounded stream fixture");
                row.before.push(movement(b"work", None, COMMIT, CLIENT, Some(false))).expect("bounded stream fixture");
                row
            }),
            ("forced_fails_on_a_rejected_push", {
                let mut row = PromiseRow::new(fails(Promise::Forced, b"owner%2Frepo/work"));
                row.before
                    .push(recorded(Observation::Rejected {
                        repository: copy_of(REPOSITORY),
                        branch: copy_of(b"work"),
                        commit: HEAD,
                        by: CLIENT,
                    }))
                    .expect("bounded stream fixture");
                row
            }),
        ],
        promised,
    );
}

#[test]
fn forced_passes_forward_moves_and_another_users_force() {
    let mut judge = judge();
    moved(&mut judge, b"work", None, HEAD, CLIENT, None);
    moved(&mut judge, b"work", Some(HEAD), COMMIT, CLIENT, Some(true));
    moved(&mut judge, b"other", Some(HEAD), COMMIT, PERSON, Some(false));
    observe(&mut judge, deleted(b"work", COMMIT, CLIENT));
    moved(&mut judge, b"work", None, COMMIT, CLIENT, Some(true));
    assert_eq!(finish(&mut judge), Verdict::Pass);
}

#[test]
#[expect(clippy::too_many_lines, reason = "the recorded foreign-ownership streams share one verdict body")]
fn unallowed_client_writes_breach_foreign_ownership_promises() {
    skein_rows::check(
        [
            ("foreign_fails_on_a_write_to_another_users_item", {
                let mut row = PromiseRow::new(fails(Promise::Foreign, b"owner%2Frepo/item/1"));
                row.before.push(recorded(pull(1, PERSON))).expect("bounded stream fixture");
                row.before.push(recorded(revised(1, CLIENT))).expect("bounded stream fixture");
                row
            }),
            ("foreign_fails_on_a_move_of_an_unowned_branch", {
                let mut row = PromiseRow::new(fails(Promise::Foreign, b"owner%2Frepo/branch/work"));
                row.before
                    .push(movement(b"work", Some(HEAD), COMMIT, CLIENT, Some(true)))
                    .expect("bounded stream fixture");
                row
            }),
            ("foreign_fails_on_a_label_outside_the_allowed_prefixes: 1", {
                let mut row = PromiseRow::new(fails(Promise::Foreign, b"owner%2Frepo/item/1"));
                let labels = &[&b"bot/ready"[..], b"person/keep"][..];

                row.profile.allowed = Box::new([Allowance::Labels(copy_of(b"bot/"))]);

                row.before.push(recorded(pull(1, PERSON))).expect("bounded stream fixture");
                row.before.push(recorded(labelled(1, &[b"person/old"], PERSON))).expect("bounded stream fixture");
                row.before.push(recorded(labelled(1, labels, CLIENT))).expect("bounded stream fixture");
                row
            }),
            ("foreign_fails_on_a_label_outside_the_allowed_prefixes: 2", {
                let mut row = PromiseRow::new(fails(Promise::Foreign, b"owner%2Frepo/item/1"));
                let labels = &[];

                row.profile.allowed = Box::new([Allowance::Labels(copy_of(b"bot/"))]);

                row.before.push(recorded(pull(1, PERSON))).expect("bounded stream fixture");
                row.before.push(recorded(labelled(1, &[b"person/old"], PERSON))).expect("bounded stream fixture");
                row.before.push(recorded(labelled(1, labels, CLIENT))).expect("bounded stream fixture");
                row
            }),
            ("foreign_fails_on_an_edit_over_another_users_edit: 1", {
                let (entry, subject) = (revised(1, CLIENT), &b"owner%2Frepo/item/1"[..]);
                let mut row = PromiseRow::new(fails(Promise::Foreign, subject));

                row.before.push(recorded(pull(1, CLIENT))).expect("bounded stream fixture");
                row.before.push(recorded(comment(1, 1, b"mine", CLIENT))).expect("bounded stream fixture");
                row.before.push(recorded(revised(1, PERSON))).expect("bounded stream fixture");
                row.before.push(recorded(edited(1, PERSON))).expect("bounded stream fixture");
                row.before.push(recorded(entry)).expect("bounded stream fixture");
                row
            }),
            ("foreign_fails_on_an_edit_over_another_users_edit: 2", {
                let (entry, subject) = (edited(1, CLIENT), &b"owner%2Frepo/comment/1"[..]);
                let mut row = PromiseRow::new(fails(Promise::Foreign, subject));

                row.before.push(recorded(pull(1, CLIENT))).expect("bounded stream fixture");
                row.before.push(recorded(comment(1, 1, b"mine", CLIENT))).expect("bounded stream fixture");
                row.before.push(recorded(revised(1, PERSON))).expect("bounded stream fixture");
                row.before.push(recorded(edited(1, PERSON))).expect("bounded stream fixture");
                row.before.push(recorded(entry)).expect("bounded stream fixture");
                row
            }),
            ("foreign_fails_each_unallowed_item_write: 1", {
                let mut row = PromiseRow::new(fails(Promise::Foreign, b"owner%2Frepo/item/1"));
                let entry = Observation::Depends {
                    repository: copy_of(REPOSITORY),
                    number: 1,
                    dependencies: Box::new([]),
                    by: CLIENT,
                };

                row.before.push(recorded(entry)).expect("bounded stream fixture");
                row.decisions.push(decision(1, HEAD)).expect("bounded stream fixture");
                row
            }),
            ("foreign_fails_each_unallowed_item_write: 2", {
                let mut row = PromiseRow::new(fails(Promise::Foreign, b"owner%2Frepo/item/1"));
                let entry = Observation::Requested {
                    repository: copy_of(REPOSITORY),
                    number: 1,
                    reviewers: Box::new([]),
                    by: CLIENT,
                };

                row.before.push(recorded(entry)).expect("bounded stream fixture");
                row.decisions.push(decision(1, HEAD)).expect("bounded stream fixture");
                row
            }),
            ("foreign_fails_each_unallowed_item_write: 3", {
                let mut row = PromiseRow::new(fails(Promise::Foreign, b"owner%2Frepo/item/1"));
                let entry = Observation::Reviewed {
                    repository: copy_of(REPOSITORY),
                    number: 1,
                    id: 1,
                    commit: HEAD,
                    verdict: ReviewVerdict::Approve,
                    body: copy_of(b""),
                    by: CLIENT,
                };

                row.before.push(recorded(entry)).expect("bounded stream fixture");
                row.decisions.push(decision(1, HEAD)).expect("bounded stream fixture");
                row
            }),
            ("foreign_fails_each_unallowed_item_write: 4", {
                let mut row = PromiseRow::new(fails(Promise::Foreign, b"owner%2Frepo/item/1"));
                let entry = Observation::Closed { repository: copy_of(REPOSITORY), number: 1, by: CLIENT };

                row.before.push(recorded(entry)).expect("bounded stream fixture");
                row.decisions.push(decision(1, HEAD)).expect("bounded stream fixture");
                row
            }),
            ("foreign_fails_each_unallowed_item_write: 5", {
                let mut row = PromiseRow::new(fails(Promise::Foreign, b"owner%2Frepo/item/1"));
                let entry = Observation::Reopened { repository: copy_of(REPOSITORY), number: 1, by: CLIENT };

                row.before.push(recorded(entry)).expect("bounded stream fixture");
                row.decisions.push(decision(1, HEAD)).expect("bounded stream fixture");
                row
            }),
            ("foreign_fails_each_unallowed_item_write: 6", {
                let mut row = PromiseRow::new(fails(Promise::Foreign, b"owner%2Frepo/item/1"));
                let entry = comment(1, 1, b"", CLIENT);

                row.before.push(recorded(entry)).expect("bounded stream fixture");
                row.decisions.push(decision(1, HEAD)).expect("bounded stream fixture");
                row
            }),
            ("foreign_fails_each_unallowed_item_write: 7", {
                let mut row = PromiseRow::new(fails(Promise::Foreign, b"owner%2Frepo/item/1"));
                let entry = enqueued(1, HEAD);

                row.before.push(recorded(entry)).expect("bounded stream fixture");
                row.decisions.push(decision(1, HEAD)).expect("bounded stream fixture");
                row
            }),
            ("foreign_fails_each_unallowed_item_write: 8", {
                let mut row = PromiseRow::new(fails(Promise::Foreign, b"owner%2Frepo/item/1"));
                let entry = merged(1, HEAD, CLIENT);

                row.before.push(recorded(entry)).expect("bounded stream fixture");
                row.decisions.push(decision(1, HEAD)).expect("bounded stream fixture");
                row
            }),
            ("foreign_fails_on_preexisting_comment_edits_and_removals: 1", {
                let mut row = PromiseRow::new(fails(Promise::Foreign, b"owner%2Frepo/comment/1"));
                let entry = edited(1, CLIENT);

                row.before.push(recorded(entry)).expect("bounded stream fixture");
                row
            }),
            ("foreign_fails_on_preexisting_comment_edits_and_removals: 2", {
                let mut row = PromiseRow::new(fails(Promise::Foreign, b"owner%2Frepo/comment/1"));
                let entry = Observation::Removed { repository: copy_of(REPOSITORY), number: 1, id: 1, by: CLIENT };

                row.before.push(recorded(entry)).expect("bounded stream fixture");
                row
            }),
            ("foreign_fails_on_an_unowned_deletion_at_the_stream_end", {
                let mut row = PromiseRow::new(fails(Promise::Foreign, b"owner%2Frepo/branch/work"));
                row.before.push(recorded(deleted(b"work", HEAD, CLIENT))).expect("bounded stream fixture");
                row
            }),
            ("foreign_does_not_exempt_an_ordinary_queue_named_branch", {
                let mut row =
                    PromiseRow::new(fails(Promise::Foreign, b"owner%2Frepo/branch/gh-readonly-queue%2Fmain%2Fpr-1-30"));
                row.before
                    .push(movement(b"gh-readonly-queue/main/pr-1-30", Some(HEAD), COMMIT, CLIENT, Some(true)))
                    .expect("bounded stream fixture");
                row
            }),
        ],
        promised,
    );
}

#[test]
fn foreign_passes_allowed_writes_and_owned_objects() {
    let mut profile = profile();
    profile.allowed = Box::new([Allowance::Comment, Allowance::Labels(copy_of(b"bot/"))]);
    let mut judge = Judge::new(profile);
    observe(&mut judge, pull(2, PERSON));
    observe(&mut judge, comment(2, 1, b"allowed", CLIENT));
    observe(&mut judge, labelled(2, &[b"bot/ready"], CLIENT));
    observe(&mut judge, labelled(2, &[], CLIENT));
    moved(&mut judge, b"bot/work", Some(HEAD), COMMIT, CLIENT, Some(true));
    moved(&mut judge, b"mine", None, HEAD, CLIENT, None);
    moved(&mut judge, b"mine", Some(HEAD), COMMIT, CLIENT, Some(true));
    observe(&mut judge, deleted(b"mine", COMMIT, CLIENT));
    observe(&mut judge, pull(1, CLIENT));
    observe(&mut judge, revised(1, CLIENT));
    observe(&mut judge, edited(1, CLIENT));
    judge.decide(&decision(1, HEAD));
    observe(&mut judge, merged(1, HEAD, CLIENT));
    moved(&mut judge, b"main", Some(HEAD), COMMIT, CLIENT, Some(true));
    // queue.rs: Group then Moved for creation, Deleted then Group for ending.
    observe(&mut judge, enqueued(1, HEAD));
    observe(&mut judge, group(false, false));
    moved(&mut judge, b"gh-readonly-queue/main/pr-1-30", None, COMMIT, CLIENT, None);
    observe(&mut judge, deleted(b"gh-readonly-queue/main/pr-1-30", COMMIT, CLIENT));
    observe(&mut judge, group(true, true));
    observe(&mut judge, labelled(3, &[b"person/baseline"], CLIENT));
    assert_eq!(finish(&mut judge), Verdict::Pass);
}

#[test]
fn foreign_passes_each_named_item_allowance() {
    let mut profile = profile();
    profile.allowed = Box::new([Allowance::Request, Allowance::Review, Allowance::Close, Allowance::Merge]);
    let mut judge = Judge::new(profile);
    observe(&mut judge, pull(1, PERSON));
    for entry in [
        Observation::Requested {
            repository: copy_of(REPOSITORY),
            number: 1,
            reviewers: Box::new([PERSON]),
            by: CLIENT,
        },
        Observation::Reviewed {
            repository: copy_of(REPOSITORY),
            number: 1,
            id: 1,
            commit: HEAD,
            verdict: ReviewVerdict::Approve,
            body: copy_of(b""),
            by: CLIENT,
        },
        Observation::Closed { repository: copy_of(REPOSITORY), number: 1, by: CLIENT },
        Observation::Reopened { repository: copy_of(REPOSITORY), number: 1, by: CLIENT },
        enqueued(1, HEAD),
        merged(1, HEAD, CLIENT),
    ] {
        observe(&mut judge, entry);
    }
    judge.decide(&decision(1, HEAD));
    assert_eq!(finish(&mut judge), Verdict::Pass);
}

#[test]
fn dependencies_do_not_overwrite_the_item_body() {
    let mut judge = judge();
    observe(&mut judge, pull(1, CLIENT));
    observe(&mut judge, revised(1, PERSON));
    observe(
        &mut judge,
        Observation::Depends { repository: copy_of(REPOSITORY), number: 1, dependencies: Box::new([]), by: CLIENT },
    );
    assert_eq!(finish(&mut judge), Verdict::Pass);
}

#[test]
fn merge_order_requires_matching_repository_item_and_head_decisions() {
    skein_rows::check(
        [
            ("order_fails_on_a_merge_without_its_decision", {
                let mut row = PromiseRow::new(fails(Promise::Order, b"owner%2Frepo/1/30"));
                row.before.push(recorded(pull(1, CLIENT))).expect("bounded stream fixture");
                row.before.push(recorded(merged(1, HEAD, CLIENT))).expect("bounded stream fixture");
                row.decisions.push(decision(2, HEAD)).expect("bounded stream fixture");
                row
            }),
            ("order_fails_on_an_enqueue_decided_at_another_head", {
                let mut row = PromiseRow::new(fails(Promise::Order, b"owner%2Frepo/1/30"));
                row.before.push(recorded(pull(1, CLIENT))).expect("bounded stream fixture");
                row.decisions.push(decision(1, COMMIT)).expect("bounded stream fixture");
                row.after.push(recorded(enqueued(1, HEAD))).expect("bounded stream fixture");
                row
            }),
            ("order_fails_on_a_merge_with_no_decisions_given", {
                let mut row = PromiseRow::new(fails(Promise::Order, b"owner%2Frepo/1/30"));
                row.before.push(recorded(pull(1, CLIENT))).expect("bounded stream fixture");
                row.before.push(recorded(merged(1, HEAD, CLIENT))).expect("bounded stream fixture");
                row
            }),
            ("order_does_not_use_another_repositorys_decision", {
                let mut row = PromiseRow::new(fails(Promise::Order, b"owner%2Frepo/1/30"));
                row.before.push(recorded(pull(1, CLIENT))).expect("bounded stream fixture");
                row.decisions
                    .push(Decision { repository: copy_of(OTHER), number: 1, head: HEAD })
                    .expect("bounded stream fixture");
                row.after.push(recorded(enqueued(1, HEAD))).expect("bounded stream fixture");
                row
            }),
        ],
        promised,
    );
}

#[test]
fn order_passes_merges_and_enqueues_decided_at_their_heads() {
    let mut judge = judge();
    observe(&mut judge, pull(1, CLIENT));
    observe(&mut judge, merged(1, HEAD, CLIENT));
    judge.decide(&decision(1, HEAD));
    observe(&mut judge, pull(2, CLIENT));
    judge.decide(&decision(2, HEAD));
    observe(&mut judge, enqueued(2, HEAD));
    observe(&mut judge, merged(2, HEAD, CLIENT));
    assert_eq!(finish(&mut judge), Verdict::Pass);
}

#[test]
fn a_stream_with_lost_entries_is_undecided() {
    let mut judge = judge();
    moved(&mut judge, b"bot/work", Some(HEAD), COMMIT, CLIENT, Some(false));
    judge.observe(&Recorded::Lost(1));
    judge.observe(&Recorded::Lost(3));
    judge.decided();
    assert_eq!(
        judge.verdict(),
        Verdict::Undecided(Why::Lost(3), Box::new([breach(Promise::Forced, b"owner%2Frepo/bot%2Fwork")]))
    );
}

#[test]
fn a_stream_without_its_final_count_is_undecided() {
    let mut judge = judge();
    judge.observe(&Recorded::Lost(0));
    observe(&mut judge, pull(1, CLIENT));
    assert_eq!(judge.verdict(), Verdict::Undecided(Why::Truncated, Box::new([])));
}

#[test]
fn order_without_decisions_is_undecided() {
    let mut judge = judge();
    observe(&mut judge, pull(1, CLIENT));
    observe(&mut judge, merged(1, HEAD, CLIENT));
    judge.observe(&Recorded::Lost(0));
    assert_eq!(
        judge.verdict(),
        Verdict::Undecided(Why::NoDecisions, Box::new([breach(Promise::Order, b"owner%2Frepo/1/30")]))
    );
}

#[test]
fn object_tables_distinguish_capacity_from_one_additional_object() {
    skein_rows::check(
        [
            ("a_table_past_its_bound_is_undecided", (2, Verdict::Undecided(Why::Full, Box::new([])))),
            ("a_table_at_its_bound_passes", (1, Verdict::Pass)),
        ],
        |(count, expected)| {
            let mut profile = profile();
            profile.bounds.objects = 1;
            let mut judge = Judge::new(profile);
            for number in 1..=count {
                observe(&mut judge, pull(number, CLIENT));
            }
            assert_eq!(finish(&mut judge), expected);
        },
    );
}

#[test]
fn per_repository_tables_have_separate_bounds() {
    let mut profile = profile();
    profile.bounds.objects = 2;
    let mut judge = Judge::new(profile);
    for repository in [REPOSITORY, OTHER] {
        for number in [1, 2] {
            observe(&mut judge, opened(repository, number, Kind::Pull, b"", CLIENT));
        }
    }
    assert_eq!(finish(&mut judge), Verdict::Pass);
}

#[test]
fn decisions_are_bounded_at_and_past_capacity() {
    for (count, expected) in [(1, Verdict::Pass), (2, Verdict::Undecided(Why::Full, Box::new([])))] {
        let mut profile = profile();
        profile.bounds.decisions = 1;
        let mut judge = Judge::new(profile);
        for number in 1..=count {
            judge.decide(&decision(number, HEAD));
        }
        assert_eq!(finish(&mut judge), expected);
    }
}

#[test]
fn a_move_without_forward_is_undecided() {
    let mut judge = judge();
    moved(&mut judge, b"bot/work", Some(HEAD), COMMIT, CLIENT, None);
    assert_eq!(finish(&mut judge), Verdict::Undecided(Why::Unforwarded, Box::new([])));
}

#[test]
fn breaches_print_as_promise_and_subject() {
    let mut judge = judge();
    moved(&mut judge, b"bot/\x80", Some(HEAD), COMMIT, CLIENT, Some(false));
    match finish(&mut judge) {
        Verdict::Fail(breaches) => assert_eq!(&*breaches[0].line(), b"promise/forced/owner%2Frepo/bot%2F%80"),
        Verdict::Pass | Verdict::Undecided(..) => panic!("a forced move fails"),
    }
    for (promise, expected) in [
        (Promise::Once, &b"promise/once/subject"[..]),
        (Promise::Foreign, b"promise/foreign/subject"),
        (Promise::Order, b"promise/order/subject"),
    ] {
        assert_eq!(&*breach(promise, b"subject").line(), expected);
    }
}

#[test]
fn one_breach_per_promise_and_subject() {
    let mut judge = judge();
    for _ in 0_u32..2 {
        moved(&mut judge, b"work", Some(HEAD), COMMIT, CLIENT, Some(false));
        observe(&mut judge, revised(1, CLIENT));
    }
    assert_eq!(
        finish(&mut judge),
        Verdict::Fail(Box::new([
            breach(Promise::Forced, b"owner%2Frepo/work"),
            breach(Promise::Foreign, b"owner%2Frepo/branch/work"),
            breach(Promise::Foreign, b"owner%2Frepo/item/1"),
        ]))
    );
}

#[test]
fn breach_order_follows_the_stream_across_repositories() {
    let mut judge = judge();
    for repository in [REPOSITORY, OTHER, REPOSITORY] {
        observe(
            &mut judge,
            Observation::Rejected {
                repository: copy_of(repository),
                branch: copy_of(b"work"),
                commit: HEAD,
                by: CLIENT,
            },
        );
    }
    assert_eq!(
        finish(&mut judge),
        Verdict::Fail(Box::new([
            breach(Promise::Forced, b"owner%2Frepo/work"),
            breach(Promise::Forced, b"owner%2Fother/work"),
        ]))
    );
}

#[test]
fn ignored_observations_carry_no_promises() {
    let mut judge = judge();
    for entry in [
        Observation::Defined { repository: copy_of(REPOSITORY), label: copy_of(b"foreign"), by: CLIENT },
        Observation::Reported {
            repository: copy_of(REPOSITORY),
            commit: HEAD,
            context: copy_of(b"ci"),
            state: crate::api::Check::Passed,
            by: CLIENT,
        },
        Observation::Wiki {
            repository: copy_of(REPOSITORY),
            name: copy_of(b"page"),
            content: Some(copy_of(b"text")),
            revision: 1,
            by: CLIENT,
        },
        Observation::Refused {
            repository: copy_of(REPOSITORY),
            what: Operation::Merge,
            number: Some(1),
            commit: Some(HEAD),
            error: Error::Stale,
            by: CLIENT,
        },
        Observation::Dequeued { repository: copy_of(REPOSITORY), number: 1, reason: QueueRemoval::CiFailure },
    ] {
        observe(&mut judge, entry);
    }
    assert_eq!(finish(&mut judge), Verdict::Pass);
}

#[test]
fn loss_wins_over_a_truncated_tail() {
    let mut judge = judge();
    judge.observe(&Recorded::Lost(2));
    observe(&mut judge, pull(1, CLIENT));
    assert_eq!(judge.verdict(), Verdict::Undecided(Why::Lost(2), Box::new([])));
}

#[test]
fn truncation_wins_over_a_full_table() {
    let mut profile = profile();
    profile.bounds.decisions = 0;
    let mut judge = Judge::new(profile);
    judge.decide(&decision(1, HEAD));
    assert_eq!(judge.verdict(), Verdict::Undecided(Why::Truncated, Box::new([])));
}

#[test]
fn a_full_table_wins_over_missing_decisions() {
    let mut profile = profile();
    profile.bounds.decisions = 0;
    let mut judge = Judge::new(profile);
    observe(&mut judge, pull(1, CLIENT));
    observe(&mut judge, enqueued(1, HEAD));
    judge.decide(&decision(1, HEAD));
    judge.observe(&Recorded::Lost(0));
    assert_eq!(
        judge.verdict(),
        Verdict::Undecided(Why::Full, Box::new([breach(Promise::Order, b"owner%2Frepo/1/30")]))
    );
}

#[test]
fn missing_decisions_win_over_a_missing_forward_answer() {
    let mut judge = judge();
    observe(&mut judge, pull(1, CLIENT));
    observe(&mut judge, enqueued(1, HEAD));
    moved(&mut judge, b"bot/work", Some(HEAD), COMMIT, CLIENT, None);
    judge.observe(&Recorded::Lost(0));
    assert_eq!(
        judge.verdict(),
        Verdict::Undecided(Why::NoDecisions, Box::new([breach(Promise::Order, b"owner%2Frepo/1/30")]))
    );
}

#[test]
fn each_ownership_and_label_table_refuses_one_past_its_bound() {
    let rows: Box<[Box<[Observation]>]> = Box::new([
        // Branches.
        Box::new([
            Observation::Moved {
                repository: copy_of(REPOSITORY),
                branch: copy_of(b"one"),
                from: None,
                to: HEAD,
                by: CLIENT,
            },
            Observation::Moved {
                repository: copy_of(REPOSITORY),
                branch: copy_of(b"two"),
                from: None,
                to: HEAD,
                by: CLIENT,
            },
        ]),
        // Comments.
        Box::new([pull(1, CLIENT), comment(1, 1, b"", CLIENT), comment(1, 2, b"", CLIENT)]),
        // Last editors of comments which predate the stream.
        Box::new([edited(1, PERSON), edited(2, PERSON)]),
        // Last editors of items which predate the stream.
        Box::new([revised(1, PERSON), revised(2, PERSON)]),
        // Removed labels remain remembered.
        Box::new([labelled(1, &[b"first"], PERSON), labelled(1, &[], PERSON), labelled(1, &[b"second"], PERSON)]),
        // A snapshot cannot hide excess labels.
        Box::new([labelled(1, &[b"first", b"second"], PERSON)]),
        // Repositories.
        Box::new([pull(1, CLIENT), opened(OTHER, 1, Kind::Pull, b"", CLIENT)]),
    ]);
    for row in &rows {
        let mut profile = profile();
        profile.bounds.objects = 1;
        let mut judge = Judge::new(profile);
        for entry in row {
            observe(&mut judge, entry.clone());
        }
        assert_eq!(finish(&mut judge), Verdict::Undecided(Why::Full, Box::new([])));
    }
}

#[test]
fn keys_and_evidence_are_bounded_too() {
    for (body, expected) in [
        (&b"<key>second</key>"[..], Verdict::Undecided(Why::Full, Box::new([]))),
        (b"<key>first</key>", fails(Promise::Once, b"owner%2Frepo/pull/first")),
    ] {
        let mut profile = profile();
        profile.bounds.objects = 1;
        let mut judge = Judge::new(profile);
        // Repeated identity isolates the key bound from the item bound.
        observe(&mut judge, opened(REPOSITORY, 1, Kind::Pull, b"<key>first</key>", CLIENT));
        observe(&mut judge, opened(REPOSITORY, 1, Kind::Pull, body, CLIENT));
        assert_eq!(finish(&mut judge), expected);
    }
}

#[test]
fn full_evidence_keeps_the_breaches_that_fit() {
    let mut profile = profile();
    profile.bounds.objects = 1;
    let mut judge = Judge::new(profile);
    moved(&mut judge, b"work", Some(HEAD), COMMIT, CLIENT, Some(false));
    assert_eq!(
        finish(&mut judge),
        Verdict::Undecided(Why::Full, Box::new([breach(Promise::Forced, b"owner%2Frepo/work")]))
    );
}

#[test]
fn pending_deletion_evidence_checks_its_bound() {
    let mut profile = profile();
    profile.bounds.objects = 1;
    let mut judge = Judge::new(profile);
    moved(&mut judge, b"bot/work", Some(HEAD), COMMIT, CLIENT, Some(false));
    observe(&mut judge, deleted(b"foreign", HEAD, CLIENT));
    assert_eq!(
        finish(&mut judge),
        Verdict::Undecided(Why::Full, Box::new([breach(Promise::Forced, b"owner%2Frepo/bot%2Fwork")]))
    );
}

#[test]
fn queue_adjacency_survives_a_checkpoint() {
    let mut judge = judge();
    observe(&mut judge, group(false, false));
    judge.observe(&Recorded::Lost(0));
    moved(&mut judge, b"gh-readonly-queue/main/pr-1-30", None, COMMIT, CLIENT, None);
    observe(&mut judge, deleted(b"gh-readonly-queue/main/pr-1-30", COMMIT, CLIENT));
    judge.observe(&Recorded::Lost(0));
    observe(&mut judge, group(true, false));
    assert_eq!(finish(&mut judge), Verdict::Pass);
}

#[test]
fn verdict_does_not_consume_a_pending_deletion() {
    let mut judge = judge();
    observe(&mut judge, deleted(b"group", COMMIT, CLIENT));
    judge.observe(&Recorded::Lost(0));
    assert_eq!(judge.verdict(), fails(Promise::Foreign, b"owner%2Frepo/branch/group"));
    observe(&mut judge, group(true, false));
    assert_eq!(finish(&mut judge), Verdict::Pass);
}

#[test]
fn a_rebase_exemption_requires_the_exact_following_move() {
    for (repository, branch, from, to, by) in [
        (OTHER, b"work".as_slice(), HEAD, COMMIT, CLIENT),
        (REPOSITORY, b"other", HEAD, COMMIT, CLIENT),
        (REPOSITORY, b"work", HEAD + 1, COMMIT, CLIENT),
        (REPOSITORY, b"work", HEAD, COMMIT + 1, CLIENT),
        (REPOSITORY, b"work", HEAD, COMMIT, PERSON),
    ] {
        let mut judge = judge();
        moved(&mut judge, b"work", None, HEAD, CLIENT, None);
        observe(
            &mut judge,
            Observation::Rebased { repository: copy_of(repository), branch: copy_of(branch), from, to, by },
        );
        moved(&mut judge, b"work", Some(HEAD), COMMIT, CLIENT, Some(false));
        assert_eq!(finish(&mut judge), fails(Promise::Forced, b"owner%2Frepo/work"));
    }
}

#[test]
fn an_intervening_observation_ends_a_rebase_exemption() {
    let mut judge = judge();
    moved(&mut judge, b"work", None, HEAD, CLIENT, None);
    observe(
        &mut judge,
        Observation::Rebased {
            repository: copy_of(REPOSITORY),
            branch: copy_of(b"work"),
            from: HEAD,
            to: COMMIT,
            by: CLIENT,
        },
    );
    observe(
        &mut judge,
        Observation::Reported {
            repository: copy_of(REPOSITORY),
            commit: COMMIT,
            context: copy_of(b"ci"),
            state: crate::api::Check::Passed,
            by: PERSON,
        },
    );
    moved(&mut judge, b"work", Some(HEAD), COMMIT, CLIENT, Some(false));
    assert_eq!(finish(&mut judge), fails(Promise::Forced, b"owner%2Frepo/work"));
}

struct PromiseRow {
    profile: Profile,
    before: List<Recorded>,
    decisions: List<Decision>,
    after: List<Recorded>,
    expected: Verdict,
}

impl PromiseRow {
    fn new(expected: Verdict) -> Self {
        Self {
            profile: profile(),
            before: List::with_capacity(OBJECTS),
            decisions: List::with_capacity(DECISIONS),
            after: List::with_capacity(OBJECTS),
            expected,
        }
    }
}

fn recorded(observation: Observation) -> Recorded {
    Recorded::Observed { observation, forward: None }
}

fn movement(branch: &[u8], from: Option<u64>, to: u64, by: u64, forward: Option<bool>) -> Recorded {
    Recorded::Observed {
        observation: Observation::Moved { repository: copy_of(REPOSITORY), branch: copy_of(branch), from, to, by },
        forward,
    }
}

fn promised(row: PromiseRow) {
    let mut judge = Judge::new(row.profile);
    for entry in row.before.into_boxed() {
        judge.observe(&entry);
    }
    for decision in row.decisions.into_boxed() {
        judge.decide(&decision);
    }
    for entry in row.after.into_boxed() {
        judge.observe(&entry);
    }
    assert_eq!(finish(&mut judge), row.expected);
}
