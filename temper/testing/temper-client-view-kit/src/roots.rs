use super::*;
use jig_client::contract as c;
use skein_lib::{Env, Queue, Time, Token, Wall, Writer};
use temper_client_protocol_wire::{self as wire, v1 as w};

#[must_use]
pub fn root_in_project(project: Option<u32>) -> temper_client_domain::Domain {
    root_with_watch(project).0
}
#[must_use]
pub fn root_with_watch(project: Option<u32>) -> (temper_client_domain::Domain, Token) {
    let (root, me, _) = root_at(
        temper_client_domain::Address::Inbox {
            filter: temper_client_domain::address::Filter { project, ..temper_client_domain::address::Filter::ALL },
        },
        &domain_limits(),
    );
    (root, me)
}
#[must_use]
pub fn root_at(
    address: temper_client_domain::Address,
    limits: &Limits,
) -> (temper_client_domain::Domain, Token, Option<Token>) {
    let (root, me, page, _) = root_at_with_tree(address, limits);
    (root, me, page)
}
#[must_use]
pub fn root_at_with_tree(
    address: temper_client_domain::Address,
    limits: &Limits,
) -> (temper_client_domain::Domain, Token, Option<Token>, Option<Token>) {
    root_at_with_tree_named(address, limits, b"Shop")
}
#[must_use]
pub fn root_at_with_tree_named(
    address: temper_client_domain::Address,
    limits: &Limits,
    project_name: &[u8],
) -> (temper_client_domain::Domain, Token, Option<Token>, Option<Token>) {
    let (root, me, page, tree, _) = root_at_full(address, limits, project_name);
    (root, me, page, tree)
}
#[must_use]
pub fn account_root() -> (temper_client_domain::Domain, Token) {
    let (root, _, _, _, accounts) = root_at_full(temper_client_domain::Address::Accounts, &domain_limits(), b"Shop");
    (root, accounts.expect("global accounts watch"))
}
#[must_use]
pub fn root_at_full(
    address: temper_client_domain::Address,
    limits: &Limits,
    project_name: &[u8],
) -> (temper_client_domain::Domain, Token, Option<Token>, Option<Token>, Option<Token>) {
    root_at_with_playground(address, limits, project_name, None)
}
#[must_use]
pub fn root_at_with_playground(
    address: temper_client_domain::Address,
    limits: &Limits,
    project_name: &[u8],
    playground: Option<(u32, bool)>,
) -> (temper_client_domain::Domain, Token, Option<Token>, Option<Token>, Option<Token>) {
    let state = if let temper_client_domain::Address::Welcome { .. } = address {
        w::PlaygroundState::Starting
    } else {
        w::PlaygroundState::Running
    };
    root_at_with_activity(
        address,
        limits,
        project_name,
        match playground {
            Some((project, model)) => Some((project, model, state)),
            None => None,
        },
        None,
        None,
    )
}
#[must_use]
pub fn root_at_with_activity(
    address: temper_client_domain::Address,
    limits: &Limits,
    project_name: &[u8],
    playground: Option<(u32, bool, w::PlaygroundState)>,
    activity: Option<c::ActivityPage>,
    forge_rows: Option<List<w::ForgeRow>>,
) -> (temper_client_domain::Domain, Token, Option<Token>, Option<Token>, Option<Token>) {
    let (root, me, page, tree, accounts, _) =
        root_at_with_forges(address, limits, project_name, playground, activity, forge_rows);
    (root, me, page, tree, accounts)
}

#[must_use]
pub fn root_at_with_forges(
    address: temper_client_domain::Address,
    limits: &Limits,
    project_name: &[u8],
    playground: Option<(u32, bool, w::PlaygroundState)>,
    activity: Option<c::ActivityPage>,
    forge_rows: Option<List<w::ForgeRow>>,
) -> (temper_client_domain::Domain, Token, Option<Token>, Option<Token>, Option<Token>, Option<Token>) {
    root_at_with_dismissals(address, limits, project_name, playground, activity, forge_rows, Some(&[]))
}

#[must_use]
pub fn root_at_with_needs(
    address: temper_client_domain::Address,
    limits: &Limits,
) -> (temper_client_domain::Domain, Token, Option<Token>, Token) {
    let (root, me, page, _, _, _, needs) =
        root_at_with_dismissals_and_needs(address, limits, b"Shop", None, None, None, Some(&[]));
    (root, me, page, needs.expect("standing needs watch"))
}
#[must_use]
pub fn root_at_with_dismissals(
    address: temper_client_domain::Address,
    limits: &Limits,
    project_name: &[u8],
    playground: Option<(u32, bool, w::PlaygroundState)>,
    activity: Option<c::ActivityPage>,
    forge_rows: Option<List<w::ForgeRow>>,
    dismissed: Option<&[u32]>,
) -> (temper_client_domain::Domain, Token, Option<Token>, Option<Token>, Option<Token>, Option<Token>) {
    let (root, me, page, tree, accounts, forges, _, _, _) = root_at_with_values(
        address,
        limits,
        project_name,
        playground,
        activity,
        forge_rows,
        dismissed,
        Some(forge::ways::Ways::STOCK),
        false,
        true,
    );
    (root, me, page, tree, accounts, forges)
}

#[expect(clippy::type_complexity, reason = "the fixture returns session and page watch tokens")]
#[must_use]
pub fn root_at_with_dismissals_and_needs(
    address: temper_client_domain::Address,
    limits: &Limits,
    project_name: &[u8],
    playground: Option<(u32, bool, w::PlaygroundState)>,
    activity: Option<c::ActivityPage>,
    forge_rows: Option<List<w::ForgeRow>>,
    dismissed: Option<&[u32]>,
) -> (temper_client_domain::Domain, Token, Option<Token>, Option<Token>, Option<Token>, Option<Token>, Option<Token>) {
    let (root, me, page, tree, accounts, forges, _, needs, _) = root_at_with_values(
        address,
        limits,
        project_name,
        playground,
        activity,
        forge_rows,
        dismissed,
        Some(forge::ways::Ways::STOCK),
        false,
        true,
    );
    (root, me, page, tree, accounts, forges, needs)
}

#[must_use]
pub fn root_with_ways(
    address: temper_client_domain::Address,
    ways: Option<forge::ways::Ways>,
) -> temper_client_domain::Domain {
    root_at_with_values(address, &domain_limits(), b"Shop", None, None, None, Some(&[]), ways, false, true).0
}

#[must_use]
pub fn root_with_refused_ways(address: temper_client_domain::Address) -> temper_client_domain::Domain {
    root_at_with_values(address, &domain_limits(), b"Shop", None, None, None, Some(&[]), None, true, true).0
}

#[must_use]
pub fn home_root(
    limits: &Limits,
    rows: List<w::ForgeRow>,
    dismissed: Option<&[u32]>,
    goals: bool,
) -> (temper_client_domain::Domain, Token, Token, Token, Token, Token) {
    let (root, me, _, _, accounts, _, settings, needs, away) = root_at_with_values(
        temper_client_domain::Address::Models { project: PROJECT },
        limits,
        b"Shop",
        None,
        None,
        Some(rows),
        dismissed,
        Some(forge::ways::Ways { goals, ..forge::ways::Ways::STOCK }),
        false,
        false,
    );
    (
        root,
        me,
        accounts.expect("accounts watch"),
        settings.expect("session settings watch"),
        needs.expect("standing needs watch"),
        away.expect("away watch"),
    )
}

