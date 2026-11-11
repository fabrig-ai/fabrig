//! The switch and decisions cross both roots (temper/docs/design/client/pages/README.md
//! , Away, 4.6 and 5.13; temper/docs/design/domain/forge/attention.md, 4 and 5).
use jig_client_kit::harness::{App, ClientId, Duty, Front};
use skein_lib::{Duration, Env, Queue, RequestEnvelope, Time, Wall};
use skein_ui::{Role, State};
use smith_domain_session::llm::{self, Block, Message};
use temper_client_domain::{
    Action, Address, Event, Request,
    forge::away::AwayRow,
    pages::{Act, all_home, away, home, needs},
};
use temper_client_protocol_wire::{self as wire, v1 as w};
use temper_client_world::{
    app::{Root, Temper, View},
    fixtures,
    front::{HostAnswer, Joint},
};
use temper_engine_domain::{Key as StoreKey, Record};
use temper_engine_domain_forge as forge;

const CLIENT: ClientId = ClientId(0);
const SECOND_DEVICE: ClientId = ClientId(1);
const PROJECT: u32 = 1;
const OTHER_PROJECT: u32 = 2;
const PUMP_ROUNDS: usize = 32;
const OFFLINE: Duration = Duration::from_millis(30);
const WORKER: skein_lib::Token = skein_lib::Token::new(7);
const LOCAL_DECISION: &[u8] = br#"{"question":"Which local implementation?","options":["Small","Broad"],"took":0,"why":"The small implementation fits the design.","kind":"implementation"}"#;

