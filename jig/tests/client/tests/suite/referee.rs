//! Faulty client and front run through real callbacks, trees, patches and people.
use jig_client_view::Owner;
use jig_client_world::{Until, harness::*};
use skein_fake_person::{Face, Find, Found, Step, TreeFace};
use skein_ui::{Act, Binding};
use skein_ui::{Key, NodeId, Patch, Role, Run, Tree, View};

use skein_lib::{Duration, Env, Queue, Time, Token, Wall};

struct Broken<const BAD: u8>;
struct Root {
    answered: bool,
    shown: u64,
}
enum Event {
    Start,
    Press,
    Answer,
}
struct Wire;
const KEY: Key = Key { owner: Owner::Jig.opaque(), kind: 1, a: 1, b: 1 };
const LIMITS: skein_ui::Limits = skein_ui::Limits {
    nodes: 24,
    depth: 4,
    text_per_node: 64,
    text_bytes: 1024,
    patches: 128,
    bindings: 2,
    markdown_depth: 1,
    ..skein_ui::Limits::EMPTY
};
impl<const BAD: u8> App for Broken<BAD> {
    type Root = Root;
    type View = View;
    type Limits = ();
    type Event = Event;
    type Request = Wire;
    type Record = ();
    type Key = ();
    type Address = ();
    fn limits() {}
    fn new(_: u64) -> (Root, View) {
        (Root { answered: false, shown: 0 }, View::new(&LIMITS))
    }
    fn start((): (), _: Vec<()>, _: i32) -> Event {
        Event::Start
    }
    fn went((): ()) -> Event {
        Event::Start
    }
    fn step(root: &mut Root, _: &Env<()>, event: Event, out: &mut Queue<Wire>) {
        match event {
            Event::Start => {}
            Event::Press => {
                out.push(Wire);
            }
            Event::Answer => root.answered = true,
        }
        root.shown += 1;
    }
    fn fire(_: &mut Root, _: &Env<()>, _: &mut Queue<Wire>) {}
    fn settle(_: &mut Root, _: &Env<()>, _: &mut Queue<Wire>) {}
    fn take(_: &mut Root, _: &Env<()>, _: &mut Queue<Wire>) {}
    fn next_deadline(_: &Root) -> Option<Time> {
        None
    }
    fn build(view: &mut View, root: &Root, out: &mut Queue<Patch>) {
        let mut b = view.builder();
        b.open(Role::Page);
        b.open(Role::Banner);
        for region in [
            jig_client_view::regions::JigRegion::Link,
            jig_client_view::regions::JigRegion::Needs,
            jig_client_view::regions::JigRegion::Party,
        ] {
            b.open(Role::Region);
            b.region(region.region());
            b.close();
        }
        b.close();
        b.open(Role::Main);
        b.open(Role::Status);
        b.name(if BAD == 9 { b"Offline" } else { b"Online" });
        b.close();
        if !(BAD == 5 && root.answered) {
            b.open(Role::Item);
            b.key(KEY);
            b.open(Role::Status);
            b.name(if root.answered { if BAD == 10 { b"Leaving" } else { b"Done" } } else { b"Waiting" });
            b.close();
            b.text(Run::Plain, b"Revision 7");
            b.open(Role::Button);
            b.name(b"Choose");
            b.bind(Binding { owner: Owner::Jig.opaque(), token: Token::new(1) });
            b.close();
            b.close();
        }
        if root.answered {
            b.open(Role::Status);
            b.name(b"Sent words");
            b.text(Run::Plain, b"Words");
            b.close();
        }
        b.close();
        b.close();
        view.finish(b, root.shown, out);
    }
    fn tree(view: &View) -> &Tree {
        view.tree()
    }
    fn act(_: &mut View, _: NodeId, _: Act) -> Option<Event> {
        Some(Event::Press)
    }
    fn duty(_: Wire) -> Duty<Self> {
        Duty::Wire(Wire)
    }
    fn record_key((): &()) {}
    fn unsent(_: Wire) -> Option<Event> {
        None
    }
    fn view_limits(_: &View) -> skein_ui::Limits {
        LIMITS
    }
    fn patch_limit(_: &View) -> u32 {
        LIMITS.patches
    }
    fn sent(_: &Wire) -> Option<Sent> {
        Some(Sent {
            application: false,
            attempt: Token::new(1),
            key: [1; 16],
            card: Some(CardKey::of(KEY)),
            revision: Some((0, if BAD == 4 { 8 } else { 7 })),
            words: Some((9, b"Words".to_vec())),
        })
    }
    fn received(event: &Event) -> Vec<Receipt> {
        match event {
            Event::Answer => vec![Receipt::Answer {
                attempt: Token::new(1),
                final_answer: true,
                unconfigured_model: false,
                closed_project: false,
                words: Some((9, 1)),
            }],
            Event::Start | Event::Press => vec![],
        }
    }
    fn card(_: &Tree, _: NodeId) -> Option<CardKey> {
        Some(CardKey::of(KEY))
    }
    fn seen(tree: &Tree) -> Vec<Seen> {
        let has = |name: &[u8]| matches!(TreeFace::new(tree).find(&Find::named(Role::Status, name)), Found::Node(_));
        let mut seen = vec![Seen::Link { live: has(b"Online") }];
        if has(b"Waiting") || has(b"Done") || has(b"Leaving") {
            seen.push(Seen::Card {
                key: CardKey::of(KEY),
                state: if has(b"Leaving") {
                    CardSeen::Left { reason: false }
                } else if has(b"Done") {
                    CardSeen::Done
                } else {
                    CardSeen::Waiting
                },
                revision: Some(7),
            });
        }
        if has(b"Sent words") {
            seen.push(Seen::Words { task: 9, message: Some(1), words: b"Words".to_vec() });
        }
        seen
    }
}
enum Progress {
    Quiet,
    Sent,
    Reported,
    Received,
    RetentionChecked,
}
struct FrontOf<const BAD: u8> {
    client: Option<ClientId>,
    progress: Progress,
    truth: Vec<Observed>,
}
impl<const BAD: u8> Front<Broken<BAD>> for FrontOf<BAD> {
    fn bind(&mut self, client: ClientId, _: u64) {
        self.client = Some(client);
        if BAD == 6 || BAD == 9 {
            self.truth.push(Observed::Reachable { live: BAD == 9 });
        }
    }
    fn wire(&mut self, _: ClientId, _: Time, _: Wall, _: Wire) {
        self.progress = Progress::Sent;
    }
    fn poll(&mut self, now: Time, _: Wall) -> Vec<(ClientId, Event)> {
        if matches!(self.progress, Progress::Received) && (BAD == 11 || BAD == 12) {
            self.progress = Progress::RetentionChecked;
            self.truth.push(Observed::Durable { commit: 2, at: now });
            if BAD == 12 {
                self.truth.push(Observed::Ended { task: 9, commit: 2 });
            }
            self.truth.push(Observed::Retained { task: 9, messages: vec![], commit: 2 });
            return vec![];
        }
        if !matches!(self.progress, Progress::Sent) {
            return vec![];
        }
        self.progress = Progress::Reported;
        if BAD != 1 {
            self.truth.push(Observed::Durable { commit: 1, at: now });
            self.truth.push(Observed::Made { party: 7, key: [1; 16], commit: 1 });
        }
        if BAD == 2 {
            self.truth.push(Observed::Made { party: 7, key: [1; 16], commit: 1 });
        }
        if BAD != 3 {
            self.truth.push(Observed::Kept {
                task: 9,
                message: 1,
                words: b"Words".to_vec(),
                commit: u64::from(BAD != 1),
            });
        }
        if BAD == 8 { vec![] } else { vec![(self.client.expect("bound"), Event::Answer)] }
    }
    fn observed(&mut self) -> Vec<Observed> {
        std::mem::take(&mut self.truth)
    }
    fn next_due(&self) -> Option<Time> {
        None
    }
    fn fault(&mut self, _: Time, _: FrontFault) {}
    fn reload(&mut self, _: ClientId, _: Time) {}
    fn received(&mut self, _: ClientId, event: &Event) {
        if matches!(event, Event::Answer) {
            self.progress = Progress::Received;
        }
    }
    fn sign_in(&mut self, _: ClientId, _: Time, _: Wall) {}
    fn sign_out(&mut self, _: ClientId, _: Time, _: Wall) {}
}
fn run<const BAD: u8>() -> Result<Outcome, String> {
    let settings = Settings {
        memory: BAD == 7,
        heap_ceiling: if BAD == 7 { Some(0) } else { None },
        heartbeat: Duration::from_millis(5),
        request_within: Duration::from_millis(20),
        link_within: Duration::from_millis(5),
        ..Settings::default()
    };
    let mut w = World::<Broken<BAD>, _>::new(
        settings,
        FrontOf::<BAD> { client: None, progress: Progress::Quiet, truth: vec![] },
    );
    let c = w.try_client(7, ())?;
    if BAD != 6 && BAD != 9 {
        w.person(
            c,
            vec![
                Step::Press { find: Find::named(Role::Button, b"Choose") },
                Step::Pause { span: Duration::from_millis(10) },
            ],
        );
    }
    w.try_run(Until::Done)
}
#[test]
fn correct_boundaries_meet_the_request_and_keep_the_words() {
    assert_eq!(run::<0>().unwrap().verdict, skein_world::domain::Verdict::Passed);
}
#[test]
fn an_answer_before_the_write_is_caught() {
    let finding = run::<1>().unwrap_err();
    assert!(finding.contains("nothing shown done before durable"), "{finding}");
}
#[test]
fn two_makes_for_one_party_key_are_caught() {
    let finding = run::<2>().unwrap_err();
    assert!(finding.contains("made once"), "{finding}");
}
#[test]
fn sent_words_missing_from_committed_activity_are_caught() {
    let finding = run::<3>().unwrap_err();
    assert!(finding.contains("nothing sent lost"), "{finding}");
}
#[test]
fn an_act_with_a_different_revision_is_caught() {
    let finding = run::<4>().unwrap_err();
    assert!(finding.contains("what they saw is what they decided"), "{finding}");
}
#[test]
fn a_card_removed_without_a_visible_reason_is_caught() {
    let finding = run::<5>().unwrap_err();
    assert!(finding.contains("a card leaves for a reason"), "{finding}");
}
#[test]
fn unreachable_truth_without_offline_is_caught() {
    let finding = run::<6>().unwrap_err();
    assert!(finding.contains("Offline"), "{finding}");
    assert!(finding.contains("was not met by"), "{finding}");
}
#[test]
fn a_client_exceeding_its_heap_envelope_is_caught() {
    let finding = run::<7>().unwrap_err();
    assert!(finding.contains("bounded"), "{finding}");
}
#[test]
fn a_request_without_a_final_answer_misses_its_deadline() {
    let finding = run::<8>().unwrap_err();
    assert!(finding.contains("Request"), "{finding}");
    assert!(finding.contains("was not met by"), "{finding}");
}
#[test]
fn recovered_truth_without_an_online_view_misses_its_deadline() {
    let finding = run::<9>().unwrap_err();
    assert!(finding.contains("Online"), "{finding}");
    assert!(finding.contains("was not met by"), "{finding}");
}

