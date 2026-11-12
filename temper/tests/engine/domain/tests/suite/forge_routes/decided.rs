//! A task's named decision is live and durable (temper/docs/design/domain/forge/attention.md, 5).
use super::*;
use skein_lib::Wall;
use temper_protocol_reaches::{Class, Classes, Item, Kind, Offer, PathSet, Reach, Reason, Subject, Way};

const CHANNEL: Token = Token::new(7);
const CALL: u64 = 96;
const STEPS: u32 = 200; // A root call's durable round trip, without forge IO.

fn start(tracked: bool, words: &[u8]) -> (World, engine::Assignment) {
    start_as(tracked, !tracked, people::EndedBy::Result, words)
}

fn start_as(tracked: bool, conversation: bool, ended_by: people::EndedBy, words: &[u8]) -> (World, engine::Assignment) {
    let mut world = World::configured_limits(
        true,
        true,
        true,
        1000,
        Gate::default(),
        None,
        StoryLimits { slots: Some(3), kinds: true, ..StoryLimits::default() },
    );
    world.adopt();
    world.send(protocol::hello_slots(CHANNEL, 3, Box::new([])));
    let people::Ask::Create { project, mut batch } = world.chat(1, words) else { panic!("chat template") };
    batch[0].tracked = tracked.then_some(0);
    batch[0].flags.conversation = conversation;
    batch[0].flags.ended_by = ended_by;
    batch[0].budget = people::Budget::AsAllowed { cap: Some(400) };
    batch[0].authority.tools = 1 | 2 | 4 | 512;
    batch[0].authority.delegation.tasks = 11;
    batch[0].authority.delegation.depth = 3;
    let run = assigned_chat(&mut world, 92, people::Ask::Create { project, batch });
    world.until(Until::Ready);
    (world, run)
}

fn away(world: &mut World, on: bool, until: Option<Wall>) {
    world.send(engine::Event::Forge(Box::new(forge_top::Event::SetAway {
        request: Token::new(301),
        token: Token::new(302),
        party: 1,
        project: 1,
        on,
        until,
    })));
    world.until(Until::Ready);
}

fn tool(kind: temper_protocol_reaches::Kind) -> engine::Tool {
    engine::Tool::Decided(forge_top::DecidedCall {
        question: Box::from(b"Which implementation?".as_slice()),
        options: Box::new([Box::from(b"Small".as_slice()), Box::from(b"Broad".as_slice())]),
        took: 0,
        why: Box::from(b"Fits the design".as_slice()),
        kind,
        paths: Box::new([]),
        under: 0,
    })
}

fn rows(world: &World) -> Vec<forge_top::Decided> {
    world
        .store
        .rows
        .values()
        .filter_map(|record| match record {
            Record::Forge { row } => match row.as_ref() {
                forge_top::Stored::Decided(row) => Some(row.clone()),
                _ => None,
            },
            Record::Core(_) => None,
        })
        .collect()
}

fn reach(
    kind: temper_protocol_reaches::Kind,
    class: temper_protocol_reaches::Class,
    away: bool,
) -> temper_protocol_reaches::Reach {
    temper_protocol_reaches::Reach {
        kind,
        class,
        reason: temper_protocol_reaches::Reason::Kind,
        under: 0,
        sets: 0,
        matched: 0,
        away,
    }
}

#[derive(Clone, Copy, Debug)]
enum Caller {
    Goal,
    Chat,
    PersonTask,
    Standing,
    ServedGoal,
    GoalsChild,
}

#[derive(Clone, Copy)]
enum After {
    Nothing,
    Narrow,
    Overturn,
}

struct Case {
    name: &'static str,
    caller: Caller,
    kind: Kind,
    away: bool,
    words: &'static [u8],
    custom: bool,
    expired: bool,
    paths: &'static [&'static [u8]],
    under: u8,
    reach: Option<Reach>,
    reason: &'static str,
    after: After,
}