#[expect(clippy::type_complexity, reason = "the fixture returns the session and page watch tokens to their callers")]
#[expect(clippy::too_many_arguments, reason = "the fixture carries independent session answers")]
#[must_use]
pub fn root_at_with_values(
    address: temper_client_domain::Address,
    limits: &Limits,
    project_name: &[u8],
    playground: Option<(u32, bool, w::PlaygroundState)>,
    activity: Option<c::ActivityPage>,
    forge_rows: Option<List<w::ForgeRow>>,
    dismissed: Option<&[u32]>,
    ways: Option<forge::ways::Ways>,
    refuse_ways: bool,
    settings_delivered: bool,
) -> (
    temper_client_domain::Domain,
    Token,
    Option<Token>,
    Option<Token>,
    Option<Token>,
    Option<Token>,
    Option<Token>,
    Option<Token>,
    Option<Token>,
) {
    root_at_with_classes(
        address,
        limits,
        project_name,
        playground,
        activity,
        forge_rows,
        dismissed,
        ways,
        refuse_ways,
        settings_delivered,
        None,
    )
}
#[expect(clippy::type_complexity, reason = "the fixture returns the session and page watch tokens to their callers")]
#[expect(clippy::too_many_arguments, reason = "the fixture carries independent session answers")]
#[expect(clippy::too_many_lines, reason = "one fixture delivers the page's documents and the session's reads together")]
#[must_use]
pub fn root_at_with_classes(
    address: temper_client_domain::Address,
    limits: &Limits,
    project_name: &[u8],
    playground: Option<(u32, bool, w::PlaygroundState)>,
    activity: Option<c::ActivityPage>,
    forge_rows: Option<List<w::ForgeRow>>,
    dismissed: Option<&[u32]>,
    ways: Option<forge::ways::Ways>,
    refuse_ways: bool,
    settings_delivered: bool,
    classes: Option<&temper_protocol_reaches::Classes>,
) -> (
    temper_client_domain::Domain,
    Token,
    Option<Token>,
    Option<Token>,
    Option<Token>,
    Option<Token>,
    Option<Token>,
    Option<Token>,
    Option<Token>,
) {
    use jig_client::contract::{Delivery, Subject, WatchEvent};
    use temper_client_domain::{Event, Request};
    let forge_rows_answered = forge_rows.is_some();
    let asking_page = if let temper_client_domain::Address::Ask { project } = address { Some(project) } else { None };
    let env = Env { now: Time::ZERO, wall: NOW, limits: *limits };
    let mut root = temper_client_domain::unlinked(limits, SEED);
    let mut out = Queue::uncapped();
    temper_client_domain::step(
        &mut root,
        &env,
        Event::Client { event: jig_client::Event::StartUnlinked { kept: List::with_capacity(0) } },
        &mut out,
    );
    let welcome = match address {
        temper_client_domain::Address::Welcome { .. } => true,
        temper_client_domain::Address::Forges
        | temper_client_domain::Address::ConnectForge { .. }
        | temper_client_domain::Address::Chats { .. }
        | temper_client_domain::Address::NewChat { .. }
        // page-view: draw Remembered.
        | temper_client_domain::Address::Told
        | temper_client_domain::Address::Remembered
        | temper_client_domain::Address::Members
        | temper_client_domain::Address::Limits
        | temper_client_domain::Address::Accounts
        | temper_client_domain::Address::AllHome
        | temper_client_domain::Address::Inbox { .. }
        | temper_client_domain::Address::Home { .. }
        | temper_client_domain::Address::Changes { .. }
        | temper_client_domain::Address::Team { .. }
        | temper_client_domain::Address::Settings { .. }
        | temper_client_domain::Address::Models { .. }
        | temper_client_domain::Address::Builders { .. }
        | temper_client_domain::Address::Role { .. }
        | temper_client_domain::Address::Rule { .. }
        | temper_client_domain::Address::Repositories { .. }
        | temper_client_domain::Address::Work { .. }
        | temper_client_domain::Address::Goals { .. }
        | temper_client_domain::Address::Ask { .. } | temper_client_domain::Address::Tell { .. }
        | temper_client_domain::Address::Adopt { .. }
        | temper_client_domain::Address::Chat { .. }
        | temper_client_domain::Address::Talk { .. }
        | temper_client_domain::Address::Task { .. }
        | temper_client_domain::Address::ConnectModel { .. } => false,
    };
    // Sign in before opening the page's one-shot activity read.
    temper_client_domain::step(
        &mut root,
        &env,
        Event::Start { address, kept: List::with_capacity(0), with: Some(jig_client::With::Transport) },
        &mut out,
    );
    temper_client_domain::step(
        &mut root,
        &env,
        Event::Client { event: jig_client::Event::Up { up: jig_client::Up::Served { served: true, versions: None } } },
        &mut out,
    );
    let mut watch = None;
    let mut inbox_watch = None;
    let mut tree_watch = None;
    let mut account_watch = None;
    let mut activity_read = None;
    let mut dismissed_watch = None;
    let mut away_watch = None;
    for _ in 0..out.len() {
        let request = out.pop();
        if let Some(Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Open { watch, subject } },
        }) = request.as_ref()
            && Some(subject.clone()) == forge::dismissed::watch(&limits.forge)
        {
            dismissed_watch = Some(*watch);
        }
        if let Some(Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Open { watch, subject } },
        }) = request.as_ref()
            && Some(subject.clone()) == forge::away::watch(&limits.forge)
        {
            away_watch = Some(*watch);
        }
        if let Some(read) = activity_token(request.as_ref()) {
            activity_read = Some(read);
        }
        if let Some(Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Open { watch: token, subject: jig_client::Watchable::Jig(subject) },
                },
        }) = request
        {
            if subject == Subject::Me {
                watch = Some(token);
            } else if subject == Subject::Accounts {
                account_watch = Some(token);
            } else if let Subject::Tree { .. } = subject {
                tree_watch = Some(token);
            } else if let Subject::Inbox { .. } | Subject::Task { .. } = subject {
                inbox_watch = Some(token);
            }
        }
    }
    let mut frame = fixture_me(project_name);
    if asking_page.is_some() {
        frame = asking_me(frame);
    }
    if forge_rows_answered {
        frame.deployment_role = Some(c::HeldRole::Owner);
    }
    temper_client_domain::step(
        &mut root,
        &env,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: watch.expect("frame opened"),
                    event: WatchEvent::Delivered(Delivery::Me(Box::new(frame))),
                },
            },
        },
        &mut out,
    );
    answer_activity(&mut root, &env, activity_read, activity, &mut out);
    let mut forges = None;
    let mut needs_watch = inbox_watch;
    for _ in 0..out.len() {
        let request = out.pop().expect("counted request");
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(Subject::Inbox { .. }) },
                },
        } = &request
        {
            needs_watch = Some(*watch);
        }
        if let Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Open { watch, subject } },
        } = &request
            && Some(subject.clone()) == forge::forges::watch(&limits.forge)
        {
            forges = Some(*watch);
        }
        if let Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Open { watch, subject } },
        } = &request
            && Some(subject.clone()) == forge::dismissed::watch(&limits.forge)
        {
            dismissed_watch = Some(*watch);
        }
        if let Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Open { watch, subject } },
        } = &request
            && Some(subject.clone()) == forge::away::watch(&limits.forge)
        {
            away_watch = Some(*watch);
        }
        out.push(request);
    }
    if let (Some(watch), Some(projects)) = (dismissed_watch, dismissed) {
        let mut dismissed = List::with_capacity(wire::DISMISSED);
        for project in projects {
            dismissed
                .push(w::PersonFact::ToBegin(
                    w::ToBegin::new(&w::CEILINGS, w::ToBeginParts { project: *project }).unwrap(),
                ))
                .unwrap();
        }
        root_event(
            &mut root,
            Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::FamilyDelivered {
                        watch,
                        record: jig_client::Family {
                            family: wire::FORGE,
                            version: wire::CLIENT_VERSION,
                            bytes: wire::dismissals(
                                &w::Dismissals::new(&w::CEILINGS, w::DismissalsParts { dismissed }).unwrap(),
                            ),
                        },
                    },
                },
            },
        );
    }
    if let Some(watch) = forges {
        let rows = match forge_rows {
            Some(rows) => rows,
            None => List::with_capacity(0),
        };
        let wire_limits = forge::wire_limits(&limits.forge).unwrap();
        temper_client_domain::step(
            &mut root,
            &env,
            Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::FamilyDelivered {
                        watch,
                        record: jig_client::Family {
                            family: wire::FORGE,
                            version: wire::CLIENT_VERSION,
                            bytes: wire::forges(
                                &w::Forges::new(&wire_limits, w::ForgesParts { forges: rows, linked: false }).unwrap(),
                            ),
                        },
                    },
                },
            },
            &mut out,
        );
    }

    let mut read = None;
    let mut settings_watch = None;
    while let Some(request) = out.pop() {
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(Subject::Settings) },
                },
        } = &request
        {
            settings_watch = Some(*watch);
            if settings_delivered {
                temper_client_domain::step(
                    &mut root,
                    &env,
                    Event::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::Watched {
                                watch: *watch,
                                event: WatchEvent::Delivered(Delivery::Settings(Box::new(c::SettingsDocument {
                                    newest: 0,
                                    settings: List::with_capacity(0),
                                    held: List::with_capacity(0),
                                    pending: List::with_capacity(0),
                                }))),
                            },
                        },
                    },
                    &mut out,
                );
            }
        }
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Read { read: token, what: jig_client::Readable::Family { record, .. } },
                },
        } = request
        {
            match wire::decode_reading(&record.bytes, &w::CEILINGS).expect("forge read").read() {
                w::Read::Deployment => deliver_deployment(&mut root, token, true, true),
                w::Read::Playground => read = Some(token),
                w::Read::Setting(setting) => {
                    if refuse_ways {
                        temper_client_domain::step(
                            &mut root,
                            &env,
                            Event::Client {
                                event: jig_client::Event::Up {
                                    up: jig_client::Up::ReadDone {
                                        read: token,
                                        result: jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::Unknown)),
                                    },
                                },
                            },
                            &mut out,
                        );
                    } else if let Some(ways) = ways {
                        let value = match setting.kind() {
                            7 => Some(ways_bytes(ways)),
                            8 => match classes {
                                Some(classes) => Some(classes_bytes(classes)),
                                None => None,
                            },
                            _ => None,
                        };
                        let answer = w::SettingAnswer::new(&w::CEILINGS, w::SettingAnswerParts { value }).unwrap();
                        temper_client_domain::step(
                            &mut root,
                            &env,
                            Event::Client {
                                event: jig_client::Event::Up {
                                    up: jig_client::Up::ReadDone {
                                        read: token,
                                        result: jig_client::Readout::Family {
                                            family: wire::FORGE,
                                            record: Some(wire::setting_answer(&answer)),
                                        },
                                    },
                                },
                            },
                            &mut out,
                        );
                    }
                }
                w::Read::RevisionSetting(_)
                | w::Read::Finding(_)
                | w::Read::ProposalItems(_)
                | w::Read::ProgramEnvironment
                | w::Read::View(_) => {}
            }
        }
    }
    if playground.is_some() || welcome || forge_rows_answered {
        if let Some((project, model, state)) = playground {
            playground_session(
                &mut root,
                read.expect("live link reads playground"),
                account_watch.expect("accounts watch"),
                project,
                model,
                state,
            );
        } else {
            empty_session(&mut root, read.expect("playground read"), account_watch.expect("accounts watch"));
        }
    }
    (
        root,
        watch.expect("me opened"),
        inbox_watch,
        tree_watch,
        account_watch,
        forges,
        settings_watch,
        needs_watch,
        away_watch,
    )
}
pub fn empty_session(root: &mut temper_client_domain::Domain, read: Token, watch: Token) {
    deliver_no_playground(root, read);
    root_event(
        root,
        temper_client_domain::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Accounts(Box::new(c::Accounts {
                        accounts: List::with_capacity(0),
                        readiness: List::with_capacity(0),
                        readiness_cut: false,
                        program_hosts: List::with_capacity(0),
                        signing: List::with_capacity(0),
                        lendable: false,
                        lending: None,
                        lent_end: None,
                        last_end: None,
                        usable: false,
                        settled: true,
                        cut: false,
                    }))),
                },
            },
        },
    );
}
#[must_use]
pub fn activity_token(request: Option<&temper_client_domain::Request>) -> Option<Token> {
    if let Some(temper_client_domain::Request::Unlinked {
        request:
            jig_client::UnlinkedRequest::Down {
                down: jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Activity { .. }) },
            },
    }) = request
    {
        Some(*read)
    } else {
        None
    }
}
pub fn answer_activity(
    root: &mut temper_client_domain::Domain,
    env: &Env<Limits>,
    mut read: Option<Token>,
    activity: Option<c::ActivityPage>,
    out: &mut Queue<temper_client_domain::Request>,
) {
    let Some(page) = activity else { return };
    // Signing in can replace a page's initial read. Answer its current one.
    for _ in 0..out.len() {
        let request = out.pop();
        if let Some(token) = activity_token(request.as_ref()) {
            read = Some(token);
        }
        if let Some(request) = request {
            out.push(request);
        }
    }
    temper_client_domain::step(
        root,
        env,
        temper_client_domain::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::ReadDone {
                    read: read.expect("activity read"),
                    result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Activity(Box::new(page)))),
                },
            },
        },
        out,
    );
}
#[must_use]
pub fn adoption_root() -> temper_client_domain::Domain {
    adoption_page(PROJECT, b"shop")
}
#[must_use]
pub fn adoption_page(project: u32, name: &[u8]) -> temper_client_domain::Domain {
    root_at_with_tree_named(temper_client_domain::Address::Adopt { project }, &domain_limits(), name).0
}
pub fn root_event(root: &mut temper_client_domain::Domain, event: temper_client_domain::Event) {
    let limits = domain_limits();
    temper_client_domain::step(root, &Env { now: Time::ZERO, wall: NOW, limits }, event, &mut Queue::uncapped());
}

