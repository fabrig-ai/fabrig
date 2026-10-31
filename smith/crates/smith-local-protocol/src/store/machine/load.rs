//! Metadata and the two bounded record passes (protocol/hosts.md, 5.3).
//! Header traversal selects the transcript window before record bytes enter its budget.
use super::Terminal;
use super::{
    Action, Box, ChatState, DeliveryRecord, Fresh, List, Queue, RecordName, Resume, State, Store, StoreFault,
    decode_state, decode_turns_bounded, digest, file, local, turn_opens_window_bounded,
};

#[derive(Debug)]
pub(super) enum Loading {
    Journal,
    Metadata { fresh: Option<ChatState> },
    Scan { state: Option<ChatState> },
    Headers { state: Option<ChatState>, entries: Box<[file::Entry]>, index: usize, window: Box<[u8]> },
    Records(Records),
}

#[derive(Debug)]
pub(super) struct Records {
    state: Option<ChatState>,
    entries: Box<[file::Entry]>,
    index: usize,
    window: Box<[u8]>,
    total: u64,
    turns: List<Box<[u8]>>,
    deliveries: List<DeliveryRecord>,
}

impl Store {
    pub(super) fn load_request(&self, loading: &Loading) -> file::Request {
        match loading {
            Loading::Journal => self.load_file(b"fresh".as_slice().into(), metadata_bytes()),
            Loading::Metadata { .. } => self.load_file(b"state".as_slice().into(), metadata_bytes()),
            Loading::Scan { .. } => self.scan_request(false),
            Loading::Headers { entries, index, .. } => {
                self.load_file(entries.get(*index).expect("selected header").name.clone(), self.limits.file_bytes)
            }
            Loading::Records(records) => self.load_file(
                records.entries.get(records.index).expect("selected record").name.clone(),
                self.limits.file_bytes,
            ),
        }
    }

    pub(super) fn load_event(&mut self, loading: Loading, event: Terminal, out: &mut Queue<Action>) -> State {
        match loading {
            Loading::Journal => match metadata(event, RecordName::Journal, self) {
                Ok(fresh) => State::Load(Loading::Metadata { fresh }),
                Err(fault) => malformed(RecordName::Journal, fault, out),
            },
            Loading::Metadata { fresh } => match metadata(event, RecordName::State, self) {
                Ok(state) => match fresh {
                    Some(state) => State::Fresh(Fresh::Scan { state, resume: Resume::Load }),
                    None => State::Load(Loading::Scan { state }),
                },
                Err(fault) => malformed(RecordName::State, fault, out),
            },
            Loading::Scan { state } => match event {
                Terminal::Scanned { entries, more: 0 } => self.header_next(state, entries, 0, Box::new([]), out),
                Terminal::Scanned { .. } => {
                    malformed(RecordName::State, StoreFault::TooLarge { file: RecordName::State }, out)
                }
                event @ (Terminal::Loaded(_)
                | Terminal::Stored(_)
                | Terminal::Removed
                | Terminal::Synced
                | Terminal::Closed
                | Terminal::Missing
                | Terminal::Committed(_)
                | Terminal::Fault(_)) => {
                    malformed(RecordName::State, super::event_fault(RecordName::State, event), out)
                }
            },
            Loading::Headers { state, entries, index, mut window } => {
                let path = entries.get(index).expect("selected header").name.clone();
                let record = super::record_name(&path).expect("selected turn");
                match event {
                    Terminal::Loaded(bytes) => {
                        match turn_opens_window_bounded(&bytes, self.limits.file_bytes) {
                            Ok(true) => window = path,
                            Ok(false) => {}
                            Err(_) => return malformed(record, StoreFault::Malformed { file: record }, out),
                        }
                        self.header_next(state, entries, index.checked_add(1).expect("bounded scan"), window, out)
                    }
                    event @ (Terminal::Scanned { .. }
                    | Terminal::Stored(_)
                    | Terminal::Removed
                    | Terminal::Synced
                    | Terminal::Closed
                    | Terminal::Missing
                    | Terminal::Committed(_)
                    | Terminal::Fault(_)) => malformed(record, super::event_fault(record, event), out),
                }
            }
            Loading::Records(records) => self.record_loaded(records, event, out),
        }
    }

    fn header_next(
        &self,
        state: Option<ChatState>,
        entries: Box<[file::Entry]>,
        start: usize,
        window: Box<[u8]>,
        out: &mut Queue<Action>,
    ) -> State {
        for index in start..entries.len() {
            match super::record_name(&entries.get(index).expect("bounded header index").name) {
                Some(RecordName::Turn(_)) => return State::Load(Loading::Headers { state, entries, index, window }),
                Some(RecordName::State | RecordName::Journal | RecordName::Delivery(_)) | None => {}
            }
        }
        self.record_next(
            Records {
                state,
                entries,
                index: 0,
                window,
                total: 0,
                turns: List::with_capacity(self.limits.turns),
                deliveries: List::with_capacity(self.limits.deliveries),
            },
            out,
        )
    }

