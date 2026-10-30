//! Complete fixture readers (jig/docs/design/protocol/store.md, 7.1, 8.3 and 14).
use super::{Store, protocol, root};
use jig_domain_core as core;
use protocol::{Event, Get, Request, codec, keys};
use skein_lib::{Queue, Reader, Token};
use std::collections::{BTreeMap, VecDeque};

const READER: Token = Token::new(0);

impl Store {
    fn readback_boundary(&self) {
        assert!(
            !self.failed
                && self.pending.is_empty()
                && self.inflight.is_empty()
                && self.held.is_empty()
                && self.replies.is_empty()
                && !self.loading(),
            "complete readback requires a quiescent fixture boundary"
        );
    }

    /// Collect the entire main map through production pages, including component
    /// and installation rows. Requires a quiescent fixture boundary; the output
    /// owns memory proportional to the complete inventory.
    pub fn retained(&mut self) -> BTreeMap<Box<[u8]>, Box<[u8]>> {
        self.readback_boundary();
        let mut found = BTreeMap::new();
        let mut after = None;
        loop {
            let page = self
                .component
                .as_mut()
                .expect("component")
                .retained_page(
                    after.as_deref(),
                    self.component_limits.store.page.rows,
                    u64::from(self.component_limits.store.page.bytes),
                )
                .expect("complete retained page");
            let mut last = after.clone();
            for row in page.rows {
                assert!(last.as_deref().is_none_or(|last| row.key.as_ref() > last), "exclusive ordered keys");
                last = Some(row.key.clone());
                assert!(found.insert(row.key, row.value).is_none(), "unique retained key");
            }
            let Some(next) = page.next else { return found };
            assert_eq!(Some(&next), last.as_ref(), "cursor is the last returned key");
            assert!(after.as_deref().is_none_or(|after| next.as_ref() > after), "retained progress");
            after = Some(next);
        }
    }

    pub(super) fn readback_query(&mut self, request: Request) -> Event {
        self.readback_boundary();
        let mut requests = VecDeque::from([request]);
        let mut above = Queue::uncapped();
        let mut below =
            protocol::Below { store: Queue::uncapped(), secrets: Queue::uncapped(), transcripts: Queue::uncapped() };
        let mut captured = None;
        loop {
            let env = self.env();
            if self.disk.is_some() {
                protocol::fire(self.component.as_mut().expect("component"), &env, &mut above, &mut below);
            }
            if let Some(event) = above.pop() {
                if matches!(event, Event::Site { .. }) {
                    // Sites need no registered flight; the fixture route
                    // answers NoSite just as it does during ordinary reads.
                    self.route(event, &mut requests);
                } else {
                    assert!(captured.replace(event).is_none(), "one helper query terminal");
                }
                continue;
            }
            if let Some(request) = requests.pop_front() {
                protocol::down(self.component.as_mut().expect("component"), &env, request, &mut above, &mut below);
                continue;
            }
            let io = if let Some(request) = below.transcripts.pop() {
                protocol::Io::Transcripts(self.file(2, request))
            } else if let Some(request) = below.store.pop() {
                protocol::Io::Store(self.file(0, request))
            } else if let Some(request) = below.secrets.pop() {
                protocol::Io::Secrets(self.file(1, request))
            } else {
                break;
            };
            protocol::up(self.component.as_mut().expect("component"), &env, io, &mut above, &mut below);
        }
        captured.expect("helper query completes")
    }

