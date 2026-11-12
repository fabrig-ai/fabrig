//! Follow fixtures use an adopted context source and a separate owned target.
use super::{Duration, LIMITS, REPO, Token, World, forge, raw, repository_config, top, translate};
impl World {
    #[must_use]
    pub fn following(seed: u64, queue: u32) -> Self {
        let mut config = repository_config(2);
        config.limits.commits = 64;
        config.limits.files = 16;
        let mut world = Self::configured_repositories(
            seed,
            LIMITS.facts,
            false,
            Some(skein_forge_client::Writer { forge: REPO.forge, author: 1 }),
            None,
            false,
            4,
            false,
            None,
            config,
            None,
        );
        world.env.limits.follow_queue = queue;
        world.env.limits.client.calls = 4; // Two pinned comparisons leave room to poll the next source tip.
        // Reconstruct the top with the scenario's queue bound before accepting work.
        world.restart();
        forge::repository(
            &mut world.forge,
            &world.forge_env.limits,
            raw::Setup {
                id: 3,
                name: b"org/second".as_slice().into(),
                default: b"main".as_slice().into(),
                tree: Box::new([]),
                labels: Box::new([]),
                workflows: Box::new([]),
                protection: None,
                hooked: false,
                checks: raw::Checks {
                    cues: Box::new([]),
                    contexts: Box::new([]),
                    latency_min: Duration::ZERO,
                    latency_max: Duration::ZERO,
                    silent: 0,
                    passes: 1000,
                    reruns: 0,
                    cue: None,
                },
            },
        );
        forge::grant(&mut world.forge, b"org/second", 1, raw::Permission::Admin);
        world.adopt_as(top::Role::Context);
        world.run_for(2);
        world.event(top::Event::Application {
            role: None,
            carried: None,
            request: Token::new(101),
            token: Token::new(101),
            retention: Duration::from_secs(60),
            ask: top::Ask::Adopt(top::Adoption {
                project: 5,
                home: true,
                forge: REPO.forge,
                host: b"forge.example".as_slice().into(),
                owner: b"org".as_slice().into(),
                name: b"second".as_slice().into(),
                prefix: b"temper/".as_slice().into(),
                role: top::Role::Owned,
                landing: b"main".as_slice().into(),
                ci: false,
                checks: Box::new([]),
            }),
        });
        world.run_for(2);
        assert_eq!(world.top.repository_role(5, REPO), Some(top::Role::Context));
        world
    }
    /// A target with a dispatch runner and an independently adopted context source.
    #[must_use]
    pub fn checking_follow(seed: u64, workflow: raw::Workflow) -> Self {
        let mut world = Self::configured_repositories(
            seed,
            LIMITS.facts,
            false,
            Some(skein_forge_client::Writer { forge: REPO.forge, author: 1 }),
            None,
            false,
            6,
            false,
            Some(workflow),
            repository_config(2),
            None,
        );
        world.adopt();
        forge::repository(
            &mut world.forge,
            &world.forge_env.limits,
            raw::Setup {
                id: 3,
                name: b"org/second".as_slice().into(),
                default: b"main".as_slice().into(),
                tree: Box::new([]),
                labels: Box::new([]),
                workflows: Box::new([]),
                protection: None,
                hooked: false,
                checks: raw::Checks {
                    cues: Box::new([]),
                    contexts: Box::new([]),
                    latency_min: Duration::ZERO,
                    latency_max: Duration::ZERO,
                    silent: 0,
                    passes: 1000,
                    reruns: 0,
                    cue: None,
                },
            },
        );
        forge::grant(&mut world.forge, b"org/second", 1, raw::Permission::Admin);
        world.event(top::Event::Application {
            role: None,
            carried: None,
            request: Token::new(102),
            token: Token::new(102),
            retention: Duration::from_secs(60),
            ask: top::Ask::Adopt(top::Adoption {
                project: 5,
                home: false,
                forge: REPO.forge,
                host: b"forge.example".as_slice().into(),
                owner: b"org".as_slice().into(),
                name: b"second".as_slice().into(),
                prefix: b"temper/".as_slice().into(),
                role: top::Role::Context,
                landing: b"main".as_slice().into(),
                ci: true,
                checks: Box::new([]),
            }),
        });
        world.run_for(2);
        assert!(
            world.top.repository(skein_forge::api::Repository { forge: 1, repository: 3 }).is_some(),
            "source adoption: {:?}",
            world.seen
        );
        assert!(world.project_landing(5, temper_engine_domain_forge_change::Way::Queue, Box::new([])));
        world
    }
    pub fn keep_follow(&mut self, number: u32, value: Option<top::settings::Follow>) {
        let edit = match value {
            Some(value) => top::settings::Edit::Set(top::settings::SizedValue {
                size: 128,
                value: top::settings::Value::Follow(value),
            }),
            None => top::settings::Edit::Remove,
        };
        self.top.keep_settings(
            &self.env,
            u64::from(number),
            None,
            &[top::settings::Asked { kind: 12, scope: 5, number, edit }],
            &mut self.top_out,
        );
        self.take_top();
    }
    #[must_use]
    pub fn follow_lag(&self, number: u32) -> top::FollowLag {
        self.top.follow_lag(5, number)
    }
    pub fn followed(&mut self, number: u32, afters: &[skein_forge::api::Commit]) {
        top::follows::followed(&mut self.top, 5, number, afters, &mut self.top_out);
        self.take_top();
    }
    pub fn allow_comparisons(&mut self) {
        self.read_script.comparisons_too_large = false;
    }
    pub fn pause_comparisons(&mut self) {
        self.held_compares = Some(Vec::new());
    }
    pub fn release_comparisons(&mut self) {
        for event in self.held_compares.take().expect("paused comparisons") {
            self.to_top.push(event);
        }
    }
    #[must_use]
    pub fn comparisons_held(&self) -> usize {
        self.held_compares.as_ref().map_or(0, Vec::len)
    }
    pub fn follow_read_limits(&mut self, rows: u32, commits: u32, files: u32, message: u32) {
        self.env.limits.client.rows = rows;
        self.env.limits.follow_commits = commits;
        self.env.limits.follow_files = files;
        self.env.limits.follow_message_bytes = message;
    }
    /// A complete source tree and message, committed by the independent git.
    pub fn push_message(&mut self, files: Box<[raw::File]>, message: &[u8]) -> skein_forge::api::Commit {
        let parent = self.branch(b"main");
        let commit = forge::commit_tree(&mut self.forge, &self.forge_env.limits, parent, None, files, message)
            .expect("follow source tree fits");
        self.external(
            raw::Op::Git(raw::Git::Push { branch: b"main".as_slice().into(), commit, expected: Some(parent) }),
            1,
        );
        translate::commit(commit)
    }
}