pub const INBOX_TASK: u64 = 9;
pub const INBOX_REPLY_TASK: u64 = 10;
pub const INBOX_QUESTION: u64 = 1;
pub const INBOX_POSITION: u64 = 1;
pub const OTHER_PARTY: u64 = 74;
#[must_use]
pub fn inbox_root() -> temper_client_domain::Domain {
    inbox_root_filtered(temper_client_domain::address::Filter::ALL)
}
#[must_use]
pub fn inbox_root_filtered(filter: temper_client_domain::address::Filter) -> temper_client_domain::Domain {
    inbox_with_document(filter, None)
}
#[must_use]
pub fn inbox_with_document(
    filter: temper_client_domain::address::Filter,
    document: Option<(jig_client::contract::Read, jig_client::contract::ReadValue)>,
) -> temper_client_domain::Domain {
    inbox_with_proposer(filter, document, c::Proposer::Task(INBOX_TASK))
}
#[must_use]
pub fn inbox_from(proposer: c::Proposer) -> temper_client_domain::Domain {
    inbox_with_proposer(temper_client_domain::address::Filter::ALL, None, proposer)
}
#[must_use]
pub fn inbox_with_proposer(
    filter: temper_client_domain::address::Filter,
    document: Option<(c::Read, c::ReadValue)>,
    proposer: c::Proposer,
) -> temper_client_domain::Domain {
    inbox_facts(filter, document, proposer, false, None)
}
#[must_use]
pub fn inbox_facts(
    filter: temper_client_domain::address::Filter,
    document: Option<(c::Read, c::ReadValue)>,
    proposer: c::Proposer,
    cut: bool,
    unread: Option<c::Count>,
) -> temper_client_domain::Domain {
    inbox_facts_at(temper_client_domain::Address::Inbox { filter }, document, proposer, cut, unread)
}
/// A question's reply on Home, sharing the inbox's delivered card fixture.
#[must_use]
pub fn reply_composer(document: (c::Read, c::ReadValue)) -> temper_client_domain::Domain {
    inbox_facts_at(
        temper_client_domain::Address::Home { project: PROJECT },
        Some(document),
        c::Proposer::Task(INBOX_TASK),
        false,
        None,
    )
}
#[expect(clippy::too_many_lines, reason = "one fixture delivers the inbox and its reply composer through public reads")]
#[must_use]
pub fn inbox_facts_at(
    address: temper_client_domain::Address,
    document: Option<(c::Read, c::ReadValue)>,
    proposer: c::Proposer,
    cut: bool,
    unread: Option<c::Count>,
) -> temper_client_domain::Domain {
    use jig_client::contract::{self, InboxGroup as Group, ObjectKey as Object};
    let mut limits = domain_limits();
    // Six card reads beside the frame's own bounded reads.
    limits.client.domain.reads = 8;
    let (mut root, watch) = if let temper_client_domain::Address::Home { .. } = address {
        let (root, _, _, watch) =
            root_at_with_needs(temper_client_domain::Address::Settings { project: PROJECT }, &limits);
        (root, Some(watch))
    } else {
        let (root, _, watch) = root_at(address.clone(), &limits);
        (root, watch)
    };
    let mut finish = false;
    if let temper_client_domain::Address::Home { .. } = address {
        finish = true;
    }
    let env = Env { now: Time::ZERO, wall: NOW, limits };
    let mut out = Queue::uncapped();
    if finish {
        temper_client_domain::step(
            &mut root,
            &env,
            temper_client_domain::Event::Went { place: temper_client_domain::Place::Address(address) },
            &mut out,
        );
    }
    let mut entries = List::with_capacity(6);
    for (object, group) in [
        (Object::Question { task: INBOX_TASK, message: INBOX_QUESTION }, Group::Questions),
        (Object::Proposal(contract::ProposalKey { proposer, number: INBOX_QUESTION }), Group::Proposals),
        (Object::Escalation(INBOX_TASK), Group::Escalations),
        (Object::PersonTask(INBOX_TASK), Group::PersonTasks),
        (Object::Result(INBOX_TASK), Group::Results),
        (Object::Task(INBOX_REPLY_TASK), Group::Replies),
    ] {
        entries
            .push(temper_fixtures::client_documents::entry(
                object,
                PROJECT,
                INBOX_POSITION,
                Wall::EPOCH,
                if group == Group::Replies {
                    let mut line = chats::line();
                    line.chip.task = INBOX_REPLY_TASK;
                    if let Some(unread) = unread {
                        line.conversation = Some(c::ConversationRead { seen: 0, unread });
                    }
                    Some(Box::new(line))
                } else {
                    None
                },
            ))
            .expect("six kinds");
    }
    let inbox = {
        let counts = contract::Needs { questions: 1, proposals: 1, escalations: 1, person_tasks: 1 };
        temper_fixtures::client_documents::inbox(
            entries,
            counts
                .questions
                .saturating_add(counts.proposals)
                .saturating_add(counts.escalations)
                .saturating_add(counts.person_tasks),
            contract::Count { value: 2, limited: false },
            0,
            cut,
        )
    };
    let query_rows = inbox.needs.clone();
    temper_client_domain::step(
        &mut root,
        &env,
        temper_client_domain::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: watch.expect("inbox watch"),
                    event: contract::WatchEvent::Delivered(contract::Delivery::Inbox(Box::new(inbox))),
                },
            },
        },
        &mut out,
    );
    if let Some((read, page)) = document {
        let mut page = Some(page);
        while let Some(request) = out.pop() {
            if let temper_client_domain::Request::Unlinked {
                request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Open { watch, subject } },
            } = &request
                && Some(subject) == forge::decided::watch(PROJECT, &env.limits.forge).as_ref()
            {
                let document = w::Decisions::new(
                    &w::CEILINGS,
                    w::DecisionsParts {
                        project: PROJECT,
                        rows: List::with_capacity(0),
                        asked: List::with_capacity(0),
                        cut: false,
                    },
                )
                .unwrap();
                temper_client_domain::step(
                    &mut root,
                    &env,
                    temper_client_domain::Event::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::FamilyDelivered {
                                watch: *watch,
                                record: jig_client::Family {
                                    family: wire::FORGE,
                                    version: wire::CLIENT_VERSION,
                                    bytes: wire::decisions(&document),
                                },
                            },
                        },
                    },
                    &mut out,
                );
                continue;
            }

            if let temper_client_domain::Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down:
                            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Inbox { group, .. }) },
                    },
            } = &request
            {
                let rows = if *group == c::InboxReadGroup::Needs { query_rows.clone() } else { List::uncapped() };
                temper_client_domain::step(
                    &mut root,
                    &env,
                    temper_client_domain::Event::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::ReadDone {
                                read: *read,
                                result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Inbox(Box::new(
                                    c::InboxPage { selection: None, cut: false, rows, before: None, after: None },
                                )))),
                            },
                        },
                    },
                    &mut out,
                );
                continue;
            }
            if let temper_client_domain::Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: jig_client::Down::Read { read: token, what: jig_client::Readable::Jig(what) },
                    },
            } = request
            {
                let result = if what == read && page.is_some() {
                    contract::ReadResult::Value(page.take().expect("one matching card read"))
                } else if finish {
                    contract::ReadResult::None(contract::ReadNone::Unknown)
                } else {
                    continue;
                };
                temper_client_domain::step(
                    &mut root,
                    &env,
                    temper_client_domain::Event::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::ReadDone { read: token, result: jig_client::Readout::Jig(result) },
                        },
                    },
                    &mut out,
                );
                if !finish {
                    break;
                }
            }
        }
    }
    root
}

