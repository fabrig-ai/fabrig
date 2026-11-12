//! Review failure walks across Smith, the worker and the change procedure.
//! Contract: temper/docs/design/domain/forge/change/gates.md, 1.
use super::{Script, VecDeque, World, agreement, call, engine, tasks};
use std::fmt::Write;
use temper_engine_domain_forge as forge;
use temper_engine_domain_forge_change as change;

fn config(seed: u64) -> engine::Config {
    let mut config = agreement::config(seed);
    let rule = engine::LandingRule {
        sets: Box::new([]),
        connector: 0,
        kind: 4,
        enforce: true,
        ci: true,
        up_to_date: true,
        pattern: jig_domain_authority::Pattern {
            segments: Box::new([b"forge".as_slice().into()]),
            last: jig_domain_authority::Last::Open(Box::default()),
        },
        gates: Box::new([engine::Gate { number: 1, blocking: true, freshness: engine::Freshness::Exact }]),
        approvals: Box::default(),
    };
    config.kinds_stock = engine::landing_stock(&[rule], &skein_lib::Map::with_capacity(0), 2, 4);
    config
}

impl World {
    /// Make a CI-checked change with a required review under the stock charters.
    #[must_use]
    pub fn review_change(seed: u64, first: Script, second: Script) -> Self {
        let chat = Script {
            cue: b"Help the person".as_slice().into(),
            turns: Box::new([
                call(b"change", br#"{"words":"Add export.csv containing fruit,count and a newline"}"#),
                call(b"wait", b"{}"),
            ]),
        };
        let producer = Script {
            cue: b"Run only what your change needs".as_slice().into(),
            turns: Box::new([
                call(b"write", br#"{"path":"export.csv","content":"fruit,count\n"}"#),
                call(b"finish", br#"{"title":"Add CSV export","body":"Add the header."}"#),
            ]),
        };
        let mut world = Self::configured_with(seed, chat.clone(), true, config);
        world.scripts = VecDeque::from([chat, producer, first, second]);
        world
    }

    /// Produce a pull with `bytes` changed content bytes and review it through
    /// the production assignment and receiving-worker fitting entrances.
    #[must_use]
    pub fn large_review_change(seed: u64, bytes: usize, review: Script) -> Self {
        let small = Self::review_change(seed, review.clone(), review);
        let chat = small.scripts.front().expect("chat script").clone();
        let mut world = Self::configured_limits(seed, chat, true, config, |limits| {
            limits.brief.read_bytes = 131_072;
            limits.brief.brief_bytes = 131_072;
            limits.brief.budgets.pull = 131_072;
            limits.forge.brief_bytes = 131_072;
            limits.journal.result_bytes = 131_072;
        });
        world.scripts = small.scripts;
        let producer = world.scripts.get_mut(1).expect("producer script");
        let mut turns = vec![call(b"read", br#"{"path":"export.csv","offset":1,"window":128}"#)];
        turns.extend(producer.turns.clone());
        producer.turns = turns.into_boxed_slice();
        world.forge.advance(b"org/repo", b"main", b"export.csv", &vec![b'x'; bytes - b"fruit,count\n".len()]);
        world
    }

    /// The fitted charter delivered to the reviewer at the process entrance.
    #[must_use]
    pub fn review_charter(&self, task: u64) -> smith_domain_run::Charter {
        let process = if self.assignment().task == task {
            &self.process
        } else {
            self.parked.get(&task).expect("review process")
        };
        smith_protocol_channel::decode_charter(
            &process.charter,
            &smith_charter::CEILINGS,
            &super::worker_protocol::endpoints(),
        )
        .expect("receiving charter")
    }

    /// Count the changed before/after contents reported by the fake forge.
    pub fn pull_diff_bytes(&mut self) -> usize {
        let number = self.reviewed_change().pull.expect("pull opened");
        let answer = self
            .forge
            .api(b"org/repo", 1, super::fake::Op::Read(super::fake::Read::PullFiles { number, page: 1, limit: 100 }))
            .expect("pull files");
        let super::fake::Answer::PullFiles { files, more, .. } = answer else { panic!("pull files answer") };
        assert!(!more);
        files
            .iter()
            .map(|file| {
                file.before.as_ref().map_or(0, |text| text.len()) + file.after.as_ref().map_or(0, |text| text.len())
            })
            .sum()
    }

    /// The durable change being judged, including CI and gate failure accounting.
    #[must_use]
    pub fn reviewed_change(&self) -> &forge::ChangeRow {
        self.store.rows.values().find_map(change_row).expect("change created")
    }

    /// The real Smith run kept for a review, including its reads and terminal failure.
    #[must_use]
    pub fn review_agent(&self, task: u64) -> &smith_agent_world::World {
        if self.assignment().task == task {
            self.agent()
        } else {
            self.parked.get(&task).expect("completed review").agent()
        }
    }

    /// Verify the fake CI passed at the head before the reviewer starts.
    pub fn assert_review_ci(&mut self) {
        let number = self.reviewed_change().pull.expect("pull opened before review");
        let head = self.reviewed_change().change.last_head.expect("review head");
        let answer = self
            .forge
            .api(b"org/repo", 1, super::fake::Op::Read(super::fake::Read::Pull { number }))
            .expect("pull read");
        let super::fake::Answer::Pull(pull) = answer else { panic!("pull answer") };
        assert_eq!(skein_fake_forge_domain::object_bytes(pull.commit).as_slice(), &head[..20]);
        assert!(!pull.statuses.is_empty());
        assert!(pull.statuses.iter().all(|status| status.state == super::fake::Check::Passed));
    }

    pub(super) fn bind_review_script(&mut self) {
        let Some(script) = self.scripts.front_mut() else { return };
        // Only the script's explicit placeholder is supplied by this world.
        let head = self.store.rows.values().find_map(change_row).and_then(|row| row.change.last_head);
        let Some(head) = head else { return };
        let mut hex = String::new();
        for byte in &head[..20] {
            write!(&mut hex, "{byte:02x}").expect("writing a string");
        }
        for turn in &mut script.turns {
            for line in &mut turn.lines {
                if let skein_fake_llm_domain::api::Line::Call { arguments, .. } = line {
                    *arguments = String::from_utf8(arguments.to_vec())
                        .expect("script JSON")
                        .replace("$HEAD", &hex)
                        .into_bytes()
                        .into_boxed_slice();
                }
            }
        }
    }

    /// Drive to a review assignment, or a terminal change hold/landing.
    pub fn until_review_or_terminal(&mut self, after: u64) {
        for _ in 0..10000 {
            self.tick();
            if self.assignment.as_ref().is_some_and(|assignment| {
                assignment.task > after && matches!(assignment.run.contract, tasks::Contract::Verdict { .. })
            }) {
                return;
            }
            if self.store.rows.values().any(|row| {
                matches!(row,
                engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))
                    if row.number == after && matches!(row.retry_notice, Some(tasks::RetryNotice::Failed { .. })))
            }) {
                return;
            }
            if self
                .store
                .rows
                .values()
                .filter_map(change_row)
                .any(|row| matches!(row.change.state, change::State::Held { .. } | change::State::Landed { .. }))
            {
                return;
            }
        }
        panic!("review did not advance: {:?}; change {:?}", self.observed, self.reviewed_change());
    }
}

pub(super) fn decode(to: skein_lib::ReplyTo, call: &jig_domain_fleet::Call) -> engine::WorkerInput {
    // The world's protocol face refuses malformed reads, as the service does.
    // Successful calls continue through its existing codec translation.
    let name = smith_domain_run::CallName { activation: 1, completion: 1, position: 1 };
    let input = smith_domain_run::HostInput::attested(call.input.clone()).expect("bounded script input");
    match temper_engine_smith::call(name, &call.tool, &input) {
        Ok(_) => engine::WorkerInput::DecodedHostCall { to, body: super::link::decode(call) },
        Err(temper_engine_smith::Problem::Range(member)) => engine::WorkerInput::RenderedHostCall {
            to,
            answer: jig_domain_core::SettledAnswer::Host {
                error: true,
                body: format!("host call out of range: {}", String::from_utf8_lossy(member)).into_bytes().into(),
            },
        },
        Err(other) => panic!(
            "unscripted decode refusal: {other:?}, tool {:?}, input {:?}",
            String::from_utf8_lossy(&call.tool),
            String::from_utf8_lossy(&call.input)
        ),
    }
}

fn change_row(record: &engine::Record) -> Option<&forge::ChangeRow> {
    if let engine::Record::Forge { row } = record
        && let forge::Stored::Change(row) = row.as_ref()
    {
        return Some(row);
    }
    None
}
