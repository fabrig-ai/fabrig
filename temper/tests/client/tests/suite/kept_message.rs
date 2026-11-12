//! A kept message at a bounded inbox, through the real clients component
//! (temper/docs/design/client/ctl/README.md, 6.2 and 9).
use jig_client::{self as client, Progress};
use jig_client_kit::harness::{ClientId, Front};
use skein_lib::{Duration, Env, List, Queue, RequestRecord, Time, Wall};
use temper_client_tools::{jig::Printed, root};
use temper_client_world::{app::Event, fixtures, front::Joint};

const SPAN: Duration = Duration::from_secs(2);
const TICK: Duration = Duration::from_millis(10);
const CALLER: ClientId = ClientId(0);

struct Story {
    front: Joint,
    root: root::Root,
    env: Env<root::Limits>,
    records: Vec<RequestRecord<client::Held>>,
    out: Queue<root::Request>,
    printed: Vec<(u32, Printed)>,
    attempts: Vec<(Time, client::Key)>,
    capture: Vec<String>,
    task: u64,
}
impl Story {
    fn new() -> Self {
        let limits = root::Limits::new(temper_client_tools::limits::client(), temper_client_tools::limits::tools());
        let mut front = Joint::open_chats(fixtures::SEED, limits.domain);
        front.answer_runs = false;
        let task = fixtures::chat(&mut front, b"A chat waiting for its worker");
        fixtures::extend_delegate_conversation(&mut front, task);
        // One whole message occupies the inbox's byte allowance. No worker
        // consumes it until clear(), while independent reads remain possible.
        let bytes = front.engine.env.limits.tasks.inbox_bytes;
        front.engine.send(temper_engine_domain::Event::Party(temper_engine_domain::PartyInput::Ask {
            models: Box::new([]),
            reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(1010)),
            sign_in: front.engine.session(),
            key: [41; 16],
            ask: jig_domain_people::Ask::Say {
                project: 1,
                task,
                words: vec![b'x'; usize::try_from(bytes).expect("inbox bytes")].into(),
            },
        }));
        front.engine.settle();
        assert_eq!(front.engine.task(task).expect("chat").inbox.len(), 1, "full inbox");
        front.bind(CALLER, fixtures::PARTY);
        let mut story = Self {
            front,
            root: root::Root::unlinked(&limits, fixtures::SEED).expect("tools limits"),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            records: Vec::new(),
            out: Queue::uncapped(),
            printed: Vec::new(),
            attempts: Vec::new(),
            capture: Vec::new(),
            task,
        };
        story.client(client::Event::StartUnlinked { kept: List::with_capacity(16) });
        story.client(client::Event::SignIn { with: client::With::Transport });
        story.settle();
        story
    }
    fn client(&mut self, event: client::Event) {
        root::step(&mut self.root, &self.env, root::Event::Client { event }, &mut self.out);
    }
    fn call(&mut self, number: u32, words: &[&[u8]]) {
        let Event::Call { call, json, .. } = fixtures::call(words) else { panic!("call") };
        root::step(&mut self.root, &self.env, root::Event::Call { number, call, json, value: None }, &mut self.out);
        self.settle();
    }
    fn message(&mut self) {
        let task = self.task.to_string();
        self.call(1, &[b"task", b"message", task.as_bytes(), b"--key", b"long-1", b"Next direction"]);
    }
    fn settle(&mut self) {
        for _ in 0..64 {
            while let Some(request) = self.out.pop() {
                match request {
                    root::Request::Unlinked {
                        request: client::UnlinkedRequest::Down { down: client::Down::Save { record } },
                    } => self.records.push(record),
                    root::Request::Unlinked {
                        request: client::UnlinkedRequest::Down { down: client::Down::Erase { key } },
                    } => self.records.retain(|r| r.key != key),
                    root::Request::Unlinked { request } => {
                        if let client::UnlinkedRequest::Down { down: client::Down::Send { key, .. } } = &request {
                            let local =
                                self.records.iter().find(|r| r.key == *key).expect("kept before send").request.key;
                            self.attempts.push((self.env.now, local));
                            self.capture.push(format!(
                                "send {}: {:?} {local:?} {:?}",
                                self.attempts.len(),
                                self.env.now,
                                self.root.client().pending(local)
                            ));
                        }
                        self.front.wire(
                            CALLER,
                            self.env.now,
                            self.env.wall,
                            temper_client_domain::Request::Unlinked { request },
                        );
                    }
                    root::Request::Printed { number, printed } => self.printed.push((number, printed)),
                    root::Request::Ready
                    | root::Request::Kept { .. }
                    | root::Request::Progress { .. }
                    | root::Request::Fact { .. } => {}
                    root::Request::Client { .. } | root::Request::Record { .. } => panic!("unlinked message"),
                }
            }
            let callbacks = self.front.poll(self.env.now, self.env.wall);
            if callbacks.is_empty() {
                return;
            }
            for (who, event) in callbacks {
                assert_eq!(who, CALLER);
                self.front.received(who, &event);
                if matches!(&event, Event::Root(temper_client_domain::Event::Went { .. })) {
                    continue;
                }
                let Event::Root(temper_client_domain::Event::Client { event }) = event else {
                    panic!("client callback")
                };
                let answered = matches!(&event, client::Event::Up { up: client::Up::Answer { .. } });
                if let client::Event::Up { up: client::Up::Answer { envelope, answer, why, .. } } = &event {
                    self.capture.push(format!("engine: {envelope:?} {answer:?} {why:?}"));
                }
                self.client(event);
                if answered && let Some((_, key)) = self.attempts.last() {
                    self.capture.push(format!("client: {:?}", self.root.client().pending(*key)));
                }
            }
        }
        panic!("message world did not settle");
    }
    fn advance(&mut self, duration: Duration) {
        let end = self.env.now.saturating_add(duration);
        while self.env.now < end {
            self.env.now = self.env.now.saturating_add(TICK).min(end);
            self.env.wall = Wall::from_nanos(self.env.now.as_nanos());
            root::fire(&mut self.root, &self.env, &mut self.out);
            self.settle();
        }
    }
    fn clear(&mut self) {
        fixtures::read_messages(&mut self.front, self.task);
    }
    fn print_capture(&self) {
        for row in &self.capture {
            eprintln!("{row}");
        }
    }
}