#[must_use]
pub fn signed_out() -> temper_client_domain::Domain {
    let mut root = root_in_project(Some(PROJECT));
    root_event(&mut root, temper_client_domain::Event::SignedOut);
    root
}
#[must_use]
pub fn out_of_date() -> temper_client_domain::Domain {
    let mut root = root_in_project(Some(PROJECT));
    root_event(
        &mut root,
        temper_client_domain::Event::Client {
            event: jig_client::Event::Up { up: jig_client::Up::Served { served: false, versions: None } },
        },
    );
    root
}
#[must_use]
pub fn chats_root() -> temper_client_domain::Domain {
    chats::listed(&[], &[], false)
}
#[must_use]
pub fn fixture_me(project_name: &[u8]) -> c::Me {
    use jig_client::contract::{self, DocumentText, HeldRole, Me, MeParty, MeProject, Needs, PartyKind};
    let mut projects = List::with_capacity(2);
    projects
        .push(MeProject {
            left_out: 0,
            defaults: List::with_capacity(0),
            public: None,
            project: PROJECT,
            name: DocumentText::complete(
                contract::Text::new(skein_lib::bytes::copy_of(project_name), 32).expect("name fits"),
            ),
            role: HeldRole::Policy { role: 1 },
            pool: None,
            templates: List::with_capacity(0),
            creating: None,
            models: List::with_capacity(0),
        })
        .expect("one project");
    let mut second = projects.as_slice()[0].clone();
    second.project = SECOND_PROJECT;
    second.name = DocumentText::complete(contract::Text::new(Box::from(*b"Second"), 32).expect("name fits"));
    projects.push(second).expect("second project");
    Me {
        providers: List::with_capacity(0),
        providers_cut: false,
        key_bytes: 256,
        redirect_bytes: 256,
        words_bytes: 8192,
        party: MeParty {
            number: 73,
            kind: PartyKind::Person,
            login: DocumentText::complete(contract::Text::new(Box::from(*b"pat"), 32).expect("login fits")),
            name: DocumentText::complete(contract::Text::new(Box::from(*b"Pat"), 32).expect("name fits")),
        },
        projects,
        needs: Needs { questions: 0, proposals: 0, escalations: 0, person_tasks: 0 },
        needs_total: 0,
        updates: contract::Count { value: 0, limited: false },
        results_seen: 0,
        cut: false,
        retention: Duration::from_secs(1000),
        deployment_role: None,
    }
}
pub const PLAYGROUND_CUE: &[u8] = b"Change the title";
pub const PLAYGROUND_OTHER_CUE: &[u8] = b"Add a test";
#[must_use]
pub fn playground_root(
    address: temper_client_domain::Address,
    project: u32,
    model: bool,
) -> temper_client_domain::Domain {
    let mut root = playground_choice_root(address.clone(), project, model, None);
    if let temper_client_domain::Address::Chats { .. } = address {
        answer_empty_chats(&mut root);
    }
    root
}
#[must_use]
pub fn playground_choice_root(
    address: temper_client_domain::Address,
    project: u32,
    model: bool,
    llm: Option<c::LlmChoice>,
) -> temper_client_domain::Domain {
    playground_frame_root(address, project, model, llm, None)
}
#[must_use]
pub fn playground_frame_root(
    address: temper_client_domain::Address,
    project: u32,
    model: bool,
    llm: Option<c::LlmChoice>,
    frame: Option<c::Me>,
) -> temper_client_domain::Domain {
    playground_frame_with_watch(address, project, model, llm, frame).0
}
#[expect(clippy::too_many_lines, reason = "the fixture delivers playground offer, preview and task through watches")]
#[must_use]
pub fn playground_frame_with_watch(
    address: temper_client_domain::Address,
    project: u32,
    model: bool,
    llm: Option<c::LlmChoice>,
    frame: Option<c::Me>,
) -> (temper_client_domain::Domain, Token) {
    let (mut root, me_watch, task_watch, _, accounts_watch) =
        root_at_with_playground(address.clone(), &domain_limits(), b"Project", Some((project, model)));
    let template = chat::messaging_root();
    let mut me = match frame {
        Some(me) => me,
        None => template.pages().me(template.client()).expect("chat template").clone(),
    };
    if let temper_client_domain::Address::Ask { .. } = address {
        me = asking_me(me);
    }
    if me.providers.is_empty() {
        me.providers = List::with_capacity(1);
        me.providers.push(playground_provider()).unwrap();
    }
    let mut models = List::with_capacity(2);
    for name in [crate::words::SCRIPTED_MODEL, b"selected"] {
        models
            .push(c::MeModel {
                new_until: None,
                number: 1,
                source: jig_client::contract::ModelSource::Stock,
                estimated: false,
                effort: None,
                shown: None,
                accounts: List::from_boxed(Box::new([2])),
                picker: true,
                name: c::Text::new(Box::from(name), 32).unwrap(),
                provider: Some(17),
                prices: c::ModelPrices { input: 1, cached: 1, cache_write: None, output: 1, unit: 1000 },
                description: None,
                window: 1024,
                output: 128,
                efforts: List::with_capacity(0),
            })
            .unwrap();
    }
    me.projects.get_mut(0).unwrap().models = models;
    me.projects.get_mut(0).unwrap().defaults = List::from_boxed(Box::new([c::MeDefault {
        template: jig_client::shapes::CHAT,
        choice: c::LlmChoice {
            later: None,
            account: None,
            main: c::ModelEffort {
                model: c::Text::new(Box::from(crate::words::SCRIPTED_MODEL), 32).unwrap(),
                effort: None,
            },
            subagents: List::with_capacity(0),
        },
    }]));
    let mut charters = List::with_capacity(2);
    charters.push(1).expect("chat charter");
    if let temper_client_domain::Address::Ask { .. } = address {
        charters.push(37).unwrap();
    }
    me.projects.get_mut(0).expect("playground project").creating = Some(c::Creating {
        within: c::CreateBounds {
            authority: Box::new(c::AuthorityDocument::complete(c::Authority {
                models: None,
                tools: 0,
                grants: List::with_capacity(0),
                delegation: c::Delegation { kinds: List::with_capacity(0), tasks: 1, depth: 2 },
                spend: 100,
                deadline: None,
                notes: 0,
                note_resources: List::with_capacity(0),
            })),
            pool_left: 100,
            charters,
            procedures: List::with_capacity(0),
            latest: None,
            budget: 100,
            maximum_budget: 100,
            maximum_deadline: None,
        },
        beyond: c::Beyond::Refused(c::Refusal::NotOffered),
    });
    root_event(
        &mut root,
        temper_client_domain::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: me_watch,
                    event: jig_client::contract::WatchEvent::Delivered(jig_client::contract::Delivery::Me(Box::new(
                        me,
                    ))),
                },
            },
        },
    );
    if let temper_client_domain::Address::Chat { task: number } = address {
        let mut task = task::document();
        task.chip.task = number;
        task.chip.title = c::DocumentText::complete(c::Text::new(b"Ship the update".as_slice().into(), 32).unwrap());
        task.project = PROJECT;
        task.llm = llm;
        task.kept_account = Some(2);
        task.budget.funder = c::Funder::Pool { project: PROJECT, person: 73, period: 1 };
        task.requester = c::Requester::Person(root.pages().me(root.client()).unwrap().party.number);
        task.acts = List::with_capacity(1);
        task.acts
            .push(c::Offered { act: c::ActKind::Message, admission: c::Admission::Allowed, effect: None })
            .unwrap();
        root_event(
            &mut root,
            temper_client_domain::Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Watched {
                        watch: task_watch.unwrap(),
                        event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task))),
                    },
                },
            },
        );
    }
    (root, accounts_watch.unwrap())
}

