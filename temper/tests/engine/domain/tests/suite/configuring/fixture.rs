//! Stock defaults with the world's scripted model and bounded store.
use super::*;
use serde_json::{Value, json};
use skein_lib::Duration;
use temper_engine_domain_world::{direct::Driver, protocol, store::Store, walking};

pub(super) const CHAT: u32 = 1;
pub(super) const CONFIGURING: u32 = 3;
pub(super) const REMOVED: u32 = 7;
pub(super) const PROJECT: u32 = 1;
pub(super) const CHANNEL: Token = Token::new(7);
pub(super) const SEED: u64 = 9761;
pub(super) const REASON: &str = "Explain running work\ntemper enforces: running work stays on its revision\nyour agents follow: push early, in charter 1";
// Stock has seven charters, fourteen grants and nine executor kinds. The
// stories need three simultaneous chats, one extra charter, and their replies.
const CHARTERS: u32 = 8;
const TASKS: u32 = 4;
const SLOTS: u32 = 3;
const GRANTS: u32 = 16;
const EXECUTORS: u32 = 10; // Nine stock kinds and the removable world's charter.
const RUN_BYTES: u32 = 16_384;
const TRANSCRIPT_BYTES: u32 = 32_768;
const MESSAGE_BYTES: u32 = 512;
// Account, three chats, a decision and up to three owner messages or edits.
const REQUESTS: u32 = 8;
// Four stock runs at this cheap model's 32-unit K, for three concurrent chats.
const CHAT_BUDGET: u64 = 128;

pub(super) fn config() -> engine::Config {
    let mut config = walking::config(SEED);
    let llm = jig_protocol_config::Llm {
        shown: Box::new([None]),
        pickers: Box::new([true]),
        endpoints: Box::new([]),
        models: walking::models(),
        descriptions: Box::new([None]),
        providers: Box::new([]),
    };
    config.core.defaults = temper_service::stock::defaults(&llm, limits().tasks.result_bytes);
    config.core.seed_projects[0].policy = config.core.defaults.policies.get(&1).expect("stock policy 1").clone();
    let project_owner = config.core.owners[0].clone();
    let mut deployment_owner = project_owner.clone();
    deployment_owner.project = people::DEPLOYMENT;
    config.core.owners = Box::new([deployment_owner, project_owner]);
    config.core.connectors = Box::new([1]);
    config.core.kinds = temper_service::stock::kinds(&limits(), 1);
    config.kinds_stock = temper_service::stock::forge(&limits());
    config
}

pub(super) fn limits() -> engine::Limits {
    let mut limits = walking::limits();
    limits.authority.grants = GRANTS;
    limits.authority.executors = EXECUTORS;
    limits.authority.roles = 4; // Owner, maintainer, member and observer.
    limits.documents.grants = GRANTS;
    limits.documents.executors = EXECUTORS;
    limits.notes.scopes = 32; // Three scalar scopes and the stock's fourteen grants.
    limits.tasks.charters = CHARTERS;
    limits.tasks.tasks = TASKS;
    limits.tasks.project_tasks = TASKS;
    limits.tasks.tree_tasks = TASKS;
    limits.tasks.authority_grants = GRANTS;
    limits.tasks.authority_segments = 8; // Stock forge patterns have one segment.
    limits.tasks.authority_bytes = 256; // Fourteen forge grants fit in 200 bytes.
    limits.tasks.executor_kinds = EXECUTORS;
    limits.tasks.message_bytes = MESSAGE_BYTES;
    limits.tasks.inbox_bytes = MESSAGE_BYTES * limits.tasks.inbox_messages;
    limits.people.initial_owners = 2; // Deployment and project holdings for the same owner.
    limits.people.amendment_bytes = 2048; // Stock role authority's request carrier.
    limits.people.words = MESSAGE_BYTES;
    limits.people.requests = REQUESTS;
    limits.people.inbox_tasks = TASKS * 2;
    limits.journal.result_bytes = MESSAGE_BYTES;
    limits.fleet.slots = SLOTS;
    limits.fleet.attempts = SLOTS;
    limits.journal.run_bytes = RUN_BYTES;
    limits.journal.transcript_bytes = TRANSCRIPT_BYTES;
    // Two four-role policy settings share a decoded page.
    limits.loads.bytes = 65_536;
    limits.loads.transcript_bytes = TRANSCRIPT_BYTES;
    limits.loads.reply_bytes = 65_536;
    limits.journal.writes = temper_engine_domain_world::journal_limits::TREE_WRITES;
    limits
}

