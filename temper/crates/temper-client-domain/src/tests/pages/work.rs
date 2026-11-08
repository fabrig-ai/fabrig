use crate::pages::work::{self, Belongs, Card, Change, Chat, Facts, Filters, Goal, Item, Where, Work};
use skein_lib::{List, Wall};

const NOW: u64 = 1_209_600;
const WEEK_AGO: u64 = 604_800;

const fn wall(seconds: u64) -> Wall {
    Wall::from_nanos(seconds.saturating_mul(1_000_000_000))
}

fn list<T: Clone>(items: &[T]) -> List<T> {
    List::from_boxed(items.into())
}

fn facts(goals: &[Goal], changes: &[Change], chats: &[Chat]) -> Facts {
    Facts { goals: list(goals), changes: list(changes), chats: list(chats) }
}

fn goal(task: u64, live: bool, activity: u64) -> Goal {
    Goal { task, live, activity: wall(activity) }
}

fn chat(task: u64, activity: u64, draft: bool) -> Chat {
    Chat { task, activity: wall(activity), draft }
}

fn queued(task: u64, place: u32, since: u64, belongs: Belongs) -> Change {
    Change { task, at: Where::Queued { place, since: wall(since) }, belongs }
}

fn landed(task: u64, at: u64, belongs: Belongs) -> Change {
    Change { task, at: Where::Landed { at: wall(at) }, belongs }
}

fn card(item: Item, changes: &[u64]) -> Card {
    Card { item, changes: list(changes) }
}

fn work(in_flight: &[Card], drafts: &[Card], landed: &[u64], filters: Filters) -> Work {
    Work { in_flight: list(in_flight), drafts: list(drafts), landed: list(landed), filters }
}