/// T1 keeps its playground account after another provider's key becomes usable.
#[must_use]
pub fn playground_kept_root() -> temper_client_domain::Domain {
    let address = temper_client_domain::Address::Chat { task: 1 };
    let (mut root, me_watch, task_watch, _, account_watch) =
        root_at_with_playground(address.clone(), &domain_limits(), b"Shop", Some((PROJECT, false)));
    let template = playground_root(address, PROJECT, false);
    let mut me = template.pages().me(template.client()).unwrap().clone();
    let mut models = List::with_capacity(3);
    let mut other = me.projects.get(0).unwrap().models.get(0).unwrap().clone();
    other.provider = Some(18);
    other.name = c::Text::new(Box::from(b"other".as_slice()), 32).unwrap();
    models.push(other).unwrap();
    for model in &me.projects.get(0).unwrap().models {
        models.push(model.clone()).unwrap();
    }
    me.projects.get_mut(0).unwrap().models = models;
    let mut doc = temper_client_domain::pages::accounts::document(root.client()).unwrap().clone();
    let mut rows = List::with_capacity(2);
    let mut other = doc.accounts.get(1).unwrap().clone();
    other.sources = doc.accounts.get(0).unwrap().sources.clone();
    other.sources.get_mut(0).unwrap().kind = c::AccountKind::Key;
    rows.push(other).unwrap();
    rows.push(doc.accounts.get(0).unwrap().clone()).unwrap();
    doc.accounts = rows;
    let mut readiness = List::with_capacity(2);
    readiness
        .push(c::PayerReadiness {
            provider: 18,
            payer: c::AccountUse::Party(73),
            readiness: c::Readiness::Usable,
            settled: Some(true),
        })
        .unwrap();
    readiness.push(*doc.readiness.get(0).unwrap()).unwrap();
    doc.readiness = readiness;
    let mut task = template.pages().chat().unwrap().task(template.client()).unwrap().clone();
    task.llm = Some(c::LlmChoice {
        later: None,
        account: None,
        main: c::ModelEffort {
            model: c::Text::new(Box::from(crate::words::SCRIPTED_MODEL), 32).unwrap(),
            effort: None,
        },
        subagents: List::with_capacity(0),
    });
    for (watch, value) in [
        (me_watch, c::Delivery::Me(Box::new(me))),
        (account_watch.unwrap(), c::Delivery::Accounts(Box::new(doc))),
        (task_watch.unwrap(), c::Delivery::Task(Box::new(task))),
    ] {
        root_event(
            &mut root,
            temper_client_domain::Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Watched { watch, event: c::WatchEvent::Delivered(value) },
                },
            },
        );
    }
    root
}

#[must_use]
pub fn hidden_playground_root(ready: Option<c::Readiness>, payer: u64) -> temper_client_domain::Domain {
    let address = temper_client_domain::Address::Chat { task: 1 };
    let (mut root, me_watch, task_watch, _, account_watch) =
        root_at_with_playground(address.clone(), &domain_limits(), b"Shop", Some((PROJECT, true)));
    let template = playground_root(address, PROJECT, true);
    let me = template.pages().me(template.client()).unwrap().clone();
    let mut task = template.pages().chat().unwrap().task(template.client()).unwrap().clone();
    task.llm = Some(c::LlmChoice {
        later: None,
        account: None,
        main: c::ModelEffort {
            model: c::Text::new(Box::from(crate::words::SCRIPTED_MODEL), 32).unwrap(),
            effort: None,
        },
        subagents: List::with_capacity(0),
    });
    task.budget.funder = c::Funder::Pool { project: PROJECT, person: payer, period: 1 };
    let mut doc = playground_accounts(true);
    doc.accounts.clear();
    match ready {
        Some(ready) => doc.readiness.get_mut(0).unwrap().readiness = ready,
        None => doc.readiness.clear(),
    }
    for (watch, value) in [
        (me_watch, c::Delivery::Me(Box::new(me))),
        (task_watch.unwrap(), c::Delivery::Task(Box::new(task))),
        (account_watch.unwrap(), c::Delivery::Accounts(Box::new(doc))),
    ] {
        root_event(
            &mut root,
            temper_client_domain::Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Watched { watch, event: c::WatchEvent::Delivered(value) },
                },
            },
        );
    }
    root
}

