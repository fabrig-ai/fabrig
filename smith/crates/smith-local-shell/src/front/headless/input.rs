//! The exec prompt's bounded stdin phase, before local policy is constructed.
//! The caller keeps one kernel for this phase and the run; IO owns stdin's close.
use crate::command::Usage;
use skein_io::{self as io, kernel};
use skein_lib::{Env, List, Queue, Time, Token, Wall, stream};
use skein_shell::{Clock, Host, Kernel};

const PROMPT_READ: u32 = 4096;

struct Input {
    io: io::Io,
    env: Env<io::Limits>,
    owner: Token,
    prompt: PromptInput,
    requests: Queue<io::Request>,
    events: Queue<io::Event>,
    completions: Queue<kernel::Complete>,
    submissions: Queue<kernel::Submit>,
}

pub(crate) fn read_prompt(
    kernel: &mut Kernel,
    clock: &Clock,
    limits: io::Limits,
    maximum: u32,
) -> Result<Box<[u8]>, Usage> {
    if !io::valid(&limits) {
        return Err(Usage("the prompt phase's IO limits are unusable".into()));
    }
    let mut io = io::Io::new(&limits);
    let owner = io.adopt_read_pipe(kernel::Fd::new(0)).map_err(|_| Usage("standard input cannot be adopted".into()))?;
    let mut input = Input {
        io,
        env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
        owner,
        prompt: PromptInput::new(maximum),
        requests: Queue::with_capacity(64),
        events: Queue::with_capacity(64),
        completions: Queue::with_capacity(64),
        submissions: Queue::with_capacity(64),
    };
    input.demand();
    let _exit = skein_shell::drive(kernel, clock, &mut input);
    input.prompt.finish()
}

impl Input {
    fn demand(&mut self) {
        self.requests.push(io::Request::Stream {
            stream: self.owner,
            down: stream::Down::Demand { read: stream::Read::Arrived { max: PROMPT_READ }, room: 0 },
        });
    }
}

impl Host for Input {
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.env.now = now;
        self.env.wall = wall;
        io::stage::up(&mut self.io, &self.env, &mut self.completions, &mut self.events, &mut self.submissions, now);
        while let Some(event) = self.events.pop() {
            match event {
                io::Event::Stream {
                    up: up @ (stream::Up::Bytes(_) | stream::Up::End | stream::Up::Failed(_)), ..
                } => {
                    if self.prompt.read_more(up) {
                        self.demand();
                    } else {
                        self.requests.push(io::Request::Close { entity: self.owner });
                    }
                }
                io::Event::Closed { .. } => {}
                io::Event::Listening { .. }
                | io::Event::Accepted { .. }
                | io::Event::Connecting { .. }
                | io::Event::Connected { .. }
                | io::Event::Stream { up: stream::Up::Room, .. }
                | io::Event::Output { .. }
                | io::Event::Spawned { .. }
                | io::Event::Exited { .. }
                | io::Event::Usage { .. }
                | io::Event::Resized
                | io::Event::Window { .. }
                | io::Event::Shutdown { .. }
                | io::Event::Failed { .. } => unreachable!("the prompt owns only an input pipe"),
            }
        }
        io::stage::down(&mut self.io, &self.env, &mut self.requests, &mut self.submissions);
        self.io.reclaim();
    }

    fn completions(&mut self) -> &mut Queue<kernel::Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<kernel::Submit> {
        &mut self.submissions
    }
    fn work_pending(&self, _now: Time) -> bool {
        !self.completions.is_empty() || !self.requests.is_empty() || !self.events.is_empty() || self.io.is_ready()
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.io.next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        None
    }
    fn is_empty(&self) -> bool {
        self.io.is_empty() && self.submissions.is_empty() && self.completions.is_empty() && self.events.is_empty()
    }
    fn operations(&self) -> u32 {
        io::operations(&self.env.limits).expect("resolved IO operations")
    }
}

struct PromptInput {
    bytes: List<u8>,
    failure: Option<Usage>,
}

impl PromptInput {
    fn new(maximum: u32) -> Self {
        Self { bytes: List::with_capacity(maximum), failure: None }
    }

    fn read_more(&mut self, up: stream::Up) -> bool {
        match up {
            stream::Up::Bytes(bytes) => {
                if bytes.iter().all(|byte| self.bytes.push(*byte).is_ok()) {
                    return true;
                }
                self.failure = Some(Usage("the prompt exceeds the message bound".into()));
            }
            stream::Up::End => {}
            stream::Up::Failed(_) => self.failure = Some(Usage("standard input failed before its end".into())),
            stream::Up::Room => unreachable!("standard input has no write demand"),
        }
        false
    }

    fn finish(self) -> Result<Box<[u8]>, Usage> {
        if let Some(failure) = self.failure {
            return Err(failure);
        }
        if self.bytes.as_slice().trim_ascii().is_empty() {
            return Err(Usage("the prompt is empty".into()));
        }
        Ok(self.bytes.into_boxed())
    }
}

#[cfg(test)]
mod tests {
    use super::{PromptInput, stream};
    use crate::compose::Refused;

    const PROMPT_BOUND: u32 = 32; // Room for either fixture's input.

    #[test]
    fn a_prompt_read_as_it_arrives_is_the_same_whatever_its_cuts() {
        let bytes = b"abcdefghijklmnopqrst";
        for cut in [bytes.len(), 1] {
            let mut prompt = PromptInput::new(PROMPT_BOUND);
            for delivery in bytes.chunks(cut) {
                assert!(prompt.read_more(stream::Up::Bytes(delivery.into())), "prompt fits its bound");
            }
            assert!(!prompt.read_more(stream::Up::End), "EOF closes the input");
            assert_eq!(prompt.finish().expect("a prompt").as_ref(), bytes);
        }
    }

    #[test]
    fn an_empty_prompt_is_a_usage_error_exiting_2() {
        let mut prompt = PromptInput::new(PROMPT_BOUND);
        for delivery in [b" \t".as_slice(), b"\n"] {
            assert!(prompt.read_more(stream::Up::Bytes(delivery.into())), "whitespace does not end the read");
        }
        assert!(!prompt.read_more(stream::Up::End), "EOF closes the input");
        let refused = Refused::Usage(prompt.finish().expect_err("a whitespace-only prompt must refuse"));
        assert_eq!(refused.code(), 2, "empty input is a usage error");
        assert_eq!(refused.line(), "smith: usage: the prompt is empty", "empty prompt line");
    }
}
