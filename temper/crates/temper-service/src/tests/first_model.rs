//! Init's model order and stock charters on one-account boxes
//! (jig/docs/design/domain/accounts.md, 4.3; temper/docs/design/domain/testing.md, 2).
#![expect(
    clippy::disallowed_methods,
    reason = "ordinary domain stories inspect generated settings and typed documents"
)]
use super::{fixtures, stock};
use jig_domain_core as core;
use jig_domain_host as host;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Duration, List, ReplyTo, Token};
use temper_engine_domain::{self as engine, Delivery, Event, PartyInput};
use temper_engine_domain_world::{direct::Driver, protocol, store::Store};

const OPENAI: &[u8] = b"gpt-6.1-sol";
const ASTRA: &[u8] = b"gpt-6-astra";
const CLAUDE: &[u8] = b"claude-opus-5-5";
const CHANNEL: Token = Token::new(7);
const LOGIN: &[u8] = b"personal";

enum OneAccount {
    Claude,
    ChatGpt,
    None,
    Declared,
}

struct StockBox {
    driver: Driver,
    config: engine::Config,
    models: Box<[core::Model]>,
    templates: Box<[core::Template]>,
    providers: jig_domain_accounts::Providers,
    login: Option<host::LoginState>,
}

fn stock_box(account: OneAccount) -> StockBox {
    let document = stock::generated();
    assert_eq!(
        fixtures::value(&document, b"engine.llm.models[0].name").as_ref(),
        br#""claude-opus-5-5""#,
        "init must put claude-opus-5-5 first: a reordered fixture hides first-usable selection"
    );
    let parsed = stock::engine(&document);
    let application_bytes = fixtures::value(&document, b"application");
    let application_document = jig_protocol_config::document(
        &application_bytes,
        &jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 },
    )
    .unwrap();
    let mut problems = crate::startup::Problems::empty();
    let application = crate::startup::application::parse(
        &application_document,
        0,
        &parsed,
        &jig_protocol_config::Directories::working(),
        &mut problems,
    )
    .expect("init application");
    assert!(problems.is_empty());
    let mut limits = crate::limits::engine_candidate(&parsed.limits, &application.limits);
    limits.memory_bytes = parsed.memory;
    let mut core = jig_protocol_config::core(&parsed, [31; 16], 91);
    core.kinds = crate::stock::kinds(&limits, 1);
    let config = engine::Config {
        host_part: None,
        period_span: crate::stock::PERIOD_SPAN,
        core,
        landings: application.landings,
        kinds_stock: crate::stock::forge(&limits),
        forge: skein_forge_client::Config { namespace: Box::new([31; 16]), writers: Box::new([]) },
        identities: Box::new([]),
        machine: None,
        forges: Box::new([]),
        under_box: false,
        playground: None,
    };
    let models = config.core.settings.models.clone();
    let templates = config.core.defaults.templates.clone();
    let providers = config.core.settings.llm_providers.clone();
    let restart = temper_engine_domain_world::oracle::config::copy(&config);
    let mut driver = Driver::cold(Store::with_limits(&limits), config, &limits);
    driver.period_sent = true;
    driver.settle();
    let login = match account {
        OneAccount::Claude => Some(host::LoginState::Usable),
        OneAccount::Declared => Some(host::LoginState::NotSignedIn),
        OneAccount::ChatGpt | OneAccount::None => None,
    };
    let mut world = StockBox { driver, config: restart, models, templates, providers, login };
    world.sign_in();
    world.ask(1, people::Ask::MakeProject { name: "Work".into(), policy: 1 });
    match account {
        OneAccount::Claude | OneAccount::Declared => {
            let provider = world.provider(CLAUDE);
            world.ask(
                2,
                people::Ask::AccountChange {
                    holder: people::AccountHolder::Own,
                    change: people::AccountChange::DeclareProgram { provider, join: None, name: LOGIN.into() },
                },
            );
            let declared = world
                .driver
                .delivered
                .iter()
                .find_map(|delivery| {
                    if let Delivery::WebReply {
                        reply: people::Reply::Outcome(people::Outcome::AccountProgramDeclared { name, .. }),
                        ..
                    } = delivery
                    {
                        Some(name.as_ref())
                    } else {
                        None
                    }
                })
                .expect("program name is durably issued");
            assert_eq!(declared, LOGIN);
        }
        OneAccount::ChatGpt => world.add_openai(),
        OneAccount::None => {}
    }
    // The document is read before a worker can claim. The declared source already
    // selects Claude even when no host has reported its login usable yet.
    world
}

