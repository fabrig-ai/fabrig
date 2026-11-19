use super::{Exchange, family_record, wire};
use crate::{Request, stream::Stream};
use jig_client_protocol_wire as w;
use jig_domain_core_vocabulary as core;
use skein_lib::{Token, Writer};
impl Exchange {
    pub fn open_stream(&mut self) -> [u8; 16] {
        self.stream_open();
        self.authenticate();
        self.component.streams.first().expect("stream opened").1.name
    }
    pub fn open_record(&mut self, stream: [u8; 16], name: u64, subject: w::Owned) -> Token {
        let value = w::Open { stream, watch: name, subject };
        let mut bytes = Writer::new(skein_lib::length::usize_of(value.measure()));
        value.encode(&mut bytes).expect("open");
        let owner = self.send(crate::Message::Watch(bytes.finish()));
        self.authenticate();
        owner
    }
    pub fn unwatch(&mut self, stream: [u8; 16], watch: u64) -> Token {
        let value = w::WatchClose { stream, watch };
        let mut body = Writer::new(skein_lib::length::usize_of(value.measure()));
        value.encode(&mut body).expect("close");
        let owner = self.send(crate::Message::Unwatch(body.finish()));
        self.authenticate();
        owner
    }
    pub fn take_watch(&mut self) -> Token {
        let request = self.out.pop().expect("watch request");
        if let Request::Core(event) = request
            && let core::Event::Watch { watcher, .. } | core::Event::WatchRun { watcher, .. } = *event
        {
            return watcher;
        }
        panic!("core watch");
    }
    pub fn stream(&self) -> &Stream {
        self.component.streams.first().expect("stream").1
    }
}
pub fn owned_subject(value: &w::Subject) -> w::Owned {
    let mut bytes = Writer::new(skein_lib::length::usize_of(value.measure()));
    value.encode(&mut bytes).expect("subject");
    w::Owned { owner: w::Owner::Jig, version: jig_client_protocol_wire::VERSION, record: bytes.finish() }
}
pub fn family_subject() -> w::Owned {
    let family = w::Family { number: 9 };
    w::Owned { owner: w::Owner::Family(family), version: 2, record: family_record(Box::new([77])) }
}
pub fn task_subject() -> w::Subject {
    w::Subject::Task(w::SubjectTask { task: 7 })
}
pub fn subject_cases() -> [(&'static str, w::Subject, core::documents::Watched, u32); 7] {
    use core::documents as d;
    let limits = wire();
    [
        ("Frame", w::Subject::Me, d::Watched::Frame, 32),
        (
            "Inbox",
            w::Subject::Inbox(w::SubjectInbox { window: 3 }),
            d::Watched::Inbox,
            3,
        ),
        ("Task", task_subject(), d::Watched::Task { task: 7 }, 32),
        (
            "Tree",
            w::Subject::Tree(w::SubjectTree { task: 7, window: 3 }),
            d::Watched::Tree { task: 7 },
            3,
        ),
        (
            "Chips",
            w::Subject::Chips(
                w::SubjectChips { tasks: skein_lib::List::with_capacity(0) },
            ),
            d::Watched::Chips { tasks: skein_lib::List::with_capacity(0) },
            32,
        ),
        (
            "Tasks",
            w::Subject::Tasks(
                w::SubjectTasks {
                        query: w::TaskQuery {
                                parts: skein_lib::List::with_capacity(0),
                                project: Some(3),
                                requester: None,
                                root: Some(7),
                                parent: None,
                                label: None,
                                phase: None,
                                executor: None,
                                tracked: None,
                                order: w::QueryOrder::Changed,
                            },
                        window: 3,
                    },
            ),
            d::Watched::Query {
                query: d::TaskQuery {
                    parts: skein_lib::List::with_capacity(0),
                    project: Some(3),
                    requester: None,
                    root: Some(7),
                    parent: None,
                    label: None,
                    phase: None,
                    executor: None,
                    tracked: None,
                    order: d::QueryOrder::Changed,
                },
            },
            3,
        ),
        ("Deployment", w::Subject::Deployment, d::Watched::Deployment, 32),
    ]
}
