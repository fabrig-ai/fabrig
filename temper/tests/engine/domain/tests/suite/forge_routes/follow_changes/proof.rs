//! Whole-engine proofs of follows.md, 3–7 and 11. All observations use durable rows or public reads.
use super::*;
use skein_forge::api::{self, Commit};

pub(crate) fn fixture() -> World {
    configured_follow(s::FollowStart::Owner, 1000, Gate::default(), true)
}
pub(crate) fn follow(world: &World) -> &FollowRow {
    row(world)
}
pub(crate) fn session(world: &mut World) {
    world.until(Until::Ready);
    // One browser session, reconnected after a restart without filling the session bound.
    if let Some(sign_in) = world.signed_in.take() {
        world.send(engine::Event::Party(engine::PartyInput::SignOut {
            reply_to: ReplyTo::new(Token::new(198)),
            sign_in,
        }));
        world.drain_cut();
    }
    world.sign_in();
}
pub(crate) fn owner(world: &mut World, key: u8, request: FollowAct) {
    assert_eq!(act(world, key, request), None);
    world.drain_cut();
}
pub(crate) fn landing(world: &mut World, path: &[u8], message: &[u8]) -> Commit {
    fake::advance(&mut world.fake, &world.fake_env, b"org/vsr", b"main", path, message, 1).expect("source push");
    let after = translate::commit(world.fake.branch(b"org/vsr", b"main").expect("source tip"));
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Hint {
        hint: api::Hint { repository: SOURCE, change: api::Change::Branch(b"main".as_slice().into()), key: None },
    })));
    after
}
pub(crate) fn heard(world: &mut World, after: Commit) {
    for _ in 0..TICKS {
        world.tick();
        if row(world).heard == Some(after) && world.store.pending.is_empty() {
            settle(world);
            world.drain_cut();
            return;
        }
    }
    panic!("source move never heard: {:?}", row(world));
}
pub(crate) fn port(world: &mut World, change: u64, producer: &engine::Assignment, red: bool) {
    push_target(world, change);
    let branch = questions::row(world, change).branch.clone();
    fake::advance(
        &mut world.fake,
        &world.fake_env,
        b"org/vsr-rs",
        &branch,
        b"failures",
        if red { b"tests/handoff\n" } else { b"" },
        1,
    )
    .expect("worker writes report cue");
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        0,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: tasks::ChangeTitle::new(b"Port handoff".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: forge_top::tag(1, 4),
                words: b"pushed".as_slice().into(),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
}
pub(crate) fn delegate(world: &mut World, change: u64, previous: u64) -> engine::Assignment {
    next_delegate(world, change, previous)
}
pub(crate) fn landed(world: &mut World, change: u64) {
    for _ in 0..temper_engine_domain_world::tier_runs::RUN_TICKS {
        world.tick();
        if world.store.rows.contains_key(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(change)))) {
            world.drain_cut();
            assert!(
                matches!(
                    questions::row(world, change).change.state,
                    temper_engine_domain_forge_change::State::Landed { .. }
                        | temper_engine_domain_forge_change::State::Cleared { .. }
                ),
                "{:?}",
                questions::row(world, change)
            );
            return;
        }
    }
    panic!("change did not land: {:?}", questions::row(world, change));
}

#[test]
fn source_paths_and_unknown_files_make_changes_but_outside_paths_do_not() {
    let mut world = fixture();
    let inside = landing(&mut world, b"src/a.c", b"handoff");
    heard(&mut world, inside);
    let first = change(&world, inside);
    let outside = landing(&mut world, b"src-old/a.c", b"outside");
    heard(&mut world, outside);
    assert_eq!(row(&world).queue.len(), 1, "src prefix excludes src-old");
    world.fail_comparisons = true;
    let unknown = landing(&mut world, b"docs/a", b"unknown");
    heard(&mut world, unknown);
    world.fail_comparisons = false;
    assert_eq!(row(&world).queue.len(), 2);
    let second = change(&world, unknown);
    assert_ne!(first, second);
    assert!(row(&world).queue.last().expect("unknown move queued").files.is_none());
    let words = &live_task(&world, second).spec.words;
    assert!(String::from_utf8_lossy(words).contains("Files: unknown; the forge's comparison was incomplete."));
    assert_eq!(live_task(&world, second).waiting_on.as_ref(), &[first]);
    world.close();
}