impl StockBox {
    fn sign_in(&mut self) {
        self.driver.send(Event::Party(PartyInput::SignedIn {
            held: None,
            reply_to: ReplyTo::new(Token::new(900)),
            identity: people::Identity {
                key: self.config.core.owners[0].identity.clone(),
                login: b"owner".as_slice().into(),
                name: b"Owner".as_slice().into(),
            },
        }));
        self.pump();
    }

    /// The shared world's worker reports no logins. Replace each queued whole
    /// inventory with this box's declaration before the root receives it.
    /// Waiting account tasks stay due, so drain inputs rather than requiring quiescence.
    fn pump(&mut self) {
        for _ in 0_u32..200 {
            for event in &mut self.driver.events {
                if let Event::Worker(engine::WorkerInput::Link {
                    link: engine::Link::Up { up: host::LinkUp::Logins { logins }, .. },
                    ..
                }) = event
                {
                    *logins = match &self.login {
                        Some(state) => Box::new([host::Login {
                            name: host::SourceName::new(LOGIN).unwrap(),
                            state: state.clone(),
                        }]),
                        None => Box::new([]),
                    };
                }
            }
            self.driver.advance(true);
        }
        assert!(
            self.driver.events.is_empty() && self.driver.store.pending.is_empty(),
            "box inputs and commits drained"
        );
    }

    fn provider(&self, name: &[u8]) -> u32 {
        self.providers
            .providers
            .iter()
            .find(|provider| provider.models.iter().any(|model| model.as_ref() == name))
            .expect("init model belongs to a provider")
            .number
    }

    fn ask(&mut self, key: u8, ask: people::Ask) {
        self.driver.send(Event::Party(PartyInput::Ask {
            models: Box::new([]),
            reply_to: ReplyTo::new(Token::new(u64::from(key))),
            sign_in: self.driver.session(),
            key: [key; 16],
            ask,
        }));
        self.pump();
        let reply = self
            .driver
            .delivered
            .iter()
            .rev()
            .find_map(|delivery| {
                if let Delivery::WebReply { to, reply } = delivery {
                    (*to == ReplyTo::new(Token::new(u64::from(key)))).then_some(reply)
                } else {
                    None
                }
            })
            .expect("party reply");
        let people::Reply::Outcome(_) = reply else { panic!("ask {key}: {reply:?}") };
    }

    fn add_openai(&mut self) {
        let digest = [10; 32];
        self.driver.send(Event::Party(PartyInput::Key {
            reply_to: ReplyTo::new(Token::new(10)),
            sign_in: self.driver.session(),
            key: [10; 16],
            ask: Box::new(people::Ask::AccountChange {
                holder: people::AccountHolder::Own,
                change: people::AccountChange::AddKey { provider: self.provider(OPENAI), join: None, digest },
            }),
            carried: core::Carried { number: 1_u64 << 63, digest },
        }));
        self.pump();
        self.driver
            .delivered
            .iter()
            .find_map(|delivery| {
                if let Delivery::WebReply {
                    reply: people::Reply::Outcome(people::Outcome::AccountKeyKept { source }),
                    ..
                } = delivery
                {
                    Some(*source)
                } else {
                    None
                }
            })
            .expect("the init OpenAI provider accepts a synthetic key");
    }