/// Two projects retain distinct repositories; only the caller's is discoverable.
pub(super) fn repository_projects() -> World {
    let mut config = config();
    let other = core::SeedProject {
        number: 2,
        name: "Other project".into(),
        policy: config.core.seed_projects[0].policy.clone(),
    };
    let mut projects = config.core.seed_projects.into_vec();
    projects.push(other);
    config.core.seed_projects = projects.into_boxed_slice();
    let mut owners = config.core.owners.into_vec();
    let mut owner = owners[1].clone();
    owner.project = 2;
    owners.push(owner);
    config.core.owners = owners.into_boxed_slice();
    let mut bounds = limits();
    bounds.authority.segments = 8; // A charter document names a six-segment forge branch.
    bounds.authority.projects = 2;
    bounds.people.projects = 3; // Two projects and the deployment roster.
    bounds.people.initial_owners = 3;
    bounds.people.holdings = 4; // The owner holds deployment and both projects.
    World::with_limits(config, &bounds)
}

pub(super) fn member_config() -> engine::Config {
    config()
}

pub(super) fn moving_config() -> engine::Config {
    let mut config = config();
    let mut charter = config
        .core
        .defaults
        .charters
        .iter()
        .find(|charter| charter.number == CHAT)
        .expect("stock chat charter")
        .clone();
    charter.number = REMOVED;
    charter.policy.instructions = b"Keep this charter's instructions.".as_slice().into();
    let mut charters = config.core.defaults.charters.into_vec();
    charters.push(charter);
    config.core.defaults.charters = charters.into_boxed_slice();
    let mut template = config
        .core
        .defaults
        .templates
        .iter()
        .find(|template| template.number == CHAT)
        .expect("stock chat template")
        .clone();
    template.number = REMOVED;
    template.name = b"removable".as_slice().into();
    template.executor = people::TemplateExecutor::Charter(REMOVED);
    let mut templates = config.core.defaults.templates.into_vec();
    templates.push(template);
    config.core.defaults.templates = templates.into_boxed_slice();
    offer_removable(&mut config.core.defaults.rules.ceiling);
    let mut policy = config.core.seed_projects[0].policy.clone();
    offer_removable(&mut policy.policy.ceiling);
    for role in &mut policy.policy.roles {
        offer_removable(&mut role.authority);
    }
    config.core.seed_projects[0].policy = policy.clone();
    config.core.defaults.policies.insert(1, policy).expect("world's policy 1");
    config
}
fn offer_removable(authority: &mut jig_domain_authority::Authority) {
    if authority.delegation.tasks == 0 {
        return;
    }
    let mut kinds = authority.delegation.kinds.clone().into_vec();
    kinds.push(jig_domain_authority::Executor::Charter(REMOVED));
    authority.delegation.kinds = kinds.into_boxed_slice();
}