#[test]
fn the_test_clients_family_revision_and_durable_outcome_are_observed() {
    use jig_client_test_domain::family;
    let mut w = jig_client_world::stories::chat_ready(20);
    let subject = family::Subject { scope: family::Scope::Project(1), kind: family::Kind::Item(1) };
    w.front().compositions.insert(
        jig_client_test_domain::first::Address { project: 1, item: 1 },
        jig_client_test_domain::family::PageComposition {
            task: 9,
            second: None,
            proposals: jig_client_world::test::list([]),
        },
    );
    let mut state = jig_client_world::scripted::Family::default();
    state.documents.insert(
        subject,
        family::Document::Connector(jig_client_test_domain::connector_values::ConnectorDocument {
            subject: jig_client_test_domain::family::Subject {
                scope: jig_client_test_domain::family::Scope::Project(1),
                kind: jig_client_test_domain::family::Kind::Item(1),
            },
            value: Box::new(jig_client_test_domain::connector_values::ClientDocument::Object {
                object: 1,
                now: jig_client_test_domain::connector_values::ApplicationObject { revision: 7, value: 1 },
            }),
        }),
    );
    w.front()
        .engine
        .act_application(Time::ZERO, Wall::EPOCH, jig_client_test_domain::FIRST, 1, state, vec![])
        .expect("family fixture fits");
    let c = w.client(
        7,
        jig_client_test_domain::Address::First(jig_client_test_domain::first::Address { project: 1, item: 1 }),
    );
    w.person(
        c,
        vec![
            Step::See { find: Find::named(Role::Button, b"Apply parameters"), within: Duration::from_secs(1) },
            Step::Press { find: Find::named(Role::Button, b"Apply parameters") },
            Step::See { find: Find::text(b"Object changed"), within: Duration::from_secs(1) },
        ],
    );
    let result = w.run(Until::Done);
    assert_eq!(result.verdict, skein_world::domain::Verdict::Passed);
    assert!(result.judged.1 > 0);
}