struct Probe {
    root: Root,
    out: Queue<Request>,
    answers: Vec<jig_client::Answered>,
    decided_watch: Option<skein_lib::Token>,
    decisions: Option<w::Decisions>,
}
struct Story {
    front: Joint,
    probes: Vec<Probe>,
    env: Env<temper_client_domain::Limits>,
    inputs: Vec<(u64, u64, u64, Box<[u8]>)>,
    keep_settings: bool,
}
impl Story {
    fn new() -> Self {
        let mut limits = fixtures::domain_limits();
        // Home may open while the previous page's handles retire.
        // The session also holds trials and a running trial task.
        limits.client.domain.handles = 18;
        limits.client.domain.watches = 18;
        Self::with_front(Joint::with_away_people(fixtures::SEED, limits), limits)
    }
    fn with_front(front: Joint, limits: temper_client_domain::Limits) -> Self {
        let mut story = Self {
            front,
            probes: Vec::new(),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            inputs: Vec::new(),
            keep_settings: false,
        };
        assert_eq!(story.client(fixtures::PARTY), CLIENT);
        story
    }
    fn client(&mut self, party: u64) -> ClientId {
        let who = ClientId(self.probes.len());
        self.front.bind(who, party);
        self.start_client(who);
        who
    }
    fn start_client(&mut self, who: ClientId) {
        assert_eq!(who.0, self.probes.len());
        self.probes.push(Probe {
            root: Root::Person(Box::new(temper_client_domain::unlinked(&self.env.limits, fixtures::SEED))),
            out: Queue::uncapped(),
            answers: Vec::new(),
            decided_watch: None,
            decisions: None,
        });
        self.event(
            who,
            Temper::start(Address::Welcome { step: temper_client_domain::address::Step::Welcome }, Vec::new(), 0),
        );
        self.event(
            who,
            Event::Client {
                event: jig_client::Event::Unwatch { handle: temper_client_domain::pages::accounts::handle() },
            }
            .into(),
        );
        // Apply needs the newest settings revision; the other stories reserve that watch for Away and Home.
        if !self.keep_settings {
            let handle = self.probes[who.0].root.person().forge().settings_handle();
            self.event(who, Event::Client { event: jig_client::Event::Unwatch { handle } }.into());
        }
    }
    fn event(&mut self, who: ClientId, event: temper_client_world::app::Event) {
        let probe = &mut self.probes[who.0];
        Temper::step(&mut probe.root, &self.env, event, &mut probe.out);
        self.pump();
    }
    fn pump(&mut self) {
        for _ in 0..PUMP_ROUNDS {
            let mut worked = false;
            for (index, probe) in self.probes.iter_mut().enumerate() {
                Temper::take(&mut probe.root, &self.env, &mut probe.out);
                while let Some(request) = probe.out.pop() {
                    worked = true;
                    if let Duty::Wire(request) = Temper::duty(request) {
                        if let Request::Unlinked {
                            request:
                                jig_client::UnlinkedRequest::Down { down: jig_client::Down::Open { watch, subject } },
                        } = &request
                            && Some(subject)
                                == temper_client_domain::forge::decided::watch(PROJECT, &self.env.limits.forge).as_ref()
                        {
                            probe.decided_watch = Some(*watch);
                        }
                        self.front.wire(ClientId(index), self.env.now, self.env.wall, request);
                    }
                }
            }
            self.hear();
            let callbacks = self.front.poll(self.env.now, self.env.wall);
            self.hear();
            if !worked && callbacks.is_empty() {
                return;
            }
            for (who, event) in callbacks {
                self.front.received(who, &event);
                let probe = &mut self.probes[who.0];
                if let Some(Event::Client {
                    event:
                        jig_client::Event::Up {
                            up: jig_client::Up::Answer { envelope: RequestEnvelope::Final, answer: Some(answer), .. },
                        },
                }) = event.root()
                {
                    probe.answers.push(answer.clone());
                }
                if let Some(Event::Client {
                    event: jig_client::Event::Up { up: jig_client::Up::FamilyDelivered { watch, record } },
                }) = event.root()
                    && Some(*watch) == probe.decided_watch
                {
                    probe.decisions = Some(
                        temper_client_domain::forge::decided::document(record, self.env.limits.forge)
                            .expect("decisions document"),
                    );
                }
                Temper::step(&mut probe.root, &self.env, event, &mut probe.out);
            }
        }
        panic!("Away clients and engine did not settle");
    }
    fn act(&mut self, act: away::Act) {
        self.event(CLIENT, Event::Act { action: Action::Pages(Act::Away(act)) }.into());
    }
    fn toggle(&mut self) {
        self.act(away::Act::Toggle { project: PROJECT });
    }
    fn after(&mut self, elapsed: Duration) {
        self.env.now = self.env.now.saturating_add(elapsed);
        self.env.wall = Wall::from_nanos(self.env.now.as_nanos());
        for probe in &mut self.probes {
            Temper::fire(&mut probe.root, &self.env, &mut probe.out);
        }
        self.pump();
    }
    fn row(&self, who: ClientId) -> Option<AwayRow> {
        self.probes[who.0].root.person().forge().away(PROJECT)
    }
    fn stored(&self) -> forge::Away {
        let Record::Forge { row } = self
            .front
            .engine
            .store
            .rows
            .get(&StoreKey::Forge(forge::Key::Away { party: fixtures::PARTY, project: PROJECT }))
            .expect("committed Away")
        else {
            panic!("forge record")
        };
        let forge::Stored::Away(row) = row.as_ref() else { panic!("Away row") };
        *row
    }
    fn home(&mut self, who: ClientId) {
        self.event(who, Temper::went(Address::Home { project: PROJECT }));
        if self.facts(who).to_begin() {
            self.event(who, Event::Act { action: Action::Pages(Act::SkipToBegin) }.into());
        }
        assert!(!self.facts(who).to_begin());
    }
    fn facts(&self, who: ClientId) -> home::Facts {
        home::facts(self.probes[who.0].root.person(), PROJECT, self.env.wall)
    }
    fn decide(&mut self, goal: u64, completion: u32) -> String {
        let HostAnswer::Tool(answer) = self.front.host_tool(
            goal,
            completion,
            temper_engine_domain::Tool::Decided(forge::DecidedCall {
                question: b"Which implementation?".as_slice().into(),
                options: Box::new([b"Small".as_slice().into(), b"Broad".as_slice().into()]),
                took: 0,
                why: b"Fits the design".as_slice().into(),
                kind: temper_protocol_reaches::Kind::Implementation,
                paths: Box::new([]),
                under: 0,
            }),
        ) else {
            panic!("tool answer")
        };
        let rendered = temper_engine_smith::answer_for_tool(
            &answer,
            b"decided",
            1024,
            temper_engine_smith::ForgeReadBounds::STOCK,
        );
        let text = String::from_utf8(rendered.text().to_vec()).expect("tool words");
        self.pump();
        text
    }
    fn decisions(&self) -> Vec<forge::Decided> {
        self.front
            .engine
            .store
            .rows
            .values()
            .filter_map(|record| {
                if let Record::Forge { row } = record
                    && let forge::Stored::Decided(row) = row.as_ref()
                {
                    Some(row.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Story {
    fn hear(&mut self) {
        for delivery in &self.front.engine.delivered {
            if let temper_engine_domain::Delivery::Link {
                down: jig_domain_host::LinkDown::Message { run, attempt, name, words, .. },
                ..
            } = delivery
            {
                let input = (run.raw(), attempt.raw(), name.raw(), words.clone());
                if !self.inputs.contains(&input) {
                    self.inputs.push(input);
                }
            }
        }
    }
    fn sequence() -> Self {
        Self::sequence_with_settings(false)
    }
    fn sequence_with_settings(keep_settings: bool) -> Self {
        let mut limits = fixtures::domain_limits();
        limits.client.domain.handles = 18;
        limits.client.domain.watches = 18;
        let mut story = Self {
            front: Joint::with_away_sequence(fixtures::SEED, limits),
            probes: Vec::new(),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            inputs: Vec::new(),
            keep_settings,
        };
        assert_eq!(story.client(fixtures::PARTY), CLIENT);
        story
    }
    fn begin_goal(&mut self) -> temper_engine_domain::Assignment {
        use jig_domain_people as people;
        use temper_engine_domain as engine;
        self.front.engine.sign_in();
        self.front.engine.settle();
        let people::Ask::Create { project, mut batch } =
            self.front.engine.chat_ask(PROJECT, b"Start a local away goal.")
        else {
            panic!("goal template")
        };
        batch[0].authority.tools = 1023;
        batch[0].authority.notes = jig_domain_authority::Scopes::GOAL.0;
        self.front.engine.send(engine::Event::Party(engine::PartyInput::Ask {
            models: Box::new([]),
            reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(2301)),
            sign_in: self.front.engine.session(),
            key: [83; 16],
            ask: people::Ask::Create { project, batch },
        }));
        self.pump();
        let chat = self.front.assignments.last().expect("chat assigned").clone();
        self.call(&chat, 1, b"goal", br#"{"words":"Walk the local away decision."}"#);
        let goal = self.front.assignments.last().expect("goal assigned").clone();
        assert_ne!(goal.task, chat.task);
        self.front.engine.answer(
            WORKER,
            chat.task,
            chat.attempt,
            1,
            jig_domain_tasks::End::Parked,
            engine::Parts::none(),
        );
        self.pump();
        assert!(self.front.engine.task(goal.task).expect("live goal").tracked.is_some());
        goal
    }
    fn visit_home(&mut self) {
        self.event(CLIENT, Temper::went(Address::Home { project: PROJECT }));
        if self.facts(CLIENT).to_begin() {
            self.press(Role::Button, b"Skip");
        }
        assert!(!self.facts(CLIENT).to_begin());
    }
    fn needs(&mut self) {
        self.event(
            CLIENT,
            Temper::went(Address::Inbox {
                filter: temper_client_domain::address::Filter {
                    project: Some(PROJECT),
                    ..temper_client_domain::address::Filter::ALL
                },
            }),
        );
    }
    fn visit_how(&mut self) {
        self.event(CLIENT, Temper::went(Address::Settings { project: PROJECT }));
        let view = self.view();
        let nodes = Temper::tree(&view).nodes();
        let (at, heading) = nodes
            .iter()
            .enumerate()
            .find(|(_, node)| {
                node.role == Role::Heading { level: 2 } && node.name.as_deref() == Some(b"What reaches you")
            })
            .expect("What reaches you heading");
        let project = nodes
            .iter()
            .skip(at)
            .nth(usize::try_from(heading.size).expect("heading subtree size"))
            .expect("project at the heading's right");
        assert_eq!(project.role, Role::Text);
        assert_eq!(project.name.as_deref(), Some(format!("on Project {PROJECT}").as_bytes()));
    }
    fn view(&self) -> View {
        let mut view = if self.keep_settings {
            let mut words = fixtures::phrasebooks(&self.env.limits);
            words.shell = temper_client_view::words::Shell::Browser;
            let bounds = fixtures::view_bounds(&self.env.limits, &words);
            View::Person(Box::new(temper_client_view::View::new(&bounds, &self.env.limits, words, fixtures::FORGE)))
        } else {
            Temper::new(fixtures::SEED).1
        };
        let View::Person(person) = &mut view else { panic!("person view") };
        temper_client_view::build(person, self.probes[CLIENT.0].root.person(), self.env.wall, &mut Queue::uncapped());
        view
    }
    fn reaches_lines(&self, group: &[u8]) -> Vec<String> {
        let view = self.view();
        let nodes = Temper::tree(&view).nodes();
        let (at, column) = nodes
            .iter()
            .enumerate()
            .find(|(_, node)| node.role == Role::Group && node.name.as_deref() == Some(group))
            .unwrap_or_else(|| panic!("column {}", String::from_utf8_lossy(group)));
        nodes
            .iter()
            .skip(at)
            .take(usize::try_from(column.size).expect("column subtree size"))
            .filter(|node| node.role == Role::Item || node.role == Role::Paragraph)
            .map(|node| String::from_utf8(node.name.as_deref().expect("column line words").to_vec()).expect("UTF-8"))
            .collect()
    }
    fn press(&mut self, role: Role, name: &[u8]) {
        let mut view = self.view();
        let node = Temper::tree(&view)
            .nodes()
            .iter()
            .find(|node| node.role == role && node.name.as_deref() == Some(name))
            .unwrap_or_else(|| panic!("control {} in {:?}", String::from_utf8_lossy(name), Temper::tree(&view)));
        assert!(!node.states.as_slice().contains(&State::Disabled));
        let id = node.id;
        let event = Temper::act(&mut view, id, skein_ui::Act::Press).expect("bound press");
        self.event(CLIENT, event);
    }
    fn open_need(&mut self, question: &[u8]) {
        let mut view = self.view();
        let nodes = Temper::tree(&view).nodes();
        let (_, row) = nodes
            .iter()
            .enumerate()
            .find(|(index, node)| {
                node.role == Role::Button
                    && nodes.iter().skip(*index).take(usize::try_from(node.size).expect("subtree size")).any(|child| {
                        child.text.iter().any(|run| run.text.windows(question.len()).any(|part| part == question))
                    })
            })
            .expect("closed need row contains the question");
        let id = row.id;
        let event = Temper::act(&mut view, id, skein_ui::Act::Press).expect("bound need row");
        self.event(CLIENT, event);
        assert!(
            Temper::tree(&self.view())
                .nodes()
                .iter()
                .any(|node| { node.role == Role::Heading { level: 1 } && node.name.as_deref() == Some(question) })
        );
    }
    fn sees(&self, words: &[u8]) {
        let view = self.view();
        assert!(
            Temper::tree(&view).nodes().iter().any(|node| {
                node.name.as_deref().is_some_and(|name| name.windows(words.len()).any(|part| part == words))
                    || node.text.iter().any(|run| run.text.windows(words.len()).any(|part| part == words))
            }),
            "{} in {:?}",
            String::from_utf8_lossy(words),
            Temper::tree(&view)
        );
    }
    fn does_not_see(&self, words: &[u8]) {
        let view = self.view();
        assert!(
            !Temper::tree(&view).nodes().iter().any(|node| {
                node.name.as_deref().is_some_and(|name| name.windows(words.len()).any(|part| part == words))
                    || node.text.iter().any(|run| run.text.windows(words.len()).any(|part| part == words))
            }),
            "{} absent in {:?}",
            String::from_utf8_lossy(words),
            Temper::tree(&view)
        );
    }
    fn call(&mut self, run: &temper_engine_domain::Assignment, completion: u32, name: &[u8], input: &[u8]) -> String {
        let (answer, error) =
            temper_engine_domain_world::host_tool::host_tool(&mut self.front.engine, run, completion, name, input);
        assert!(!error, "{}: {}", String::from_utf8_lossy(name), String::from_utf8_lossy(&answer));
        let id = format!("call-{completion}").into_bytes().into_boxed_slice();
        let mut turn =
            temper_worker_domain_world::protocol::session_turn(completion, u64::from(completion), 0, completion == 1);
        turn.messages = Box::new([
            Message {
                role: llm::Role::Assistant,
                content: Box::new([Block::ToolCall(llm::ToolCall {
                    id: id.clone(),
                    name: name.into(),
                    input: input.into(),
                    call: llm::Decoded::Historical,
                    replay: None,
                })]),
            },
            Message {
                role: llm::Role::User,
                content: Box::new([Block::ToolResult {
                    id,
                    result: llm::Returned::Text { text: answer.clone(), error, replay: None },
                }]),
            },
        ]);
        let body = smith_protocol_channel::encode_turn(
            &turn,
            &smith_transcript::CEILINGS,
            &temper_worker_domain_world::protocol::endpoints(),
        )
        .expect("goal's tool exchange encodes");
        let read = self
            .inputs
            .iter()
            .filter(|(task, attempt, _, _)| *task == run.task && *attempt == run.attempt)
            .map(|(_, _, name, _)| *name)
            .max();
        self.front.engine.send(temper_engine_domain_world::protocol::turn(
            WORKER,
            run.task,
            run.attempt,
            completion,
            u64::from(completion),
            read,
            &body,
        ));
        self.front.engine.settle();
        self.pump();
        String::from_utf8(answer.into_vec()).expect("host answer words")
    }
    fn answer(&mut self, run: &temper_engine_domain::Assignment, choice: &[u8]) {
        self.press(Role::Button, choice);
        assert!(
            !self.inputs.iter().any(|(task, attempt, _, words)| {
                *task == run.task && *attempt == run.attempt && words.as_ref() == choice
            }),
            "the answer waits for its hold"
        );
        self.after(needs::HOLD.saturating_add(Duration::from_nanos(1)));
        assert!(
            self.inputs.iter().any(|(task, attempt, _, words)| {
                *task == run.task && *attempt == run.attempt && words.as_ref() == choice
            }),
            "the run receives {} after the hold",
            String::from_utf8_lossy(choice)
        );
    }
    fn finish(&mut self, run: &temper_engine_domain::Assignment) {
        let report = b"The person answered the local question.";
        let mut turn = temper_worker_domain_world::protocol::session_turn(6, 6, 0, false);
        turn.messages[0].content = Box::new([Block::Text { text: report.as_slice().into(), replay: None }]);
        let body = smith_protocol_channel::encode_turn(
            &turn,
            &smith_transcript::CEILINGS,
            &temper_worker_domain_world::protocol::endpoints(),
        )
        .expect("report turn");
        let read = self
            .inputs
            .iter()
            .filter(|(task, attempt, _, _)| *task == run.task && *attempt == run.attempt)
            .map(|(_, _, name, _)| *name)
            .max();
        self.front.engine.send(temper_engine_domain_world::protocol::turn(
            WORKER,
            run.task,
            run.attempt,
            6,
            6,
            read,
            &body,
        ));
        self.front.engine.settle();
        self.front.engine.answer(
            WORKER,
            run.task,
            run.attempt,
            6,
            jig_domain_tasks::End::Finished {
                result: jig_domain_tasks::TaskResult::Report { words: report.as_slice().into() },
                cancel_delegates: false,
            },
            temper_engine_domain::Parts::none(),
        );
        self.pump();
        let ended = self
            .front
            .engine
            .store
            .rows
            .get(&StoreKey::Core(jig_domain_core::Key::Tasks(jig_domain_tasks::Key::Ended(run.task))));
        let Some(Record::Core(jig_domain_core::Record::Tasks(jig_domain_tasks::Stored::Ended(task)))) = ended else {
            panic!("the goal ended: {ended:?}")
        };
        assert_eq!(
            task.phase,
            jig_domain_tasks::Phase::Ended(jig_domain_tasks::Ending::Done(jig_domain_tasks::TaskResult::Report {
                words: report.as_slice().into()
            },))
        );
    }
    fn committed_calls(&self, goal: u64) -> Vec<Box<[u8]>> {
        let mut turns: Vec<_> = self
            .front
            .engine
            .store
            .rows
            .values()
            .filter_map(|row| {
                if let Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Turn(turn))) = row
                    && turn.task == goal
                {
                    Some(turn)
                } else {
                    None
                }
            })
            .collect();
        turns.sort_by_key(|turn| (turn.attempt, turn.turn));
        let mut calls = Vec::new();
        for turn in turns {
            let decoded = smith_protocol_channel::decode_turn(
                &turn.transcript,
                turn.turn,
                &smith_transcript::CEILINGS,
                &temper_worker_domain_world::protocol::endpoints(),
            )
            .expect("committed goal turn decodes");
            for message in decoded.messages {
                for block in message.content {
                    if let Block::ToolCall(call) = block {
                        calls.push(call.name);
                    }
                }
            }
        }
        calls
    }
}

#[test]
fn decisions_across_projects_overturn_only_the_named_projects_goal() {
    let mut limits = fixtures::domain_limits();
    limits.client.domain.handles = 18;
    limits.client.domain.watches = 18;
    let mut story = Story::with_front(fixtures::across_people(limits), limits);
    story.front.answer_runs = false;
    let mut goals = Vec::new();
    for project in [PROJECT, OTHER_PROJECT] {
        story.act(away::Act::Toggle { project });
        story.front.engine.sign_in();
        story.front.engine.settle();
        let jig_domain_people::Ask::Create { mut batch, .. } =
            story.front.engine.chat_ask(project, b"Make a small fix")
        else {
            panic!("goal template")
        };
        batch[0].tracked = Some(0);
        batch[0].flags.conversation = false;
        batch[0].authority.tools = 1 | 2 | 4 | 512;
        story.front.engine.send(temper_engine_domain::Event::Party(temper_engine_domain::PartyInput::Ask {
            models: Box::new([]),
            reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(2400 + u64::from(project))),
            sign_in: story.front.engine.session(),
            key: [u8::try_from(project).expect("project key"); 16],
            ask: jig_domain_people::Ask::Create { project, batch },
        }));
        story.pump();
        let goal = story.front.assignments.last().expect("project's goal assigned").task;
        assert_eq!(story.front.engine.task(goal).expect("goal").project, project);
        goals.push(goal);
        assert_eq!(story.decide(goal, 1), "Decided: carry on with Small.");
        story.after(Duration::from_secs(1));
    }
    let [first, second] = goals.as_slice() else { panic!("two goals") };
    story.event(CLIENT, Temper::went(Address::AllHome));
    let facts = all_home::facts(story.probes[CLIENT.0].root.person(), story.env.wall);
    assert_eq!(facts.showing, 2);
    assert_eq!(
        facts.decided.iter().map(|row| (row.project, row.line.task, row.line.state)).collect::<Vec<_>>(),
        [
            (PROJECT, *first, temper_client_domain::pages::decided::DecisionTask::Live),
            (OTHER_PROJECT, *second, temper_client_domain::pages::decided::DecisionTask::Live)
        ]
    );
    assert!(facts.decided[0].line.at < facts.decided[1].line.at, "oldest decision first");
    story.act(away::Act::Overturn { project: OTHER_PROJECT, number: facts.decided[1].line.number });
    let decisions = story.decisions();
    assert!(decisions.iter().any(|row| row.project == OTHER_PROJECT && row.task == *second && row.overturned));
    assert!(decisions.iter().any(|row| row.project == PROJECT && row.task == *first && !row.overturned));
    let words = b"I overturn your decision. Which implementation? You took Small. Ask me again with message, as a question, and wait for my answer.";
    assert!(
        story.inputs.iter().any(|(task, _, _, input)| task == second && input.as_ref() == words),
        "project 2's goal hears the same overturn as on project Home"
    );
    assert!(!story.inputs.iter().any(|(task, _, _, input)| task == first && input.as_ref() == words));
    story.event(CLIENT, Temper::went(fixtures::inbox()));
    let rows = all_home::decided(story.probes[CLIENT.0].root.person(), story.env.wall);
    assert_eq!(
        rows.iter().map(|row| (row.project, row.line.task, row.line.overturned)).collect::<Vec<_>>(),
        [(PROJECT, *first, false), (OTHER_PROJECT, *second, true)]
    );
    story.event(
        CLIENT,
        Temper::went(Address::Inbox {
            filter: temper_client_domain::address::Filter { project: Some(PROJECT), goal: None, kind: None },
        }),
    );
    let rows = all_home::decided(story.probes[CLIENT.0].root.person(), story.env.wall);
    assert_eq!(rows.iter().map(|row| (row.project, row.line.task)).collect::<Vec<_>>(), [(PROJECT, *first)]);
}

#[test]
fn a_goal_decides_while_away_then_waits_on_home_after_overturn_and_off() {
    let mut story = Story::sequence();
    story.event(CLIENT, Temper::went(Address::Settings { project: PROJECT }));
    story.press(Role::Option, b"I turn it off");
    assert!(
        story.row(CLIENT).is_some_and(|row| row.until.is_none()),
        "Away row {:?}; answers {:?}",
        story.row(CLIENT),
        story.probes[CLIENT.0].answers
    );
    let view = story.view();
    assert!(Temper::tree(&view).nodes().iter().any(|node| {
        node.role == Role::Option
            && node.name.as_deref() == Some(b"I turn it off")
            && node.states.as_slice().contains(&State::Selected)
    }));
    story.visit_home();
    story.press(Role::Button, b"Away");
    story.sees(b"agents decide within");
    let run = story.begin_goal();
    let goal = run.task;
    assert_eq!(story.call(&run, 1, b"decided", LOCAL_DECISION), "Decided: carry on with Small.");
    let note = format!(
        r#"{{"scope":{{"kind":"goal","project":1,"goal":{goal}}},"description":"report: I carried on with Small.","body":"I carried on with the small implementation before asking anyone."}}"#
    );
    assert!(story.call(&run, 2, b"note", note.as_bytes()).starts_with("Note "));
    story.event(CLIENT, Temper::went(Address::Task { task: goal }));
    story.sees(b"I carried on with the small implementation before asking anyone.");
    story.visit_home();
    story.sees(b"One thing was decided for you.");
    story.sees(b"The small implementation fits the design.");
    story.sees(b"Small");
    assert!(
        !Temper::tree(&story.view())
            .nodes()
            .iter()
            .any(|node| { node.role == Role::Button && node.name.as_deref() == Some(b"Try the other implementation") }),
        "Decided carries on without asking"
    );
    story.press(Role::Button, b"Overturn");
    let decisions = story.decisions();
    let [decision] = decisions.as_slice() else { panic!("one decision: {decisions:?}") };
    assert!(decision.overturned, "Overturn saves its mark");
    assert!(story.inputs.iter().any(|(task, attempt, _, words)| {
        *task == goal && *attempt == run.attempt && words.as_ref() == b"I overturn your decision. Which local implementation? You took Small. Ask me again with message, as a question, and wait for my answer."
    }), "the goal's run hears the person's overturn: {:?}", story.inputs);
    assert!(story.call(&run, 3, b"message", br#"{"target":1,"form":"question","words":"Which local implementation after overturn?\nChoice: Try the other implementation\nChoice: Keep the chosen implementation"}"#).starts_with("Message "));
    story.sees(b"Overturned: it asks you again");
    story.sees(b"Needs you");
    assert!(
        Temper::tree(&story.view())
            .nodes()
            .iter()
            .any(|node| { node.role == Role::Button && node.name.as_deref() == Some(b"Try the other implementation") })
    );
    story.press(Role::Button, b"Away");
    story.sees(b"Away is off: questions wait for you again.");
    story.answer(&run, b"Try the other implementation");
    assert!(
        story
            .call(&run, 4, b"decided", LOCAL_DECISION)
            .starts_with("Ask: implementation asks your person while they are here")
    );
    asked_after(&decisions, &story.decisions());
    assert!(story.call(&run, 5, b"message", br#"{"target":1,"form":"question","words":"Which local implementation while here?\nChoice: Use the small implementation\nChoice: Use the broad implementation"}"#).starts_with("Message "));
    story.sees(b"Which local implementation while here?");
    story.sees(b"One thing was decided for you.");
    story.answer(&run, b"Use the small implementation");
    story.finish(&run);
    let view = story.view();
    assert!(
        Temper::tree(&view)
            .nodes()
            .iter()
            .any(|node| node.role == Role::Region && node.name.as_deref() == Some(b"Chats"))
    );
    assert!(
        !Temper::tree(&view)
            .nodes()
            .iter()
            .any(|node| node.role == Role::Region && node.name.as_deref() == Some(b"Goals"))
    );
    let calls = story.committed_calls(goal);
    assert!(
        calls.starts_with(&[Box::from(b"decided".as_slice()), Box::from(b"note".as_slice())]),
        "Decided carries on before overturn: {calls:?}"
    );
}

#[test]
#[expect(clippy::too_many_lines, reason = "one story follows classes, Away, money and Stop asking through both roots")]
fn what_reaches_a_person_follows_the_classes_away_and_stop_asking() {
    use forge::settings::{Edit, SettingKey, SettingSlot, Value};
    use temper_protocol_reaches::{Class, Kind, Reason, Subject};

    let mut story = Story::sequence_with_settings(true);
    let classes_key = SettingKey { kind: 8, scope: PROJECT, number: 1, slot: SettingSlot::Change };
    assert!(
        !story.front.engine.store.rows.values().any(|record| {
            matches!(record, Record::Forge { row } if matches!(row.as_ref(),
            forge::Stored::Setting(row) if row.key.kind == 8 && row.key.scope == PROJECT))
        }),
        "the project uses Agents review's stock classes"
    );
    story.visit_home();
    story.sees(b"Away");
    assert!(!story.probes[CLIENT.0].root.person().away_on(PROJECT, story.env.wall));
    let run = story.begin_goal();
    assert_eq!(story.call(&run, 1, b"decided", LOCAL_DECISION).split_whitespace().next(), Some("Ask:"));
    let rows = story.decisions();
    let [asked] = rows.as_slice() else { panic!("one asked row: {rows:?}") };
    assert_eq!((asked.project, asked.party, asked.task), (PROJECT, fixtures::PARTY, run.task));
    assert_eq!(asked.took, None);
    assert!(asked.options.is_empty());
    assert!(asked.why.is_empty());
    assert_eq!(
        (asked.reach.kind, asked.reach.class, asked.reach.reason, asked.reach.away),
        (Kind::Implementation, Class::Ask, Reason::Kind, false)
    );
    assert!(story.call(&run, 2, b"message", br#"{"target":1,"form":"question","words":"Which local implementation?\nChoice: Use Small\nChoice: Use Broad"}"#).starts_with("Message "));
    story.needs();
    story.sees(b"Which local implementation?");
    story.sees(b"Use Small");
    story.sees("Reached you because it is an implementation choice, which asks you unless you’re away.".as_bytes());
    story.sees(b"Stop asking");

    story.visit_how();
    let away_note = "While you’re away, they are decided with the recommended choice and listed.";
    assert_eq!(
        story.reaches_lines(b"Asks you"),
        [
            "implementation choices a spec leaves open",
            "a new dependency",
            "a departure from a landed design",
            away_note,
        ]
    );
    assert_eq!(
        story.reaches_lines(b"Agents decide, then tell you"),
        [
            "wording and names inside a spec",
            "retrying or splitting a task",
            "test-only changes",
            "Listed in Decided for you for 24 hours, to overturn.",
        ]
    );
    assert_eq!(
        story.reaches_lines(b"Always waits for you"),
        [
            "money past a budget",
            "what can’t be undone: rewriting history, a force-push, deleting unlanded work",
            "changes to this page",
            "anything an agent can’t place",
            "Away or not.",
        ]
    );
    story.needs();

    story.press(Role::Button, b"Away");
    assert!(story.probes[CLIENT.0].root.person().away_on(PROJECT, story.env.wall));
    story.after(Duration::from_nanos(1));
    assert_eq!(story.call(&run, 3, b"decided", LOCAL_DECISION).split_whitespace().next(), Some("Decided:"));
    let rows = story.decisions();
    let [still_asked, away_decided] = rows.as_slice() else { panic!("asked and decided rows: {rows:?}") };
    assert_eq!(still_asked, asked);
    assert_eq!(away_decided.took, Some(0));
    assert_eq!(
        (away_decided.reach.kind, away_decided.reach.class, away_decided.reach.away),
        (Kind::Implementation, Class::Ask, true)
    );
    story.needs();
    story.sees("Decided for you · 1".as_bytes());
    story.sees("Allowed: implementation choices are decided while you’re away.".as_bytes());
    story.sees(b"Which local implementation?");
    story.sees(b"Use Small");

    story.after(Duration::from_nanos(1));
    assert_eq!(story.call(&run, 4, b"decided", br#"{"question":"Buy more capacity?","options":["Buy","Wait"],"took":0,"why":"More capacity","kind":"money"}"#).split_whitespace().next(), Some("Ask:"));
    let rows = story.decisions();
    let [first, second, money] = rows.as_slice() else { panic!("three judged rows: {rows:?}") };
    assert_eq!((first, second), (asked, away_decided));
    assert_eq!(money.took, None);
    assert_eq!((money.reach.kind, money.reach.class, money.reach.away), (Kind::Money, Class::Wait, true));
    assert!(
        story
            .call(
                &run,
                5,
                b"message",
                br#"{"target":1,"form":"question","words":"Buy more capacity?\nChoice: Buy\nChoice: Wait"}"#
            )
            .starts_with("Message ")
    );
    story.needs();
    story.open_need(b"Buy more capacity?");
    story.sees(b"Buy more capacity?");
    story.sees(b"Reached you because it spends money past a budget, which always waits for you.");
    story.does_not_see(b"Stop asking");

    story.press(Role::Button, b"Away until 08:00");
    assert!(!story.probes[CLIENT.0].root.person().away_on(PROJECT, story.env.wall));
    story.open_need(b"Which local implementation?");
    story.sees(b"Which local implementation?");
    story.sees(b"Use Small");
    story.press(Role::Button, b"Stop asking: let agents decide implementation choices");
    story.sees(b"Let agents decide implementation choices, then tell you.");
    story.press(Role::Button, b"Apply");
    let setting = story.front.engine.store.rows.get(&StoreKey::Forge(forge::Key::Setting(classes_key)));
    let Some(Record::Forge { row }) = setting else { panic!("project classes saved: {setting:?}") };
    let forge::Stored::Setting(forge::settings::SettingRow { edit: Edit::Set(value), .. }) = row.as_ref() else {
        panic!("classes set: {row:?}")
    };
    let Value::Classes(classes) = &value.value else { panic!("classes value: {value:?}") };
    assert_eq!(classes.class_of(Subject::Kind(Kind::Implementation)), Class::Decide);
    story.sees(b"Applied: the next one is decided and listed. This one still waits for you.");
    story.visit_how();
    assert_eq!(
        story.reaches_lines(b"Agents decide, then tell you"),
        [
            "wording and names inside a spec",
            "retrying or splitting a task",
            "implementation choices a spec leaves open",
            "test-only changes",
            "Listed in Decided for you for 24 hours, to overturn.",
        ]
    );
    assert_eq!(story.reaches_lines(b"Asks you"), ["a new dependency", "a departure from a landed design", away_note]);
    story.needs();
    story.sees(b"Which local implementation?");
    story.sees(b"Use Small");
    assert_eq!(story.decisions(), rows, "Apply leaves the asked rows waiting");

    story.after(Duration::from_nanos(1));
    assert_eq!(story.call(&run, 6, b"decided", LOCAL_DECISION).split_whitespace().next(), Some("Decided:"));
    let final_rows = story.decisions();
    let [first, second, third, decided] = final_rows.as_slice() else { panic!("four judged rows: {final_rows:?}") };
    assert_eq!((first, second, third), (asked, away_decided, money));
    assert_eq!(decided.took, Some(0));
    assert_eq!(
        (decided.reach.kind, decided.reach.class, decided.reach.away),
        (Kind::Implementation, Class::Decide, false)
    );
    story.needs();
    story.sees("Decided for you · 2".as_bytes());
    story.sees(b"Allowed: implementation choices are decided and listed.");
    story.sees(b"Which local implementation?");
    story.sees(b"Use Small");
}

#[test]
fn on_crosses_the_roots_and_reaches_every_device() {
    let mut story = Story::new();
    assert_eq!(story.client(fixtures::PARTY), SECOND_DEVICE);
    assert_eq!(story.row(CLIENT), None);
    assert_eq!(story.row(SECOND_DEVICE), None);
    story.toggle();
    let row = story.row(CLIENT).expect("the switch delivered its row");
    assert_eq!(row.since, story.env.wall);
    assert_eq!(row.until, Some(Wall::from_nanos(28_800_000_000_000)));
    assert_eq!((row.landed, row.off), (0, None));
    assert_eq!(story.row(SECOND_DEVICE), Some(row), "the other device hears the party's watch");
    let stored = story.stored();
    assert_eq!(
        stored,
        forge::Away {
            party: fixtures::PARTY,
            project: PROJECT,
            since: row.since,
            until: row.until,
            landed: row.landed,
            off: row.off,
        }
    );
    story.front.restart_dismissal_people(OFFLINE);
    assert_eq!(story.stored(), stored, "restart restores the committed row");
    story.front.reload(SECOND_DEVICE, story.env.now);
    story.probes[SECOND_DEVICE.0] = Probe {
        root: Root::Person(Box::new(temper_client_domain::unlinked(&story.env.limits, fixtures::SEED))),
        out: Queue::uncapped(),
        answers: Vec::new(),
        decided_watch: None,
        decisions: None,
    };
    story.event(SECOND_DEVICE, Temper::start(fixtures::inbox(), Vec::new(), 0));
    story.after(OFFLINE);
    assert_eq!(story.row(SECOND_DEVICE), Some(row), "a fresh client reads the restored row");
}

#[test]
fn off_keeps_until() {
    let mut story = Story::new();
    story.toggle();
    let first = story.stored();
    story.after(Duration::from_secs(60));
    story.toggle();
    let off = story.stored();
    assert_eq!(off.until, first.until, "off retains until");
    assert_eq!((off.since, off.landed, off.off), (first.since, first.landed, Some(story.env.wall)));
    assert!(!story.probes[CLIENT.0].root.person().away_on(PROJECT, story.env.wall));
    // Go beyond the old 08:00: the retained choice means the next 08:00 on re-entry.
    story.after(Duration::from_secs(8 * 60 * 60));
    story.toggle();
    let on = story.stored();
    assert_eq!(on.since, story.env.wall, "on again starts a new spell");
    assert!(on.since > first.since);
    assert_eq!(on.off, None);
    assert_eq!(on.until, Some(Wall::from_nanos(115_200_000_000_000)));
    assert!(story.probes[CLIENT.0].root.person().away_on(PROJECT, story.env.wall));
}

#[test]
fn a_goal_is_answered_by_the_switch() {
    let mut story = Story::new();
    let goal = story.front.begin_away_goal();
    story.pump();
    story.home(CLIENT);
    assert!(story.facts(CLIENT).decided.is_none());
    story.toggle();
    assert_eq!(story.decide(goal, 1), "Decided: carry on with Small.");
    let decisions = story.decisions();
    let [row] = decisions.as_slice() else { panic!("one committed decision: {decisions:?}") };
    assert_eq!((row.project, row.party, row.task, row.number), (PROJECT, fixtures::PARTY, goal, 1));
    assert_eq!(row.at, story.env.wall);
    assert_eq!(row.question.as_ref(), b"Which implementation?");
    assert_eq!(row.options.as_ref(), &[Box::from(b"Small".as_slice()), Box::from(b"Broad".as_slice())]);
    assert_eq!(row.took, Some(0));
    assert_eq!(row.why.as_ref(), b"Fits the design");
    assert!(!row.overturned);
    let form = story.facts(CLIENT).decided.expect("Home hears the decision on its open watch");
    let spell = story.facts(CLIENT).spell.expect("Home hears the Away spell");
    assert_eq!((spell.since, spell.end), (story.stored().since, None));
    assert_eq!(spell.landed, Some(0));
    assert_eq!(form.showing, 1);
    let [line] = form.rows.as_ref() else { panic!("one decision on Home") };
    assert_eq!((line.number, line.task, line.at), (row.number, goal, row.at));
    assert_eq!(line.question, row.question);
    assert_eq!(line.took.as_ref(), b"Small");
    assert_eq!(line.why, row.why);
    assert!(!line.overturned);
    story.after(Duration::from_secs(1));
    story.toggle();
    assert_eq!(
        story.decide(goal, 2),
        "Ask: implementation asks your person while they are here. Ask with message, as a question; if your brief says [Requested by person N], with delegate, a person task addressed to person N."
    );
    asked_after(&decisions, &story.decisions());
    assert_eq!(
        story.facts(CLIENT).spell.map(|spell| (spell.since, spell.end)),
        Some((story.stored().since, Some(story.env.wall)))
    );
}

#[test]
fn past_until_the_client_draws_off_and_the_root_says_ask() {
    let mut story = Story::new();
    let goal = story.front.begin_away_goal();
    story.pump();
    story.home(CLIENT);
    story.toggle();
    let before = story.stored();
    let until = before.until.expect("08:00 choice");
    assert_eq!(story.decide(goal, 1), "Decided: carry on with Small.");
    let decisions = story.decisions();
    story.after(Duration::from_nanos(until.as_nanos() - 1));
    assert!(story.probes[CLIENT.0].root.person().away_on(PROJECT, story.env.wall));
    story.after(Duration::from_nanos(2));
    assert!(story.env.wall > until);
    assert!(!story.probes[CLIENT.0].root.person().away_on(PROJECT, story.env.wall));
    assert_eq!(story.row(CLIENT).expect("expired row remains").ended(story.env.wall), Some(until));
    assert_eq!(story.stored(), before, "expiry does not write the Away row");
    assert_eq!(
        story.decide(goal, 2),
        "Ask: implementation asks your person while they are here. Ask with message, as a question; if your brief says [Requested by person N], with delegate, a person task addressed to person N."
    );
    asked_after(&decisions, &story.decisions());
    assert_eq!(story.stored(), before, "the call does not write the expired Away row");
    assert_eq!(story.facts(CLIENT).spell.map(|spell| (spell.since, spell.end)), Some((before.since, Some(until))));
}

#[test]
fn another_partys_home_shows_nothing() {
    const OTHER_PARTY: u64 = 4;
    const OTHER_SUBJECT: u64 = 8;
    let mut story = Story::new();
    let goal = story.front.begin_away_goal();
    story.pump();
    story.home(CLIENT);
    story.toggle();
    assert_eq!(story.decide(goal, 1), "Decided: carry on with Small.");
    assert_eq!(story.facts(CLIENT).decided.expect("owner's decision form").showing, 1);
    let decisions = story.decisions();
    let other = SECOND_DEVICE;
    // The people take turns on Home, leaving the fixed watch table room for the restart oracle.
    story.event(CLIENT, Temper::went(Address::Forges));
    story.front.bind_dismissal_person(other, OTHER_PARTY, OTHER_SUBJECT);
    story.front.add_dismissal_member(CLIENT, OTHER_PARTY, PROJECT);
    story.start_client(other);
    story.home(other);
    assert!(story.probes[other.0].decisions.as_ref().expect("the member's Home watch delivered").rows().is_empty());
    assert_eq!(story.row(other), None, "the switch belongs to its person");
    assert!(story.facts(other).decided.is_none(), "another member sees no decision form");
    story.probes[other.0].answers.clear();
    let bounds = &w::CEILINGS;
    let overturn = w::Overturn::new(bounds, w::OverturnParts { project: PROJECT, number: 1 }).expect("overturn");
    let requesting =
        w::Requesting::new(bounds, w::RequestingParts { request: w::Request::Overturn(overturn) }).expect("request");
    story.event(
        other,
        Event::Client {
            event: jig_client::Event::Send {
                key: jig_client::Key { asker: temper_client_domain::owner::Owner::Forge.asker(), number: 2302 },
                name: None,
                request: jig_client::Ask::Family {
                    scope: jig_client::contract::Scope::Project(PROJECT),
                    record: jig_client::Family {
                        family: wire::FORGE,
                        version: wire::CLIENT_VERSION,
                        bytes: wire::requesting(&requesting),
                    },
                },
            },
        }
        .into(),
    );
    let [jig_client::Answered::Family(record)] = story.probes[other.0].answers.as_slice() else {
        panic!(
            "one overturn answer: {:?}; pending {:?}; link {:?}",
            story.probes[other.0].answers,
            story.probes[other.0]
                .root
                .person()
                .client()
                .pending(jig_client::Key { asker: temper_client_domain::owner::Owner::Forge.asker(), number: 2302 }),
            story.probes[other.0].root.person().client().link()
        )
    };
    assert_eq!((record.family, record.version), (wire::FORGE, wire::CLIENT_VERSION));
    let mut reader = skein_lib::Reader::new(&record.bytes);
    let answer = w::Answer::decode(bounds, &mut reader).expect("forge answer");
    assert!(reader.is_empty());
    let w::Outcome::OverturnRefused(reason) = answer.outcome() else { panic!("overturn refused: {answer:?}") };
    assert_eq!(reason.reason(), &w::OverturnRefusal::Unknown);
    assert_eq!(story.decisions(), decisions, "the other person changed no decision");
    assert!(story.facts(other).decided.is_none());
    story.event(other, Temper::went(Address::Forges));
    story.home(CLIENT);
    assert_eq!(story.facts(CLIENT).decided.expect("owner still sees the decision").showing, 1);
}

fn asked_after(before: &[forge::Decided], after: &[forge::Decided]) {
    assert_eq!(after.len(), before.len() + 1, "a judged ask writes its own row");
    assert_eq!(&after[..before.len()], before, "earlier decisions stay");
    let row = after.last().expect("the ask appended a row");
    assert_eq!(row.took, None);
    assert!(row.options.is_empty());
    assert!(row.why.is_empty());
    assert_eq!(row.reach.class, temper_protocol_reaches::Class::Ask);
    assert!(!row.reach.away);
}