    fn create(&mut self, template: u32, named: Option<&[u8]>) -> u64 {
        let template = self.templates.iter().find(|value| value.number == template).expect("stock template");
        let mut ask = fixtures::chat(1, b"Do the work");
        let people::Ask::Create { batch, .. } = &mut ask else { panic!("create") };
        let member = batch.first_mut().expect("one member");
        member.executor = people::TaskExecutor::Agent {
            charter: match template.executor {
                people::TemplateExecutor::Charter(charter) => charter,
                people::TemplateExecutor::Chosen => 2,
            },
        };
        let tasks::Contract::Report { words } = template.contract else { panic!("report template") };
        member.contract = people::Contract::Report { words };
        member.flags = people::Flags {
            ended_by: match template.flags.ended_by {
                tasks::EndedBy::Requester => people::EndedBy::Requester,
                tasks::EndedBy::Result => people::EndedBy::Result,
            },
            conversation: template.flags.conversation,
            listed: template.flags.listed,
            retention: people::Retention::Kept,
        };
        member.authority = self
            .driver
            .store
            .rows
            .values()
            .find_map(|row| {
                if let engine::Record::Core(core::Record::People(people::Stored::Policy {
                    project: 1, value, ..
                })) = row
                {
                    Some(value.roles.iter().find(|role| role.number == 0).expect("stock owner role").authority.clone())
                } else {
                    None
                }
            })
            .expect("created project's policy");
        member.authority.delegation.tasks =
            member.authority.delegation.tasks.min(self.driver.env.limits.tasks.tree_tasks).saturating_sub(1);
        member.authority.delegation.depth = member.authority.delegation.depth.saturating_sub(1);
        if template.number != 2 {
            member.authority.spend = 0;
        }
        member.label = template.label;
        member.wake = None;
        member.llm = named.map(|name| people::LlmChoice {
            account: None,
            main: people::ModelEffort {
                model: u8::try_from(
                    self.models.iter().position(|model| model.name.as_ref() == name).expect("named model"),
                )
                .unwrap(),
                effort: self
                    .models
                    .iter()
                    .find(|model| model.name.as_ref() == name)
                    .filter(|model| !model.efforts.is_empty())
                    .map(|_| 0),
            },
            subagents: Box::new([]),
        });
        if template.number == 2 {
            member.budget = people::Budget::Named;
            member.tracked = Some(1);
        }
        self.ask(20, ask);
        self.driver
            .store
            .rows
            .values()
            .filter_map(|row| {
                if let engine::Record::Core(core::Record::Tasks(tasks::Stored::Live(row))) = row {
                    Some(row.number)
                } else {
                    None
                }
            })
            .max()
            .expect("created task")
    }

    fn document(&mut self, task: u64) -> Box<core::documents::Task> {
        self.driver.send(Event::Party(PartyInput::Read {
            reply_to: ReplyTo::new(Token::new(800)),
            sign_in: self.driver.session(),
            read: core::reads::Read::Task { task },
        }));
        self.pump();
        self.driver
            .delivered
            .iter()
            .rev()
            .find_map(|delivery| {
                if let Delivery::Read { answer: core::reads::Answer::Document(document), .. } = delivery
                    && let core::documents::Document::Task(task) = document.as_ref()
                {
                    Some(task.clone())
                } else {
                    None
                }
            })
            .expect("Task document")
    }

    fn main(&self, document: &core::documents::Task) -> &[u8] {
        let main = document.llm.as_ref().expect("agent LLM in document").main;
        &self.models.get(usize::from(main.model)).expect("document model index").name
    }