fn case(name: &'static str, caller: Caller, kind: Kind, class: Class, away: bool, reason: &'static str) -> Case {
    Case {
        name,
        caller,
        kind,
        away,
        words: b"Make a small fix",
        custom: false,
        expired: false,
        paths: &[],
        under: 0,
        reach: Some(reach(kind, class, away)),
        reason,
        after: After::Nothing,
    }
}

fn caller(case: &Case) -> (World, engine::Assignment) {
    let (mut world, parent) = match case.caller {
        Caller::Goal | Caller::GoalsChild => start(true, case.words),
        Caller::Chat | Caller::ServedGoal => start(false, case.words),
        Caller::PersonTask => start_as(false, false, people::EndedBy::Result, case.words),
        Caller::Standing => start_as(false, false, people::EndedBy::Standing, case.words),
    };
    let tool = match case.caller {
        Caller::Goal | Caller::Chat | Caller::PersonTask | Caller::Standing => return (world, parent),
        Caller::ServedGoal => engine::Tool::Goal { words: case.words.into(), advisor: false, agree: false },
        Caller::GoalsChild => {
            let mut child = temper_engine_domain_world::fixtures::report_delegate(case.words, Box::new([]));
            child.authority.tools = parent.run.authority.tools;
            engine::Tool::Delegate { batch: Box::new([child]) }
        }
    };
    let answer = world.scripted_call(&parent, 93, tool);
    let engine::CallAnswer::Delegated(made) = answer else { panic!("child made: {answer:?}") };
    world.until(Until::SecondAssignment);
    let run = world.assigned.iter().find(|run| run.task == made[0].task).expect("child assignment").clone();
    (world, run)
}

fn classes(world: &mut World, classes: Classes) {
    use forge_top::settings;
    use jig_domain_core::settings as s;
    classes.check().expect("valid classes");
    let value = settings::Value::Classes(classes);
    let bytes = temper_engine_smith::kinds::encode(&value);
    let size = u32::try_from(bytes.len()).expect("classes bytes fit u32");
    let against = temper_engine_domain_world::kinds::revision(&world.store.rows);
    world.send(engine::Event::Party(engine::PartyInput::ChangeSettings {
        reply_to: ReplyTo::new(Token::new(300)),
        sign_in: world.signed_in.expect("owner"),
        key: [30; 16],
        digest: [30; 32],
        change: Box::new(s::Change {
            against,
            items: Box::new([s::Item {
                setting: s::Setting::Application { kind: 8, scope: 1, number: 1 },
                edit: s::Edit::Set(s::Value::Application(bytes)),
            }]),
            running: s::Running::Stays,
        }),
        kinds: Box::new([engine::KindValue {
            kind: 8,
            scope: 1,
            number: 1,
            value: settings::SizedValue { size, value },
        }]),
    }));
    world.until(Until::Ready);
    assert_eq!(
        temper_engine_domain_world::kinds::revision(&world.store.rows),
        against + 1,
        "owner applies classes: {:?}",
        world.web_replies
    );
}

fn custom_classes() -> Classes {
    Classes {
        items: Box::new([
            Item { subject: Subject::Kind(Kind::Implementation), words: Box::new([]), class: Class::Decide },
            Item { subject: Subject::Set(1), words: Box::new([]), class: Class::AskAway },
            Item {
                subject: Subject::Set(1),
                words: b"Fixes that keep the wire format".as_slice().into(),
                class: Class::Decide,
            },
        ]),
        sets: Box::new([PathSet {
            number: 1,
            name: b"Core".as_slice().into(),
            patterns: Box::new([b"src/core/**".as_slice().into()]),
        }]),
    }
}

fn last_how(world: &World) -> engine::AskHow {
    let (name, _) = world.protocol.answers.last().expect("rendered answer");
    *world.protocol.asks.get(name).expect("root supplied HOW beside the decision")
}

