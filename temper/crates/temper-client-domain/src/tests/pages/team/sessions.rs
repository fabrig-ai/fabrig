//! Session grouping and Team's bounded activity reads (client/pages/team.md, 1).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "ordinary tests assemble activity pages and inspect requested reads"
)]
use super::*;
use crate::pages::team::sessions::{Line, group, retain};
use skein_lib::{Duration, Map, Time, Wall};

const MINUTE: u64 = 60_000_000_000;
const DAY: u64 = 1440 * MINUTE;
const WALL: u64 = 10 * DAY + 130 * MINUTE;
fn at(minutes: u64) -> Wall {
    Wall::from_nanos((9 * DAY).saturating_add(minutes.saturating_mul(MINUTE)))
}
fn began(attempt: u64) -> c::Activity {
    c::Activity::RunBegan { attempt, why: c::RunWhy::Ready, resumed: attempt == 3 }
}
fn ended(attempt: u64, spent: u64) -> c::Activity {
    c::Activity::RunEnded { attempt, end: c::RunEnd::Parked, spent }
}
fn boundary(attempt: u64) -> c::Activity {
    c::Activity::SessionEnded { attempt, end: c::SessionEnd::Quiet }
}
fn activity(position: u64, at: Wall, line: c::Activity) -> c::DecodedActivityLine {
    c::DecodedActivityLine { source: Box::new(c::ActivityLine { task: 52, position, at, line }), decoded: None }
}
fn pattern() -> Vec<c::DecodedActivityLine> {
    vec![
        activity(1, at(960), boundary(1)),
        activity(2, at(980), began(2)),
        activity(
            3,
            at(985),
            c::Activity::Stood {
                attempt: 2,
                words: c::Text::new(b"Triaged three findings;\nrouted two.".as_slice().into(), 512).unwrap(),
            },
        ),
        activity(4, at(990), ended(2, 13)),
        activity(5, at(995), began(3)),
        activity(6, at(1002), ended(3, 4)),
        activity(7, at(1550), boundary(2)),
        activity(8, at(1552), began(4)),
    ]
}
fn held(rows: &[c::DecodedActivityLine]) -> Map<u64, Line> {
    let mut lines = Map::uncapped();
    for row in rows {
        lines.insert(row.source.position, retain(&row.source)).unwrap();
    }
    lines
}
#[test]
fn sessions_group_runs_at_commit_boundaries_not_wake_times() {
    let wall = Wall::from_nanos(WALL);
    let rows = group(&held(&pattern()), false, wall);
    assert_eq!(
        rows.as_slice(),
        [
            team::Session { start: at(1552), end: None, words: None, spent: 0 },
            team::Session {
                start: at(980),
                end: Some(at(1002)),
                words: Some(b"Triaged three findings; routed two.".as_slice().into()),
                spent: 17
            }
        ]
    );
    assert_eq!(group(&held(&pattern()[1..]), false, wall).len(), 1, "oldest partial session is undrawn");
    assert_eq!(group(&held(&pattern()[1..]), true, wall).len(), 2, "the activity's start is a boundary");
    assert!(
        group(&held(&[activity(1, at(980), ended(2, 13)), activity(2, at(990), boundary(2))]), true, wall).is_empty(),
        "no beginning makes no row"
    );
    let last_words = vec![
        activity(1, at(980), began(1)),
        activity(
            2,
            at(990),
            c::Activity::Stood { attempt: 1, words: c::Text::new(b"First".as_slice().into(), 512).unwrap() },
        ),
        activity(
            3,
            at(985),
            c::Activity::Stood { attempt: 1, words: c::Text::new(b"Last\r\nwords".as_slice().into(), 512).unwrap() },
        ),
    ];
    assert_eq!(
        group(&held(&last_words), true, wall).get(0).unwrap().words.as_deref(),
        Some(b"Last words".as_slice()),
        "last committed Stood, not latest wall time"
    );
    let old = vec![activity(1, Wall::from_nanos(WALL - 7 * DAY - 1), began(1)), activity(2, at(980), ended(1, 1))];
    assert!(group(&held(&old), true, wall).is_empty(), "a recent end cannot make an old first run recent");
    let edge = vec![
        activity(1, Wall::from_nanos(WALL - 7 * DAY), began(1)),
        activity(2, at(980), ended(1, u64::MAX)),
        activity(3, at(990), began(2)),
        activity(4, at(1000), ended(2, 1)),
    ];
    let rows = group(&held(&edge), true, wall);
    assert_eq!(rows.get(0).unwrap().spent, u64::MAX, "cost saturates at the seven-day edge");
    let mut many = Vec::new();
    for n in 0..10 {
        many.push(activity(n * 2 + 1, at(980 + n), began(n + 1)));
        many.push(activity(n * 2 + 2, at(980 + n), boundary(n + 1)));
    }
    let rows = group(&held(&many), true, wall);
    assert_eq!(rows.len(), 8);
    assert_eq!(rows.get(0).unwrap().start, at(989), "newest first");
    assert_eq!(rows.get(7).unwrap().start, at(982));
    let rows = group(&held(&[activity(1, at(980), began(1)), activity(2, at(981), ended(2, 5))]), true, wall);
    assert_eq!(rows.get(0).unwrap().end, None, "only the matching attempt ends a run");
}
fn opening() -> Reads {
    let mut f = reads_fixture();
    f.env.wall = Wall::from_nanos(WALL);
    feed(&mut f, Feed::Open);
    let (handle, watch) = f.roles.unwrap();
    let mut role = super::super::chats::chat_line();
    role.chip.task = 52;
    role.project = PROJECT;
    role.label = 3;
    role.ended_at = None;
    delivered(
        &mut f,
        handle,
        watch,
        c::Delivery::Tasks(Box::new(c::Tasks { rows: List::from_boxed(Box::new([role])), more: false })),
    );
    assert_eq!(f.activity.last().unwrap().1, c::Read::Activity { task: 52, before: None });
    f
}
fn answer(f: &mut Reads, rows: Vec<c::DecodedActivityLine>, before: Option<c::Cursor>, cut: bool) {
    let handle = f.activity.last().unwrap().0;
    page(
        f,
        Event::Told {
            told: Told::Page {
                handle,
                page: Readout::Jig(c::ReadResult::Value(c::ReadValue::Activity(Box::new(c::ActivityPage {
                    cut,
                    rows: List::from_boxed(rows.into_boxed_slice()),
                    before,
                    after: None,
                })))),
            },
        },
    );
}
fn cursor(n: u8) -> c::Cursor {
    c::Cursor::new(Box::new([n]), 64).unwrap()
}
fn rows(f: &Reads) -> List<team::Session> {
    team::shown(&f.root, PROJECT).facts().unwrap().card.as_ref().unwrap().sessions.clone()
}
fn moved(f: &mut Reads, activity: u64) {
    let (_, handle, watch) = f.card.unwrap();
    let mut task = fixtures::chat::task();
    task.chip.task = 52;
    task.activity = activity;
    delivered(f, handle, watch, c::Delivery::Task(Box::new(task)));
}
#[test]
fn sessions_read_back_then_merge_a_moving_tail_by_position() {
    let mut f = opening();
    let pattern = pattern();
    answer(&mut f, pattern[4..].iter().rev().cloned().collect(), Some(cursor(1)), true);
    assert_eq!(
        f.activity.last().unwrap().1,
        c::Read::Activity { task: 52, before: Some(cursor(1)) },
        "cut is not continuation"
    );
    assert_eq!(rows(&f).len(), 1, "the newest session draws as soon as its older boundary is held");
    moved(&mut f, 8);
    assert_eq!(f.activity.len(), 2, "a document moving during a read does not overlap it");
    answer(&mut f, pattern[..4].iter().rev().cloned().collect(), None, false);
    assert_eq!(rows(&f).len(), 2);
    assert_eq!(f.activity.len(), 3, "the pending move reads from the newest line afterwards");
    assert_eq!(f.activity.last().unwrap().1, c::Read::Activity { task: 52, before: None });
    answer(&mut f, vec![activity(9, at(1560), ended(4, 3)), pattern[7].clone()], Some(cursor(2)), false);
    assert_eq!(f.activity.len(), 3, "a held position stops the reread");
    assert_eq!(rows(&f).len(), 2, "older lines survive the merged reread");
    assert_eq!(rows(&f).get(0).unwrap().spent, 3);
    assert_eq!(rows(&f).get(1).unwrap().spent, 17);
    let expired = team::shown_at(&f.root, PROJECT, Wall::from_nanos(WALL + 8 * DAY));
    assert!(
        expired.facts().unwrap().card.as_ref().unwrap().sessions.is_empty(),
        "week expiry follows the view wall without another activity delivery"
    );
    page(&mut f, Event::Went { address: Address::Home { project: PROJECT } });
    assert!(rows(&f).is_empty(), "leaving Team closes the reader");
    page(&mut f, Event::Went { address: Address::Team { project: PROJECT, role: Some(52) } });
    assert_eq!(f.activity.len(), 4, "reopening starts fresh");
    let stale = f.activity.last().unwrap().0;
    let (handle, watch) = f.roles.unwrap();
    let mut first = super::super::chats::chat_line();
    first.chip.task = 41;
    first.project = PROJECT;
    first.label = 3;
    first.ended_at = None;
    let mut second = first.clone();
    second.chip.task = 52;
    delivered(
        &mut f,
        handle,
        watch,
        c::Delivery::Tasks(Box::new(c::Tasks { rows: List::from_boxed(Box::new([first, second])), more: false })),
    );
    page(&mut f, Event::Went { address: Address::Team { project: PROJECT, role: Some(41) } });
    assert_eq!(
        f.activity.last().unwrap().1,
        c::Read::Activity { task: 41, before: None },
        "changing the card replaces its read"
    );
    page(
        &mut f,
        Event::Told {
            told: Told::Page {
                handle: stale,
                page: Readout::Jig(c::ReadResult::Value(c::ReadValue::Activity(Box::new(c::ActivityPage {
                    rows: List::from_boxed(pattern.into_boxed_slice()),
                    before: None,
                    after: None,
                    cut: false,
                })))),
            },
        },
    );
    assert!(rows(&f).is_empty(), "the old card's late answer is ignored");
}
#[test]
fn sessions_reads_stop_at_age_eight_sessions_or_sixteen_pages() {
    for old in [began(1), boundary(1)] {
        let mut f = opening();
        answer(&mut f, vec![activity(1, Wall::from_nanos(WALL - 7 * DAY - 1), old)], Some(cursor(1)), false);
        assert_eq!(f.activity.len(), 1, "old boundary or beginning stops reading");
        assert!(rows(&f).is_empty());
    }
    let mut f = opening();
    f.env.wall = Wall::from_nanos(WALL + DAY);
    answer(&mut f, vec![activity(1, Wall::from_nanos(WALL - 7 * DAY + 1), began(1))], Some(cursor(1)), false);
    assert_eq!(f.activity.len(), 1, "the stop uses the answer's wall, not the request's");
    let mut f = opening();
    let mut many = Vec::new();
    for n in 0..9 {
        many.push(activity(n * 2 + 1, at(980 + n), began(n + 1)));
        many.push(activity(n * 2 + 2, at(980 + n), boundary(n + 1)));
    }
    many.reverse();
    answer(&mut f, many, Some(cursor(1)), false);
    assert_eq!(f.activity.len(), 1, "eight fully bounded sessions stop reading");
    assert_eq!(rows(&f).len(), 8);
    let mut f = opening();
    for n in 0..16 {
        answer(&mut f, vec![activity(32 - n, at(980), began(1))], Some(cursor(u8::try_from(n + 1).unwrap())), false);
        assert_eq!(f.activity.len(), u32::try_from((n + 2).min(16)).unwrap());
    }
    assert!(rows(&f).is_empty(), "the session stopped inside at sixteen pages is undrawn");
}
#[test]
fn sessions_failed_reads_hide_rows_and_retry_with_activity_asking() {
    let mut f = opening();
    answer(&mut f, pattern().into_iter().rev().collect(), None, false);
    assert_eq!(rows(&f).len(), 2);
    moved(&mut f, 9);
    for refused in [true, false] {
        let handle = f.activity.last().unwrap().0;
        if refused {
            page(
                &mut f,
                Event::Told {
                    told: Told::Page { handle, page: Readout::Jig(c::ReadResult::None(c::ReadNone::Refused)) },
                },
            );
        } else {
            page(&mut f, Event::Told { told: Told::Ended { handle, why: jig_client::Why::Limit } });
        }
        assert!(rows(&f).is_empty());
        let deadline = pages::next_deadline(&f.root.pages).unwrap();
        assert_eq!(deadline, Time::ZERO.saturating_add(Duration::from_secs(if refused { 1 } else { 3 })));
        let mut out = Queue::uncapped();
        f.env.now = deadline;
        pages::fire(&mut f.root.pages, &f.env, &f.root.client, &mut out);
        let mut retried = false;
        for request in &out {
            if let Request::Client(jig_client::Event::Read {
                handle: h,
                read: Readable::Jig(c::Read::Activity { task: 52, before: None }),
            }) = request
            {
                retried |= *h == handle;
            }
        }
        assert!(retried);
    }
    answer(&mut f, vec![pattern()[7].clone()], Some(cursor(1)), false);
    assert_eq!(rows(&f).len(), 2, "successful retry restores merged rows");
}
#[test]
fn sessions_parts_need_rows_in_the_shown_card() {
    let mut facts = team::gather(&project(Box::new([]), false), YOU, None, &[], None, Wall::EPOCH);
    facts.roles = List::from_boxed(Box::new([role(52)]));
    facts.card = Some(Box::new(card(52)));
    assert!(!team::parts(&facts).sessions);
    facts.card.as_mut().unwrap().sessions = group(&held(&pattern()), false, Wall::from_nanos(WALL));
    assert!(team::parts(&facts).sessions && team::parts(&facts).right);
    facts.card.as_mut().unwrap().task = 41;
    assert!(!team::parts(&facts).sessions && !team::parts(&facts).right, "an old card has no parts");
}