#[test]
fn an_owner_started_port_repairs_the_follow_check_and_ctl_reads_green_parity() {
    let mut world = fixture();
    let after = landing(&mut world, b"src/a.c", b"handoff");
    heard(&mut world, after);
    let change = change(&world, after);
    assert_eq!(follow(&world).project, 2);
    assert_eq!(row(&world).follow, 1);
    assert!(questions::row(&world, change).delegate.is_none());
    owner(&mut world, 190, FollowAct::Start { after });
    let producer = delegate(&mut world, change, 0);
    let context =
        producer.workspace.repositories.iter().find(|r| r.name.as_ref() == b"vsr").expect("source context checkout");
    assert!(context.push.is_none() && context.save.is_none() && !context.delivers);
    port(&mut world, change, &producer, true);
    let repair = delegate(&mut world, change, producer.task);
    assert!(
        repair.sections.iter().any(|section| match &section.body {
            engine::BriefBody::Text(words) => {
                let text = String::from_utf8_lossy(words);
                text.contains("tests/handoff") && text.contains("The follow's check failed at")
            }
            engine::BriefBody::Missing(_) => false,
        }),
        "repair brief: {:?}",
        repair.sections
    );
    assert_eq!(row(&world).parity.as_ref().expect("red verdict").verdict, forge_top::FollowVerdict::Failing(1));
    assert!(world.writes.iter().any(|write| matches!(write, api::Write::Dispatch { inputs, .. }
        if inputs.iter().any(|input| input.name.as_ref() == b"temper_follows" && input.value == forge_top::commit_text(&after)))));
    port(&mut world, change, &repair, false);
    landed(&mut world, change);
    assert!(row(&world).queue.is_empty());
    let dispatches: Vec<_> = world
        .writes
        .iter()
        .filter_map(|write| match write {
            api::Write::Dispatch { workflow, inputs, .. } if workflow.as_ref() == b"test.yml" => Some(inputs),
            _ => None,
        })
        .collect();
    assert_eq!(dispatches.len(), 2, "the red and repaired heads each run their own check");
    assert!(dispatches.iter().all(|inputs| {
        inputs
            .iter()
            .any(|input| input.name.as_ref() == b"temper_follows" && input.value == forge_top::commit_text(&after))
    }));
    let parity = row(&world).parity.as_ref().expect("green verdict");
    assert_eq!((parity.source, &parity.verdict), (after, &forge_top::FollowVerdict::Passed));
    session(&mut world);
    let printed = ctl_list(&mut world);
    let after = String::from_utf8(forge_top::commit_text(&after).into_vec()).expect("commit hex is UTF-8");
    assert_eq!(
        printed,
        format!("follow 1: org/vsr main → org/vsr-rs main · caught up · check passes at {}\n", &after[..12])
    );
    world.close();
}

#[test]
fn cold_engine_restart_keeps_three_entries_live_work_and_reasks_the_inflight_read() {
    let mut world = fixture();
    let first = push(&mut world, b"first");
    let first_change = change(&world, first);
    owner(&mut world, 190, FollowAct::Start { after: first });
    let producer = delegate(&mut world, first_change, 0);
    let follow_task = row(&world).task.expect("standing follow");
    party_task(&mut world, 191, people::Ask::Stop { project: 2, task: follow_task });
    let second = landing(&mut world, b"src/a.c", b"second");
    heard(&mut world, second);
    let third = landing(&mut world, b"src/a.c", b"third");
    heard(&mut world, third);
    assert_eq!(
        row(&world).queue.iter().filter(|entry| entry.change.is_some()).count(),
        1,
        "one change live at the cut"
    );
    world.drain_cut();
    let saved = row(&world).clone();
    world.held_comparisons = Some(Vec::new());
    let fourth = landing(&mut world, b"include/a.h", b"fourth");
    for _ in 0..TICKS {
        world.tick();
        if world.held_comparisons.as_ref().is_some_and(|calls| !calls.is_empty()) {
            break;
        }
    }
    assert!(!world.held_comparisons.as_ref().expect("comparison held by shell").is_empty(), "cut has a read in flight");
    world.drain_cut();
    assert_eq!(row(&world), &saved);
    // A cold shell drops the old comparison, not a durable cursor or queue entry.
    world.held_comparisons = None;
    world.restart_same();
    heard(&mut world, fourth);
    assert_eq!(row(&world).queue.len(), 4);
    assert_eq!(&row(&world).queue[..3], saved.queue.as_ref());
    assert_eq!(row(&world).queue[3].before, third);
    session(&mut world);
    let revision = live_task(&world, follow_task).revisions.hold;
    party_task(&mut world, 192, people::Ask::Release { project: 2, task: follow_task, revision, again: None });
    let all = [change(&world, first), change(&world, second), change(&world, third), change(&world, fourth)];
    assert_eq!(all[0], first_change, "the live change survives the restart");
    for pair in all.windows(2) {
        assert_eq!(live_task(&world, pair[1]).waiting_on.as_ref(), &[pair[0]]);
    }
    let made: Vec<_> = world
        .store
        .rows
        .values()
        .filter_map(|record| match record {
            Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task)))
                if task.executor == (tasks::Executor::Procedure { connector: 0, code: 2 }) =>
            {
                Some(task.number)
            }
            _ => None,
        })
        .collect();
    assert_eq!(made, all, "one live change per entry, none remade on restart");
    assert_eq!(
        live_task(&world, producer.task).requester,
        tasks::Party::Task(first_change),
        "the producer remains the same delegate"
    );
    world.drain_cut();
    world.restart_same();
    heard(&mut world, fourth);
    assert_eq!(
        row(&world).queue.iter().map(|entry| entry.change.expect("remade entry has its change")).collect::<Vec<_>>(),
        all
    );
    world.close();
}

