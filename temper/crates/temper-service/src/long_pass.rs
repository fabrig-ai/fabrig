//! A long pass of the service's loop (protocol/README.md, 6.1).
use crate::Stage;
use skein_lib::{Duration, Time};

pub const LONG_PASS: Duration = Duration::from_secs(1);
pub const LINE_EVERY: Duration = Duration::from_secs(10);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Part {
    Service(Stage),
    Names,
    Drain,
}
impl Part {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Service(Stage::Io) => "io",
            Self::Service(Stage::Route) => "route",
            Self::Service(Stage::Engine) => "engine",
            Self::Service(Stage::Worker) => "worker",
            Self::Service(Stage::Out) => "out",
            Self::Names => "names",
            Self::Drain => "drain",
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct LongPass {
    pub took: Duration,
    pub part: Part,
    pub part_took: Duration,
    pub more: u32,
}
#[derive(Debug)]
pub struct Watch {
    began: Time,
    ended: Time,
    pass: LongPass,
    last_line: Option<Time>,
    held: Option<LongPass>,
}
impl Watch {
    #[must_use]
    #[expect(clippy::new_without_default, reason = "step code names no Default (programming-model.md, 10.2)")]
    pub const fn new() -> Self {
        Self {
            began: Time::ZERO,
            ended: Time::ZERO,
            pass: LongPass { took: Duration::ZERO, part: Part::Service(Stage::Io), part_took: Duration::ZERO, more: 0 },
            last_line: None,
            held: None,
        }
    }
    pub fn begin(&mut self, at: Time) {
        self.began = at;
        self.ended = at;
        self.pass =
            LongPass { took: Duration::ZERO, part: Part::Service(Stage::Io), part_took: Duration::ZERO, more: 0 };
    }
    pub fn ended(&mut self, part: Part, at: Time) {
        let took = at.saturating_since(self.ended);
        if took > self.pass.part_took {
            self.pass.part = part;
            self.pass.part_took = took;
        }
        self.pass.took = at.saturating_since(self.began);
        self.ended = at;
    }
    #[must_use]
    pub fn pass_ended(&mut self) -> Option<LongPass> {
        if self.pass.took > LONG_PASS {
            self.held = Some(match self.held {
                Some(held) => {
                    let more = held.more.saturating_add(1);
                    if self.pass.took > held.took { LongPass { more, ..self.pass } } else { LongPass { more, ..held } }
                }
                None => self.pass,
            });
        }
        let due = match self.last_line {
            Some(at) => self.ended.saturating_since(at) >= LINE_EVERY,
            None => true,
        };
        if due && self.held.is_some() {
            self.last_line = Some(self.ended);
            self.held.take()
        } else {
            None
        }
    }
    #[must_use]
    pub const fn held(&self) -> Option<LongPass> {
        self.held
    }
}
