use super::*;
use skein_ui::{Role, Run};
use temper_fixtures::first_run;
fn text(b: &mut skein_ui::Builder, role: Role, words: &[u8]) {
    b.open(role);
    b.text(Run::Plain, words);
    b.close();
}
pub fn page() -> skein_ui::Tree {
    let mut b = skein_ui::Builder::new(&tree_limits());
    synthetic_frame(&mut b, false);
    b.open(Role::Main);
    text(&mut b, Role::Heading { level: 1 }, "T9 · Hello".as_bytes());
    text(&mut b, Role::Status, b"Writing");
    text(&mut b, Role::Paragraph, &spend(7, 100, 2));
    b.open(Role::Log);
    b.name(b"Conversation");
    b.bind(binding());
    text(&mut b, Role::Paragraph, b"You");
    text(&mut b, Role::Paragraph, b"Hello");
    text(&mut b, Role::Paragraph, b"Temper");
    golden_cards::report(&mut b, b"I can help.");
    text(&mut b, Role::Paragraph, b"Writing an answer");
    text(&mut b, Role::Status, "▌".as_bytes());
    b.close();
    for (role, name) in [
        (Role::TextArea, b"Message".as_slice()),
        (Role::Button, b"Send"),
        (Role::Button, b"Pause"),
        (Role::Button, b"Close"),
    ] {
        b.open(role);
        b.name(name);
        b.bind(binding());
        b.close();
    }
    b.close();
    b.close();
    b.finish()
}