fn assert_answer(world: &World, answer: &engine::CallAnswer, expected: Option<Reach>, reason: &str) {
    let took = expected.filter(Reach::decided).map(|_| Box::from(b"Small".as_slice()));
    let words = if took.is_some() {
        "Decided: carry on with Small.".to_owned()
    } else {
        match last_how(world) {
            engine::AskHow::Message => format!("Ask: {reason}. Ask your requester with message, as a question."),
            engine::AskHow::Person(person) => {
                format!("Ask: {reason}. Ask person {person} with delegate, a person task addressed to them.")
            }
            engine::AskHow::None => format!(
                "Not decided: {reason}. Nothing you hold asks anyone: decide within your brief, or leave the choice and your recommendation in your result."
            ),
        }
    };
    assert_eq!(*answer, engine::CallAnswer::Decided { took, reach: expected });
    let event = temper_engine_service::render_for_task_page(
        ReplyTo::new(Token::new(1)),
        answer,
        b"decided",
        1024,
        1,
        1024,
        temper_engine_smith::ForgeReadBounds::STOCK,
        None,
        None,
        last_how(world),
    );
    let engine::Event::Worker(engine::WorkerInput::RenderedHostCall { answer, .. }) = event else { panic!("rendered") };
    let jig_domain_core::SettledAnswer::Host { body, error } = answer else { panic!("host text") };
    assert_eq!(body.as_ref(), words.as_bytes());
    assert!(!error);
}

fn assert_row(world: &World, run: &engine::Assignment, expected: Option<Reach>) {
    let rows = rows(world);
    let Some(reach) = expected else {
        assert!(rows.is_empty(), "no person's question means no forge row");
        return;
    };
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!((row.project, row.party, row.task, row.number), (1, 1, run.task, 1));
    assert_eq!(row.reach, reach);
    assert_eq!(row.question.as_ref(), b"Which implementation?");
    assert_eq!(row.took, reach.decided().then_some(0));
    if reach.decided() {
        assert_eq!(row.options.as_ref(), &[Box::from(b"Small".as_slice()), Box::from(b"Broad".as_slice())]);
        assert_eq!(row.why.as_ref(), b"Fits the design");
    } else {
        assert!(row.options.is_empty());
        assert!(row.why.is_empty());
    }
    assert!(!row.overturned);
}

fn narrow(world: &mut World, run: &engine::Assignment, first: &engine::CallAnswer, completion: u32) {
    let engine::CallAnswer::Decided { reach: Some(reach), .. } = first else { panic!("asked reach") };
    let stock = Classes::stock(Way::AgentsReview);
    classes(
        world,
        temper_protocol_reaches::narrowed(&stock, reach, Offer::Decide(Subject::Kind(Kind::Implementation)), b"")
            .expect("narrowed kind"),
    );
    let event = world.protocol.call(
        CHANNEL,
        run.task,
        run.attempt,
        CALL + 1,
        engine::Call { completion, position: 2, tool: tool(Kind::Implementation) },
    );
    world.send(event);
    world.until(Until::Ready);
    let answer = world.answers.last().expect("next call answered").clone();
    assert_answer(world, &answer, Some(self::reach(Kind::Implementation, Class::Decide, false)), "");
    assert_eq!(rows(world).len(), 2);
    assert_eq!(rows(world)[1].task, run.task);
    assert_eq!(rows(world)[1].party, 1);
    assert_eq!(rows(world)[1].took, Some(0));
    assert_eq!(rows(world)[1].reach, self::reach(Kind::Implementation, Class::Decide, false));
    let before = rows(world);
    let event = world.protocol.call(
        CHANNEL,
        run.task,
        run.attempt,
        CALL + 2,
        engine::Call { completion, position: 1, tool: tool(Kind::Implementation) },
    );
    world.send(event);
    world.until(Until::Ready);
    assert_eq!(world.answers.last(), Some(first), "Stop asking changes only later calls");
    assert_eq!(rows(world), before, "replay writes no row");
}