fn party_task(world: &mut World, key: u8, ask: people::Ask) {
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in: world.signed_in.expect("owner session"),
        key: [key; 16],
        ask,
    }));
    settle(world);
    world.drain_cut();
}

#[test]
fn two_owner_waiting_entries_keep_the_hold_reason_then_merge_land_and_skip() {
    let mut world = fixture();
    let a = push(&mut world, b"first");
    let b = push(&mut world, b"second");
    assert!(questions::row(&world, change(&world, a)).delegate.is_none());
    assert!(questions::row(&world, change(&world, b)).delegate.is_none());
    owner(&mut world, 190, FollowAct::Hold { after: a, reason: b"wait for source fix".as_slice().into() });
    assert_eq!(row(&world).queue.len(), 2);
    assert_eq!(row(&world).queue[0].state, FollowState::HeldByOwner(b"wait for source fix".as_slice().into()));
    owner(&mut world, 191, FollowAct::Merge { afters: Box::new([a, b]) });
    let change = change(&world, a);
    owner(&mut world, 192, FollowAct::Start { after: a });
    let producer = delegate(&mut world, change, 0);
    port(&mut world, change, &producer, false);
    landed(&mut world, change);
    assert!(row(&world).queue.is_empty(), "landing counts both merged entries as followed");
    let c = push(&mut world, b"already ported");
    session(&mut world);
    owner(&mut world, 193, FollowAct::Skip { after: c, reason: b"already ported".as_slice().into() });
    assert!(row(&world).queue.is_empty());
    assert_eq!(row(&world).skips.last().expect("kept skip").reason.as_ref(), b"already ported");
    world.close();
}

#[test]
fn a_middle_skip_rewires_the_four_change_chain_before_cancellation() {
    let mut world = fixture();
    let a = push(&mut world, b"first");
    let b = push(&mut world, b"second");
    let c = push(&mut world, b"third");
    let d = push(&mut world, b"fourth");
    let changes = [change(&world, a), change(&world, b), change(&world, c), change(&world, d)];
    owner(&mut world, 190, FollowAct::Skip { after: b, reason: b"not needed".as_slice().into() });
    assert_eq!(live_task(&world, changes[2]).waiting_on.as_ref(), &[changes[0]]);
    assert_eq!(live_task(&world, changes[3]).waiting_on.as_ref(), &[changes[2]]);
    no_holds(&world, &changes);
    assert_eq!(row(&world).queue.iter().map(|entry| entry.after).collect::<Vec<_>>(), [a, c, d]);
    assert_eq!(row(&world).skips.last().expect("skip").after, b);
    world.close();
}

#[test]
fn a_follow_without_the_targets_context_adoption_is_refused() {
    let mut world = unaccepted_follow(1000, Gate::default(), true, false);
    let before = revision(&world);
    change_setting(&mut world, Some(setting(s::FollowStart::Owner)), 110);
    assert_eq!(revision(&world), before, "unadopted context must not enter settings");
    assert!(!world.store.rows.contains_key(&Key::Forge(forge_top::Key::Follow { project: 2, follow: 1 })));
    assert!(
        world.web_replies.iter().any(|reply| matches!(reply,
        people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Judged { why, .. }))
        if why.as_ref() == b"follow 1: its source 1/3 is not adopted by project 2 as context")),
        "{:?}",
        world.web_replies
    );
    world.close();
}