pub(super) struct World {
    pub driver: Driver,
    pub owner: u64,
    templates: Box<[core::Template]>,
}
impl World {
    pub fn stock() -> Self {
        Self::new(config())
    }
    pub fn new(config: engine::Config) -> Self {
        Self::with_limits(config, &limits())
    }
    /// Stock setup with story-specific entrance bounds.
    pub fn with_limits(config: engine::Config, limits: &engine::Limits) -> Self {
        let identity = config.core.owners[0].identity.clone();
        let templates = config.core.defaults.templates.clone();
        let period_budget = config.core.defaults.rules.period_spend;
        let mut driver = Driver::cold(Store::with_limits(limits), config, limits);
        driver.period_sent = true;
        driver.settle();
        driver.send(Event::Party(PartyInput::SignedIn {
            held: None,
            reply_to: ReplyTo::new(Token::new(101)),
            identity: people::Identity {
                key: identity,
                login: b"person".as_slice().into(),
                name: b"Person".as_slice().into(),
            },
        }));
        driver.settle();
        let owner = driver.session();
        // A fake key envelope exercises account admission without a cold replay
        // of charter versions: every story begins at the first-start revision.
        driver.send(Event::Party(PartyInput::Key {
            reply_to: ReplyTo::new(Token::new(10)),
            sign_in: owner,
            key: [10; 16],
            ask: Box::new(people::Ask::AccountChange {
                holder: people::AccountHolder::Deployment,
                change: people::AccountChange::AddKey { provider: 1, join: None, digest: [10; 32] },
            }),
            carried: core::Carried { number: 1_u64 << 63, digest: [10; 32] },
        }));
        driver.settle();
        let mut greeting = protocol::hello_slots(CHANNEL, SLOTS, Box::new([]));
        let Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello, .. }, .. }) = &mut greeting
        else {
            panic!("hello")
        };
        hello.bounds.time = Duration::from_secs(28_800); // Stock work charter run time.
        hello.bounds.turns = 800; // Stock's largest charter.
        hello.bounds.message_bytes = MESSAGE_BYTES;
        driver.send(greeting);
        driver.settle();
        driver.send(Event::Party(PartyInput::Period {
            project: PROJECT,
            period: 1,
            budget: period_budget,
            next: None,
        }));
        driver.settle();
        Self { driver, owner, templates }
    }
    pub fn ask(&mut self, session: u64, key: u8, ask: people::Ask) {
        self.driver.send(Event::Party(PartyInput::Ask {
            models: Box::new([]),
            reply_to: ReplyTo::new(Token::new(u64::from(key))),
            sign_in: session,
            key: [key; 16],
            ask,
        }));
        self.driver.settle();
    }
    pub fn reply(&self, key: u8) -> &people::Reply {
        self.driver
            .delivered
            .iter()
            .rev()
            .find_map(|delivery| {
                if let Delivery::WebReply { to, reply } = delivery
                    && *to == ReplyTo::new(Token::new(u64::from(key)))
                {
                    Some(reply)
                } else {
                    None
                }
            })
            .expect("party's answer")
    }
    pub fn member(&mut self) -> u64 {
        self.driver.send(Event::Party(PartyInput::SignedIn {
            held: None,
            reply_to: ReplyTo::new(Token::new(11)),
            identity: people::Identity {
                key: people::IdentityKey { provider: 0, subject: 8_u64.to_be_bytes().into() },
                login: b"member".as_slice().into(),
                name: b"Member".as_slice().into(),
            },
        }));
        self.driver.settle();
        let people::Reply::SignedIn { person: member, sign_in, .. } = *self.reply(11) else {
            panic!("member signed in")
        };
        let owner = self
            .driver
            .delivered
            .iter()
            .find_map(|delivery| {
                if let Delivery::WebReply { reply: people::Reply::SignedIn { person, sign_in, .. }, .. } = delivery
                    && *sign_in == self.owner
                {
                    Some(*person)
                } else {
                    None
                }
            })
            .expect("owner person");
        // The world's policy gives members the stock request authority. The
        // member holds no role in the deployment and cannot decide proposals.
        self.ask(
            self.owner,
            12,
            people::Ask::SetRoles {
                project: PROJECT,
                holdings: Box::new([
                    people::Holding { person: owner, role: people::Role::Owner },
                    people::Holding { person: member, role: people::Role::Member },
                ]),
            },
        );
        assert_eq!(self.reply(12), &people::Reply::Outcome(people::Outcome::RolesSet { project: PROJECT }));
        sign_in
    }
    pub fn chat(&mut self, template: u32, session: u64, key: u8) -> engine::Assignment {
        self.chat_words(template, session, key, b"Explain how our work runs")
    }
    /// Start a stock chat with the person's opening words.
    pub fn chat_words(&mut self, template: u32, session: u64, key: u8, words: &[u8]) -> engine::Assignment {
        let template = self.templates.iter().find(|value| value.number == template).expect("stock template");
        let mut ask = self.driver.chat_ask(PROJECT, words);
        let people::Ask::Create { batch, .. } = &mut ask else { panic!("chat") };
        let people::TemplateExecutor::Charter(charter) = template.executor else { panic!("chat charter") };
        let tasks::Contract::Report { words } = template.contract else { panic!("chat report") };
        batch[0].executor = people::TaskExecutor::Agent { charter };
        batch[0].contract = people::Contract::Report { words };
        batch[0].flags = people::Flags {
            ended_by: people::EndedBy::Requester,
            conversation: template.flags.conversation,
            listed: template.flags.listed,
            retention: people::Retention::Kept,
        };
        batch[0].label = template.label;
        batch[0].llm = None;
        batch[0].budget = people::Budget::AsAllowed { cap: Some(CHAT_BUDGET) };
        batch[0].wake = None;
        self.ask(session, key, ask);
        self.driver
            .delivered
            .iter()
            .rev()
            .find_map(|delivery| {
                if let Delivery::Assigned { assignment, .. } = delivery { Some(assignment.clone()) } else { None }
            })
            .unwrap_or_else(|| {
                panic!("chat not assigned: {:?}; task: {:?}", self.driver.delivered, self.driver.task(1))
            })
    }
    pub fn head(&self) -> &s::Head {
        self.driver
            .store
            .rows
            .values()
            .filter_map(|row| {
                if let engine::Record::Core(core::Record::Core(core::CoreRecord::Settings(s::Record::Revision {
                    history: false,
                    revision,
                }))) = row
                {
                    Some(&revision.head)
                } else {
                    None
                }
            })
            .max_by_key(|head| head.number)
            .expect("newest revision")
    }
    pub fn settings_rows(&self) -> Vec<s::Record> {
        self.driver
            .store
            .rows
            .values()
            .filter_map(|row| {
                if let engine::Record::Core(core::Record::Core(core::CoreRecord::Settings(record))) = row {
                    Some(record.clone())
                } else {
                    None
                }
            })
            .collect()
    }
    pub fn change(&mut self, against: u32, key: u8, items: Box<[s::Item]>) {
        self.driver.send(Event::Party(PartyInput::ChangeSettings {
            kinds: Box::new([]),
            reply_to: ReplyTo::new(Token::new(u64::from(key))),
            sign_in: self.owner,
            key: [key; 16],
            digest: [key; 32],
            change: Box::new(s::Change { against, items, running: s::Running::Stays }),
        }));
        self.driver.settle();
        assert_eq!(self.head().number, against + 1, "owner's own change accepted");
    }
    pub fn review(&mut self, run: &engine::Assignment, proposal: u64) -> core::documents::Proposal {
        self.driver.send(Event::Party(PartyInput::Read {
            reply_to: ReplyTo::new(Token::new(90)),
            sign_in: self.owner,
            read: core::reads::Read::Proposal { proposer: people::Proposer::Task(run.task), proposal },
        }));
        self.driver.settle();
        self.driver
            .delivered
            .iter()
            .rev()
            .find_map(|delivery| {
                if let Delivery::Read { answer: core::reads::Answer::Document(document), .. } = delivery
                    && let core::documents::Document::Proposal(proposal) = document.as_ref()
                {
                    Some(*proposal.clone())
                } else {
                    None
                }
            })
            .expect("owner's proposal review")
    }
    pub fn decide(
        &mut self,
        run: &engine::Assignment,
        proposal: u64,
        key: u8,
        decision: people::ProposalDecision,
        running: people::Running,
    ) {
        let running = if matches!(decision, people::ProposalDecision::Accept) { Some(running) } else { None };
        self.ask(
            self.owner,
            key,
            people::Ask::DecideProposal {
                project: people::DEPLOYMENT,
                proposer: people::Proposer::Task(run.task),
                proposal,
                decision,
                running,
            },
        );
        let answer = self.reply(key);
        assert!(matches!(answer, people::Reply::Outcome(people::Outcome::ProposalDecided { .. })), "{answer:?}");
    }
    pub fn next(&mut self, run: &engine::Assignment) -> engine::Assignment {
        self.driver.answer(CHANNEL, run.task, run.attempt, 0, tasks::End::Parked, engine::Parts::none());
        self.driver.settle();
        let next = self.driver.last_assignment();
        assert_eq!(next.task, run.task);
        assert_ne!(next.attempt, run.attempt);
        next
    }
    pub fn park(&mut self, run: &engine::Assignment) {
        self.driver.answer(CHANNEL, run.task, run.attempt, 0, tasks::End::Parked, engine::Parts::none());
        self.driver.settle();
        assert_eq!(self.driver.task(run.task).expect("parked task").settings.run, None);
    }
    pub fn wake(&mut self, run: &engine::Assignment, key: u8) -> engine::Assignment {
        self.ask(
            self.owner,
            key,
            people::Ask::Say { project: PROJECT, task: run.task, words: b"Continue the work.".as_slice().into() },
        );
        let next = self.driver.last_assignment();
        assert_eq!(next.task, run.task);
        assert_ne!(next.attempt, run.attempt);
        next
    }
}