#[test]
fn a_kept_message_reaches_its_answer_while_its_chat_cannot_take_it() {
    let mut story = Story::new();
    story.message();
    story.advance(Duration::from_secs(12));
    // A read concurrent with the pending act proves the engine is responsive.
    story.call(2, &[b"me"]);
    assert_eq!(story.printed.iter().find(|(n, _)| *n == 2).expect("concurrent read").1.status, 0);
    let key = story.attempts.last().expect("attempt").1;
    assert_eq!(
        story.root.client().pending(key).expect("pending act").progress,
        Progress::Retrying { why: client::Retry::Busy }
    );
    assert_eq!(story.records.len(), 1);
    story.clear();
    story.advance(SPAN);
    story.print_capture();
    let printed = &story.printed.iter().find(|(n, _)| *n == 1).expect("answer within a span of room returning").1;
    assert_eq!(printed.status, 0);
    assert!(printed.stdout.starts_with(b"done:"), "{printed:?}");
    assert_eq!(story.records.len(), 0);
}

#[test]
fn a_kept_message_asked_again_under_its_key_gets_the_first_answer() {
    let mut story = Story::new();
    story.message();
    story.advance(temper_client_tools::limits::DEADLINE);
    assert!(!story.printed.iter().any(|(n, _)| *n == 1));
    assert_eq!(story.records.len(), 1);
    // A fresh invocation loads the saved request and joins it by its name.
    let mut kept = List::with_capacity(16);
    for record in &story.records {
        kept.push(record.clone()).expect("one kept request");
    }
    story.front.reload(CALLER, story.env.now);
    story.root = root::Root::unlinked(&story.env.limits, fixtures::SEED).expect("tools limits");
    story.client(client::Event::StartUnlinked { kept });
    story.client(client::Event::SignIn { with: client::With::Transport });
    story.settle();
    story.advance(TICK);
    assert_eq!(story.root.client().link().state, client::Link::Live);
    assert!(story.root.replayed(), "host waits for replay dispatch before its own call");
    story.message();
    assert!(!story.printed.iter().any(|(n, _)| *n == 1), "rerun joins the still-busy request");
    story.clear();
    story.advance(SPAN);
    story.print_capture();
    let printed = &story.printed.iter().find(|(n, _)| *n == 1).expect("same-key answer").1;
    assert_eq!(printed.status, 0);
    let first = printed.stdout.clone();
    story.call(3, &[b"task", b"message", story.task.to_string().as_bytes(), b"--key", b"long-1", b"Next direction"]);
    assert_eq!(story.printed.iter().find(|(n, _)| *n == 3).expect("retained answer").1.stdout, first);
    assert_eq!(
        story
            .front
            .engine
            .task(story.task)
            .expect("chat")
            .inbox
            .iter()
            .filter(|w| w.words.as_ref() == b"Next direction")
            .count(),
        1
    );
}
