//! Init's offers, template previews and runs for the party's usable accounts
//! (jig/docs/design/client/domain/contract/documents.md, 1;
//! temper/docs/design/protocol/configuration/stock.md; temper/docs/design/testing/README.md, 2).
#![expect(
    clippy::disallowed_methods,
    reason = "ordinary domain stories inspect generated settings and typed documents"
)]
use super::{fixtures, stock};
use jig_client_protocol_wire::v21 as wire;
use jig_domain_accounts as accounts;
use jig_domain_core as core;
use jig_domain_host as host;
use jig_domain_link::{Bounds, EndpointBound, Login, LoginState, PROGRAM_ENDPOINTS, SourceName};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Duration, List, Reader, ReplyTo, Token, Writer};
use temper_engine_domain::{self as engine, Delivery, Event, PartyInput};
use temper_engine_domain_world::{direct::Driver, protocol, store::Store};

const OPENAI: &[u8] = b"gpt-6.1-sol";
const ASTRA: &[u8] = b"gpt-6-astra";
const CLAUDE: &[u8] = b"claude-opus-5-5";
const CHANNEL: Token = Token::new(7);
const LOGIN: &[u8] = b"personal";
const ME_WATCH: Token = Token::new(802);

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
    charters: Box<[core::Charter]>,
    providers: jig_domain_accounts::Providers,
    client_models: Box<[jig_protocol_engine_clients::Model]>,
    client_limits: jig_protocol_engine_clients::Limits,
    login: Option<LoginState>,
}

fn stock_box(account: OneAccount) -> StockBox {
    let document = stock::generated();
    let parsed = stock::engine(&document);
    assert_eq!(parsed.llm.models[0].model.name.as_ref(), CLAUDE, "stock must put Claude first");
    let client_models = jig_protocol_config::clients::models(&parsed.llm);
    let client_limits = parsed.limits.clients;
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
        ways_stock: Box::default(),
        forge: temper_engine_domain_forge::Config { push: None, namespace: Box::new([31; 16]), writers: Box::new([]) },
        identities: Box::new([]),
        machine: None,
        forges: Box::new([]),
        under_box: false,
        playground: None,
    };
    let models = config.core.settings.models.clone();
    let templates = config.core.defaults.templates.clone();
    let charters = config.core.defaults.charters.clone();
    let providers = config.core.settings.llm_providers.clone();
    let restart = temper_engine_domain_world::oracle::config::copy(&config);
    let mut driver = Driver::cold(Store::with_limits(&limits), config, &limits);
    driver.period_sent = true;
    driver.settle();
    let login = match account {
        OneAccount::Claude => Some(LoginState::Usable),
        OneAccount::Declared => Some(LoginState::NotSignedIn),
        OneAccount::ChatGpt | OneAccount::None => None,
    };
    let mut world = StockBox {
        driver,
        config: restart,
        models,
        templates,
        charters,
        providers,
        client_models,
        client_limits,
        login,
    };
    world.sign_in();
    world.ask(1, people::Ask::MakeProject { name: "Work".into(), policy: 1 });
    world.ask(3, people::Ask::MakeProject { name: "Other work".into(), policy: 1 });
    match account {
        OneAccount::Claude | OneAccount::Declared => world.declare_claude(),
        OneAccount::ChatGpt => world.add_openai(),
        OneAccount::None => {}
    }
    world
}