    fn record_loaded(&mut self, mut records: Records, event: Terminal, out: &mut Queue<Action>) -> State {
        let path = records.entries.get(records.index).expect("selected record").name.clone();
        let record = super::record_name(&path).expect("selected record name");
        let bytes = match event {
            Terminal::Loaded(bytes) => bytes,
            event @ (Terminal::Scanned { .. }
            | Terminal::Stored(_)
            | Terminal::Removed
            | Terminal::Synced
            | Terminal::Closed
            | Terminal::Missing
            | Terminal::Committed(_)
            | Terminal::Fault(_)) => return malformed(record, super::event_fault(record, event), out),
        };
        records.total = match records.total.checked_add(u64::try_from(bytes.len()).expect("file bytes fit")) {
            Some(total) if total <= u64::from(self.limits.file_bytes) => total,
            Some(_) | None => return malformed(record, StoreFault::TooLarge { file: record }, out),
        };
        match record {
            RecordName::Turn(_) => {
                if records.turns.push(bytes).is_err() {
                    return malformed(record, StoreFault::TooLarge { file: record }, out);
                }
            }
            RecordName::Delivery(_) => {
                if self.versions.insert(path, digest(&bytes)).is_err() {
                    return malformed(record, StoreFault::TooLarge { file: record }, out);
                }
                match crate::decode_delivery(&bytes, self.limits.file_bytes.min(super::DELIVERY_BYTES)) {
                    Ok(value) => {
                        if records.deliveries.push(value).is_err() {
                            return malformed(record, StoreFault::TooLarge { file: record }, out);
                        }
                    }
                    Err(_) => return malformed(record, StoreFault::Malformed { file: record }, out),
                }
            }
            RecordName::State | RecordName::Journal => unreachable!("record scan selects turns and deliveries"),
        }
        records.index = records.index.checked_add(1).expect("bounded scan");
        self.record_next(records, out)
    }

    fn record_next(&self, mut records: Records, out: &mut Queue<Action>) -> State {
        for index in records.index..records.entries.len() {
            let entry = records.entries.get(index).expect("bounded record index");
            let selected = match super::record_name(&entry.name) {
                Some(RecordName::Turn(_)) => entry.name >= records.window,
                Some(RecordName::Delivery(_)) => true,
                Some(RecordName::State | RecordName::Journal) | None => false,
            };
            if selected {
                records.index = index;
                return State::Load(Loading::Records(records));
            }
        }
        self.finish_load(records, out)
    }

    fn finish_load(&self, mut records: Records, out: &mut Queue<Action>) -> State {
        match decode_turns_bounded(
            records.turns.as_slice(),
            &smith_transcript::CEILINGS,
            &self.endpoints,
            self.limits.file_bytes,
        ) {
            Ok((transcript, read)) => {
                if transcript.is_some() {
                    match &mut records.state {
                        Some(state) => state.read = read,
                        None => {
                            return malformed(
                                RecordName::State,
                                StoreFault::Malformed { file: RecordName::State },
                                out,
                            );
                        }
                    }
                }
                Self::finished(
                    local::Event::Loaded {
                        state: records.state,
                        transcript,
                        deliveries: records.deliveries.into_boxed(),
                    },
                    out,
                )
            }
            Err(_) => malformed(RecordName::State, StoreFault::Malformed { file: RecordName::State }, out),
        }
    }
}

fn metadata(event: Terminal, record: RecordName, store: &mut Store) -> Result<Option<ChatState>, StoreFault> {
    match event {
        Terminal::Loaded(bytes) => match decode_state(&bytes) {
            Ok(state) => {
                let path = match record {
                    RecordName::Journal => b"fresh".as_slice(),
                    RecordName::State => b"state".as_slice(),
                    RecordName::Turn(_) | RecordName::Delivery(_) => unreachable!("metadata records only"),
                };
                store.versions.insert(path.into(), digest(&bytes)).expect("bounded metadata versions");
                Ok(Some(state))
            }
            Err(_) => Err(StoreFault::Malformed { file: record }),
        },
        Terminal::Missing => Ok(None),
        event @ (Terminal::Scanned { .. }
        | Terminal::Stored(_)
        | Terminal::Removed
        | Terminal::Synced
        | Terminal::Closed
        | Terminal::Committed(_)
        | Terminal::Fault(_)) => Err(super::event_fault(record, event)),
    }
}

fn malformed(record: RecordName, fault: StoreFault, out: &mut Queue<Action>) -> State {
    Store::failure(record, fault, true, out)
}

fn metadata_bytes() -> u32 {
    u32::try_from(
        crate::local_store_v1::State::encoded_bound(&crate::local_store_v1::CEILINGS).expect("fixed metadata"),
    )
    .expect("metadata fits u32")
}