#[test]
fn sessions_prefer_their_last_now_line_without_lending_it() {
    let mut f = opening();
    let now = |attempt, line: &[u8]| c::Activity::NowWritten {
        attempt,
        line: c::Text::new(line.into(), 512).unwrap(),
    };
    let stood = |attempt, words: &[u8]| c::Activity::Stood {
        attempt,
        words: c::Text::new(words.into(), 512).unwrap(),
    };
    let lines = vec![
        activity(1, at(980), began(1)),
        activity(2, at(981), stood(1, b"done")),
        activity(3, at(982), now(1, b"Earlier Now")),
        activity(4, at(983), ended(1, 1)),
        activity(5, at(984), began(2)),
        activity(6, at(981), now(2, b"Reproduced finding 91;\r\nasked you about its fix.")),
        activity(7, at(985), stood(2, b"done again")),
        activity(8, at(986), ended(2, 2)),
        activity(9, at(987), boundary(1)),
        activity(10, at(988), began(3)),
        activity(11, at(989), stood(3, b"Fallback\nwords")),
        activity(12, at(990), boundary(3)),
        activity(13, at(991), began(4)),
    ];
    // NowWritten crosses a page edge and its wall is older: commit order wins.
    answer(&mut f, lines[6..].iter().rev().cloned().collect(), Some(cursor(1)), false);
    answer(&mut f, lines[..6].iter().rev().cloned().collect(), None, false);
    let rows = rows(&f);
    assert_eq!(rows.len(), 3);
    assert_eq!(rows.get(2).unwrap().words.as_deref(), Some(b"Reproduced finding 91; asked you about its fix.".as_slice()));
    assert_eq!(rows.get(1).unwrap().words.as_deref(), Some(b"Fallback words".as_slice()));
    assert_eq!(rows.get(0).unwrap().words, None, "an earlier session lends no Now line");
}
