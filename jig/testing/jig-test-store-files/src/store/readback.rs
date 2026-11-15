//! Complete fixture readers (jig/docs/design/protocol/store/loads.md, 1, and jig/docs/design/protocol/store/transcripts.md, 3, and jig/docs/design/protocol/store/README.md, 9).
use super::{Store, protocol, root};
use jig_domain_core as core;
use protocol::{Event, Get, Request, codec, keys};
use skein_lib::{Queue, Reader, Token};
use std::collections::{BTreeMap, VecDeque};

const READER: Token = Token::new(0);

/// The first finding of a complete readback (jig/docs/design/domain/testing.md, 7).
#[derive(Debug)]
pub enum ReadbackFinding {
    NotQuiescent,
    ApplicationKey {
        key: Box<[u8]>,
    },
    ApplicationValue {
        key: Box<[u8]>,
    },
    Transcript {
        task: u64,
        backwards: bool,
        after: Option<u64>,
        terminal: Box<Event>,
        reason: &'static str,
    },
    /// A retained page, component row or decoded record disagrees with its stored inventory.
    Inventory {
        key: Option<Box<[u8]>>,
        reason: String,
    },
}

fn inventory(key: &[u8], reason: impl Into<String>) -> ReadbackFinding {
    ReadbackFinding::Inventory { key: Some(key.into()), reason: reason.into() }
}

impl Store {
    fn readback_boundary(&self) -> Result<(), ReadbackFinding> {
        if self.failed
            || !self.pending.is_empty()
            || !self.inflight.is_empty()
            || !self.held.is_empty()
            || !self.replies.is_empty()
            || self.loading()
        {
            return Err(ReadbackFinding::NotQuiescent);
        }
        Ok(())
    }