#[must_use]
pub fn playground_provider() -> c::ProviderOffer {
    c::ProviderOffer {
        provider: 17,
        name: c::DocumentText::complete(c::Text::new(Box::from(crate::words::SCRIPTED_MODEL), 32).unwrap()),
        key_bytes: 4096,
        sign_in: false,
        key: true,
        programs: List::with_capacity(0),
        programs_cut: false,
        none: false,
        signed_in_plan: false,
        manage_usage: None,
    }
}

#[must_use]
pub fn playground_accounts(model: bool) -> c::Accounts {
    let mut rows = List::with_capacity(2);
    let mut sources = List::with_capacity(1);
    sources
        .push(c::AccountSource {
            name: None,
            place: 1,
            number: 2,
            kind: c::AccountKind::None,
            state: c::AccountState::Granted,
            usable: true,
            expiring: false,
            attention: false,
            spent_until: None,
        })
        .unwrap();
    rows.push(c::AccountLine {
        list: None,
        facts: List::with_capacity(0),
        number: 2,
        place: 1,
        uses: None,
        plan: false,
        provider: Some(17),
        provider_name: None,
        manage_usage: None,
        display: c::DocumentText::complete(c::Text::new(Box::from(b"Playground".as_slice()), 32).unwrap()),
        holder: c::AccountHolder::Deployment,
        sources,
        cut: false,
    })
    .unwrap();
    if !model {
        rows.push(c::AccountLine {
            list: None,
            facts: List::with_capacity(0),
            number: 1,
            place: 1,
            uses: None,
            plan: false,
            provider: Some(18),
            provider_name: None,
            manage_usage: None,
            display: c::DocumentText::complete(c::Text::new(b"Other".as_slice().into(), 32).unwrap()),
            holder: c::AccountHolder::Party(73),
            sources: List::with_capacity(0),
            cut: true,
        })
        .unwrap();
    }
    let mut readiness = List::with_capacity(1);
    readiness
        .push(c::PayerReadiness {
            provider: 17,
            payer: c::AccountUse::Party(73),
            readiness: c::Readiness::Usable,
            settled: Some(true),
        })
        .unwrap();
    c::Accounts {
        program_hosts: List::with_capacity(0),
        readiness,
        readiness_cut: false,
        accounts: rows,
        signing: List::with_capacity(0),
        lendable: false,
        lending: None,
        lent_end: None,
        last_end: None,
        usable: true,
        settled: true,
        cut: false,
    }
}

/// Playground read fixtures (temper/docs/design/client/forge/family.md, 7).
pub fn deliver_playground(
    root: &mut temper_client_domain::Domain,
    read: Token,
    state: w::PlaygroundState,
    forge: u16,
    project: Option<u32>,
) {
    let mut cues = List::with_capacity(2);
    cues.push(Box::from(PLAYGROUND_CUE)).expect("first cue");
    cues.push(Box::from(PLAYGROUND_OTHER_CUE)).expect("second cue");
    let answer = w::PlaygroundAnswer::new(
        &w::CEILINGS,
        w::PlaygroundAnswerParts {
            state,
            project,
            forge,
            provider: 17,
            owner: b"acme".as_slice().into(),
            name: b"shop".as_slice().into(),
            cues,
        },
    )
    .expect("playground answer");
    playground_record(root, read, Some(wire::playground_answer(&answer)));
}
pub fn deliver_deployment(root: &mut temper_client_domain::Domain, read: Token, welcome: bool, owned: bool) {
    let answer = w::DeploymentAnswer::new(
        &w::CEILINGS,
        w::DeploymentAnswerParts { push: None, welcome, owned, program_login: None },
    )
    .expect("deployment answer");
    playground_record(root, read, Some(wire::deployment_answer(&answer)));
}
pub fn deliver_no_playground(root: &mut temper_client_domain::Domain, read: Token) {
    playground_record(root, read, None);
}
pub fn playground_record(root: &mut temper_client_domain::Domain, read: Token, record: Option<Box<[u8]>>) {
    root_event(
        root,
        temper_client_domain::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::ReadDone {
                    read,
                    result: jig_client::Readout::Family { family: wire::FORGE, record },
                },
            },
        },
    );
}
pub fn playground_read_answer(
    root: &mut temper_client_domain::Domain,
    read: Token,
    project: u32,
    state: w::PlaygroundState,
) {
    match state {
        w::PlaygroundState::Starting => {
            deliver_playground(root, read, w::PlaygroundState::Starting, wire::FORGE, Some(project));
        }
        w::PlaygroundState::Running => {
            deliver_playground(root, read, w::PlaygroundState::Running, wire::FORGE, Some(project));
        }
        w::PlaygroundState::Resetting => deliver_no_playground(root, read),
    }
}

pub fn playground_session(
    root: &mut temper_client_domain::Domain,
    read: Token,
    watch: Token,
    project: u32,
    model: bool,
    state: w::PlaygroundState,
) {
    playground_read_answer(root, read, project, state);
    let accounts = playground_accounts(model);
    root_event(
        root,
        temper_client_domain::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Accounts(Box::new(accounts))),
                },
            },
        },
    );
}

/// First-run facts answered, with or without the deployment's playground.
#[must_use]
pub fn welcome_root(step: temper_client_domain::Step, playground: bool) -> temper_client_domain::Domain {
    let address = temper_client_domain::Address::Welcome { step };
    root_at_with_playground(address, &domain_limits(), b"Shop", if playground { Some((PROJECT, true)) } else { None }).0
}

/// A flow's rows delivered through the session watch.
#[must_use]
pub fn forge_root(address: temper_client_domain::Address, rows: List<w::ForgeRow>) -> temper_client_domain::Domain {
    forge_root_accounts(address, rows).0
}

#[must_use]
pub fn forge_root_accounts(
    address: temper_client_domain::Address,
    rows: List<w::ForgeRow>,
) -> (temper_client_domain::Domain, Token) {
    let (root, _, _, _, accounts) = root_at_with_activity(address, &domain_limits(), b"Shop", None, None, Some(rows));
    (root, accounts.unwrap())
}
pub fn welcome_account(root: &mut temper_client_domain::Domain, watch: Token) {
    root_event(
        root,
        temper_client_domain::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Accounts(Box::new(playground_accounts(true)))),
                },
            },
        },
    );
}

/// A playground and explicit forge rows share the session watch.
#[must_use]
pub fn playground_forges_root(
    address: temper_client_domain::Address,
    rows: List<w::ForgeRow>,
) -> temper_client_domain::Domain {
    playground_forges_state_root(address, rows, w::PlaygroundState::Running)
}
#[must_use]
pub fn playground_forges_state_root(
    address: temper_client_domain::Address,
    rows: List<w::ForgeRow>,
    state: w::PlaygroundState,
) -> temper_client_domain::Domain {
    root_at_with_activity(address, &domain_limits(), b"Shop", Some((PROJECT, true, state)), None, Some(rows)).0
}

pub fn playground_start_attempt(root: &mut temper_client_domain::Domain) -> Token {
    let limits = domain_limits();
    let mut out = Queue::uncapped();
    temper_client_domain::step(
        root,
        &Env { now: Time::ZERO, wall: NOW, limits },
        temper_client_domain::Event::Act {
            action: temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Playground),
        },
        &mut out,
    );
    while let Some(request) = out.pop() {
        if let temper_client_domain::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Send { attempt, request: jig_client::Ask::Family { record, .. }, .. },
                },
        } = request
            && *wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().request() == w::Request::StartPlayground
        {
            return attempt;
        }
    }
    panic!("playground start sent");
}
pub fn playground_start_answer(
    root: &mut temper_client_domain::Domain,
    attempt: Token,
    outcome: w::Outcome,
) -> Option<Token> {
    let limits = domain_limits();
    let mut out = Queue::uncapped();
    temper_client_domain::step(
        root,
        &Env { now: Time::ZERO, wall: NOW, limits },
        temper_client_domain::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    why: None,
                    answer: Some(jig_client::Answered::Family(jig_client::Family {
                        family: wire::FORGE,
                        version: wire::CLIENT_VERSION,
                        bytes: wire::answer(&w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome }).unwrap()),
                    })),
                },
            },
        },
        &mut out,
    );
    while let Some(request) = out.pop() {
        if let temper_client_domain::Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Read { read, .. } },
        } = request
        {
            return Some(read);
        }
    }
    None
}