    /// Discover records from the main map and read each through Get.
    /// Component metadata is validated and each indexed transcript is read in
    /// both directions, contributing its Turn records to the result.
    /// Requires a quiescent fixture boundary. Raw inventory,
    /// decoded output and transient pages overlap in memory; `rows` is untouched.
    pub fn readback(&mut self) -> BTreeMap<root::Key, root::Record> {
        let inventory = self.retained();
        let mut found = BTreeMap::new();
        let mut transcripts = BTreeMap::new();
        let mut attempts = Vec::new();
        let installation = keys::installation(self.component_limits.store.key).expect("installation key");
        for (key, value) in &inventory {
            if let Some(key) = keys::decode(key) {
                let Event::Got { owner: READER, row: Some(protocol::Row::Jig(row)) } =
                    self.readback_query(Request::Get { owner: READER, key: Get::Jig(key.clone()) })
                else {
                    panic!("discovered jig Get must be present and typed")
                };
                assert_eq!(row.key(), key, "Get key agrees with inventory");
                let row = root::Record::Core(row);
                assert!(found.insert(row.key(), row).is_none(), "unique decoded key");
            } else if let Some(application) = keys::of_application(key) {
                let (number, record_key) = jig_test_store::keys::decode(application, &self.limits.connector)
                    .expect("recognized application key");
                let Event::Got { owner: READER, row: Some(protocol::Row::Application { key, value: bytes }) } =
                    self.readback_query(Request::Get { owner: READER, key: Get::Application(application.into()) })
                else {
                    panic!("discovered application Get must be present and typed")
                };
                assert_eq!(key.as_ref(), application, "application Get key agreement");
                assert_eq!(&bytes, value, "Get reads the inventoried bytes");
                let record = jig_test_store::records::decode(&bytes, &self.limits.connector)
                    .expect("application record decodes");
                assert_eq!(record.key(), record_key, "application record key agreement");
                let row = root::Record::Connector { number, record };
                assert!(found.insert(row.key(), row).is_none(), "unique decoded key");
            } else if *key == installation {
                codec::accounts::Installation::decode(value).expect("installation row");
            } else {
                match keys::decode_own(key).expect("recognized retained key") {
                    keys::Own::Format => {
                        codec::component::Format::decode(value).expect("format row");
                    }
                    keys::Own::Layouts => {
                        codec::component::Layouts::decode(value).expect("layouts row");
                    }
                    keys::Own::Wires => {
                        protocol::component_v3::Wires::decode(
                            &protocol::component_v3::CEILINGS,
                            &mut Reader::new(value),
                        )
                        .expect("wires row");
                    }
                    keys::Own::Transcript { task } => {
                        let index = codec::component::Transcript::decode(value).expect("transcript index");
                        assert!(index.window <= index.length, "window within committed transcript");
                        assert!(transcripts.insert(task, index).is_none());
                    }
                    keys::Own::Attempt { task, attempt } => {
                        let index = codec::component::Attempt::decode(value).expect("attempt index");
                        attempts.push((task, attempt, index));
                    }
                }
            }
        }
        for (task, _, index) in attempts {
            let transcript = transcripts.get(&task).expect("attempt has transcript metadata");
            assert!(index.first < transcript.length, "attempt begins within committed transcript");
        }
        for task in transcripts.keys() {
            let forwards = self.transcript(*task, false);
            let mut backwards = self.transcript(*task, true);
            backwards.reverse();
            assert_eq!(forwards, backwards, "complete transcript frame orders agree");
            for turn in forwards {
                let row = root::Record::Core(core::Record::Core(core::CoreRecord::Turn(turn)));
                assert!(found.insert(row.key(), row).is_none(), "unique transcript turn key");
            }
        }
        found
    }

    /// Read every committed frame in the chosen order. Every Unread, including
    /// after a successful prefix, fails. Requires a quiescent fixture boundary;
    /// output owns memory proportional to the whole transcript.
    pub fn transcript(&mut self, task: u64, backwards: bool) -> Vec<core::TurnRecord> {
        self.readback_boundary();
        let mut found = Vec::new();
        let mut after = None;
        loop {
            let event = self.readback_query(Request::Transcript {
                owner: READER,
                read: protocol::TranscriptRead::Page { task, backwards, after },
                rows: self.component_limits.store.page.rows,
                bytes: u64::from(self.component_limits.store.page.bytes),
            });
            let Event::Turns { owner: READER, turns, next } = event else {
                panic!("complete transcript requires successful Turns: {event:?}")
            };
            assert!(turns.iter().all(|turn| turn.task == task), "transcript task agreement");
            if let Some(next) = next {
                assert!(!turns.is_empty(), "nonterminal transcript page has turns");
                assert!(backwards || next > 0, "forward transcript cursor advances from zero");
                assert!(
                    after.is_none_or(|after| if backwards { next < after } else { next > after }),
                    "transcript progress"
                );
            }
            // Production returns each backwards page in chronological order.
            // Reverse within the page to return the requested frame order.
            if backwards {
                found.extend(turns.into_vec().into_iter().rev());
            } else {
                found.extend(turns);
            }
            let Some(next) = next else { return found };
            after = Some(next);
        }
    }
}