/// The first run, through the production chat view and delegation card.
pub fn delegation_page(landed: bool) -> skein_ui::Tree {
    first_run_page(client::Address::Chat { task: 1 }, landed)
}
pub fn settings() -> skein_ui::Tree {
    first_run_page(client::Address::Settings { project: 1 }, true)
}
pub fn first_run() -> skein_ui::Tree {
    first_run_page(client::Address::NewChat { draft: false }, true)
}
fn first_run_page(address: client::Address, landed: bool) -> skein_ui::Tree {
    first_run_sheet(address, landed, false)
}
pub fn closing_page() -> skein_ui::Tree {
    first_run_sheet(client::Address::Chat { task: 1 }, false, true)
}
pub fn long_line_page(address: client::Address, line: &contract::Activity) -> skein_ui::Tree {
    first_run_words(address, false, false, Some(line), None)
}
fn first_run_sheet(address: client::Address, landed: bool, closing: bool) -> skein_ui::Tree {
    first_run_words(address, landed, closing, None, None)
}
#[expect(clippy::too_many_lines, reason = "the fixture pumps one first-run page and optionally opens its confirmation")]
fn first_run_words(
    address: client::Address,
    landed: bool,
    closing: bool,
    line: Option<&contract::Activity>,
    typed: Option<&[u8]>,
) -> skein_ui::Tree {
    use jig_client::contract as c;
    use skein_lib::{Env, List, Queue, Time, Wall};
    use temper_client_domain::{Event, Request};
    let limits = crate::limits::native().client;
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut root = client::unlinked(&limits, 16);
    let mut out = Queue::uncapped();
    client::step(&mut root, &env, Event::Start { address, kept: List::with_capacity(0), with: None }, &mut out);
    for event in [
        jig_client::Event::StartUnlinked { kept: List::with_capacity(0) },
        jig_client::Event::SignIn { with: jig_client::With::Transport },
        jig_client::Event::Up { up: jig_client::Up::Served { served: true, versions: None } },
    ] {
        client::step(&mut root, &env, Event::Client { event }, &mut out);
    }
    for _ in 0..128 {
        let Some(request) = out.pop() else { break };
        let up = match request {
            Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: jig_client::Down::Open { watch, subject: jig_client::Watchable::Family { record, .. } },
                    },
            } if temper_client_protocol_wire::decode_watching(&record.bytes, &wire_limits())
                .expect("forge watch")
                .watch()
                == &temper_client_protocol_wire::v1::Watch::Forges =>
            {
                Some(jig_client::Up::FamilyDelivered {
                    watch,
                    record: jig_client::Family {
                        family: temper_client_protocol_wire::FORGE,
                        version: temper_client_protocol_wire::CLIENT_VERSION,
                        bytes: demo_forges(),
                    },
                })
            }
            Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(subject) },
                    },
            } => {
                let delivery = match subject {
                    c::Subject::Settings => Some(c::Delivery::Settings(Box::new(c::SettingsDocument {
                        newest: 0,
                        settings: List::with_capacity(0),
                        held: List::with_capacity(0),
                        pending: List::with_capacity(0),
                    }))),
                    c::Subject::Me => Some(c::Delivery::Me(Box::new(demo_me()))),
                    c::Subject::Task { task } => {
                        let mut document = delegation_task(task, landed);
                        if line.is_some() {
                            document.activity = 1;
                        }
                        if closing && task == 1 {
                            document.chip.details.phase = c::ChipPhase::Active(c::RunStage::Running { attempt: 1 });
                            document.acts = List::with_capacity(1);
                            document
                                .acts
                                .push(c::Offered {
                                    act: c::ActKind::Cancel,
                                    admission: c::Admission::Allowed,
                                    effect: Some(c::Effect::Cancel(c::Impact {
                                        tasks: c::ImpactCounts {
                                            waiting: 1,
                                            running: 0,
                                            held: 0,
                                            closing: 0,
                                            ended: 0,
                                            done: 0,
                                            failed: 0,
                                            cancelled: 0,
                                        },
                                        live_runs: 1,
                                        proposals: 2,
                                    })),
                                })
                                .expect("cancel");
                        }
                        Some(c::Delivery::Task(Box::new(document)))
                    }
                    c::Subject::Accounts if landed => Some(c::Delivery::Accounts(Box::new(demo_accounts()))),
                    c::Subject::Deployment
                    | c::Subject::Tree { .. }
                    | c::Subject::Chips { .. }
                    | c::Subject::Inbox { .. }
                    | c::Subject::Tasks { .. }
                    | c::Subject::Run { .. }
                    | c::Subject::Accounts => None,
                };
                let Some(delivery) = delivery else { continue };
                Some(jig_client::Up::Watched { watch, event: c::WatchEvent::Delivered(delivery) })
            }
            Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down:
                            jig_client::Down::Read {
                                read,
                                what: jig_client::Readable::Jig(c::Read::Activity { before, .. }),
                            },
                    },
            } => Some(jig_client::Up::ReadDone {
                read,
                result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Activity(Box::new(
                    if let Some(line) = line {
                        let mut rows = List::with_capacity(1);
                        rows.push(c::DecodedActivityLine {
                            source: Box::new(c::ActivityLine {
                                task: 1,
                                position: 1,
                                at: Wall::EPOCH,
                                line: line.clone(),
                            }),
                            decoded: None,
                        })
                        .expect("whole line");
                        c::ActivityPage { rows, before: None, after: None, cut: false }
                    } else {
                        first_run::page(&delegation_activity(landed), before.as_ref(), limits.client.domain.rows)
                    },
                )))),
            }),
            Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: jig_client::Down::Read { read, what: jig_client::Readable::Family { record, .. } },
                    },
            } if temper_client_protocol_wire::decode_reading(&record.bytes, &wire_limits())
                .expect("forge reading")
                .read()
                == &temper_client_protocol_wire::v1::Read::Deployment =>
            {
                use temper_client_protocol_wire::{self as wire, v1 as w};
                Some(jig_client::Up::ReadDone {
                    read,
                    result: jig_client::Readout::Family {
                        family: wire::FORGE,
                        record: Some(wire::deployment_answer(
                            &w::DeploymentAnswer::new(
                                &w::CEILINGS,
                                w::DeploymentAnswerParts {
                                    push: None,
                                    welcome: true,
                                    owned: true,
                                    program_login: None,
                                },
                            )
                            .expect("deployment answer"),
                        )),
                    },
                })
            }
            Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: jig_client::Down::Read { read, what: jig_client::Readable::Family { record, .. } },
                    },
            } if landed
                && temper_client_protocol_wire::decode_reading(&record.bytes, &wire_limits())
                    .expect("forge reading")
                    .read()
                    == &temper_client_protocol_wire::v1::Read::Playground =>
            {
                Some(jig_client::Up::ReadDone {
                    read,
                    result: jig_client::Readout::Family {
                        family: temper_client_protocol_wire::FORGE,
                        record: Some(demo_answer()),
                    },
                })
            }
            Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: jig_client::Down::Read { read, what: jig_client::Readable::Family { record, .. } },
                    },
            } => {
                use temper_client_protocol_wire::{self as wire, v1 as w};
                if let w::Read::Setting(_) = wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap().read() {
                    Some(jig_client::Up::ReadDone {
                        read,
                        result: jig_client::Readout::Family {
                            family: wire::FORGE,
                            record: Some(wire::setting_answer(
                                &w::SettingAnswer::new(&w::CEILINGS, w::SettingAnswerParts { value: None }).unwrap(),
                            )),
                        },
                    })
                } else {
                    None
                }
            }
            Request::Client { .. }
            | Request::Unlinked { .. }
            | Request::Show { .. }
            | Request::Screen { .. }
            | Request::KeepLook { .. }
            | Request::KeepSignIn { .. }
            | Request::Tab { .. }
            | Request::Leave { .. }
            | Request::SignOut
            | Request::Dropped { .. }
            | Request::Copy { .. }
            | Request::Fact { .. } => None,
        };
        if let Some(up) = up {
            client::step(&mut root, &env, Event::Client { event: jig_client::Event::Up { up } }, &mut out);
        }
    }
    assert!(out.is_empty(), "first run fixture settles");
    if closing {
        client::step(
            &mut root,
            &env,
            Event::Act { action: client::Action::Pages(client::pages::Act::Close) },
            &mut out,
        );
    }
    if let Some(words) = typed {
        client::step(
            &mut root,
            &env,
            Event::Act { action: client::Action::Pages(client::pages::Act::MessageWords { words: words.into() }) },
            &mut out,
        );
        client::step(&mut root, &env, Event::Act { action: client::Action::Pages(client::pages::Act::Send) }, &mut out);
        assert!(out.is_empty(), "an oversized Send sends nothing");
        assert_eq!(root.pages().chat().unwrap().words.as_ref(), words);
    }
    let bounds = if line.is_some() || typed.is_some() {
        temper_client_view::Limits::of(&limits, &page_words()).expect("whole-line bounds")
    } else {
        temper_client_view::Limits { tree: tree_limits() }
    };
    let mut view = temper_client_view::View::new(&bounds, &limits, page_words(), 1);
    temper_client_view::build(&mut view, &root, super::NOW, &mut Queue::with_capacity(bounds.tree.patches));
    copy_tree_at(view.tree(), &bounds.tree)
}
fn demo_me() -> contract::Me {
    use jig_client::contract as c;
    use skein_lib::List;
    let mut me = jig_client_kit::stories::frame(1);
    me.deployment_role = Some(c::HeldRole::Owner);
    let mut models = List::with_capacity(1);
    models
        .push(c::MeModel {
            new_until: None,
            number: 1,
            source: jig_client::contract::ModelSource::Stock,
            estimated: false,
            effort: None,
            shown: None,
            accounts: List::with_capacity(0),
            picker: true,
            name: delegation_text(b"demo"),
            provider: Some(3),
            prices: c::ModelPrices { input: 1, cached: 1, cache_write: None, output: 1, unit: 1000 },
            description: None,
            window: 1024,
            output: 128,
            efforts: List::with_capacity(0),
        })
        .expect("demo model");
    me.projects.get_mut(0).expect("demo project").models = models;
    me.projects.get_mut(0).expect("demo project").name = c::DocumentText::complete(delegation_text(b"demo"));
    me
}
fn delegation_task(number: u64, landed: bool) -> contract::Task {
    use jig_client::contract as c;
    let mut task = jig_client_kit::stories::conversations::chat(number, b"First run", skein_lib::Wall::EPOCH);
    task.chip.task = number;
    task.chip.title = c::DocumentText::complete(delegation_text(if number == 1 {
        b"Add CSV export to widget"
    } else {
        b"Export widget's table as CSV"
    }));
    task.hold = None;
    task.kept_account = Some(1);
    task.budget.funder = c::Funder::Pool { project: 1, person: 1, period: 1 };
    task.result = None;
    task.chip.phase = c::Phase::Running;
    task.proposals.clear();
    task.activity = if number == 1 { if landed { 9 } else { 5 } } else { 0 };
    task.spec.specification.words = c::DocumentText::complete(delegation_text(first_run::CSV));
    if number == 1 {
        task.requester = c::Requester::Person(1);
        task.wake.words = c::WakeRule::Immediate;
        task.waits_on = if landed { c::WaitsOn::News } else { c::WaitsOn::Delegates };
        let c::ChipPhaseFacts { details, .. } = task.chip.details.as_mut();
        // The first-run fixture has one change, still live until its result arrives.
        details.live_delegates = u32::from(!landed);
        details.wake = task.wake;
        if !landed {
            task.delegates = skein_lib::List::with_capacity(1);
            task.delegates
                .push(c::LiveDelegate { chip: delegation_task(2, false).chip, takeover_budget: 0 })
                .expect("live change");
        }
    }
    if number == 2 {
        task.requester = c::Requester::Task(1);
        task.result_contract = c::TaskContractDocument::Change { connector: 1, kind: 1, words: 256 };
        if landed {
            task.result = Some(c::TaskResult::Ending(Box::new(c::ChipEnding::Done(c::ResultValue::Change {
                kind: 1,
                part: c::Part { connector: 1, kind: c::PartKind::Result, key: c::PartKey::Number(2) },
                words: c::DocumentText::complete(delegation_text(b"1a2b3c4abcdef0123456789abcdef0123456789abcd")),
            }))));
        }
    }
    task
}
fn delegation_activity(landed: bool) -> contract::ActivityPage {
    first_run::activity(false, landed, crate::limits::native().client.client.domain.turn_text)
}