    fn worker(&mut self) {
        let mut greeting = protocol::hello_at(CHANNEL, 0, 1, Box::new([]));
        let Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello, .. }, .. }) = &mut greeting
        else {
            panic!("hello")
        };
        let mut endpoints = List::with_capacity(host::PROGRAM_ENDPOINTS);
        for model in &self.models {
            if model.program.is_some() && !endpoints.as_slice().contains(&model.endpoint) {
                endpoints.push(model.endpoint).expect("configured program endpoints");
            }
        }
        hello.bounds = host::Bounds {
            program_endpoints: endpoints.into_boxed(),
            models: self
                .models
                .iter()
                .map(|model| host::ModelBound { name: model.name.clone(), window: model.window, output: model.output })
                .collect(),
            turns: 800,
            spend: u64::MAX,
            time: Duration::from_secs(28_800),
            waiting: Duration::from_secs(60),
            checks: Duration::from_secs(1),
            message_bytes: self.driver.env.limits.tasks.message_bytes,
            inbox: self.driver.env.limits.tasks.inbox_messages,
            unacknowledged: self.driver.env.limits.fleet.turns,
        };
        self.driver.send(greeting);
        self.pump();
    }

    fn assert_claim(&mut self, task: u64, model: &[u8]) -> engine::Assignment {
        assert_eq!(self.document(task).run_wait, None, "a connected worker can run the task");
        let assignment = self.driver.last_assignment();
        assert_eq!(assignment.task, task);
        assert_eq!(assignment.run.policy.model.name.as_ref(), model, "claim's main");
        assert_eq!(assignment.run.policy.model.default_effort().unwrap(), b"high");
        // Complete this activation before the world's cold-restart oracle.
        let end = match assignment.run.ended_by {
            tasks::EndedBy::Requester => tasks::End::Parked,
            tasks::EndedBy::Result => tasks::End::Finished {
                result: tasks::TaskResult::Report { words: b"Done".as_slice().into() },
                cancel_delegates: false,
            },
        };
        let mut event = self.driver.protocol.answer(CHANNEL, task, assignment.attempt, 0, end, engine::Parts::none());
        let Event::Worker(engine::WorkerInput::Link {
            link: engine::Link::Up { up: host::LinkUp::Answer { answer, .. }, .. },
            ..
        }) = &mut event
        else {
            panic!("run answer")
        };
        answer.read = assignment.messages.last().map(|word| Token::new(word.number));
        self.driver.send(event);
        self.pump();
        assignment
    }
}

#[test]
fn claude_alone_runs_the_stock_chat() {
    let mut world = stock_box(OneAccount::Claude);
    let task = world.create(1, None);
    let document = world.document(task);
    assert_eq!(world.main(&document), CLAUDE, "Task before claim");
    world.worker();
    world.assert_claim(task, CLAUDE);
}

#[test]
fn claude_alone_runs_the_stock_configuring_chat() {
    let mut world = stock_box(OneAccount::Claude);
    let task = world.create(3, None);
    let document = world.document(task);
    assert_eq!(world.main(&document), CLAUDE, "template 3 Task before claim");
    world.worker();
    let assignment = world.assert_claim(task, CLAUDE);
    assert_eq!(assignment.charter, 3);
}

#[test]
fn claude_alone_runs_the_goals_coding_task() {
    let mut world = stock_box(OneAccount::Claude);
    let task = world.create(2, None);
    let document = world.document(task);
    assert_eq!(world.main(&document), CLAUDE, "goal Task before claim");
    assert_eq!(document.goal, Some(task));
    world.worker();
    let assignment = world.assert_claim(task, CLAUDE);
    assert_eq!(assignment.charter, 2);
}

#[test]
fn chatgpt_alone_runs_the_stock_chat() {
    let mut world = stock_box(OneAccount::ChatGpt);
    let task = world.create(1, None);
    let document = world.document(task);
    assert_eq!(world.main(&document), OPENAI, "Task before claim");
    world.worker();
    world.assert_claim(task, OPENAI);
}

#[test]
fn chatgpt_alone_runs_the_goals_coding_task() {
    let mut world = stock_box(OneAccount::ChatGpt);
    let task = world.create(2, None);
    let document = world.document(task);
    assert_eq!(world.main(&document), OPENAI, "goal Task before claim");
    assert_eq!(document.goal, Some(task));
    world.worker();
    world.assert_claim(task, OPENAI);
}

#[test]
fn a_named_openai_model_waits_on_the_claude_box() {
    let mut world = stock_box(OneAccount::Claude);
    let task = world.create(1, Some(OPENAI));
    world.worker();
    let document = world.document(task);
    assert_eq!(world.main(&document), OPENAI);
    assert_eq!(
        document.run_wait,
        Some(core::documents::TaskRunWait::Account {
            account: None,
            source: None,
            why: core::documents::TaskWaitWhy::Empty,
        })
    );
    assert!(world.driver.task(task).unwrap().kept_account.is_none());
}