pub(super) fn read_charter(driver: &mut Driver, run: &engine::Assignment, completion: u32, number: u32) -> Value {
    let input = json!({"setting":{"kind":"charter","number":number}});
    read_value(driver, run, completion, input)
}
pub(super) fn read_value(driver: &mut Driver, run: &engine::Assignment, completion: u32, input: Value) -> Value {
    let (text, _) = read_whole(driver, run, completion, input);
    let (_, text) = text.split_once('\n').expect("setting heading");
    serde_json::from_str(text.trim()).expect("setting JSON")
}
pub(super) fn omit_removable(authority: &mut Value) {
    let kinds = authority["delegation"]["kinds"].as_array_mut().expect("delegation kinds");
    if kinds.is_empty() {
        return;
    }
    let before = kinds.len();
    kinds.retain(|kind| kind != &json!({"charter":REMOVED}));
    assert_eq!(kinds.len() + 1, before, "one charter reference removed");
}
pub(super) fn read_whole(
    driver: &mut Driver,
    run: &engine::Assignment,
    completion: u32,
    mut input: Value,
) -> (String, u32) {
    let mut from = 0;
    let mut text = Vec::new();
    let mut pages = 0;
    loop {
        input["from"] = json!(from);
        let (body, error) = host_tool(driver, run, completion + pages, b"settings", input.to_string().as_bytes());
        assert!(!error, "{input}: {}", String::from_utf8_lossy(&body));
        assert!(
            body.len() <= usize::try_from(smith_agent_world::LIMITS.run.host_reply_bytes).expect("host reply bound")
        );
        let page = std::str::from_utf8(&body).expect("settings UTF-8");
        pages += 1;
        if let Some((body, next)) = page.rsplit_once("next: ") {
            let next: u32 = next.trim().parse().expect("continuation offset");
            assert!(next > from, "pagination advances");
            let bytes = usize::try_from(next - from).expect("page size");
            text.extend_from_slice(body.as_bytes().get(..bytes).expect("original page bytes"));
            from = next;
        } else {
            text.extend_from_slice(page.strip_suffix("end\n").expect("terminal page").as_bytes());
            return (String::from_utf8(text).expect("whole settings UTF-8"), pages);
        }
    }
}