fn ctl_list(world: &mut World) -> String {
    use jig_client::{self as client, contract as c};
    use skein_lib::{Env, List, Queue};
    use temper_client_tools::{self as tools, root};
    let limits = root::Limits::new(tools::limits::client(), tools::limits::tools());
    let env = Env { now: world.env.now, wall: world.env.wall, limits };
    let mut root = root::Root::unlinked(&env.limits, 71).expect("client limits");
    let mut out = Queue::uncapped();
    for event in [
        client::Event::StartUnlinked { kept: List::with_capacity(0) },
        client::Event::SignIn { with: client::With::Transport },
        client::Event::Up { up: client::Up::Served { served: true, versions: None } },
    ] {
        root::step(&mut root, &env, root::Event::Client { event }, &mut out);
    }
    let mut me_watch = None;
    while let Some(request) = out.pop() {
        if let root::Request::Unlinked {
            request:
                client::UnlinkedRequest::Down {
                    down: client::Down::Open { watch, subject: client::Watchable::Jig(c::Subject::Me) },
                },
        } = request
        {
            me_watch = Some(watch);
        }
    }
    let jig_domain_core::reads::Answer::Document(document) = world.party_read(jig_domain_core::reads::Read::Frame)
    else {
        panic!("owner frame")
    };
    let jig_domain_core::documents::Document::Frame(frame) = *document else { panic!("frame document") };
    let preset = tools::limits::client().client;
    let link = jig_client_protocol_link::Limits::new(preset.domain, preset.exchanges).expect("client limits");
    let models = walking::models();
    let model = models.first().expect("fixture's model");
    let catalogue = jig_protocol_engine_clients::Model {
        new_until: None,
        number: 1,
        source: cs::ModelSource::Stock,
        estimated: false,
        name: model.name.clone(),
        provider: Some(model.provider),
        window: model.window,
        output: model.output,
        efforts: model.efforts.clone(),
        effort: model.effort.clone(),
        prices: jig_protocol_engine_clients::Prices {
            input: model.input_price,
            cached: model.cached_price,
            cache_write: model.cache_write_price,
            output: model.output_price,
            unit: model.price_unit,
        },
        description: None,
        shown: None,
        picker: true,
    };
    let frame = jig_protocol_engine_clients::translate::owner::frame(&frame, &[catalogue], link.wire())
        .expect("owner's frame on wire");
    let me = jig_client_protocol_link::translate::documents::frame(frame, &link).expect("client frame");
    let event = client::Event::Up {
        up: client::Up::Watched {
            watch: me_watch.expect("ctl watches me"),
            event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))),
        },
    };
    root::step(&mut root, &env, root::Event::Client { event }, &mut out);
    let tools::jig::call::Parsed::Call { call, json } =
        tools::table::parse(&[b"follow", b"list", b"--project", b"2"]).expect("ctl follow list arguments")
    else {
        panic!("ctl call")
    };
    root::step(&mut root, &env, root::Event::Call { number: 1, call, json, value: None }, &mut out);
    let mut watch = None;
    while let Some(request) = out.pop() {
        if let root::Request::Unlinked {
            request:
                client::UnlinkedRequest::Down {
                    down: client::Down::Open { watch: token, subject: client::Watchable::Family { .. } },
                },
        } = request
        {
            watch = Some(token);
        }
    }
    let document = world
        .oracle_document(world.signed_in.expect("owner session"), 2, forge_top::follows_subject(2))
        .expect("authenticated follows document");
    let bytes = temper_service::family::document(
        document,
        &temper_service::forges::Forges::with_capacity(0),
        &temper_client_protocol_wire::v1::CEILINGS,
    )
    .expect("follows document fits wire limits");
    let event = client::Event::Up {
        up: client::Up::FamilyDelivered {
            watch: watch.expect("ctl watches follows"),
            record: client::Family {
                family: temper_client_protocol_wire::FORGE,
                version: temper_client_protocol_wire::CLIENT_VERSION,
                bytes,
            },
        },
    };
    root::step(&mut root, &env, root::Event::Client { event }, &mut out);
    while let Some(request) = out.pop() {
        if let root::Request::Printed { printed, .. } = request {
            assert_eq!(printed.status, 0, "{printed:?}");
            return String::from_utf8(printed.stdout.into_vec()).expect("ctl prints UTF-8");
        }
    }
    panic!("ctl follow list did not print");
}