impl StockBox {
    fn declare_claude(&mut self) {
        let provider = self.provider(CLAUDE);
        self.ask(
            2,
            people::Ask::AccountChange {
                holder: people::AccountHolder::Own,
                change: people::AccountChange::DeclareProgram { provider, join: None, name: LOGIN.into() },
            },
        );
        let declared = self
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
                        Some(state) => Box::new([Login {
                            name: SourceName::new(LOGIN).unwrap(),
                            state: state.clone(),
                            email: None,
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
        let model = self.models.iter().find(|model| model.name.as_ref() == name).expect("configured init model");
        self.providers.get(model.provider).expect("configured init provider").number
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
        self.driver.send(Event::Party(PartyInput::Ask {
            models: Box::new([]),
            reply_to: ReplyTo::new(Token::new(10)),
            sign_in: self.driver.session(),
            key: [10; 16],
            ask: people::Ask::AccountSignIn { holder: people::AccountHolder::Own, provider: self.provider(OPENAI) },
        }));
        self.pump();
        let source = self
            .driver
            .protocol
            .accounts
            .iter()
            .find_map(|request| {
                if let accounts::Request::Signing(accounts::Signing::SignIn { source, .. }) = request {
                    Some(*source)
                } else {
                    None
                }
            })
            .expect("ChatGPT plan sign-in");
        self.driver.send(Event::Account(accounts::Event::Visited {
            source,
            address: b"http://issuer/visit".as_slice().into(),
        }));
        self.pump();
        self.driver.send(Event::Account(accounts::Event::SignedIn {
            source,
            subject: b"plan-owner".as_slice().into(),
            display: b"ChatGPT plan".as_slice().into(),
            declined: false,
            said: jig_domain_accounts::Said::default(),
        }));
        self.pump();
        self.driver.send(Event::Account(accounts::Event::Credential {
            number: source,
            generation: 1,
            valid: Duration::from_secs(3600),
        }));
        self.pump();
    }

    fn frame(&mut self) -> core::documents::Frame {
        self.driver.send(Event::Party(PartyInput::Read {
            reply_to: ReplyTo::new(Token::new(801)),
            sign_in: self.driver.session(),
            read: core::reads::Read::Frame,
        }));
        self.pump();
        self.driver
            .delivered
            .iter()
            .rev()
            .find_map(|delivery| {
                if let Delivery::Read { answer: core::reads::Answer::Document(document), .. } = delivery
                    && let core::documents::Document::Frame(frame) = document.as_ref()
                {
                    Some(frame.clone())
                } else {
                    None
                }
            })
            .expect("me document")
    }

    fn watched_frame(&self, after: usize) -> &core::documents::Frame {
        self.driver
            .delivered
            .iter()
            .skip(after)
            .rev()
            .find_map(|delivery| {
                if let Delivery::Document { watcher, document } = delivery
                    && *watcher == ME_WATCH
                    && let core::documents::Document::Frame(frame) = document.as_ref()
                {
                    Some(frame)
                } else {
                    None
                }
            })
            .expect("the watched me is updated")
    }

    fn me(&mut self) -> wire::Me {
        let frame = self.frame();
        jig_protocol_engine_clients::translate::owner::frame(&frame, &self.client_models, &self.client_limits.wire)
            .expect("stock me fits the client wire")
    }

    fn assert_offer(&mut self, expected: &[(&[u8], &[u8])]) {
        let record = self.me();
        let mut projects = Reader::new(record.projects());
        let mut count = 0_u32;
        while let Some(bytes) = skein_codec::piece::split(&mut projects).unwrap() {
            count = count.checked_add(1).unwrap();
            let project = wire::MeProject::decode(&self.client_limits.wire, &mut Reader::new(&bytes)).unwrap();
            assert_eq!(project.models().len(), u32::try_from(expected.len()).unwrap());
            for (index, (name, effort)) in expected.iter().enumerate() {
                let model = project.models().get(u32::try_from(index).unwrap()).unwrap();
                assert_eq!(model.name(), *name, "offer order and provider filtering");
                assert_eq!(model.effort().as_deref(), Some(*effort));
                let account = self
                    .driver
                    .store
                    .rows
                    .values()
                    .find_map(|row| {
                        if let engine::Record::Core(core::Record::Accounts(accounts::Stored::Account {
                            number,
                            provider,
                            ..
                        })) = row
                        {
                            (*provider == self.provider(name)).then_some(*number)
                        } else {
                            None
                        }
                    })
                    .expect("offered model's account");
                assert_eq!(model.accounts().as_slice(), [account]);
            }
        }
        assert_eq!(count, 2, "both of the party's projects are present");
    }

    fn assert_previews(&mut self, chat: Option<&[u8]>, asking: Option<&[u8]>) {
        let record = self.me();
        let mut projects = Reader::new(record.projects());
        while let Some(bytes) = skein_codec::piece::split(&mut projects).unwrap() {
            let project = wire::MeProject::decode(&self.client_limits.wire, &mut Reader::new(&bytes)).unwrap();
            assert_eq!(project.defaults().len(), if chat.is_some() { 4 } else { 0 });
            for (template, expected) in [(1, chat), (4, chat), (5, asking), (6, chat)] {
                let preview = (0..project.defaults().len()).find_map(|at| {
                    let preview = project.defaults().get(at).unwrap();
                    (preview.template() == template).then_some(preview)
                });
                match expected {
                    Some(name) => {
                        let preview = preview.expect("usable template preview");
                        assert_eq!(preview.choice().main().model(), name);
                        assert_eq!(preview.choice().main().effort().as_deref(), Some(b"high".as_slice()));
                        assert_eq!(*preview.choice().account(), None, "a preview reserves no account");
                    }
                    None => assert_eq!(preview, None),
                }
            }
        }
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
                tasks::EndedBy::Standing => people::EndedBy::Standing,
                tasks::EndedBy::Closer(closer) => {
                    people::EndedBy::Closer(people::Closer { connector: closer.connector, code: closer.code })
                }
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
            later: None,
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
        let mut endpoints = List::with_capacity(PROGRAM_ENDPOINTS);
        for model in &self.models {
            if model.program.is_some() && !endpoints.as_slice().contains(&model.endpoint) {
                endpoints.push(model.endpoint).expect("configured program endpoints");
            }
        }
        hello.bounds = Bounds {
            program_endpoints: endpoints.into_boxed(),
            endpoints: self
                .models
                .iter()
                .map(|model| EndpointBound { endpoint: model.endpoint, window: model.window, output: model.output })
                .collect(),
            turns: 800,
            spend: u64::MAX,
            time: Duration::from_secs(28_800),
            waiting: Duration::from_secs(60),
            checks: Duration::from_secs(1),
            message_bytes: self.driver.env.limits.tasks.message_bytes,
            inbox: self.driver.env.limits.tasks.inbox_messages,
            unacknowledged: self.driver.env.limits.fleet.turns,
            reply_bytes: None,
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
        let charter = self.charters.iter().find(|charter| charter.number == assignment.charter).unwrap();
        assert!(assignment.run.budget <= charter.spend, "claim stays within its charter's spend");
        assert!(
            assignment.run.budget > core::reserve_floor(&assignment.run.policy).unwrap(),
            "claim has its admission reserve"
        );
        // Complete this activation before the world's cold-restart oracle.
        let end = match assignment.run.ended_by {
            tasks::EndedBy::Requester | tasks::EndedBy::Standing => tasks::End::Parked,
            tasks::EndedBy::Result | tasks::EndedBy::Closer(_) => tasks::End::Finished {
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
    world.worker();
    world.assert_offer(&[(CLAUDE, b"high"), (b"claude-sonnet-5-5", b"high"), (b"claude-fable-5-1", b"high")]);
    world.assert_previews(Some(CLAUDE), Some(CLAUDE));
    let task = world.create(1, None);
    let document = world.document(task);
    assert_eq!(world.main(&document), CLAUDE, "Task before claim");
    world.assert_claim(task, CLAUDE);
}

#[test]
fn claude_alone_runs_the_stock_ask_temper_chat() {
    let mut world = stock_box(OneAccount::Claude);
    let task = world.create(5, None);
    let document = world.document(task);
    assert_eq!(world.main(&document), CLAUDE, "Ask temper Task before claim");
    world.worker();
    let assignment = world.assert_claim(task, CLAUDE);
    assert_eq!(assignment.charter, 11);
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
    world.worker();
    world.assert_offer(&[
        (OPENAI, b"high"),
        (ASTRA, b"high"),
        (b"gpt-6.1-terra", b"medium"),
        (b"gpt-6-luna", b"medium"),
    ]);
    world.assert_previews(Some(OPENAI), Some(ASTRA));
    let task = world.create(1, None);
    let document = world.document(task);
    assert_eq!(world.main(&document), OPENAI, "Task before claim");
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
fn no_account_offers_nothing_and_an_unnamed_chat_names_no_model() {
    let mut world = stock_box(OneAccount::None);
    world.assert_offer(&[]);
    world.assert_previews(None, None);
    let task = world.create(1, None);
    world.worker();
    let document = world.document(task);
    assert_eq!(document.llm, None, "no account offers a model in the task document");
    let row = world.driver.task(task).expect("live chat");
    let fallback = world
        .driver
        .root
        .tasks()
        .default_llm_at(1, row.settings.task, row.authority.models)
        .expect("the charter keeps its first model for run policy");
    assert_eq!(world.models.get(usize::from(fallback.main.model)).expect("charter model").name.as_ref(), CLAUDE);
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
fn chatgpt_alone_runs_the_stock_ask_temper_chat() {
    let mut world = stock_box(OneAccount::ChatGpt);
    let task = world.create(5, None);
    let document = world.document(task);
    assert_eq!(world.main(&document), ASTRA, "Ask temper Task before claim");
    world.worker();
    assert_eq!(world.assert_claim(task, ASTRA).charter, 11);
}

#[test]
fn both_accounts_offer_every_model_and_preview_the_users_first_choice() {
    let mut world = stock_box(OneAccount::Claude);
    world.add_openai();
    world.worker();
    world.assert_offer(&[
        (CLAUDE, b"high"),
        (b"claude-sonnet-5-5", b"high"),
        (b"claude-fable-5-1", b"high"),
        (OPENAI, b"high"),
        (ASTRA, b"high"),
        (b"gpt-6.1-terra", b"medium"),
        (b"gpt-6-luna", b"medium"),
    ]);
    world.assert_previews(Some(CLAUDE), Some(CLAUDE));
}

#[test]
fn no_account_keeps_an_explicitly_named_model_while_waiting() {
    let mut world = stock_box(OneAccount::None);
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
}

#[test]
fn a_lapsed_login_withdraws_its_offer_and_marks_me_but_keeps_the_tasks_model() {
    let mut world = stock_box(OneAccount::Claude);
    let task = world.create(1, None);
    world.worker();
    world.assert_claim(task, CLAUDE);
    let account = world.document(task).kept_account.expect("the run kept its Claude account");
    world.driver.send(Event::Party(PartyInput::Watch {
        watcher: ME_WATCH,
        sign_in: world.driver.session(),
        subject: core::documents::Watched::Frame,
        window: core::documents::Window { rows: world.driver.env.limits.documents.rows, after: None },
    }));
    world.pump();
    assert!(world.watched_frame(0).projects.iter().all(|project| project.offer.len() == 3));
    world.driver.send(Event::Party(PartyInput::Delivered { watcher: ME_WATCH }));
    world.pump();
    let before_lapse = world.driver.delivered.len();
    world.login = Some(LoginState::NotSignedIn);
    world.driver.send(protocol::up(
        CHANNEL,
        host::LinkUp::Logins {
            logins: Box::new([Login {
                name: SourceName::new(LOGIN).unwrap(),
                state: LoginState::NotSignedIn,
                email: None,
            }]),
        },
        engine::Parts::none(),
    ));
    world.pump();
    assert!(
        world
            .watched_frame(before_lapse)
            .projects
            .iter()
            .all(|project| project.offer.is_empty() && project.defaults.is_empty()),
        "the login lapse invalidates me's watched offer and previews"
    );
    world.assert_offer(&[]);
    world.assert_previews(None, None);
    let document = world.document(task);
    assert_eq!(document.kept_account, Some(account));
    assert_eq!(world.main(&document), CLAUDE);
}

#[test]
fn init_limits_hold_seven_models_and_five_previews_per_project_on_the_wire() {
    let mut world = stock_box(OneAccount::Claude);
    world.add_openai();
    world.worker();
    let mut frame = world.frame();
    let limits = world.driver.env.limits;
    let mut project = frame.projects.as_slice().first().unwrap().clone();
    assert_eq!(project.offer.len(), 7);
    // Size the fourth preview too: a project's chosen goal executor may be resolved
    // to the coding charter. The stock leaves that executor for the person to pick.
    let choice = project.defaults.as_slice().first().unwrap().1.clone();
    project.defaults.push((2, choice)).unwrap();
    project.templates.iter_mut().find(|template| template.number == 2).unwrap().executor =
        people::TemplateExecutor::Charter(2);
    for offered in project.offer.iter_mut() {
        offered.accounts = List::with_capacity(limits.accounts.accounts);
        for account in 1..=limits.accounts.accounts {
            offered.accounts.push(account).unwrap();
        }
    }
    assert_eq!(project.defaults.len(), 5);
    project.name.bytes = skein_lib::bytes::zeroed(usize::try_from(limits.documents.text_bytes).unwrap());
    project.name.bytes.fill(b'p');
    let count = limits.people.projects.min(limits.documents.lines);
    frame.projects = List::with_capacity(count);
    for number in 1..=count {
        let mut row = project.clone();
        row.project = number;
        frame.projects.push(row).unwrap();
    }
    frame.login.bytes = skein_lib::bytes::zeroed(usize::try_from(limits.documents.text_bytes).unwrap());
    frame.login.bytes.fill(b'l');
    frame.name.bytes.clone_from(&frame.login.bytes);
    let record =
        jig_protocol_engine_clients::translate::owner::frame(&frame, &world.client_models, &world.client_limits.wire)
            .expect("init's largest me translates to the client wire");
    assert!(
        record.measure() <= world.client_limits.message_bytes,
        "me fits the client message bound: {} <= {}",
        record.measure(),
        world.client_limits.message_bytes
    );
    assert!(u64::from(record.measure()) <= wire::Me::encoded_bound(&world.client_limits.wire).unwrap());
    let mut writer = Writer::new(usize::try_from(record.measure()).unwrap());
    record.encode(&mut writer).unwrap();
    let decoded = wire::Me::decode(&world.client_limits.wire, &mut Reader::new(&writer.finish())).unwrap();
    assert_eq!(decoded, record);
    let mut projects = Reader::new(decoded.projects());
    let mut seen = 0_u32;
    while let Some(bytes) = skein_codec::piece::split(&mut projects).unwrap() {
        let project = wire::MeProject::decode(&world.client_limits.wire, &mut Reader::new(&bytes)).unwrap();
        assert_eq!(project.models().len(), 7);
        assert_eq!(project.defaults().len(), 5);
        seen = seen.checked_add(1).unwrap();
    }
    assert_eq!(seen, count);
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