fn overturn(world: &mut World, run: &engine::Assignment) {
    world.scripted_read_inbox(run);
    let turns = live_task(world, run.task).turn;
    park_after_turn(world, run, None, turns);
    world.until(Until::Ready);
    let attempts = world.assigned.len();
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Overturn {
        request: Token::new(80),
        token: Token::new(80),
        party: 1,
        project: 1,
        number: 1,
    })));
    world.until(Until::Ready);
    assert!(rows(world)[0].overturned);
    assert_eq!(world.assigned.len(), attempts, "Overturn alone sends no message");
    let words = b"I overturn your decision. Ask me again with message, as a question, and wait for my answer.";
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(81)),
        sign_in: world.signed_in.expect("owner"),
        key: [81; 16],
        ask: people::Ask::Say { project: 1, task: run.task, words: words.as_slice().into() },
    }));
    for _ in 0..STEPS {
        world.tick();
        if world.assigned.len() > attempts {
            break;
        }
    }
    let message = world
        .web_replies
        .iter()
        .find_map(|reply| match reply {
            people::Reply::Outcome(people::Outcome::Said { task, message }) if *task == run.task => Some(*message),
            _ => None,
        })
        .expect("message accepted");
    let woke = world.assigned.last().expect("woken task").clone();
    assert_eq!(woke.task, run.task);
    assert!(woke.attempt > run.attempt, "overturned task wakes");
    world.until(Until::Ready);
    assert!(
        woke.messages.iter().any(|word| word.number == message)
            || world
                .delivered_messages
                .get(&(woke.task, woke.attempt))
                .is_some_and(|messages| messages.contains(&message)),
        "the woken task hears the client's message"
    );
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table holds each caller and reach story without another world test")]
fn each_callers_person_and_reach_are_kept() {
    let set_reach =
        Reach { reason: Reason::Set(1), sets: 1, matched: 1, ..reach(Kind::Implementation, Class::AskAway, true) };
    let worded = Reach { under: 3, sets: 1, ..reach(Kind::Implementation, Class::Decide, true) };
    let always = Some(Reach { reason: Reason::Always, ..reach(Kind::Wording, Class::AskAway, true) });
    for case in [
        case(
            "goal here",
            Caller::Goal,
            Kind::Implementation,
            Class::Ask,
            false,
            "implementation asks your person while they are here",
        ),
        Case {
            away: true,
            expired: true,
            ..case(
                "past until",
                Caller::Goal,
                Kind::Implementation,
                Class::Ask,
                false,
                "implementation asks your person while they are here",
            )
        },
        case("goal away", Caller::Goal, Kind::Implementation, Class::Ask, true, ""),
        case("chat", Caller::Chat, Kind::Wording, Class::Decide, false, ""),
        case("person-made untracked task", Caller::PersonTask, Kind::Wording, Class::Decide, false, ""),
        case("standing role", Caller::Standing, Kind::Wording, Class::Decide, false, ""),
        case("served goal", Caller::ServedGoal, Kind::Wording, Class::Decide, false, ""),
        Case {
            reach: None,
            ..case(
                "Goals child",
                Caller::GoalsChild,
                Kind::Wording,
                Class::Decide,
                false,
                "no person is asked through you",
            )
        },
        Case {
            words: b"Asks: always",
            reach: always,
            ..case("always whole", Caller::Goal, Kind::Wording, Class::AskAway, true, "your words say Asks: always")
        },
        Case {
            words: b"Asks: always\nMake a small fix",
            reach: always,
            ..case("always first", Caller::Goal, Kind::Wording, Class::AskAway, true, "your words say Asks: always")
        },
        Case {
            words: b"Make a small fix\nAsks: always",
            reach: always,
            ..case("always last", Caller::Goal, Kind::Wording, Class::AskAway, true, "your words say Asks: always")
        },
        Case {
            words: b"Make a small fix\nAsks: always\nKeep it small",
            reach: always,
            ..case("always middle", Caller::Goal, Kind::Wording, Class::AskAway, true, "your words say Asks: always")
        },
        case("money", Caller::Goal, Kind::Money, Class::Wait, true, "money always waits for your person"),
        case("other", Caller::Goal, Kind::Other, Class::Wait, true, "other always waits for your person"),
        Case {
            words: b"Make a small fix\nAsks: always later",
            ..case("not an always line", Caller::Goal, Kind::Wording, Class::Decide, false, "")
        },
        Case { custom: true, ..case("custom kind", Caller::Goal, Kind::Implementation, Class::Decide, false, "") },
        Case {
            custom: true,
            paths: &[b"src/core/a.rs"],
            reach: Some(set_reach),
            ..case(
                "custom path",
                Caller::Goal,
                Kind::Implementation,
                Class::AskAway,
                true,
                "changes to path set 1 ask your person, even when away",
            )
        },
        Case {
            custom: true,
            paths: &[b"src/core/a.rs"],
            under: 3,
            reach: Some(worded),
            ..case("custom worded item", Caller::Goal, Kind::Implementation, Class::Decide, true, "")
        },
        Case {
            custom: true,
            ..case(
                "missing kind",
                Caller::Goal,
                Kind::Dependency,
                Class::AskAway,
                true,
                "dependency asks your person, even when away",
            )
        },
        Case {
            after: After::Narrow,
            ..case(
                "Stop asking",
                Caller::Goal,
                Kind::Implementation,
                Class::Ask,
                false,
                "implementation asks your person while they are here",
            )
        },
        Case { after: After::Overturn, ..case("overturn chat", Caller::Chat, Kind::Wording, Class::Decide, false, "") },
        Case {
            after: After::Overturn,
            ..case("overturn standing role", Caller::Standing, Kind::Wording, Class::Decide, false, "")
        },
        Case {
            after: After::Overturn,
            ..case("overturn untracked task", Caller::PersonTask, Kind::Wording, Class::Decide, false, "")
        },
    ] {
        let (mut world, run) = caller(&case);
        if case.custom {
            classes(&mut world, custom_classes());
        }
        let until = case.expired.then_some(world.env.wall);
        away(&mut world, case.away, until);
        let engine::Tool::Decided(mut call) = tool(case.kind) else { panic!("decided") };
        call.paths = case.paths.iter().map(|path| Box::from(*path)).collect();
        call.under = case.under;
        world.scripted_read_inbox(&run);
        eprintln!("{}", case.name);
        let answer = world.scripted_call(&run, CALL, engine::Tool::Decided(call));
        let completion = live_task(&world, run.task).turn;
        let how = match case.caller {
            Caller::Goal | Caller::Chat | Caller::PersonTask | Caller::Standing => engine::AskHow::Person(1),
            Caller::ServedGoal | Caller::GoalsChild => engine::AskHow::Message,
        };
        assert_eq!(last_how(&world), how);
        assert_answer(&world, &answer, case.reach, case.reason);
        assert_row(&world, &run, case.reach);
        match case.after {
            After::Nothing => {}
            After::Narrow => narrow(&mut world, &run, &answer, completion),
            After::Overturn => overturn(&mut world, &run),
        }
        world.close();
    }
}