pub(super) fn charter_item(mut value: Value, instructions: &[u8]) -> Value {
    value["instructions"] = Value::String(std::str::from_utf8(instructions).expect("instructions").into());
    json!([{"setting":{"kind":"charter","number":CHAT},"set":value}])
}
pub(super) fn owner_charter(instructions: &[u8]) -> s::Item {
    let mut charter = config()
        .core
        .defaults
        .charters
        .into_vec()
        .into_iter()
        .find(|value| value.number == CHAT)
        .expect("stock chat charter");
    charter.policy.instructions = instructions.into();
    s::Item { setting: s::Setting::Charter(CHAT), edit: s::Edit::Set(s::Value::Charter(Box::new(charter))) }
}
pub(super) fn propose(
    driver: &mut Driver,
    run: &engine::Assignment,
    completion: u32,
    against: u32,
    items: &Value,
) -> u64 {
    let input = json!({"against":against,"reason":REASON,"items":items});
    let (body, error) = host_tool(driver, run, completion, b"propose_settings", input.to_string().as_bytes());
    assert!(!error, "{}", String::from_utf8_lossy(&body));
    std::str::from_utf8(&body)
        .expect("proposal answer")
        .strip_prefix(&format!("proposed as t{}/", run.task))
        .expect("proposal")
        .strip_suffix(" to the deployment's owners")
        .expect("deployment owners")
        .parse()
        .expect("proposal number")
}
pub(super) fn assert_heard(run: &engine::Assignment, expected: &[u8]) {
    let smith_domain::Event::Start(smith_domain::Start { messages, .. }) = temper_engine_smith::start(
        run,
        None,
        None,
        ReplyTo::new(Token::new(99)),
        Token::new(run.task),
        smith_domain::Window { turns: 1, bytes: 100_000 },
        &smith_agent_world::LIMITS.run,
    ) else {
        panic!("smith start")
    };
    assert!(messages.iter().any(|message| message.text.as_ref() == expected), "{messages:?}");
}