/// A forge flow that can receive subsequent watch deliveries.
#[must_use]
pub fn forge_root_watch(
    address: temper_client_domain::Address,
    rows: List<w::ForgeRow>,
) -> (temper_client_domain::Domain, Token) {
    let (root, _, _, _, _, watch) = root_at_with_forges(address, &domain_limits(), b"Shop", None, None, Some(rows));
    (root, watch.expect("forges watch"))
}

#[must_use]
pub fn pair(
    first: temper_client_domain::Domain,
    second: temper_client_domain::Domain,
) -> Box<[temper_client_domain::Domain]> {
    let mut roots = List::with_capacity(2);
    roots.push(first).expect("first root");
    roots.push(second).expect("second root");
    roots.into_boxed()
}

/// A signed-in root with no project and a deployment without the welcome.
#[must_use]
pub fn no_project_root() -> temper_client_domain::Domain {
    chats::no_project_root(Some(false))
}
#[must_use]
pub fn reading_me_at(address: temper_client_domain::Address) -> temper_client_domain::Domain {
    use temper_client_domain::Event;
    let mut root = temper_client_domain::unlinked(&domain_limits(), SEED);
    for event in [
        Event::Client { event: jig_client::Event::StartUnlinked { kept: List::uncapped() } },
        Event::Start { address, kept: List::uncapped(), with: Some(jig_client::With::Transport) },
        Event::Client { event: jig_client::Event::Up { up: jig_client::Up::Served { served: true, versions: None } } },
    ] {
        root_event(&mut root, event);
    }
    root
}
pub fn deliver_me(root: &mut temper_client_domain::Domain, watch: Token, me: c::Me) {
    root_event(
        root,
        temper_client_domain::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched { watch, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))) },
            },
        },
    );
}

/// Template 5's charter deliberately differs from its number and its name is editable.
#[must_use]
pub fn asking_me(mut me: c::Me) -> c::Me {
    let project = me.projects.get_mut(0).expect("project");
    let stock = match project.templates.get(0) {
        Some(stock) => stock.clone(),
        None => {
            let source = chat::messaging_root();
            source.pages().me(source.client()).unwrap().projects.get(0).unwrap().templates.get(0).unwrap().clone()
        }
    };
    let mut asking = stock.clone();
    asking.number = 5;
    asking.name = c::DocumentText::complete(first_run::text(b"A renamed role"));
    asking.executor = c::TemplateExecutor::Charter(37);
    let mut custom = stock.clone();
    custom.number = 7;
    custom.executor = c::TemplateExecutor::Charter(38);
    project.templates = List::with_capacity(3);
    project.templates.push(stock).expect("stock");
    project.templates.push(custom).expect("custom conversation template");
    project.templates.push(asking).expect("asking");
    me
}

#[must_use]
pub fn root_with_classes(
    address: temper_client_domain::Address,
    classes: &temper_protocol_reaches::Classes,
) -> temper_client_domain::Domain {
    root_at_with_classes(
        address,
        &domain_limits(),
        b"Shop",
        None,
        None,
        None,
        Some(&[]),
        Some(forge::ways::Ways::STOCK),
        false,
        true,
        Some(classes),
    )
    .0
}
#[must_use]
pub fn classes_bytes(classes: &temper_protocol_reaches::Classes) -> Box<[u8]> {
    let record = temper_protocol_reaches::classes_record(classes).unwrap();
    let mut writer = Writer::new(usize::try_from(record.measure()).unwrap());
    record.encode(&mut writer).unwrap();
    writer.finish()
}
#[must_use]
pub fn ways_bytes(ways: forge::ways::Ways) -> Box<[u8]> {
    let value = w::ProjectWays::new(
        &w::CEILINGS,
        w::ProjectWaysParts { drafts: ways.drafts, goals: ways.goals, changes: ways.changes, way: ways.way },
    )
    .unwrap();
    let mut writer = Writer::new(usize::try_from(value.measure()).unwrap());
    value.encode(&mut writer).unwrap();
    writer.finish()
}