fn asking_change_world(agent: bool) -> (World, engine::Assignment) {
    let mut world = World::configured_limits(
        true,
        true,
        false,
        if agent { 1000 } else { 0 },
        Gate { agent, ..Gate::default() },
        None,
        StoryLimits { slots: Some(3), ..StoryLimits::default() },
    );
    world.adopt();
    world.send(protocol::hello_slots(CHANNEL, 3, Box::new([])));
    let people::Ask::Create { project, mut batch } = adoption_chat(&world) else { panic!("chat template") };
    batch[0].budget = people::Budget::AsAllowed { cap: Some(400) };
    batch[0].authority.tools |= jig_domain_core::ToolFamilies::JIG.control.0;
    let requester = assigned_chat(&mut world, 92, people::Ask::Create { project, batch });
    world.until(Until::Ready);
    (world, requester)
}

#[test]
fn a_changes_producer_and_repair_ask_their_requester() {
    let (mut world, requester) = asking_change_world(false);
    let change = super::questions::make_change(&mut world, &requester, CALL, engine::ChangeQuestions::Up);
    let producer = super::questions::next_producer(&mut world, change);
    let answer = world.scripted_call(&producer, CALL + 1, tool(Kind::Implementation));
    assert_eq!(last_how(&world), engine::AskHow::Message);
    assert_answer(&world, &answer, None, "no person is asked through you");
    assert!(rows(&world).is_empty());
    let branch = super::questions::row(&world, change).branch.clone();
    let assigned = world.assigned.len();
    failed_ci_repair(&mut world, &producer, &branch, &[b"build"]);
    let repair = super::reviews::next_assignment(&mut world, assigned);
    assert!(live_task(&world, repair.task).spec.words.starts_with(b"Repair"));
    let answer = world.scripted_call(&repair, CALL + 2, tool(Kind::Implementation));
    assert_eq!(last_how(&world), engine::AskHow::Message);
    assert_answer(&world, &answer, None, "no person is asked through you");
    assert!(rows(&world).is_empty());
    world.close();
}

