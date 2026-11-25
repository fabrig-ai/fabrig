//! Independent one-slot lanes for the placement story (temper/docs/design/testing/tiers.md, 1.1).
use super::{Lane, STEPS, Script, Token, World, engine, open_config, signed_in_party, worker_limits};

const SECOND_CHANNEL: Token = Token::new(8);
const POSITIONS: u32 = 2;
const PLACES: u32 = 1;
pub(super) const CHAT_SPEND: u64 = 100;

impl World {
    /// Open the usual owner chat world with two independently driven workers.
    #[must_use]
    pub fn two_workers(seed: u64, script: Script, words: &[u8]) -> Self {
        let mut world = Self::configured_limits(seed, script.clone(), false, config, limits);
        world.party = signed_in_party(words);
        world.lanes.push(Lane::new(SECOND_CHANNEL, 1, seed, script, worker_limits(), Box::new([])));
        for lane in &mut world.lanes {
            lane.process.held_query = Some(1);
            lane.process.starts = Some(Vec::new());
        }
        world
    }

    /// Wait until both workers reach the model before releasing either response.
    pub fn release_both_models(&mut self) {
        assert_eq!(self.lanes.len(), POSITIONS as usize);
        for _ in 0..STEPS {
            self.tick();
            if self.lanes.iter().all(|lane| lane.process.model_held()) {
                for lane in &mut self.lanes {
                    lane.process.release_model();
                }
                return;
            }
        }
        panic!("both model responses were not held: {:?}", self.observed);
    }

    /// Worker-side starts, retaining repeated starts for each position.
    pub fn worker_starts(&self) -> impl Iterator<Item = &[(u64, u64)]> {
        self.lanes.iter().map(|lane| lane.process.starts.as_deref().expect("tracking worker starts"))
    }

    /// Close one numbered chat through the authenticated party and await its durable end.
    pub fn close_numbered_chat(&mut self, task: u64, key: [u8; 16]) {
        self.act(super::parties::Ask::Cancel { task: super::parties::Task::Number(task), grace: None }, key);
        self.await_chat_end(task);
    }
}

fn config(seed: u64) -> engine::Config {
    let mut config = open_config(seed);
    config.core.defaults.workers = Box::new([worker(1, b"first-worker"), worker(2, b"second-worker")]);
    config
}

fn worker(number: u32, name: &[u8]) -> jig_domain_core::settings::WorkerSetting {
    jig_domain_core::settings::WorkerSetting {
        number,
        name: name.into(),
        places: PLACES,
        digest: Some([7; 32]),
        withdrawn: false,
    }
}

fn limits(limits: &mut engine::Limits) {
    limits.fleet.configured_workers = POSITIONS;
    limits.fleet.workers = POSITIONS;
    limits.fleet.slots = PLACES;
    limits.people.program_hosts = POSITIONS + 1;
    limits.accounts.program_hosts = POSITIONS + 1;
    limits.journal.deliveries = limits.journal.deliveries.max(64);
    limits.journal.held = 3 * limits.journal.deliveries + 16;
}