#[test]
fn adding_chatgpt_keeps_the_chats_claude_account() {
    let mut world = stock_box(OneAccount::Claude);
    let task = world.create(1, None);
    world.worker();
    let first = world.assert_claim(task, CLAUDE);
    let account = world.driver.task(task).unwrap().kept_account.expect("claimed Claude account");
    world.add_openai();
    let document = world.document(task);
    assert_eq!(world.main(&document), CLAUDE);
    assert_eq!(document.kept_account, Some(account));
    world.driver.say(task, 21);
    let next = world.assert_claim(task, CLAUDE);
    assert_ne!(first.attempt, next.attempt);
    assert_eq!(world.driver.task(task).unwrap().kept_account, Some(account));
    assert_eq!(next.run.accounts.first().map(|binding| binding.account), Some(account));
}

#[test]
fn a_declared_login_without_sign_in_names_its_host_wait() {
    let mut world = stock_box(OneAccount::Declared);
    let task = world.create(1, None);
    world.worker();
    let document = world.document(task);
    assert_eq!(world.main(&document), CLAUDE);
    let Some(core::documents::TaskRunWait::Account {
        account: Some(_),
        source: Some(_),
        why: core::documents::TaskWaitWhy::Host,
    }) = document.run_wait
    else {
        panic!("declared login waits for its source on a host: {:?}", document.run_wait)
    };
}

#[test]
fn no_account_keeps_the_charters_first_model_and_names_an_empty_wait() {
    let mut world = stock_box(OneAccount::None);
    let task = world.create(1, None);
    world.worker();
    let document = world.document(task);
    assert_eq!(world.main(&document), CLAUDE);
    assert_eq!(
        document.run_wait,
        Some(core::documents::TaskRunWait::Account {
            account: None,
            source: None,
            why: core::documents::TaskWaitWhy::Empty,
        })
    );
}

#[test]
fn chatgpt_alone_runs_the_stock_configuring_chat() {
    let mut world = stock_box(OneAccount::ChatGpt);
    let task = world.create(3, None);
    let document = world.document(task);
    assert_eq!(world.main(&document), ASTRA, "configuring Task before claim");
    world.worker();
    world.assert_claim(task, ASTRA);
}

#[test]
fn a_stored_charter_keeps_its_value_across_stock_changes() {
    use core::settings::{Changed, Record, Setting, Value};
    let mut world = stock_box(OneAccount::Claude);
    let mut stored = world.config.core.defaults.charters[0].clone();
    stored.policy.instructions = b"The stored charter's instructions.".as_slice().into();
    world.driver.store.seed(
        Box::new([engine::Record::Core(core::Record::Core(core::CoreRecord::Settings(Record::Changed {
            setting: Setting::Charter(1),
            changed: Changed { revision: 1, value: Some(Value::Charter(Box::new(stored.clone()))) },
        })))]),
        Box::new([]),
    );
    let mut changed = temper_engine_domain_world::oracle::config::copy(&world.config);
    changed.core.defaults.charters[0].policy.instructions = b"The new stock's instructions.".as_slice().into();
    changed.core.defaults.charters[1].policy.instructions = b"The new coding stock's instructions.".as_slice().into();
    world.driver.reopen(changed);
    world.driver.settle();
    for row in world.driver.store.rows.values() {
        if let engine::Record::Core(core::Record::Core(core::CoreRecord::Settings(Record::Revision {
            revision, ..
        }))) = row
        {
            assert!(
                !revision.items.iter().any(|item| item.setting == Setting::Charter(1)),
                "a stock release skips the stored charter"
            );
        }
    }
    world.sign_in();
    let task = world.create(1, None);
    world.worker();
    let assignment = world.assert_claim(task, CLAUDE);
    assert_eq!(assignment.run.policy.instructions, stored.policy.instructions);
}