    /// Collect the entire main map through production pages, including component
    /// and installation rows. Requires a quiescent fixture boundary; the output
    /// owns memory proportional to the complete inventory.
    #[expect(clippy::type_complexity, reason = "complete raw inventory owns both keys and values")]
    pub fn retained(&mut self) -> Result<BTreeMap<Box<[u8]>, Box<[u8]>>, ReadbackFinding> {
        self.readback_boundary()?;
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
                .map_err(|failure| ReadbackFinding::Inventory {
                    key: after.clone(),
                    reason: format!("complete retained page: {failure:?}"),
                })?;
            let mut last = after.clone();
            for row in page.rows {
                if last.as_deref().is_some_and(|last| row.key.as_ref() <= last) {
                    return Err(inventory(&row.key, "exclusive ordered keys"));
                }
                last = Some(row.key.clone());
                if found.insert(row.key, row.value).is_some() {
                    return Err(ReadbackFinding::Inventory { key: last, reason: "unique retained key".into() });
                }
            }
            let Some(next) = page.next else { return Ok(found) };
            if Some(&next) != last.as_ref() {
                return Err(inventory(&next, "cursor is the last returned key"));
            }
            if after.as_deref().is_some_and(|after| next.as_ref() <= after) {
                return Err(inventory(&next, "retained progress"));
            }
            after = Some(next);
        }
    }

    pub(super) fn readback_query(&mut self, request: Request) -> Result<Event, ReadbackFinding> {
        self.readback_boundary()?;
        let mut requests = VecDeque::from([request]);
        let mut above = Queue::uncapped();
        let mut below =
            protocol::Below { store: Queue::uncapped(), secrets: Queue::uncapped(), transcripts: Queue::uncapped() };
        let mut captured = None;
        loop {
            let env = self.env();
            if let Some(disk) = &mut self.disk {
                disk.drive(self.component.as_mut().expect("component"), &env, &mut above);
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
                if let Some(disk) = &mut self.disk {
                    disk.request(self.component.as_mut().expect("component"), &env, request, &mut above);
                } else {
                    protocol::down(self.component.as_mut().expect("component"), &env, request, &mut above, &mut below);
                }
                continue;
            }
            let io = if let Some(request) = below.transcripts.pop() {
                protocol::Io::Transcripts(self.file(request))
            } else if let Some(request) = below.store.pop() {
                protocol::Io::Store(self.file(request))
            } else if let Some(request) = below.secrets.pop() {
                protocol::Io::Secrets(self.file(request))
            } else {
                break;
            };
            protocol::up(self.component.as_mut().expect("component"), &env, io, &mut above, &mut below);
        }
        Ok(captured.expect("helper query completes"))
    }

    /// Discover records from the main map and read each through Get.
    /// Component metadata is validated and each indexed transcript is read in
    /// both directions, contributing its Turn records to the result.
    /// Requires a quiescent fixture boundary. Raw inventory,
    /// decoded output and transient pages overlap in memory; `rows` is untouched.
    pub fn readback(&mut self) -> Result<BTreeMap<root::Key, root::Record>, ReadbackFinding> {
        let inventory = self.retained()?;
        self.readback_of(&inventory)
    }

    /// Read back an inventory `retained` just returned, with nothing changed between.
    #[expect(
        clippy::too_many_lines,
        reason = "one inventory walk decodes each family and checks its cross-index links"
    )]
    pub fn readback_of(
        &mut self,
        rows: &BTreeMap<Box<[u8]>, Box<[u8]>>,
    ) -> Result<BTreeMap<root::Key, root::Record>, ReadbackFinding> {
        self.readback_boundary()?;
        let mut found = BTreeMap::new();
        let mut transcripts = BTreeMap::new();
        let mut attempts = Vec::new();
        let installation = keys::installation(self.component_limits.store.key).expect("installation key");
        for (key, value) in rows {
            if let Some(decoded) = keys::decode(key) {
                let event = self.readback_query(Request::Get { owner: READER, key: Get::Jig(decoded.clone()) })?;
                let Event::Got { owner: READER, row: Some(protocol::Row::Jig(row)) } = event else {
                    return Err(inventory(key, format!("discovered jig Get must be present and typed: {event:?}")));
                };
                if row.key() != decoded {
                    return Err(inventory(key, "Get key agrees with inventory"));
                }
                let row = root::Record::Core(row);
                if found.insert(row.key(), row).is_some() {
                    return Err(inventory(key, "unique decoded key"));
                }
            } else if let Some(application) = keys::of_application(key) {
                let (number, record_key) = jig_test_store::keys::decode(application, &self.limits.connector)
                    .ok_or_else(|| ReadbackFinding::ApplicationKey { key: application.into() })?;
                let event =
                    self.readback_query(Request::Get { owner: READER, key: Get::Application(application.into()) })?;
                let Event::Got { owner: READER, row: Some(protocol::Row::Application { key: got, value: bytes }) } =
                    event
                else {
                    return Err(inventory(
                        key,
                        format!("discovered application Get must be present and typed: {event:?}"),
                    ));
                };
                if got.as_ref() != application || &bytes != value {
                    return Err(inventory(key, "application Get key and bytes agree with inventory"));
                }
                let record = jig_test_store::records::decode(&bytes, &self.limits.connector)
                    .map_err(|_| ReadbackFinding::ApplicationValue { key: application.into() })?;
                if record.key() != record_key {
                    return Err(inventory(key, "application record key agreement"));
                }
                let row = root::Record::Connector { number, record };
                if found.insert(row.key(), row).is_some() {
                    return Err(inventory(key, "unique decoded key"));
                }
            } else if *key == installation {
                codec::accounts::Installation::decode(value)
                    .map_err(|failure| inventory(key, format!("installation row: {failure:?}")))?;
            } else if key.starts_with(&protocol::logs::EXTENTS) {
                if key.len() != 10 || value.len() != 8 {
                    return Err(inventory(key, "log extent key and value widths"));
                }
                Reader::new(value).u64().ok_or_else(|| inventory(key, "log extent"))?;
            } else {
                match keys::decode_own(key).ok_or_else(|| inventory(key, "recognized retained key"))? {
                    keys::Own::History { .. } => {
                        codec::component::HistoryLog::decode(value)
                            .map_err(|failure| inventory(key, format!("history log: {failure:?}")))?;
                    }
                    keys::Own::TurnLog { .. } => {
                        codec::component::TurnLog::decode(value)
                            .map_err(|failure| inventory(key, format!("turn log: {failure:?}")))?;
                    }
                    keys::Own::LogAttempt { .. } => {
                        codec::component::LogAttempt::decode(value)
                            .map_err(|failure| inventory(key, format!("log attempt: {failure:?}")))?;
                    }
                    keys::Own::Format => {
                        codec::component::Format::decode(value)
                            .map_err(|failure| inventory(key, format!("format row: {failure:?}")))?;
                    }
                    keys::Own::Layouts => {
                        codec::component::Layouts::decode(value)
                            .map_err(|failure| inventory(key, format!("layouts row: {failure:?}")))?;
                    }
                    keys::Own::Wires => {
                        protocol::component_v3::Wires::decode(
                            &protocol::component_v3::CEILINGS,
                            &mut Reader::new(value),
                        )
                        .map_err(|failure| inventory(key, format!("wires row: {failure:?}")))?;
                    }
                    keys::Own::Layout { .. } => {
                        protocol::registry_v1::Layout::decode(
                            &protocol::registry_v1::CEILINGS,
                            &mut Reader::new(value),
                        )
                        .expect("layout row");
                    }
                    keys::Own::Text(_) => {
                        protocol::registry_v1::Text::decode(&protocol::registry_v1::CEILINGS, &mut Reader::new(value))
                            .expect("text row");
                    }
                    keys::Own::Job { .. } => {
                        protocol::registry_v1::Job::decode(&protocol::registry_v1::CEILINGS, &mut Reader::new(value))
                            .expect("job row");
                    }
                    keys::Own::Opening(worker) => {
                        let opening = protocol::openings_v1::WorkerOpening::decode(
                            &protocol::openings_v1::CEILINGS,
                            &mut Reader::new(value),
                        )
                        .map_err(|failure| inventory(key, format!("opening row: {failure:?}")))?;
                        if opening.worker() != worker.as_bytes() {
                            return Err(inventory(key, "opening key agreement"));
                        }
                    }
                    keys::Own::Transcript { task } => {
                        let index = codec::component::Transcript::decode(value)
                            .map_err(|failure| inventory(key, format!("transcript index: {failure:?}")))?;
                        if index.window > index.length {
                            return Err(inventory(key, "window within committed transcript"));
                        }
                        if transcripts.insert(task, index).is_some() {
                            return Err(inventory(key, "unique transcript index"));
                        }
                    }
                    keys::Own::Attempt { task, attempt } => {
                        let index = codec::component::Attempt::decode(value)
                            .map_err(|failure| inventory(key, format!("attempt index: {failure:?}")))?;
                        attempts.push((key, task, attempt, index));
                    }
                }
            }
        }
        for (key, task, _, index) in attempts {
            // A drop leaves attempt places for bounded later commits (protocol/store/transcripts.md, 5).
            let Some(transcript) = transcripts.get(&task) else { continue };
            if index.first >= transcript.length {
                return Err(inventory(key, "attempt begins within committed transcript"));
            }
        }
        for task in transcripts.keys() {
            let forwards = self.transcript(*task, false)?;
            let mut backwards = self.transcript(*task, true)?;
            backwards.reverse();
            let key = keys::encode_own(&keys::Own::Transcript { task: *task }, self.component_limits.store.key)
                .expect("transcript key");
            if forwards != backwards {
                return Err(inventory(&key, "complete transcript frame orders agree"));
            }
            for turn in forwards {
                let row = root::Record::Core(core::Record::Core(core::CoreRecord::Turn(turn)));
                if found.insert(row.key(), row).is_some() {
                    return Err(inventory(&key, "unique transcript turn key"));
                }
            }
        }
        Ok(found)
    }

    /// Read every committed frame in the chosen order. Every Unread, including
    /// after a successful prefix, is a finding. Requires a quiescent fixture boundary;
    /// output owns memory proportional to the whole transcript.
    pub fn transcript(&mut self, task: u64, backwards: bool) -> Result<Vec<core::TurnRecord>, ReadbackFinding> {
        self.readback_boundary()?;
        let mut found = Vec::new();
        let mut after = None;
        loop {
            let event = self.readback_query(Request::Transcript {
                owner: READER,
                read: protocol::TranscriptRead::Page { task, backwards, after },
                rows: self.component_limits.store.page.rows,
                bytes: u64::from(self.component_limits.store.page.bytes),
            })?;
            let reason = match &event {
                Event::Turns { owner: READER, turns, next, cut: None } => {
                    if !turns.iter().all(|turn| turn.task == task) {
                        Some("transcript task agreement")
                    } else if let Some(next) = next {
                        if turns.is_empty() {
                            Some("nonterminal transcript page has turns")
                        } else if !backwards && *next == 0 {
                            Some("forward transcript cursor advances from zero")
                        } else if !after.is_none_or(|after| if backwards { *next < after } else { *next > after }) {
                            Some("transcript progress")
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                _ => Some("complete transcript requires successful Turns"),
            };
            if let Some(reason) = reason {
                return Err(ReadbackFinding::Transcript { task, backwards, after, terminal: Box::new(event), reason });
            }
            let Event::Turns { turns, next, .. } = event else { unreachable!("validated Turns") };
            // Production returns each backwards page in chronological order.
            // Reverse within the page to return the requested frame order.
            if backwards {
                found.extend(turns.into_vec().into_iter().rev());
            } else {
                found.extend(turns);
            }
            let Some(next) = next else { return Ok(found) };
            after = Some(next);
        }
    }
}