/// The party's Away watch and project frame, without an optimistic row.
#[must_use]
pub fn away_root(until: Option<Wall>, off: Option<Wall>) -> temper_client_domain::Domain {
    away_frame(until, off).0
}
#[must_use]
pub fn away_frame(until: Option<Wall>, off: Option<Wall>) -> (temper_client_domain::Domain, Token) {
    let (mut root, me, _, _, _, _, _, _, watch) = root_at_with_values(
        temper_client_domain::Address::Home { project: PROJECT },
        &domain_limits(),
        b"Shop",
        None,
        None,
        None,
        Some(&[]),
        Some(forge::ways::Ways::STOCK),
        false,
        true,
    );
    let row = w::AwayRow::new(
        &w::CEILINGS,
        w::AwayRowParts {
            project: PROJECT,
            since: 0,
            until: match until {
                Some(wall) => Some(wall.as_nanos()),
                None => None,
            },

            landed: 0,
            off: match off {
                Some(wall) => Some(wall.as_nanos()),
                None => None,
            },
        },
    )
    .unwrap();
    root_event(
        &mut root,
        temper_client_domain::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::FamilyDelivered {
                    watch: watch.expect("Away watch"),
                    record: jig_client::Family {
                        family: wire::FORGE,
                        version: wire::CLIENT_VERSION,
                        bytes: wire::aways(
                            &w::Aways::new(&w::CEILINGS, w::AwaysParts { rows: List::from_boxed(Box::new([row])) })
                                .unwrap(),
                        ),
                    },
                },
            },
        },
    );
    (root, me)
}
/// Send the switch's act through the domain and return its transport attempt.
pub fn away_toggle(root: &mut temper_client_domain::Domain) -> Token {
    root_event(
        root,
        temper_client_domain::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::FamilyServed {
                    family: wire::FORGE,
                    versions: Some(c::VersionRange { first: wire::CLIENT_VERSION, last: wire::CLIENT_VERSION }),
                },
            },
        },
    );
    let mut out = Queue::uncapped();
    temper_client_domain::step(
        root,
        &Env { now: Time::ZERO, wall: NOW, limits: domain_limits() },
        temper_client_domain::Event::Act {
            action: temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Away(
                temper_client_domain::pages::away::Act::Toggle { project: PROJECT },
            )),
        },
        &mut out,
    );
    while let Some(request) = out.pop() {
        if let temper_client_domain::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Send { attempt, request: jig_client::Ask::Family { record, .. }, .. },
                },
        } = request
            && let w::Request::SetAway(_) = wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().request()
        {
            return attempt;
        }
    }
    panic!("Away request sent")
}
/// A page whose watches open but whose list reads remain under the test's control.
pub struct ReadPage {
    pub root: temper_client_domain::Domain,
    pub out: Queue<temper_client_domain::Request>,
    pub inbox: Option<Token>,
    pub me: Option<Token>,
    pub chats: Option<Token>,
    pub reads: List<(Token, c::Read)>,
}
impl ReadPage {
    #[must_use]
    pub fn new(address: temper_client_domain::Address) -> Self {
        use jig_client::{Down, Readable, Up, Watchable};
        use temper_client_domain::{Event, Request};
        let mut page = Self {
            root: temper_client_domain::unlinked(&domain_limits(), SEED),
            out: Queue::uncapped(),
            inbox: None,
            me: None,
            chats: None,
            reads: List::uncapped(),
        };
        page.event(Event::Start { address, kept: List::with_capacity(0), with: Some(jig_client::With::Transport) });
        page.up(Up::Served { served: true, versions: None });
        while let Some(request) = page.out.pop() {
            let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down } } = request else { continue };
            match down {
                Down::Open { watch, subject: Watchable::Jig(c::Subject::Me) } => {
                    page.me = Some(watch);
                    let source = chat::messaging_root();
                    let mut me = source.pages().me(source.client()).unwrap().clone();
                    me.projects.get_mut(0).unwrap().name = fixture_me(b"Shop").projects.get(0).unwrap().name.clone();
                    page.up(Up::Watched { watch, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))) });
                }
                Down::Open { watch, subject: Watchable::Jig(c::Subject::Inbox { .. }) } => {
                    page.inbox = Some(watch);
                    page.up(Up::Watched { watch, event: c::WatchEvent::Opened });
                }
                Down::Open { watch, subject: Watchable::Jig(c::Subject::Tasks { .. }) } => {
                    page.chats = Some(watch);
                    page.up(Up::Watched { watch, event: c::WatchEvent::Opened });
                }
                Down::Open { watch, subject }
                    if Some(subject.clone()) == forge::forges::watch(&domain_limits().forge) =>
                {
                    let rows = List::with_capacity(0);
                    page.up(Up::FamilyDelivered {
                        watch,
                        record: jig_client::Family {
                            family: wire::FORGE,
                            version: wire::CLIENT_VERSION,
                            bytes: wire::forges(
                                &w::Forges::new(&w::CEILINGS, w::ForgesParts { forges: rows, linked: false }).unwrap(),
                            ),
                        },
                    });
                }
                Down::Read { read, what: Readable::Jig(what) } => {
                    page.reads.push((read, what)).unwrap();
                }
                Down::Open { .. }
                | Down::Read { .. }
                | Down::Continue { .. }
                | Down::Close { .. }
                | Down::Save { .. }
                | Down::Erase { .. }
                | Down::Send { .. } => {}
            }
        }
        page
    }
    pub fn event(&mut self, event: temper_client_domain::Event) {
        temper_client_domain::step(
            &mut self.root,
            &Env { now: Time::ZERO, wall: NOW, limits: domain_limits() },
            event,
            &mut self.out,
        );
    }
    pub fn up(&mut self, up: jig_client::Up) {
        self.event(temper_client_domain::Event::Client { event: jig_client::Event::Up { up } });
    }
    pub fn collect(&mut self) {
        while let Some(request) = self.out.pop() {
            if let temper_client_domain::Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: jig_client::Down::Read { read, what: jig_client::Readable::Jig(what) },
                    },
            } = request
            {
                self.reads.push((read, what)).unwrap();
            }
        }
    }
    pub fn drop_inbox(&mut self) {
        self.up(jig_client::Up::WatchEnded { watch: self.inbox.unwrap(), end: c::WatchEnd::Dropped });
        if self.root.pages().inbox().is_none() {
            self.up(jig_client::Up::WatchEnded { watch: self.me.unwrap(), end: c::WatchEnd::Dropped });
        }
        self.collect();
    }
    pub fn live_chats(&mut self, lines: &[c::TaskLine]) {
        self.up(jig_client::Up::Watched {
            watch: self.chats.unwrap(),
            event: c::WatchEvent::Delivered(c::Delivery::Tasks(Box::new(c::Tasks {
                rows: List::from_boxed(lines.into()),
                more: false,
            }))),
        });
        self.collect();
    }
    pub fn refresh_inbox(&mut self) {
        self.up(jig_client::Up::Watched {
            watch: self.inbox.unwrap(),
            event: c::WatchEvent::Delivered(c::Delivery::Inbox(Box::new(c::Inbox {
                needs: List::with_capacity(0),
                updates: List::with_capacity(0),
                next: None,
                party: chat::PARTY,
                cached_needs: 0,
                cached_updates: c::Count { value: 0, limited: false },
                results_seen: 0,
                updates_time_complete: true,
                selection: None,
            }))),
        });
        self.collect();
    }
    pub fn answer(&mut self, read: Token, result: c::ReadResult) {
        self.up(jig_client::Up::ReadDone { read, result: jig_client::Readout::Jig(result) });
        self.collect();
    }
    pub fn closed(&mut self, lines: &[c::TaskLine]) {
        self.collect();
        let mut read = None;
        for (token, what) in &self.reads {
            if let c::Read::Tasks { .. } = what {
                read = Some(*token);
            }
        }
        let read = read.expect("closed read");
        self.answer(
            read,
            c::ReadResult::Value(c::ReadValue::Tasks(Box::new(c::TaskPage {
                live: List::with_capacity(0),
                live_after: None,
                ended: c::EndedGroup::Rows { rows: List::from_boxed(lines.into()), after: None, cut: false },
            }))),
        );
    }
    pub fn sections(&mut self, lost: bool) {
        self.collect();
        let reads = self.reads.clone();
        for (read, what) in &reads {
            if let c::Read::Inbox { .. } = what {
                let result = if lost {
                    c::ReadResult::None(c::ReadNone::Unreachable)
                } else {
                    c::ReadResult::Value(c::ReadValue::Inbox(Box::new(c::InboxPage {
                        rows: List::with_capacity(0),
                        before: None,
                        after: None,
                        cut: false,
                        selection: None,
                    })))
                };
                self.answer(*read, result);
            }
        }
    }
}

/// Answers the list that an empty Chats fixture asserts on.
pub fn answer_empty_chats(root: &mut temper_client_domain::Domain) {
    use temper_client_domain::{Address, Event, Place, Request};
    let mut out = Queue::uncapped();
    let env = Env { now: Time::ZERO, wall: NOW, limits: domain_limits() };
    for address in [
        Address::Inbox { filter: temper_client_domain::address::Filter::ALL },
        Address::Chats { filter: temper_client_domain::ChatsFilter::All },
    ] {
        temper_client_domain::step(root, &env, Event::Went { place: Place::Address(address) }, &mut out);
    }
    while let Some(request) = out.pop() {
        let up = match request {
            Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down:
                            jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Tasks { .. }) },
                    },
            } => jig_client::Up::Watched {
                watch,
                event: c::WatchEvent::Delivered(c::Delivery::Tasks(Box::new(c::Tasks {
                    rows: List::with_capacity(0),
                    more: false,
                }))),
            },
            Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Tasks { .. }) },
                    },
            } => jig_client::Up::ReadDone {
                read,
                result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Tasks(Box::new(c::TaskPage {
                    live: List::with_capacity(0),
                    live_after: None,
                    ended: c::EndedGroup::Rows { rows: List::with_capacity(0), after: None, cut: false },
                })))),
            },
            Request::Screen { .. }
            | Request::KeepLook { .. }
            | Request::KeepSignIn { .. }
            | Request::Tab { .. }
            | Request::Copy { .. }
            | Request::Client { .. }
            | Request::Unlinked { .. }
            | Request::Show { .. }
            | Request::Leave { .. }
            | Request::SignOut
            | Request::Dropped { .. }
            | Request::Push(_)
            | Request::Fact { .. } => continue,
        };
        temper_client_domain::step(root, &env, Event::Client { event: jig_client::Event::Up { up } }, &mut out);
    }
}

#[must_use]
pub fn sheet_root() -> temper_client_domain::Domain {
    let mut rows = List::with_capacity(1);
    let mut row =
        connect_forge::row(1, w::Kind::Forgejo, w::Mode::Owned, connect_forge::connected(), w::Offer::NotOffered)
            .into_parts();
    row.repositories = List::from_boxed(Box::new([w::Adoption::new(
        &w::CEILINGS,
        w::AdoptionParts { owner: b"acme".as_slice().into(), name: b"kept".as_slice().into(), project: PROJECT },
    )
    .unwrap()]));
    row.makings = List::from_boxed(Box::new([w::RepositoryMaking::new(
        &w::CEILINGS,
        w::RepositoryMakingParts {
            org: b"acme".as_slice().into(),
            name: b"copy".as_slice().into(),
            from: w::MakeFrom::Empty,
            into: None,
            state: w::MakingState::Making,
        },
    )
    .unwrap()]));
    rows.push(w::ForgeRow::new(&w::CEILINGS, row).unwrap()).unwrap();
    forge_root(temper_client_domain::Address::Adopt { project: PROJECT }, rows)
}

/// An in-flight model sign-in for the shared waiting page's direct render checks.
#[must_use]
pub fn waiting_root() -> temper_client_domain::Domain {
    connect_model::pasting().0
}
