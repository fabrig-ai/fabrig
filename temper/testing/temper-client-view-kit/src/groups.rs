//! Questions about triager-held findings, read through the public watches.
use super as f;
use jig_client::{Down, Up, Watchable, contract as c};
use skein_lib::{Env, List, Queue, Time, Wall};
use temper_client_domain::{self as d, Address, Event};
use temper_client_protocol_wire::{self as wire, v1 as w};
use temper_fixtures::client_documents as documents;

pub const QUESTION: &str = "Start a goal for each?\n\nNone blocks a landing.\nGroup: bugs · in temper\nFindings: 2/2 2/8 2/10\nSaw: No change is blocked.\nChoice: One goal for all\nChoice: A goal for each (recommended)\nChoice: Backlog them";
pub const PROJECT: u32 = 2;
pub const NOW: Wall = Wall::from_nanos(86_400_000_000_000);

fn limits() -> d::Limits {
    let mut limits = f::home::home_limits();
    limits.client.domain.text = 512;
    limits.client.domain.turn_text = 512;
    limits.client.domain.long_text = 4096;
    limits
}
fn event(root: &mut d::Domain, event: Event, out: &mut Queue<d::Request>) {
    d::step(root, &Env { now: Time::ZERO, wall: NOW, limits: limits() }, event, out);
}

#[must_use]
pub fn row(number: u32, title: &str, severity: w::Severity, kind: w::FindingKind, hours_old: u64) -> w::FindingRow {
    w::FindingRow {
            number,
            at: NOW.as_nanos() - hours_old * 3_600_000_000_000,
            kind,
            severity,
            area: None,
            state: w::FindingState::New,
            title: title.as_bytes().into(),
            holder: w::Holder::Task(w::HolderTask { number: 55 }),
            why: None,
            hander: None,
        }
}

#[must_use]
pub fn bugs() -> Box<[w::FindingRow]> {
    Box::new([
        row(2, "The model picker keeps a later choice", w::Severity::High, w::FindingKind::Qa, 24),
        row(8, "A resolution ignores later files", w::Severity::Low, w::FindingKind::Review, 16),
        row(10, "A hung forge read holds too long", w::Severity::Medium, w::FindingKind::Review, 15),
    ])
}

/// Every question belongs to project 2; the mine read is separate from its triager's.
#[must_use]
pub fn fixture(words: &[&str], rows: &[w::FindingRow], mine: &[w::FindingRow], cut: bool) -> d::Domain {
    at(
        words,
        rows,
        mine,
        cut,
        Address::Inbox { filter: d::address::Filter { project: Some(PROJECT), ..d::address::Filter::ALL } },
    )
}

#[must_use]
pub fn at(words: &[&str], rows: &[w::FindingRow], mine: &[w::FindingRow], cut: bool, address: Address) -> d::Domain {
    at_with(words, rows, mine, cut, address, false)
}

/// A report question asked by an untracked standing task, without goal ancestry.
#[must_use]
pub fn standing_person(words: &str, rows: &[w::FindingRow]) -> d::Domain {
    at_with(&[words], rows, &[], false, Address::Inbox { filter: d::address::Filter::ALL }, true)
}