#[test]
fn the_test_clients_sent_words_are_found_in_committed_activity() {
    use jig_client_domain::contract::{
        Activity, ActivityLine, Answer, Ask, DecodedActivityLine, MessageKind, Outcome, Read, Requester,
    };
    use jig_client_world::stories;
    let mut w = stories::chat_ready(21);
    w.front()
        .engine
        .script(jig_client_scripted_engine::Rule {
            party: Some(7),
            ask: Ask::Message { project: 1, task: 9, words: stories::text(b"Words") },
            answer: Answer::Done(Outcome::Document(jig_client::contract::DocumentOutcome::Said {
                task: 9,
                message: 1,
            })),
            changes: vec![jig_client_scripted_engine::Change::Source(jig_client_scripted_engine::Source {
                party: 7,
                read: Read::Activity { task: 9, before: None },
                rows: jig_client_scripted_engine::Rows::Activity(vec![DecodedActivityLine {
                    source: Box::new(ActivityLine {
                        task: 9,
                        position: 1,
                        at: skein_lib::Wall::EPOCH,
                        line: Activity::Words {
                            by: Requester::Person(7),
                            message: 1,
                            kind: MessageKind::Words,
                            words: stories::text(b"Words"),
                        },
                    }),
                    decoded: None,
                }]),
            })],
        })
        .expect("words script fits");
    let c = w.client(7, stories::chat());
    w.person(
        c,
        vec![
            Step::See { find: Find::named(Role::Button, b"Write"), within: Duration::from_secs(1) },
            Step::Press { find: Find::named(Role::Button, b"Write") },
            Step::Type { find: Find::named(Role::TextArea, b"Your answer"), words: Box::from(*b"Words") },
            Step::Press { find: Find::named(Role::Button, b"Confirm") },
            Step::See { find: Find::named(Role::Status, b"Sent"), within: Duration::from_secs(1) },
            Step::Pause { span: Duration::from_millis(30) },
        ],
    );
    let result = w.run(Until::Done);
    assert_eq!(result.verdict, skein_world::domain::Verdict::Passed);
    assert!(result.judged.1 > 0);
}

#[test]
fn a_departure_after_a_write_still_needs_an_explanation() {
    let finding = run::<10>().unwrap_err();
    assert!(finding.contains("explanation absent"), "{finding}");
}
#[test]
fn previously_sent_words_cannot_leave_later_activity() {
    let finding = run::<11>().unwrap_err();
    assert!(finding.contains("nothing sent lost"), "{finding}");
}
#[test]
fn ending_the_task_discharges_word_retention() {
    assert_eq!(run::<12>().unwrap().verdict, skein_world::domain::Verdict::Passed);
}