#[test]
fn a_changes_review_is_not_told_to_ask_with_a_tool_it_lacks() {
    let (mut world, requester) = asking_change_world(true);
    let change = super::questions::make_change(&mut world, &requester, CALL, engine::ChangeQuestions::Up);
    let producer = super::questions::next_producer(&mut world, change);
    let assigned = world.assigned.len();
    let branch = super::questions::row(&world, change).branch.clone();
    super::reviews::produce(&mut world, &producer, &branch);
    let review = super::reviews::next_assignment(&mut world, assigned);
    let row = live_task(&world, review.task);
    let revision = row.revisions.spec;
    let read = jig_domain_core::ToolFamilies::JIG.read.0;
    let control = jig_domain_core::ToolFamilies::JIG.control.0;
    // The requester grants Control so the review can call decided, but no Message.
    let mut authority = row.authority.clone();
    authority.tools.0 |= control;
    let amendment = temper_engine_domain_world::fixtures::task_amendment(
        None,
        Some(authority),
        b"Judge this choice within the review",
    );
    world.scripted_call(
        &requester,
        CALL + 1,
        engine::Tool::Amend { target: review.task, revision, amendment, keep_parameters: false },
    );
    assert_eq!(live_task(&world, review.task).authority.tools.0, read | control, "{:?}", world.answers);
    let answer = world.scripted_call(&review, CALL + 2, tool(Kind::Implementation));
    assert_eq!(last_how(&world), engine::AskHow::None);
    assert_answer(&world, &answer, None, "no person is asked through you");
    world.close();
}

#[test]
fn a_replayed_decision_reads_how_from_the_caller_now() {
    let (mut world, requester) = asking_change_world(false);
    let change = super::questions::make_change(&mut world, &requester, CALL, engine::ChangeQuestions::Up);
    let producer = super::questions::next_producer(&mut world, change);
    let answer = world.scripted_call(&producer, CALL + 1, tool(Kind::Implementation));
    let completion = live_task(&world, producer.task).turn;
    assert_eq!(last_how(&world), engine::AskHow::Message);
    world.until(Until::Ready);
    // Restore the saved decision with a caller that no longer holds Message.
    let mut caller = live_task(&world, producer.task).clone();
    caller.authority.tools.0 &= !jig_domain_core::ToolFamilies::JIG.message.0;
    world.store.seed(
        Box::new([Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(Box::new(caller))))]),
        Box::new([]),
    );
    world.restart_same();
    world.until(Until::Ready);
    let before = world.store.rows.clone();
    let event = world.protocol.call(
        CHANNEL,
        producer.task,
        producer.attempt,
        CALL + 3,
        engine::Call { completion, position: 1, tool: tool(Kind::Implementation) },
    );
    world.send(event);
    world.until(Until::Ready);
    assert_eq!(world.answers.last(), Some(&answer));
    assert_eq!(last_how(&world), engine::AskHow::None);
    assert_answer(&world, &answer, None, "no person is asked through you");
    assert_eq!(world.store.rows, before, "rendering HOW writes no new decision or settled record");
    world.close();
}