fn delegation_text(bytes: &[u8]) -> contract::Text {
    contract::Text::new(Box::from(bytes), 256).expect("first run text fits")
}

fn demo_accounts() -> contract::Accounts {
    use jig_client::contract as c;
    use skein_lib::List;
    let mut readiness = List::with_capacity(1);
    readiness
        .push(c::PayerReadiness {
            provider: 3,
            payer: c::AccountUse::Party(1),
            readiness: c::Readiness::Usable,
            settled: Some(true),
        })
        .expect("demo readiness");
    let mut sources = List::with_capacity(1);
    sources
        .push(c::AccountSource {
            name: None,
            place: 1,
            number: 1,
            kind: c::AccountKind::None,
            state: c::AccountState::Granted,
            usable: true,
            expiring: false,
            attention: false,
            spent_until: None,
        })
        .expect("demo source");
    let mut accounts = List::with_capacity(1);
    accounts
        .push(c::AccountLine {
            list: None,
            facts: List::with_capacity(0),
            number: 1,
            place: 1,
            uses: None,
            plan: false,
            provider: Some(3),
            provider_name: None,
            manage_usage: None,
            display: c::DocumentText::complete(delegation_text(b"Demo")),
            holder: c::AccountHolder::Deployment,
            sources,
            cut: false,
        })
        .expect("demo account");
    contract::Accounts {
        program_hosts: skein_lib::List::with_capacity(0),
        readiness,
        readiness_cut: false,
        accounts,
        signing: skein_lib::List::with_capacity(0),
        last_end: None,
        usable: true,
        settled: true,
        cut: false,
    }
}
fn demo_answer() -> Box<[u8]> {
    use temper_client_protocol_wire::{self as wire, v1 as w};
    let mut cues = skein_lib::List::with_capacity(3);
    for cue in [
        b"Add CSV export to widget".as_slice(),
        b"Add a dark theme to widget",
        "Fix widget’s empty search state".as_bytes(),
    ] {
        cues.push(cue.into()).expect("three demo cues");
    }
    wire::playground_answer(
        &w::PlaygroundAnswer::new(
            &wire_limits(),
            w::PlaygroundAnswerParts {
                state: w::PlaygroundState::Running,
                project: Some(1),
                forge: 1,
                provider: 3,
                owner: b"playground".as_slice().into(),
                name: b"widget".as_slice().into(),
                cues,
            },
        )
        .expect("demo answer fits"),
    )
}