struct Case {
    name: &'static str,
    facts: Facts,
    wall: Wall,
    want: Work,
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table compares Work for each placement rule and the whole project")]
fn each_item_is_placed_by_its_own_facts() {
    let cases = [
        Case { name: "empty project", facts: Facts::default(), wall: wall(NOW), want: Work::default() },
        Case {
            name: "live goals get cards; ended goals still count",
            facts: facts(&[goal(1, true, 10), goal(2, false, 20)], &[], &[]),
            wall: wall(NOW),
            want: work(&[card(Item::Goal(1), &[])], &[], &[], Filters { goals: 2, ..Filters::default() }),
        },
        Case {
            name: "a live goal's queued change has no card of its own",
            facts: facts(&[goal(1, true, 10)], &[queued(11, 1, 20, Belongs::Goal(1))], &[]),
            wall: wall(NOW),
            want: work(&[card(Item::Goal(1), &[11])], &[], &[], Filters { goals: 1, changes: 1, ..Filters::default() }),
        },
        Case {
            name: "an ended goal's queued change has its own card",
            facts: facts(&[goal(1, false, 10)], &[queued(11, 1, 20, Belongs::Goal(1))], &[]),
            wall: wall(NOW),
            want: work(
                &[card(Item::Change(11), &[])],
                &[],
                &[],
                Filters { goals: 1, changes: 1, ..Filters::default() },
            ),
        },
        Case {
            name: "an unlisted goal's queued change has its own card",
            facts: facts(&[], &[queued(11, 1, 20, Belongs::Goal(1))], &[]),
            wall: wall(NOW),
            want: work(&[card(Item::Change(11), &[])], &[], &[], Filters { changes: 1, ..Filters::default() }),
        },
        Case {
            name: "a listed chat holds its queued change",
            facts: facts(&[], &[queued(11, 1, 20, Belongs::Chat(1))], &[chat(1, 10, false)]),
            wall: wall(NOW),
            want: work(&[card(Item::Chat(1), &[11])], &[], &[], Filters { changes: 1, chats: 1, ..Filters::default() }),
        },
        Case {
            name: "an unlisted chat's queued change has its own card",
            facts: facts(&[], &[queued(11, 1, 20, Belongs::Chat(1))], &[]),
            wall: wall(NOW),
            want: work(&[card(Item::Change(11), &[])], &[], &[], Filters { changes: 1, ..Filters::default() }),
        },
        Case {
            name: "a queued change belonging to nothing has its own card",
            facts: facts(&[], &[queued(11, 1, 20, Belongs::Nothing)], &[]),
            wall: wall(NOW),
            want: work(&[card(Item::Change(11), &[])], &[], &[], Filters { changes: 1, ..Filters::default() }),
        },
        Case {
            name: "a chat with only old landed work counts without an In flight card",
            facts: facts(&[], &[landed(11, WEEK_AGO - 1, Belongs::Chat(1))], &[chat(1, 10, false)]),
            wall: wall(NOW),
            want: work(&[], &[], &[], Filters { changes: 1, chats: 1, ..Filters::default() }),
        },
        Case {
            name: "a draft with a queued change belongs to both sections",
            facts: facts(&[], &[queued(11, 1, 20, Belongs::Chat(1))], &[chat(1, 10, true)]),
            wall: wall(NOW),
            want: work(
                &[card(Item::Chat(1), &[11])],
                &[card(Item::Draft(1), &[])],
                &[],
                Filters { drafts: 1, changes: 1, chats: 1, goals: 0 },
            ),
        },
        Case {
            name: "a draft alone is no chat with work; an empty chat gets no card",
            facts: facts(&[], &[], &[chat(1, 10, true), chat(2, 20, false)]),
            wall: wall(NOW),
            want: work(&[], &[card(Item::Draft(1), &[])], &[], Filters { drafts: 1, ..Filters::default() }),
        },
        Case {
            name: "a goal holds queued changes by place then all its landed changes newest first",
            facts: facts(
                &[goal(1, true, 10)],
                &[
                    landed(14, WEEK_AGO - 1, Belongs::Goal(1)),
                    queued(12, 2, 50, Belongs::Goal(1)),
                    landed(13, NOW, Belongs::Goal(1)),
                    queued(11, 1, 20, Belongs::Goal(1)),
                    landed(15, NOW + 1, Belongs::Goal(1)),
                ],
                &[],
            ),
            wall: wall(NOW),
            want: work(
                &[card(Item::Goal(1), &[11, 12, 15, 13, 14])],
                &[],
                &[13],
                Filters { goals: 1, changes: 5, ..Filters::default() },
            ),
        },
        Case {
            name: "a chat holds queued changes by place then landed changes newest first",
            facts: facts(
                &[],
                &[
                    landed(14, NOW - 1, Belongs::Chat(1)),
                    queued(12, 2, 50, Belongs::Chat(1)),
                    landed(13, NOW, Belongs::Chat(1)),
                    queued(11, 1, 20, Belongs::Chat(1)),
                ],
                &[chat(1, 10, false)],
            ),
            wall: wall(NOW),
            want: work(
                &[card(Item::Chat(1), &[11, 12, 13, 14])],
                &[],
                &[13, 14],
                Filters { changes: 4, chats: 1, ..Filters::default() },
            ),
        },
        Case {
            name: "In flight uses each card's own activity, never its changes' times or queue place",
            facts: facts(
                &[goal(1, true, 20)],
                &[
                    queued(11, 1, 80, Belongs::Goal(1)),
                    queued(12, 3, 30, Belongs::Nothing),
                    queued(13, 2, 5, Belongs::Chat(2)),
                ],
                &[chat(2, 40, false)],
            ),
            wall: wall(NOW),
            want: work(
                &[card(Item::Chat(2), &[13]), card(Item::Change(12), &[]), card(Item::Goal(1), &[11])],
                &[],
                &[],
                Filters { goals: 1, changes: 3, chats: 1, drafts: 0 },
            ),
        },
        Case {
            name: "In flight ties use the lower task across all three kinds",
            facts: facts(
                &[goal(3, true, 20)],
                &[queued(1, 2, 20, Belongs::Nothing), queued(11, 1, 90, Belongs::Chat(2))],
                &[chat(2, 20, false)],
            ),
            wall: wall(NOW),
            want: work(
                &[card(Item::Change(1), &[]), card(Item::Chat(2), &[11]), card(Item::Goal(3), &[])],
                &[],
                &[],
                Filters { goals: 1, changes: 2, chats: 1, drafts: 0 },
            ),
        },
        Case {
            name: "drafts use newest activity, then lower task",
            facts: facts(&[], &[], &[chat(3, 10, true), chat(2, 20, true), chat(1, 20, true)]),
            wall: wall(NOW),
            want: work(
                &[],
                &[card(Item::Draft(1), &[]), card(Item::Draft(2), &[]), card(Item::Draft(3), &[])],
                &[],
                Filters { drafts: 3, ..Filters::default() },
            ),
        },
        Case {
            name: "Landed includes wall",
            facts: facts(&[], &[landed(1, NOW, Belongs::Nothing)], &[]),
            wall: wall(NOW),
            want: work(&[], &[], &[1], Filters { changes: 1, ..Filters::default() }),
        },
        Case {
            name: "Landed includes exactly seven days ago",
            facts: facts(&[], &[landed(1, WEEK_AGO, Belongs::Nothing)], &[]),
            wall: wall(NOW),
            want: work(&[], &[], &[1], Filters { changes: 1, ..Filters::default() }),
        },
        Case {
            name: "Landed excludes one second before the week",
            facts: facts(&[], &[landed(1, WEEK_AGO - 1, Belongs::Nothing)], &[]),
            wall: wall(NOW),
            want: work(&[], &[], &[], Filters { changes: 1, ..Filters::default() }),
        },
        Case {
            name: "Landed excludes the future",
            facts: facts(&[], &[landed(1, NOW + 1, Belongs::Nothing)], &[]),
            wall: wall(NOW),
            want: work(&[], &[], &[], Filters { changes: 1, ..Filters::default() }),
        },
        Case {
            name: "Landed sorts newest first, then lower task",
            facts: facts(
                &[],
                &[
                    landed(1, NOW - 1, Belongs::Nothing),
                    landed(3, NOW, Belongs::Nothing),
                    landed(2, NOW, Belongs::Nothing),
                ],
                &[],
            ),
            wall: wall(NOW),
            want: work(&[], &[], &[2, 3, 1], Filters { changes: 3, ..Filters::default() }),
        },
        Case {
            name: "the week saturates at the epoch",
            facts: facts(&[], &[landed(1, 0, Belongs::Nothing)], &[]),
            wall: wall(1),
            want: work(&[], &[], &[1], Filters { changes: 1, ..Filters::default() }),
        },
        Case {
            name: "the week preserves nanoseconds",
            facts: Facts {
                changes: list(&[
                    Change { task: 1, at: Where::Landed { at: wall(WEEK_AGO) }, belongs: Belongs::Nothing },
                    Change {
                        task: 2,
                        at: Where::Landed { at: Wall::from_nanos(wall(WEEK_AGO).as_nanos() + 1) },
                        belongs: Belongs::Nothing,
                    },
                ]),
                ..Facts::default()
            },
            wall: Wall::from_nanos(wall(NOW).as_nanos() + 1),
            want: work(&[], &[], &[2], Filters { changes: 2, ..Filters::default() }),
        },
        Case {
            name: "first goal row wins, including ended before live",
            facts: facts(&[goal(1, false, 10), goal(1, true, 20)], &[queued(11, 1, 20, Belongs::Goal(1))], &[]),
            wall: wall(NOW),
            want: work(
                &[card(Item::Change(11), &[])],
                &[],
                &[],
                Filters { goals: 1, changes: 1, ..Filters::default() },
            ),
        },
        Case {
            name: "first live goal row wins, including its activity",
            facts: facts(&[goal(2, true, 10), goal(2, false, 30), goal(1, true, 20)], &[], &[]),
            wall: wall(NOW),
            want: work(
                &[card(Item::Goal(1), &[]), card(Item::Goal(2), &[])],
                &[],
                &[],
                Filters { goals: 2, ..Filters::default() },
            ),
        },
        Case {
            name: "first change row wins before sorting, including its owner and state",
            facts: facts(
                &[goal(1, true, 10)],
                &[queued(11, 2, 20, Belongs::Goal(1)), landed(11, NOW, Belongs::Chat(2))],
                &[chat(2, 30, false)],
            ),
            wall: wall(NOW),
            want: work(&[card(Item::Goal(1), &[11])], &[], &[], Filters { goals: 1, changes: 1, ..Filters::default() }),
        },
        Case {
            name: "first chat row wins, including draft and activity",
            facts: facts(
                &[],
                &[queued(11, 1, 50, Belongs::Chat(2)), queued(12, 2, 50, Belongs::Nothing)],
                &[chat(2, 10, true), chat(2, 90, false)],
            ),
            wall: wall(NOW),
            want: work(
                &[card(Item::Change(12), &[]), card(Item::Chat(2), &[11])],
                &[card(Item::Draft(2), &[])],
                &[],
                Filters { drafts: 1, goals: 0, changes: 2, chats: 1 },
            ),
        },
        Case {
            name: "task numbers are deduplicated within each kind alone",
            facts: facts(&[goal(1, true, 10)], &[queued(1, 1, 20, Belongs::Chat(1))], &[chat(1, 30, true)]),
            wall: wall(NOW),
            want: work(
                &[card(Item::Chat(1), &[1]), card(Item::Goal(1), &[])],
                &[card(Item::Draft(1), &[])],
                &[],
                Filters { drafts: 1, goals: 1, changes: 1, chats: 1 },
            ),
        },
        whole_project(),
    ];
    for case in cases {
        let before = case.facts.clone();
        assert_eq!(work::place(&case.facts, case.wall), case.want, "{}", case.name);
        assert_eq!(case.facts, before, "{} leaves its facts alone", case.name);
    }
}

// The whole story's work.html: two goals, an independent change, a chat and two drafts.
fn whole_project() -> Case {
    Case {
        name: "the whole small project",
        facts: facts(
            &[goal(1, true, 100), goal(2, true, 90)],
            &[
                queued(11, 1, 10, Belongs::Goal(1)),
                queued(12, 3, 80, Belongs::Nothing),
                queued(13, 2, 20, Belongs::Chat(3)),
                landed(14, NOW - 20, Belongs::Goal(1)),
                landed(15, NOW - 10, Belongs::Goal(2)),
                landed(16, NOW, Belongs::Nothing),
                landed(17, NOW - 30, Belongs::Chat(3)),
                landed(18, WEEK_AGO - 1, Belongs::Goal(2)),
                landed(19, WEEK_AGO - 2, Belongs::Nothing),
            ],
            &[chat(3, 70, false), chat(4, 60, true), chat(5, 50, true)],
        ),
        wall: wall(NOW),
        want: work(
            &[
                card(Item::Goal(1), &[11, 14]),
                card(Item::Goal(2), &[15, 18]),
                card(Item::Change(12), &[]),
                card(Item::Chat(3), &[13, 17]),
            ],
            &[card(Item::Draft(4), &[]), card(Item::Draft(5), &[])],
            &[16, 15, 14, 17],
            Filters { drafts: 2, goals: 2, changes: 9, chats: 1 },
        ),
    }
}