#[expect(clippy::too_many_lines, reason = "the fixture answers its independent question and finding watches together")]
fn at_with(
    words: &[&str],
    rows: &[w::FindingRow],
    mine: &[w::FindingRow],
    cut: bool,
    address: Address,
    person: bool,
) -> d::Domain {
    let (mut root, me_watch, _, _, _, _, _, standing, _) = f::root_at_with_classes(
        Address::Settings { project: PROJECT },
        &limits(),
        b"Shop",
        None,
        None,
        None,
        Some(&[]),
        Some(d::forge::ways::Ways::STOCK),
        true,
        true,
        None,
    );
    let mut out = Queue::uncapped();
    let mut me = root.pages().me(root.client()).unwrap().clone();
    me.projects.truncate(1);
    me.projects.get_mut(0).unwrap().project = PROJECT;
    event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: Up::Watched { watch: me_watch, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))) },
            },
        },
        &mut out,
    );
    event(&mut root, Event::Went { place: d::Place::Address(address) }, &mut out);
    let entries: Vec<_> = words
        .iter()
        .enumerate()
        .map(|(i, _)| {
            let message = u64::try_from(i).unwrap() + 1;
            let object = if person { c::ObjectKey::PersonTask(f::INBOX_TASK) } else { f::needs::object(message) };
            documents::entry(object, PROJECT, message, Wall::EPOCH, None)
        })
        .collect();
    let entries = List::from_boxed(entries.into_boxed_slice());
    event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: Up::Watched {
                    watch: standing.unwrap(),
                    event: c::WatchEvent::Delivered(c::Delivery::Inbox(Box::new(documents::inbox(
                        entries.clone(),
                        u32::try_from(words.len()).unwrap(),
                        c::Count { value: 0, limited: false },
                        0,
                        cut,
                    )))),
                },
            },
        },
        &mut out,
    );
    while let Some(request) = out.pop() {
        match request {
            d::Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down { down: Down::Read { read, what: jig_client::Readable::Jig(what) } },
            } => {
                let value = match what {
                    c::Read::Inbox { group, .. } => c::ReadValue::Inbox(Box::new(c::InboxPage {
                        selection: None,
                        before: None,
                        rows: if group == c::InboxReadGroup::Needs { entries.clone() } else { List::with_capacity(0) },
                        after: None,
                        cut,
                    })),
                    c::Read::Question { message, .. } => {
                        let text = words[usize::try_from(message - 1).unwrap()];
                        let mut task = f::task::document().chip;
                        task.task = f::INBOX_TASK;
                        task.title = f::task::document_words(b"Triager");
                        c::ReadValue::Question(Box::new(c::Question {
                            task,
                            from: None,
                            message,
                            words: c::DocumentText::complete(c::Text::new(text.as_bytes().into(), 4096).unwrap()),
                            since: Wall::EPOCH,
                            acts: List::from_boxed(Box::new([c::Offered {
                                act: c::ActKind::Answer,
                                admission: c::Admission::Allowed,
                                effect: None,
                            }])),
                            ending: None,
                        }))
                    }
                    c::Read::PersonTask { task } if person => {
                        assert_eq!(task, f::INBOX_TASK);
                        let mut asked = f::needs::person();
                        asked.task.task = task;
                        asked.task.executor = c::ChipExecutor::Party { party: 73 };
                        asked.requester = c::Requester::Task(55);
                        asked.address = c::PersonAddress::Party(73);
                        asked.task.details.details.executor = c::Executor::Person(asked.address);
                        asked.contract = c::TaskContractDocument::Report { words: 4096 };
                        asked.spec.specification.words =
                            c::DocumentText::complete(c::Text::new(words[0].as_bytes().into(), 4096).unwrap());
                        asked.since = Some(Wall::EPOCH);
                        asked.acts = f::needs::offers(&[c::ActKind::Answer]);
                        c::ReadValue::PersonTask(Box::new(asked))
                    }
                    c::Read::Task { task } if person => {
                        assert_eq!(task, 55);
                        let mut standing = f::task::document();
                        standing.chip.task = task;
                        standing.chip.title = f::task::document_words(b"Standing triager");
                        standing.chip.executor = c::ChipExecutor::Agent { charter: 1 };
                        standing.chip.details.details.executor = c::Executor::Agent { charter: 1 };
                        standing.flags.ended_by = c::EndedBy::Standing;
                        standing.project = PROJECT;
                        standing.requester = c::Requester::Person(73);
                        standing.priority = None;
                        standing.goal = None;
                        standing.lineage = List::with_capacity(0);
                        standing.lineage_chips = List::with_capacity(0);
                        c::ReadValue::Task(Box::new(standing))
                    }
                    _ => continue,
                };
                event(
                    &mut root,
                    Event::Client {
                        event: jig_client::Event::Up {
                            up: Up::ReadDone { read, result: jig_client::Readout::Jig(c::ReadResult::Value(value)) },
                        },
                    },
                    &mut out,
                );
            }
            d::Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: Down::Open { watch, subject: Watchable::Family { record, .. } },
                    },
            } => {
                let Some(watching) = wire::decode_watching(&record.bytes, &w::CEILINGS) else { continue };
                let bytes = match &watching.watch {
                    w::Watch::Findings(subject) => {
                        let holder = w::Holder::Party(
                            w::HolderParty { number: 1 },
                        );
                        wire::findings(
                            &w::Findings {
                                    project: subject.project,
                                    rows: List::from_boxed(if subject.project != PROJECT {
                                        Box::default()
                                    } else if subject.mine {
                                        mine.into()
                                    } else {
                                        rows.into()
                                    }),
                                    areas: List::with_capacity(0),
                                    triager: None,
                                    holder,
                                    cut: false,
                                },
                        )
                    }
                    w::Watch::Decided(subject) => wire::decisions(
                        &w::Decisions {
                                project: subject.project,
                                rows: List::with_capacity(0),
                                asked: List::with_capacity(0),
                                cut: false,
                            },
                    ),
                    _ => continue,
                };
                event(
                    &mut root,
                    Event::Client {
                        event: jig_client::Event::Up {
                            up: Up::FamilyDelivered {
                                watch,
                                record: jig_client::Family {
                                    family: wire::FORGE,
                                    version: wire::CLIENT_VERSION,
                                    bytes,
                                },
                            },
                        },
                    },
                    &mut out,
                );
            }
            _ => {}
        }
    }
    root
}
