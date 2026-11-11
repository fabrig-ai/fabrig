//! A parked producer's delivered and saved edits across a moved landing
//! branch (temper/docs/design/domain/worker/checkouts.md, **Saved work**).
use super::{Script, VecDeque, World, call, engine, questioning_config, run, tasks};

// The goal creates the waiting change and its producer before the independent change runs.
const CHANGE: u64 = 2;
const PRODUCER: u64 = 4;
// The clean story resumes three times; each activation uses the composed world's tick bound.
const RESUMES: u32 = 3;
const STEPS: u32 = 10000;

impl World {
    /// A goal's producer delivers one edit, writes two more, and asks;
    /// the independent change moves the goal to `moved_path`.
    #[must_use]
    pub fn parked_edits(seed: u64, moved_path: &[u8], script: Script) -> Self {
        let goal = Script {
            cue: b"Help the person with their work.".as_slice().into(),
            turns: Box::new([
                call(b"change", br#"{"changes":[{"words":"Make edits and keep saved work","landing":"goal","questions":"up"},{"words":"Advance the goal","landing":"goal"}]}"#),
                call(b"wait", b"{}"),
            ]),
        };
        let mut world = Self::configured_with(seed, goal.clone(), true, questioning_config);
        world.goal_branch = true;
        let write = serde_json::to_vec(&serde_json::json!({
            "path": std::str::from_utf8(moved_path).expect("fixture path"),
            "content": "goal moved\n",
        }))
        .expect("fixture write");
        let moving = Script {
            cue: b"Read only what the change needs".as_slice().into(),
            turns: Box::new([
                call(b"write", &write),
                call(b"finish", br#"{"title":"Advance the goal","body":"Give the waiting producer new context."}"#),
            ]),
        };
        world.scripts = VecDeque::from([goal, script, moving]);
        world
    }

    /// Answers each question in turn, then checks the delivered head.
    pub fn continue_parked_edits(&mut self, files: &[(&[u8], &[u8])]) {
        self.until_producer_answer(0);
        assert!(matches!(self.agent().answer(), run::Answer::Parked { .. }));
        let branch = self.assignment().workspace.repositories[0].push.clone().expect("producer push branch");
        let first = self.forge.branch(b"org/repo", &branch).expect("first delivery exists");
        let base = self.reviewed_change().base.clone();
        let goal = self.advance_goal_for_producer(&base);
        let mut starts = Vec::new();
        for _ in 0..RESUMES {
            for _ in 0..STEPS {
                let key = engine::Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(CHANGE)));
                if let Some(engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))) =
                    self.store.rows.get(&key)
                    && !row.questions.is_empty()
                {
                    break;
                }
                self.tick();
            }
            let attempt = self.assignment().attempt;
            let spawns = self.observed.spawns;
            self.answer_producer_question();
            for _ in 0..STEPS {
                self.tick();
                if self.assignment().task == PRODUCER
                    && self.assignment().attempt > attempt
                    && self.observed.spawns > spawns
                    && !self.process.prepared()
                {
                    break;
                }
            }
            assert_eq!(self.assignment().task, PRODUCER, "the same producer resumes");
            let repository = &self.assignment().workspace.repositories[0];
            starts.push((
                repository.start.clone(),
                repository.save.clone(),
                self.agent().disk().files().get(b"repo/kept.txt".as_slice()).map(|bytes| bytes.to_vec()),
            ));
            self.until_producer_answer(attempt);
            if matches!(self.agent().answer(), run::Answer::Accepted { .. }) {
                let head = self.forge.branch(b"org/repo", &branch).expect("final delivery exists");
                for (path, content) in files {
                    self.assert_producer_file(head, path, content);
                }
                assert!(self.forge.is_ancestor(first, head), "the first delivery remains an ancestor");
                assert!(self.forge.is_ancestor(goal, head), "the goal tip remains an ancestor");
                for (start, save, kept) in starts {
                    assert!(
                        matches!(start, engine::ForgeStart::Merge { branch, .. } if Some(branch.as_ref()) == save.as_deref()),
                        "each resumed producer merges its saved branch"
                    );
                    assert_eq!(
                        kept.as_deref(),
                        Some(b"saved work\n".as_slice()),
                        "saved edits are on the worker's disk at the run's start"
                    );
                }
                self.run();
                return;
            }
            assert!(matches!(self.agent().answer(), run::Answer::Parked { .. }));
            self.run();
        }
        panic!("the producer did not finish after its questions");
    }
}