#[test]
fn a_replay_answers_the_same_after_the_switch_moved() {
    for first in [false, true] {
        let (mut world, run) = start(true, b"Make a small fix");
        away(&mut world, first, None);
        let call =
            engine::Call { completion: 1, position: 1, tool: tool(temper_protocol_reaches::Kind::Implementation) };
        let event = world.protocol.call(CHANNEL, run.task, run.attempt, CALL, call.clone());
        world.send(event);
        world.until(Until::Ready);
        let original = world.answers.last().unwrap().clone();
        let before = rows(&world);
        away(&mut world, !first, None);
        let answers = world.answers.len();
        let event = world.protocol.call(CHANNEL, run.task, run.attempt, CALL + 1, call);
        world.send(event);
        world.until(Until::Ready);
        assert_eq!(world.answers.len(), answers + 1);
        assert_eq!(world.answers.last(), Some(&original));
        assert_eq!(rows(&world), before, "replay writes no second decision");
        world.close();
    }
}

#[test]
fn an_advisor_is_beyond_authority() {
    let (mut world, goal) = start(true, b"Make a small fix");
    let engine::CallAnswer::Delegated(made) = world.scripted_call(
        &goal,
        93,
        engine::Tool::Goal { words: Box::from(b"Survey".as_slice()), advisor: true, agree: false },
    ) else {
        panic!("advisor delegated")
    };
    world.until(Until::SecondAssignment);
    let run = world.assigned.iter().find(|run| run.task == made[0].task).expect("child assignment").clone();
    away(&mut world, true, None);
    let answer = world.scripted_call(&run, CALL, tool(temper_protocol_reaches::Kind::Implementation));
    assert!(matches!(answer, engine::CallAnswer::ToolDenied { .. }), "{answer:?}");
    let rendered =
        temper_engine_smith::answer_for_tool(&answer, b"decided", 1024, temper_engine_smith::ForgeReadBounds::STOCK);
    assert!(String::from_utf8_lossy(rendered.text()).contains("control"));
    assert!(rows(&world).is_empty());
    world.close();
}

#[test]
fn decided_answers_in_its_decision() {
    let (mut world, run) = start(true, b"Make a small fix");
    away(&mut world, true, None);
    world.scripted_read_inbox(&run);
    world.until(Until::Ready);
    let event = world.protocol.call(
        CHANNEL,
        run.task,
        run.attempt,
        CALL,
        engine::Call { completion: 1, position: 1, tool: tool(temper_protocol_reaches::Kind::Implementation) },
    );
    world.send(event);
    let mut found = false;
    for _ in 0..STEPS {
        for (_, writes) in &world.store.pending {
            if writes.iter().any(|write| matches!(write, engine::Write::Save(Record::Forge { row }) if matches!(row.as_ref(), forge_top::Stored::Decided(_)))) {
                assert!(writes.iter().any(|write| matches!(write, engine::Write::Save(Record::Forge { row }) if matches!(row.as_ref(), forge_top::Stored::Call { answer: forge_top::CallPayload::Decided { took: Some(_), .. }, .. }))), "decision row and named answer share a commit");
                assert!(writes.iter().any(|write| matches!(write, engine::Write::Save(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Call(_)))))), "core call shares the decision");
                found = true;
            }
        }
        if found {
            break;
        }
        if let Some(event) = world.events.pop_front() {
            world.send(event);
        } else {
            world.tick();
        }
    }
    assert!(found, "the decoded call answers in its decision");
    world.until(Until::Ready);
    world.close();
}