fn demo_forges() -> Box<[u8]> {
    use temper_client_protocol_wire::{self as wire, v1 as w};
    let limits = wire_limits();
    let mut forges = skein_lib::List::with_capacity(1);
    forges
        .push(
            w::ForgeRow::new(
                &limits,
                w::ForgeRowParts {
                    starts_with: w::StartsWith::Repository,
                    makings: skein_lib::List::with_capacity(0),
                    host: b"git.example:3000".as_slice().into(),
                    number: 1,
                    kind: w::Kind::Forgejo,
                    mode: w::Mode::Fake,
                    connection: w::Connection::Connected(
                        w::Connected::new(&limits, w::ConnectedParts { id: 1, login: b"temper".as_slice().into() })
                            .expect("demo account"),
                    ),
                    offer: w::Offer::NotOffered,
                    first: None,
                    installations: skein_lib::List::with_capacity(0),
                    ended: None,
                    app: None,
                    web: None,
                    repositories: skein_lib::List::with_capacity(0),
                    owned: None,
                    admin: false,
                    read_only: false,
                    ended_stop: None,
                },
            )
            .expect("demo forge"),
        )
        .expect("one demo forge");
    wire::forges(&w::Forges::new(&limits, w::ForgesParts { forges, linked: false }).expect("demo forges"))
}

pub fn composer_over_page(words: &[u8]) -> skein_ui::Tree {
    first_run_words(client::Address::Chat { task: 1 }, false, false, None, Some(words))
}
