//! The forge family's records at the clients' boundary (client/engine.md).
//! Trials: temper/docs/design/domain/trial.md, 6.
use crate::generated::client_wire as maps;
use alloc::boxed::Box;
use jig_protocol_engine_clients as kit;
use skein_codec::{Reason, convert};
use skein_lib::{List, Token, Writer};
use temper_client_protocol_wire::v1::Record;
use temper_client_protocol_wire::{self as wire, v1 as w};
use temper_engine_domain as root;
use temper_engine_domain_forge as forge;
pub use temper_engine_domain_forge::parts;
use temper_engine_domain_forge_change as change;
mod checkpoints;
mod findings;
mod history;
mod landing_views;
mod milestones;
mod refusals;
mod ways;
pub(crate) use findings::{finding_blame_act_to_domain, finding_blame_to_record};
pub(crate) use history::{history_cursor_to_domain, history_cursor_to_record, priority_to_record};
pub(crate) use landing_views::{prefix_gate_to_record, train_to_record};
pub use ways::ways_panel;
pub(crate) use ways::{proposal_item_to_record, roles_view_to_record, way_role_to_record};

// Each direction passes only the read-only inputs its owner uses.
pub(crate) enum Context<'a> {
    Receiving(&'a jig_protocol_config::Directories),
    Sending(&'a w::Limits),
}

impl Context<'_> {
    fn limits(&self) -> &w::Limits {
        match self {
            Self::Sending(limits) => limits,
            Self::Receiving(_) => unreachable!("only record translation reads wire limits"),
        }
    }

    fn directories(&self) -> &jig_protocol_config::Directories {
        match self {
            Self::Receiving(directories) => directories,
            Self::Sending(_) => unreachable!("only domain translation resolves program paths"),
        }
    }
}

// client/forge/records.md: connector documents and their wire share every bound.
const _: () = {
    assert!(forge::PULL_ADDRESS_BYTES == wire::ADDRESS_BYTES, "pull address bound");
    assert!(forge::FIRST_NAME_BYTES == w::CEILINGS.first_ask_org, "first repository org bound");
    assert!(forge::FIRST_NAME_BYTES == w::CEILINGS.first_ask_repository, "first repository name bound");
    assert!(forge::PANEL_AHEAD == wire::TASKS_AHEAD, "panel predecessor bound");
    assert!(forge::PANEL_GATES == wire::GATES, "panel gate bound");
    assert!(forge::PANEL_NAME_BYTES == w::CEILINGS.change_panel_owner, "panel owner bound");
    assert!(forge::PANEL_NAME_BYTES == w::CEILINGS.change_panel_name, "panel repository bound");
    assert!(forge::PANEL_NAME_BYTES == w::CEILINGS.change_panel_branch, "panel branch bound");
    assert!(forge::PANEL_NAME_BYTES == w::CEILINGS.change_panel_landing, "panel landing branch bound");
};

/// The start's deployment facts (temper/docs/design/client/forge/records.md).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Deployment {
    pub welcome: bool,
    pub owned: bool,
    pub program_login: Option<Box<[u8]>>,
    pub push: Option<[u8; 65]>,
}

/// Decode the keyed request, preserving its admission envelope and supplied digest.
#[expect(clippy::too_many_lines, reason = "one exhaustive match translates the family's typed requests")]
pub fn to_root(
    directories: &jig_protocol_config::Directories,
    exchange: kit::ToApplication,
    forges: &crate::forges::Forges,
    demo: Option<&crate::settings::Demo>,
    deployment: &Deployment,
    playground: Option<&forge::Playground>,
    limits: &w::Limits,
) -> Result<root::Event, Box<kit::Event>> {
    match exchange {
        kit::ToApplication::ProviderClientChanged { .. } => {
            unreachable!("provider changes are not client family requests")
        }
        kit::ToApplication::Request { reply_to, sign_in, key, family, version, record, digest, scope, carried } => {
            let request = match wire::owner(family) {
                Some(wire::Owner::Forge) if version == wire::CLIENT_VERSION => wire::decode_requesting(&record, limits),
                Some(wire::Owner::Forge | wire::Owner::Pages | wire::Owner::Runner) | None => None,
            };
            let request = match request {
                Some(request) => request.request,
                None => {
                    return Err(Box::new(kit::Event::Application(kit::FromApplication::Undecodable { to: reply_to })));
                }
            };
            let request = match request {
                w::Request::Checkpoint => {
                    return Ok(root::Event::Party(root::PartyInput::Application {
                        scope,
                        reply_to,
                        sign_in,
                        key,
                        digest,
                        carried,
                        request: root::Application::Checkpoint,
                    }));
                }
                w::Request::OpenChange(value) => {
                    return Ok(root::Event::Party(root::PartyInput::Application {
                        scope,
                        reply_to,
                        sign_in,
                        key,
                        digest,
                        carried,
                        request: root::Application::OpenChange(maps::open_change_to_domain(value)),
                    }));
                }
                w::Request::SetDraft(value) => {
                    return Ok(root::Event::Party(root::PartyInput::Application {
                        scope,
                        reply_to,
                        sign_in,
                        key,
                        digest,
                        carried,
                        request: root::Application::Forge(forge::Ask::SetDraft(maps::set_draft_to_domain(value))),
                    }));
                }
                w::Request::TierAct(value) => {
                    return Ok(root::Event::Party(root::PartyInput::Application {
                        scope,
                        reply_to,
                        sign_in,
                        key,
                        digest,
                        carried,
                        request: root::Application::Tier(maps::tier_act_request_to_domain(value)),
                    }));
                }
                w::Request::FollowAct(value) => {
                    let w::FollowActRequest { project, follow, act } = value;
                    let act = maps::follow_act_to_domain(act);
                    return Ok(root::Event::Party(root::PartyInput::Application {
                        scope,
                        reply_to,
                        sign_in,
                        key,
                        digest,
                        carried,
                        request: root::Application::Follow { project, follow, act },
                    }));
                }
                w::Request::SetUp(value) => {
                    let Ok(forge) = maps::making_to_domain(value.forge, &Context::Receiving(directories)) else {
                        return Err(Box::new(kit::Event::Application(kit::FromApplication::Undecodable {
                            to: reply_to,
                        })));
                    };
                    return Ok(root::Event::Party(root::PartyInput::Application {
                        scope,
                        reply_to,
                        sign_in,
                        key,
                        digest,
                        carried,
                        request: temper_engine_domain::Application::Forge(forge::Ask::SetUp {
                            forge,
                            first: setup_first(value.first),
                        }),
                    }));
                }
                value @ (w::Request::Connect(_)
                | w::Request::Replace(_)
                | w::Request::Install(_)
                | w::Request::Accept(_)) => {
                    let request = match value {
                        w::Request::Connect(value) => {
                            let how = match value.how {
                                w::How::ForgejoAdmin(admin) => {
                                    let first = match admin.first {
                                        Some(first) => Some(maps::first_ask_to_domain(first)),
                                        None => None,
                                    };
                                    forge::How::ForgejoAdmin { first }
                                }
                                w::How::ForgejoTokens(_) => forge::How::ForgejoTokens,
                                w::How::GithubApp(app) => forge::How::GitHubApp {
                                    app: core::num::NonZeroU64::new(app.id).expect("decoded nonzero App"),
                                    read_only: app.read_only.unwrap_or(false),
                                },
                                w::How::GithubManifest(manifest) => {
                                    let back = manifest.back;
                                    if let Some(back) = &back
                                        && !return_path(back)
                                    {
                                        return Err(Box::new(kit::Event::Application(
                                            kit::FromApplication::Undecodable { to: reply_to },
                                        )));
                                    }
                                    forge::How::GitHubManifest { back }
                                }
                            };
                            forge::Ask::Connect {
                                forge: match maps::making_to_domain(value.forge, &Context::Receiving(directories)) {
                                    Ok(forge) => forge,
                                    Err(_problem) => {
                                        return Err(Box::new(kit::Event::Application(
                                            kit::FromApplication::Undecodable { to: reply_to },
                                        )));
                                    }
                                },
                                how,
                            }
                        }
                        w::Request::Replace(value) => forge::Ask::Replace { forge: value.forge },
                        w::Request::Accept(value) => {
                            let Some(installation) = core::num::NonZeroU64::new(value.installation) else {
                                return Err(Box::new(kit::Event::Application(kit::FromApplication::Undecodable {
                                    to: reply_to,
                                })));
                            };
                            forge::Ask::Accept { forge: value.forge, installation }
                        }
                        w::Request::Install(value) => {
                            if let Some(back) = &value.back
                                && !return_path(back)
                            {
                                return Err(Box::new(kit::Event::Application(kit::FromApplication::Undecodable {
                                    to: reply_to,
                                })));
                            }
                            forge::Ask::Install { forge: value.forge, back: value.back }
                        }
                        w::Request::DismissMaking(_)
                        | w::Request::MakeRepository(_)
                        | w::Request::Reachable(_)
                        | w::Request::FollowAct(_)
                        | w::Request::OpenChange(_)
                        | w::Request::TierAct(_)
                        | w::Request::Adopt(_)
                        | w::Request::SetUp(_)
                        | w::Request::FileFinding(_)
                        | w::Request::HandFinding(_)
                        | w::Request::TriageFinding(_)
                        | w::Request::SetDraft(_)
                        | w::Request::TryAccount(_)
                        | w::Request::Dismiss(_)
                        | w::Request::AddDevice(_)
                        | w::Request::RemoveDevice(_)
                        | w::Request::SetAway(_)
                        | w::Request::Overturn(_)
                        | w::Request::Looked(_)
                        | w::Request::RemoveForge(_)
                        | w::Request::StartPlayground
                        | w::Request::Checkpoint
                        | w::Request::EndPlayground => {
                            unreachable!("connect or install")
                        }
                    };
                    return Ok(root::Event::Party(root::PartyInput::Application {
                        scope,
                        reply_to,
                        sign_in,
                        key,
                        digest,
                        carried,
                        request: root::Application::Forge(request),
                    }));
                }
                w::Request::RemoveForge(value) => {
                    return Ok(root::Event::Party(root::PartyInput::Application {
                        scope,
                        reply_to,
                        sign_in,
                        key,
                        digest,
                        carried,
                        request: root::Application::Forge(forge::Ask::RemoveForge { forge: value.forge }),
                    }));
                }
                w::Request::StartPlayground => {
                    return Ok(root::Event::Party(root::PartyInput::Application {
                        scope,
                        reply_to,
                        sign_in,
                        key,
                        digest,
                        carried,
                        request: root::Application::Forge(forge::Ask::StartPlayground),
                    }));
                }
                w::Request::EndPlayground => {
                    return Ok(root::Event::Party(root::PartyInput::Application {
                        scope,
                        reply_to,
                        sign_in,
                        key,
                        digest,
                        carried,
                        request: root::Application::Forge(forge::Ask::EndPlayground),
                    }));
                }
                w::Request::Reachable(value) => {
                    return Ok(root::Event::Party(root::PartyInput::Application {
                        scope,
                        reply_to,
                        sign_in,
                        key,
                        digest,
                        carried,
                        request: root::Application::Forge(forge::Ask::Reachable {
                            forge: value.forge,
                            installation: value.installation,
                            page: value.page,
                        }),
                    }));
                }
                w::Request::DismissMaking(value) => {
                    return Ok(root::Event::Party(root::PartyInput::Application {
                        scope,
                        reply_to,
                        sign_in,
                        key,
                        digest,
                        carried,
                        request: root::Application::Forge(forge::Ask::DismissMaking {
                            forge: value.forge,
                            org: value.org,
                            name: value.name,
                            into: value.into,
                        }),
                    }));
                }
                w::Request::MakeRepository(value) => {
                    let from = match value.from {
                        w::MakeFrom::Empty => forge::MakeFrom::Empty,
                        w::MakeFrom::Copy(copy) => {
                            let Ok(source) = wire::source::parse(&copy.address) else {
                                return Err(Box::new(kit::Event::Application(kit::FromApplication::Undecodable {
                                    to: reply_to,
                                })));
                            };
                            forge::MakeFrom::Copy { address: wire::source::address(&source) }
                        }
                    };
                    let into = match value.into {
                        Some(into) => Some(forge::MakeInto {
                            project: into.project,
                            home: into.home,
                            host: match forges.get(value.forge) {
                                Some(forge) => host(&forge.git),
                                None => Box::default(),
                            },
                        }),
                        None => None,
                    };
                    return Ok(root::Event::Party(root::PartyInput::Application {
                        scope,
                        reply_to,
                        sign_in,
                        key,
                        digest,
                        carried,
                        request: root::Application::Forge(forge::Ask::MakeRepository {
                            forge: value.forge,
                            org: value.org,
                            name: value.name,
                            from,
                            into,
                        }),
                    }));
                }
                w::Request::TryAccount(value) => {
                    return Ok(root::Event::Party(root::PartyInput::Application {
                        scope: jig_domain_people::ApplicationScope::Own,
                        reply_to,
                        sign_in,
                        key,
                        digest,
                        carried,
                        request: root::Application::Try { account: value.account, again: value.again },
                    }));
                }
                w::Request::Dismiss(value) => {
                    let what = maps::person_fact_to_domain(value.what);
                    return Ok(root::Event::Party(root::PartyInput::Application {
                        scope,
                        reply_to,
                        sign_in,
                        key,
                        digest,
                        carried,
                        request: root::Application::Dismiss { what },
                    }));
                }
                w::Request::Looked(value) => {
                    return Ok(root::Event::Party(root::PartyInput::Application {
                        scope,
                        reply_to,
                        sign_in,
                        key,
                        digest,
                        carried,
                        request: root::Application::Looked { project: value.project },
                    }));
                }
                w::Request::AddDevice(value) => {
                    return Ok(root::Event::Party(root::PartyInput::Application {
                        scope,
                        reply_to,
                        sign_in,
                        key,
                        digest,
                        carried,
                        request: root::Application::AddDevice(maps::add_device_to_domain(value)),
                    }));
                }
                w::Request::RemoveDevice(value) => {
                    return Ok(root::Event::Party(root::PartyInput::Application {
                        scope,
                        reply_to,
                        sign_in,
                        key,
                        digest,
                        carried,
                        request: root::Application::RemoveDevice { number: value.number },
                    }));
                }
                w::Request::SetAway(value) => {
                    return Ok(root::Event::Party(root::PartyInput::Application {
                        scope,
                        reply_to,
                        sign_in,
                        key,
                        digest,
                        carried,
                        request: root::Application::SetAway {
                            project: value.project,
                            on: value.on,
                            until: match value.until {
                                Some(until) => Some(convert::wall_to_domain(until)),
                                None => None,
                            },
                        },
                    }));
                }
                w::Request::Overturn(value) => {
                    return Ok(root::Event::Party(root::PartyInput::Application {
                        scope,
                        reply_to,
                        sign_in,
                        key,
                        digest,
                        carried,
                        request: root::Application::Overturn { project: value.project, number: value.number },
                    }));
                }
                w::Request::FileFinding(value) => {
                    return Ok(root::Event::Party(root::PartyInput::Application {
                        scope,
                        reply_to,
                        sign_in,
                        key,
                        digest,
                        carried,
                        request: root::Application::FileFinding(maps::file_finding_to_domain(value)),
                    }));
                }
                w::Request::HandFinding(value) => {
                    return Ok(root::Event::Party(root::PartyInput::Application {
                        scope,
                        reply_to,
                        sign_in,
                        key,
                        digest,
                        carried,
                        request: root::Application::HandFinding(maps::hand_finding_to_domain(value)),
                    }));
                }
                w::Request::TriageFinding(value) => {
                    return Ok(root::Event::Party(root::PartyInput::Application {
                        scope,
                        reply_to,
                        sign_in,
                        key,
                        digest,
                        carried,
                        request: root::Application::TriageFinding(maps::triage_finding_to_domain(value)),
                    }));
                }
                w::Request::Adopt(value) => value,
            };
            let a = request;
            let host = match forges.get(a.forge) {
                Some(forge) => host(&forge.git),
                None => Box::default(),
            };
            Ok(root::Event::Party(root::PartyInput::Application {
                scope,
                reply_to,
                sign_in,
                key,
                digest,
                request: temper_engine_domain::Application::Forge(forge::Ask::Adopt(forge::Adoption {
                    project: a.project,
                    home: a.home,
                    forge: a.forge,
                    host,
                    owner: a.owner,
                    name: a.name,
                    prefix: a.prefix,
                    role: maps::role_to_domain(a.role),
                    landing: a.landing,
                    ci: a.ci,
                    checks: a.checks.into_boxed(),
                })),
                carried,
            }))
        }
        kit::ToApplication::Read { reply, sign_in, family, version, record, scope, .. } => {
            let mut answer = None;
            if family == wire::FORGE
                && version == wire::CLIENT_VERSION
                && let Some(reading) = wire::decode_reading(&record, limits)
            {
                let query = match reading.read {
                    w::Read::Deployment => {
                        answer = Some(deployment_answer(deployment, limits));
                        None
                    }
                    w::Read::Playground => {
                        if let (Some(row), Some(demo)) = (playground, demo) {
                            answer = Some(playground_answer(row, demo, limits));
                        }
                        None
                    }
                    w::Read::History(read) => {
                        if scope == jig_domain_people::ApplicationScope::Project(read.project) {
                            forge_settings_query(w::Read::History(read))
                        } else {
                            None
                        }
                    }
                    w::Read::Milestone(read) => {
                        if scope == jig_domain_people::ApplicationScope::Project(read.project) {
                            forge_settings_query(w::Read::Milestone(read))
                        } else {
                            None
                        }
                    }
                    w::Read::Finding(read) => {
                        if scope == jig_domain_people::ApplicationScope::Project(read.project) {
                            forge_settings_query(w::Read::Finding(read))
                        } else {
                            None
                        }
                    }
                    w::Read::View(read) => {
                        if scope == jig_domain_people::ApplicationScope::Project(read.project) {
                            forge_settings_query(w::Read::View(read))
                        } else {
                            None
                        }
                    }
                    read @ (w::Read::Limits(_)
                    | w::Read::LimitHolds(_)
                    | w::Read::Setting(_)
                    | w::Read::RevisionSetting(_)
                    | w::Read::ProposalItems(_)
                    | w::Read::ProgramEnvironment) => forge_settings_query(read),
                };
                if let Some(query) = query {
                    return Ok(root::Event::Party(root::PartyInput::ForgeRead { reply, sign_in, scope, query }));
                }
            }
            Err(Box::new(kit::Event::Application(kit::FromApplication::Read { reply, family, record: answer })))
        }
        kit::ToApplication::Watch { watcher, sign_in, family, version, record, scope, window } => {
            if family == wire::FORGE
                && version == wire::CLIENT_VERSION
                && let Some(watching) = wire::decode_watching(&record, limits)
            {
                let subject = match watching.watch {
                    w::Watch::Forges => jig_domain_core::documents::Watched::Application {
                        scope: jig_domain_people::ApplicationScope::Deployment,
                        connector: wire::FORGE,
                        subject: Token::new(0),
                    },
                    w::Watch::Change(subject) if subject.task > 0 => jig_domain_core::documents::Watched::Application {
                        scope,
                        connector: wire::FORGE,
                        subject: convert::token_to_domain(subject.task),
                    },
                    w::Watch::Landing(subject) if subject.task > 0 => {
                        jig_domain_core::documents::Watched::Application {
                            scope,
                            connector: wire::FORGE,
                            subject: convert::token_to_domain(subject.task),
                        }
                    }
                    w::Watch::Follows(subject) => jig_domain_core::documents::Watched::Application {
                        scope,
                        connector: wire::FORGE,
                        subject: Token::new(forge::follows_subject(subject.project)),
                    },
                    w::Watch::Landings(subject) => jig_domain_core::documents::Watched::Application {
                        scope,
                        connector: wire::FORGE,
                        subject: Token::new(forge::landings_subject(subject.project, subject.ended)),
                    },
                    w::Watch::Changes(subject) => jig_domain_core::documents::Watched::Application {
                        scope,
                        connector: wire::FORGE,
                        subject: Token::new(forge::changes_subject(subject.project)),
                    },
                    w::Watch::Ways(subject) => jig_domain_core::documents::Watched::Application {
                        scope: jig_domain_people::ApplicationScope::Project(subject.project),
                        connector: wire::FORGE,
                        subject: Token::new(root::ways::subject(subject.project)),
                    },
                    w::Watch::Approvals => jig_domain_core::documents::Watched::Application {
                        scope: jig_domain_people::ApplicationScope::Own,
                        connector: wire::FORGE,
                        subject: Token::new(root::APPROVALS_SUBJECT),
                    },
                    w::Watch::Trials => jig_domain_core::documents::Watched::Application {
                        scope: jig_domain_people::ApplicationScope::Own,
                        connector: wire::FORGE,
                        subject: Token::new(root::TRIALS_SUBJECT),
                    },
                    w::Watch::Dismissed => jig_domain_core::documents::Watched::Application {
                        scope: jig_domain_people::ApplicationScope::Own,
                        connector: wire::FORGE,
                        subject: Token::new(forge::DISMISSED_SUBJECT),
                    },
                    w::Watch::Status(subject) => jig_domain_core::documents::Watched::Application {
                        scope,
                        connector: wire::FORGE,
                        subject: Token::new(forge::status_subject(subject.project)),
                    },
                    w::Watch::Devices => jig_domain_core::documents::Watched::Application {
                        scope: jig_domain_people::ApplicationScope::Own,
                        connector: wire::FORGE,
                        subject: Token::new(forge::DEVICES_SUBJECT),
                    },
                    w::Watch::Away => jig_domain_core::documents::Watched::Application {
                        scope: jig_domain_people::ApplicationScope::Own,
                        connector: wire::FORGE,
                        subject: Token::new(forge::AWAY_SUBJECT),
                    },
                    w::Watch::Findings(subject) => jig_domain_core::documents::Watched::Application {
                        scope: jig_domain_people::ApplicationScope::Project(subject.project),
                        connector: wire::FORGE,
                        subject: Token::new(
                            forge::findings_subject(subject.project, maps::findings_filter_to_domain(subject.filter))
                                .unwrap_or(u64::MAX),
                        ),
                    },
                    w::Watch::Decided(subject) => jig_domain_core::documents::Watched::Application {
                        scope: jig_domain_people::ApplicationScope::Project(subject.project),
                        connector: wire::FORGE,
                        subject: Token::new(forge::decided_subject(subject.project)),
                    },
                    w::Watch::Drafts(subject) => jig_domain_core::documents::Watched::Application {
                        scope: jig_domain_people::ApplicationScope::Project(subject.project),
                        connector: wire::FORGE,
                        subject: Token::new(forge::drafts_subject(subject.project)),
                    },
                    w::Watch::Change(_) | w::Watch::Landing(_) => {
                        return Err(Box::new(kit::Event::WatchRefused {
                            watcher,
                            why: jig_domain_core::WatchRefusal::Unknown,
                        }));
                    }
                };
                return Ok(root::Event::Party(root::PartyInput::Watch { watcher, sign_in, subject, window }));
            }
            Err(Box::new(kit::Event::WatchRefused { watcher, why: jig_domain_core::WatchRefusal::Unknown }))
        }
    }
}

/// Encode the whole retained panel, deriving its public address from configuration
/// (client/forge/served.md, 2). No domain record carries a deployment's web address.
#[must_use]
pub fn panel(value: forge::Panel, forges: &crate::forges::Forges, limits: &w::Limits) -> Option<Box<[u8]>> {
    let landing_hold = value.landing_hold();
    let pull = match value.pull {
        Some(number) => Some(w::Pull { number, address: address(&value, number, forges) }),
        None => None,
    };
    let gates = panel_gates(value.gates)?;
    let approvals = panel_approvals(value.approvals, limits)?;
    let unmapped_reviews = panel_unmapped(value.unmapped_reviews, limits)?;
    let record = w::ChangePanel {
        way: maps::way_to_record(&value.way),
        in_landing: value.in_landing,
        task: value.task,
        forge: value.forge,
        owner: value.owner,
        name: value.name,
        branch: value.branch,
        landing: value.landing,
        head: value.head,
        pull,
        step: panel_step(value.step, limits)?,
        hold: if let Some(reason) = landing_hold {
            Some(w::Hold::Authority(w::AuthorityHold { reason: Box::from(reason) }))
        } else {
            panel_hold(value.hold)
        },
        wait: match &value.wait {
            Some(value) => Some(maps::change_wait_to_record(value)),
            None => None,
        },
        runs: value.runs,
        ci_required: value.ci_required,
        ci: maps::ci_to_record(&value.ci),
        gates,
        approvals,
        unmapped_reviews,
        repairs: maps::used_to_record(&value.repairs),
        resolutions: maps::used_to_record(&value.resolutions),
        updates: maps::used_to_record(&value.updates),
        questions: panel_questions(value.questions, limits)?,
        source: match &value.source {
            Some(value) => Some(maps::panel_source_to_record(value)),
            None => None,
        },
    };
    record.encode(limits).ok()
}

fn panel_gates(value: Box<[forge::PanelGate]>) -> Option<List<w::Gate>> {
    let mut gates = List::with_capacity(forge::PANEL_GATES);
    for gate in &value {
        gates.push(maps::gate_to_record(gate)).ok()?;
    }
    Some(gates)
}

#[expect(clippy::trivially_copy_pass_by_ref, reason = "the generated adapter borrows the domain value")]
pub(crate) fn gate_kind_to_record(value: &change::GateKind) -> w::GateKind {
    match value {
        change::GateKind::Agent => w::GateKind::Review,
        change::GateKind::Person => w::GateKind::Approval,
        change::GateKind::Check | change::GateKind::Follow => w::GateKind::Check,
    }
}
fn panel_approvals(value: Box<[forge::PanelApproval]>, limits: &w::Limits) -> Option<List<w::Approval>> {
    let mut approvals = List::with_capacity(limits.change_panel_approvals);
    for line in &value {
        if line.counted.len() > skein_lib::usize_of(limits.approval_counted) {
            return None;
        }
        approvals.push(maps::approval_to_record(line).ok()?).ok()?;
    }
    Some(approvals)
}

pub(crate) fn counted_review_to_record(value: &forge::Reviewer) -> w::CountedReview {
    w::CountedReview { person: value.person, head: value.head }
}
fn panel_unmapped(value: Box<[forge::UnmappedReview]>, limits: &w::Limits) -> Option<List<w::UnmappedReview>> {
    let mut reviews = List::with_capacity(limits.change_panel_unmapped_reviews);
    for review in value {
        reviews
            .push(w::UnmappedReview {
                account: review.account,
                head: review.head,
                verdict: match review.verdict {
                    skein_forge::api::Verdict::Approve => w::Verdict::Approved,
                    skein_forge::api::Verdict::Comment => w::Verdict::None,
                    skein_forge::api::Verdict::RequestChanges => w::Verdict::Changes,
                },
            })
            .ok()?;
    }
    Some(reviews)
}

pub(crate) fn question_to_record(value: &forge::PanelQuestion) -> w::Question {
    let (stands, blocked) = match value.stands {
        forge::PanelStands::Taken => (w::Stands::Taken, None),
        forge::PanelStands::Asked => (w::Stands::Asked, None),
        forge::PanelStands::Held { blocked } => {
            let blocked = match blocked {
                Some(blocked) => Some(maps::question_blocker_to_record(&blocked)),
                None => None,
            };
            (w::Stands::Held, blocked)
        }
    };
    w::Question { asker: value.asker, number: value.number, line: value.line.clone(), stands, blocked }
}

fn panel_questions(value: forge::PanelQuestions, limits: &w::Limits) -> Option<w::Questions> {
    if value.open.len() > 8 {
        return None;
    }
    let record = maps::questions_to_record(&value).ok()?;
    record.measure(limits).ok()?;
    Some(record)
}

fn panel_step(value: forge::PanelStep, limits: &w::Limits) -> Option<w::Step> {
    if let forge::PanelStep::Queued { ahead, .. } = &value
        && ahead.len() > skein_lib::usize_of(forge::PANEL_AHEAD)
    {
        return None;
    }
    let record = maps::step_to_record(&value).ok()?;
    record.measure(limits).ok()?;
    Some(record)
}

pub(crate) fn reason_to_record(value: &change::Repair) -> w::Reason {
    match value {
        change::Repair::Ci => w::Reason::Ci,
        change::Repair::Review { .. } => w::Reason::Review,
        change::Repair::Gate(gate) => w::Reason::Gate(w::GateReason { gate: *gate }),
        change::Repair::Semantic => w::Reason::Semantic,
        change::Repair::LandingAt { gate, .. } | change::Repair::Landing { gate } => {
            w::Reason::Landing(w::GateReason { gate: u64::from(*gate) })
        }
    }
}

fn panel_hold(value: Option<change::Hold>) -> Option<w::Hold> {
    Some(match value? {
        change::Hold::Stalled => w::Hold::Stalled,
        change::Hold::Repairs => w::Hold::Repairs,
        change::Hold::Questions => w::Hold::Questions,
        change::Hold::Resolutions => w::Hold::Resolutions,
        change::Hold::Updates => w::Hold::Updates,
        change::Hold::BranchMoved => w::Hold::Moved,
        change::Hold::BranchMissing => w::Hold::Missing,
        change::Hold::PullClosed => w::Hold::Closed,
        change::Hold::Retargeted => w::Hold::Retargeted,
        change::Hold::Failed | change::Hold::Runner => w::Hold::Failed,
        change::Hold::Flake => w::Hold::Flake,
        change::Hold::NoTest => w::Hold::NoTest,
        change::Hold::Rejected => w::Hold::Rejected,
        change::Hold::Authority(cause) => {
            w::Hold::Authority(w::AuthorityHold { reason: change::authority_words(cause) })
        }
    })
}

fn address(panel: &forge::Panel, number: u64, forges: &crate::forges::Forges) -> Box<[u8]> {
    pull_address(panel.forge, &panel.owner, &panel.name, number, forges)
}
fn pull_address(
    forge_number: u16,
    owner: &[u8],
    name: &[u8],
    number: u64,
    forges: &crate::forges::Forges,
) -> Box<[u8]> {
    let Some(forge) = forges.get(forge_number) else { return Box::default() };
    if forge.mode == forge::Mode::Fake {
        return Box::default();
    }
    let kind = match forge.kind {
        crate::settings::Kind::GitHub => forge::Kind::GitHub,
        crate::settings::Kind::Forgejo => forge::Kind::Forgejo,
    };
    forge::pull_address(kind, &forge.git, forge.web.as_deref(), owner, name, number).unwrap_or_default()
}
/// Keep the authority of an already checked git URL, including its explicit port.
#[must_use]
pub fn host(git: &[u8]) -> Box<[u8]> {
    forge::host(git)
}
/// Encode one durable terminal, or the unknown terminal past its retention.
#[must_use]
#[expect(clippy::too_many_lines, reason = "one exhaustive family outcome translation")]
pub fn answer(
    outcome: Option<temper_engine_domain::ApplicationAnswer>,
    limits: &w::Limits,
    jig: &jig_client_protocol_wire::Limits,
) -> Option<Box<[u8]>> {
    let outcome = match outcome {
        Some(root::ApplicationAnswer::Checkpoint(value)) => checkpoints::answer(value, limits)?,
        Some(root::ApplicationAnswer::Reachable(forge::ReachableAnswer::Page(page))) => {
            if page.rows.len() > skein_lib::usize_of(limits.page_rows) {
                return None;
            }
            w::Outcome::Page(maps::page_to_record(&page).ok()?)
        }
        Some(
            root::ApplicationAnswer::Follow(refusal)
            | root::ApplicationAnswer::Adopt(forge::ApplicationOutcome::Follow(refusal)),
        ) => w::Outcome::Follow(w::FollowOutcome { refusal }),
        Some(root::ApplicationAnswer::Make(answer)) => match answer {
            forge::MakeAnswer::Kept => w::Outcome::MakeKept,
            forge::MakeAnswer::Refused(why) => {
                w::Outcome::MakeRefused(w::MakeRefused { why: maps::make_refusal_to_record(&why) })
            }
        },
        Some(root::ApplicationAnswer::Adopt(
            forge::ApplicationOutcome::MakeKept | forge::ApplicationOutcome::MakeRefused(_),
        )) => return None,
        Some(root::ApplicationAnswer::Tier(value)) => {
            let refusal = match value {
                Ok(()) => None,
                Err(why) => Some(maps::tier_refusal_to_record(&why)),
            };
            w::Outcome::Tier(w::TierOutcome { refusal })
        }
        Some(root::ApplicationAnswer::FileFinding(value)) => findings::file_answer(value)?,
        Some(root::ApplicationAnswer::HandFinding(value)) => findings::hand_answer(value)?,
        Some(root::ApplicationAnswer::TriageFinding(value)) => findings::triage_answer(value)?,
        Some(root::ApplicationAnswer::Try(root::TryAnswer::Tried)) => w::Outcome::Tried,
        Some(root::ApplicationAnswer::Try(root::TryAnswer::NotTheirs)) => w::Outcome::NotTheirs,
        Some(root::ApplicationAnswer::Dismiss(forge::DismissAnswer::Dismissed)) => w::Outcome::Dismissed,
        Some(root::ApplicationAnswer::Looked(forge::LookedAnswer::Looked)) => w::Outcome::Looked,
        Some(root::ApplicationAnswer::Device(answer)) => match answer {
            forge::DeviceAnswer::Added { number } => w::Outcome::DeviceAdded(w::DeviceAdded { number }),
            forge::DeviceAnswer::Removed => w::Outcome::DeviceRemoved,
            forge::DeviceAnswer::Refused(why) => {
                w::Outcome::DeviceRefused(w::DeviceRefused { reason: maps::device_refusal_to_record(&why) })
            }
        },
        Some(root::ApplicationAnswer::SetAway(forge::SetAwayAnswer::Away)) => w::Outcome::Away,
        Some(root::ApplicationAnswer::SetDraft(answer)) => match answer {
            forge::SetDraftAnswer::Kept(view) => w::Outcome::Drafted(landed_draft(&view, limits)?),
            forge::SetDraftAnswer::Refused(why) => {
                w::Outcome::DraftRefused(w::DraftRefused { reason: maps::draft_refusal_to_record(&why) })
            }
        },
        Some(root::ApplicationAnswer::Overturn(answer)) => match answer {
            forge::OverturnAnswer::Overturned => w::Outcome::Overturned,
            forge::OverturnAnswer::Already => {
                w::Outcome::OverturnRefused(w::OverturnRefused { reason: w::OverturnRefusal::Already })
            }
            forge::OverturnAnswer::Unknown => {
                w::Outcome::OverturnRefused(w::OverturnRefused { reason: w::OverturnRefusal::Unknown })
            }
        },
        Some(
            root::ApplicationAnswer::Dismiss(forge::DismissAnswer::TooLarge)
            | root::ApplicationAnswer::Looked(forge::LookedAnswer::TooLarge)
            | root::ApplicationAnswer::SetAway(forge::SetAwayAnswer::TooLarge),
        ) => w::Outcome::ForgeRefused(w::ForgeRefused { reason: w::ForgeReason::TooLarge }),
        Some(root::ApplicationAnswer::Adopt(forge::ApplicationOutcome::Opened { task })) => {
            w::Outcome::Opened(w::Opened { task })
        }
        Some(root::ApplicationAnswer::Adopt(forge::ApplicationOutcome::OpenRefused(reason))) => {
            if let forge::OpenRefusal::Several(names) = &reason
                && names.len() > skein_lib::usize_of(limits.several_repositories_repositories)
            {
                return None;
            }
            w::Outcome::OpenRefused(w::OpenRefused { reason: maps::open_refusal_to_record(&reason).ok()? })
        }
        Some(temper_engine_domain::ApplicationAnswer::Adopt(forge::ApplicationOutcome::Adopted(a))) => {
            w::Outcome::Adopted(w::Adopted {
                forge: a.repository.forge,
                repository: a.repository.repository,
                // A made repository can name a plan branch past the client's adoption bound.
                landing: if a.landing.len() <= skein_lib::usize_of(limits.adopted_landing) {
                    a.landing
                } else {
                    Box::new([])
                },
                owner: a.owner,
                name: a.name,
                role: maps::role_to_record(&a.role),
                home: a.home,
                kinds: maps::kinds_to_record(&a.kinds),
                protection: maps::protection_to_record(&a.protection, &Context::Sending(limits)).ok()?,
            })
        }
        Some(temper_engine_domain::ApplicationAnswer::Adopt(forge::ApplicationOutcome::CoreRefused(value))) => {
            let value = refusals::refusal(value, jig).ok()?;
            let refusal = value.encode(jig).ok()?;
            w::Outcome::CoreRefused(w::CoreRefused { refusal })
        }
        Some(
            temper_engine_domain::ApplicationAnswer::Adopt(forge::ApplicationOutcome::ForgeRefused(reason))
            | root::ApplicationAnswer::Reachable(forge::ReachableAnswer::ForgeRefused(reason))
            | root::ApplicationAnswer::Accept(forge::AcceptAnswer::ForgeRefused(reason)),
        ) => {
            w::Outcome::ForgeRefused(w::ForgeRefused {
                reason: match reason {
                    forge::AdoptionRefusal::NoSuchForge => w::ForgeReason::NoSuchForge,
                    forge::AdoptionRefusal::NotFound => w::ForgeReason::NotFound,
                    // The client wire has no not-installed reason (client/forge/family.md, 6).
                    forge::AdoptionRefusal::NotInstalled | forge::AdoptionRefusal::NoAccess => w::ForgeReason::NoAccess,
                    forge::AdoptionRefusal::Refused => w::ForgeReason::Refused,
                    forge::AdoptionRefusal::InFlight => w::ForgeReason::InFlight,
                    forge::AdoptionRefusal::TooLarge => w::ForgeReason::TooLarge,
                    forge::AdoptionRefusal::Unreachable => w::ForgeReason::Unreachable,
                    forge::AdoptionRefusal::InvalidAnswer => w::ForgeReason::InvalidAnswer,
                    forge::AdoptionRefusal::AnotherProject => w::ForgeReason::AnotherProject,
                    forge::AdoptionRefusal::HomeHeld => w::ForgeReason::HomeHeld,
                    forge::AdoptionRefusal::PrefixOccupied => w::ForgeReason::PrefixOccupied,
                },
            })
        }
        Some(root::ApplicationAnswer::Accept(forge::AcceptAnswer::Accepted { installation })) => {
            w::Outcome::Accepted(w::AcceptedInstallation { installation })
        }
        Some(
            root::ApplicationAnswer::Connect(forge::ConnectAnswer::Connecting)
            | root::ApplicationAnswer::Install(forge::InstallAnswer::Connecting),
        ) => w::Outcome::Connecting,
        Some(
            root::ApplicationAnswer::Install(forge::InstallAnswer::Link(address))
            | root::ApplicationAnswer::Connect(forge::ConnectAnswer::Link(address)),
        ) => w::Outcome::Link(w::Link { address }),
        Some(
            root::ApplicationAnswer::Accept(forge::AcceptAnswer::Refused(why))
            | root::ApplicationAnswer::Reachable(forge::ReachableAnswer::Refused(why))
            | root::ApplicationAnswer::Connect(forge::ConnectAnswer::Refused(why))
            | root::ApplicationAnswer::Install(forge::InstallAnswer::Refused(why)),
        ) => w::Outcome::ConnectRefused(w::ConnectRefused { why: maps::forge_refusal_to_record(&why) }),
        Some(root::ApplicationAnswer::StartPlayground(answer)) => start_answer(answer, limits)?,
        Some(root::ApplicationAnswer::RemoveForge(forge::RemoveAnswer::Removed)) => w::Outcome::Removed,
        Some(root::ApplicationAnswer::RemoveForge(forge::RemoveAnswer::Refused(why))) => {
            w::Outcome::RemoveRefused(w::RemoveRefused { why: maps::forge_refusal_to_record(&why) })
        }
        Some(root::ApplicationAnswer::EndPlayground(forge::EndAnswer::Reset)) => w::Outcome::Reset,
        Some(root::ApplicationAnswer::EndPlayground(forge::EndAnswer::NotOffered)) => {
            w::Outcome::EndRefused(w::EndRefused { why: w::EndRefusal::NotOffered })
        }
        None => w::Outcome::Unknown,
        Some(temper_engine_domain::ApplicationAnswer::SetUp(forge::SetUpAnswer::SettingUp)) => w::Outcome::SettingUp,
        Some(temper_engine_domain::ApplicationAnswer::SetUp(forge::SetUpAnswer::Refused(why))) => {
            w::Outcome::SetUpRefused(w::SetUpRefused { why: maps::set_up_refusal_to_record(&why) })
        }
    };
    let answer = w::Answer { outcome };
    answer.encode(limits).ok()
}
fn start_answer(answer: forge::StartAnswer, limits: &w::Limits) -> Option<w::Outcome> {
    let value = match answer {
        forge::StartAnswer::Running => w::Outcome::Running,
        forge::StartAnswer::NotOffered => {
            w::Outcome::StartRefused(w::StartRefused { why: w::StartRefusal::NotOffered })
        }
        forge::StartAnswer::Failed(why) => {
            w::Outcome::StartFailed(w::StartFailed { why: maps::start_failure_to_record(&why) })
        }
    };
    value.measure(limits).ok()?;
    Some(value)
}
pub(crate) fn rule_to_record(
    value: &skein_forge::api::Protection,
    context: &Context<'_>,
) -> Result<w::Rule, w::Problem> {
    let mut contexts = List::with_capacity(context.limits().rule_contexts);
    for name in &value.contexts {
        if contexts.push(name.clone()).is_err() {
            return Err(w::paths::RULE_CONTEXTS.problem(Reason::Bound));
        }
    }
    Ok(w::Rule {
        branch: value.branch.clone(),
        contexts,
        approvals: value.approvals,
        dismiss_stale: value.dismiss_stale,
    })
}

pub(crate) fn reached_to_record(row: &forge::Reached) -> w::Reached {
    w::Reached {
        id: row.id,
        owner: w::Account { id: row.owner_id, login: row.owner.clone() },
        name: row.name.clone(),
        private: row.private,
    }
}

/// Encode the connector's whole document without adding policy from settings.
#[must_use]
pub fn forges(value: forge::Forges, linked: bool, limits: &w::Limits) -> Option<Box<[u8]>> {
    let mut forges = List::with_capacity(limits.forges_forges);
    for row in value.rows {
        forges.push(forge_row(row, limits)?).ok()?;
    }
    let mut full = List::with_capacity(2);
    for row in value.full {
        full.push(maps::full_table_to_record(&row)).ok()?;
    }
    (w::Forges { forges, full, linked }).encode(limits).ok()
}

fn forge_row(row: forge::Row, limits: &w::Limits) -> Option<w::ForgeRow> {
    let ended = match row.ended {
        Some(why) => Some(w::ConnectStopped { why }),
        None => None,
    };
    let mut makings = List::with_capacity(limits.forge_row_makings);
    for row in row.makings {
        makings.push(repository_making(row, limits)?).ok()?;
    }
    {
        let value = w::ForgeRow {
            catching_up: match &row.catching_up {
                Some(value) => Some(maps::catching_up_to_record(value)),
                None => None,
            },
            number: row.number,
            host: row.host,
            makings,
            read_only: row.read_only,
            kind: maps::kind_to_record(&row.kind),
            mode: maps::mode_to_record(&row.mode),
            connection: connection(row.connection, row.failed, row.fetched, limits)?,
            offer: maps::offer_to_record(&row.offer),
            first: match row.first {
                Some(first) => Some(first_repository(first, limits)?),
                None => None,
            },
            starts_with: starts_with(&row.starts_with, limits)?,
            installations: forge_installations(row.installations, limits)?,
            ended,
            app: match &row.app {
                Some(value) => Some(maps::app_way_to_record(value)),
                None => None,
            },
            web: row.web,
            repositories: forge_adoptions(row.repositories, limits)?,
            owned: owned_forge(row.owned, limits),
            admin: row.admin,
            ended_stop: match row.ended_stop {
                Some(stop) => Some(admin_stop(stop, limits)?),
                None => None,
            },
        };
        value.measure(limits).ok()?;
        Some(value)
    }
}

fn repository_making(row: forge::RepositoryMaking, limits: &w::Limits) -> Option<w::RepositoryMaking> {
    let into = match row.into {
        Some(into) => Some(w::MakeInto { project: into.project, home: into.home }),
        None => None,
    };
    let value = w::RepositoryMaking {
        org: row.key.org,
        name: row.key.name,
        from: maps::make_from_to_record(&row.from),
        into,
        state: maps::making_state_to_record(&row.state),
    };
    value.measure(limits).ok()?;
    Some(value)
}
fn forge_adoptions(values: Box<[forge::AdoptedRow]>, limits: &w::Limits) -> Option<List<w::Adoption>> {
    let mut repositories = List::with_capacity(limits.forge_row_repositories);
    for adopted in values {
        repositories.push(maps::adoption_to_record(&adopted)).ok()?;
    }
    Some(repositories)
}
fn forge_installations(values: Box<[(u64, forge::Installation)]>, limits: &w::Limits) -> Option<List<w::Installation>> {
    let mut installations = List::with_capacity(limits.forge_row_installations);
    for (id, installation) in values {
        let account = match installation.account {
            Some((id, login)) => Some(w::Account { id, login }),
            None => None,
        };
        installations
            .push(w::Installation { id, account, suspended: installation.suspended, stray: installation.stray })
            .ok()?;
    }
    Some(installations)
}
fn owned_forge(owned: Option<forge::Owned>, limits: &w::Limits) -> Option<w::OwnedForge> {
    match owned {
        Some(owned) => {
            let value = maps::owned_forge_to_record(&owned);
            value.measure(limits).ok()?;
            Some(value)
        }
        None => None,
    }
}

fn connection(
    value: forge::Connection,
    failed: Option<forge::FailedStep>,
    fetched: Option<forge::SetUpFetched>,
    limits: &w::Limits,
) -> Option<w::Connection> {
    let value = {
        Some(match value {
            forge::Connection::NotConnected => w::Connection::NotConnected,
            forge::Connection::NotSetUp => w::Connection::NotSetUp,
            forge::Connection::SettingUp { step } => w::Connection::SettingUp(w::SettingUp {
                fetched: match &fetched {
                    Some(value) => Some(maps::set_up_fetched_to_record(value)),
                    None => None,
                },
                step: match &step {
                    Some(value) => Some(maps::set_up_step_to_record(value)),
                    None => None,
                },
            }),
            forge::Connection::SetUpFailed { why } => w::Connection::SetUpFailed(w::SetUpFailed {
                why,
                at: match &failed {
                    Some(value) => Some(maps::failed_step_to_record(value)),
                    None => None,
                },
            }),
            forge::Connection::Connecting => w::Connection::Connecting,
            forge::Connection::Connected { id, login } => w::Connection::Connected(w::Connected { id, login }),
            forge::Connection::Stopped { reason } => w::Connection::Stopped(w::Stopped { reason }),
        })
    }?;
    value.measure(limits).ok()?;
    Some(value)
}

fn first_repository(first: forge::First, limits: &w::Limits) -> Option<w::FirstRepository> {
    let record = maps::first_repository_to_record(&first);
    record.measure(limits).ok()?;
    Some(record)
}

/// A bounded path on this clients listener, checked before root admission (client/engine.md).
#[must_use]
pub fn return_path(path: &[u8]) -> bool {
    if path.is_empty() || path.len() > 512 || path.first() != Some(&b'/') || path.get(1) == Some(&b'/') {
        return false;
    }
    for byte in path {
        if !(0x21..=0x7e).contains(byte) || *byte == b'\\' {
            return false;
        }
    }
    for byte in path {
        match byte {
            b'/' | b'?' | b'#' => break,
            b':' => return false,
            _ => {}
        }
    }
    true
}

fn admin_stop(stop: forge::AdminStop, limits: &w::Limits) -> Option<w::AdminStop> {
    let record = maps::admin_stop_to_record(&stop);
    record.measure(limits).ok()?;
    Some(record)
}

fn project_status(value: forge::StatusView, limits: &w::Limits) -> Option<Box<[u8]>> {
    maps::project_status_to_record(&value).encode(limits).ok()
}

pub(crate) fn status_brief_to_record(brief: &forge::Brief) -> w::StatusBrief {
    w::StatusBrief {
        sentence: brief.sentence.clone(),
        lede: brief.lede.clone(),
        at: convert::wall_to_record(&brief.at),
        landed: brief.landed,
    }
}

fn follows_document(value: forge::FollowsView, limits: &w::Limits) -> Option<Box<[u8]>> {
    if value.follows.len() > skein_lib::usize_of(limits.follows_panel_follows)
        || value.dependencies.len() > skein_lib::usize_of(limits.follows_panel_dependencies)
    {
        return None;
    }
    for row in &value.follows {
        if row.entries.len() > skein_lib::usize_of(limits.follow_view_entries)
            || row.skips.len() > skein_lib::usize_of(limits.follow_view_skips)
        {
            return None;
        }
    }
    let record = maps::follows_panel_to_record(&value).ok()?;
    // The embedded setting also has to fit as a settings kind at its own ceiling.
    for row in &record.follows {
        row.setting.measure(&w::CEILINGS).ok()?;
    }
    record.encode(limits).ok()
}

/// Encode a whole forge document, with deployment addresses at this boundary.
#[must_use]
pub fn document(value: forge::Document, forges: &crate::forges::Forges, limits: &w::Limits) -> Option<Box<[u8]>> {
    match value {
        forge::Document::Change(value) => panel(value, forges, limits),
        forge::Document::Follows(value) => follows_document(value, limits),
        forge::Document::Dismissals(rows) => {
            let mut dismissed = List::with_capacity(u32::try_from(rows.len()).ok()?);
            for row in rows {
                let value = maps::person_fact_to_record(&row);
                dismissed.push(value).ok()?;
            }
            let document = w::Dismissals { dismissed };
            document.encode(limits).ok()
        }
        forge::Document::Landing(value) => {
            let search = if value.round.search.is_some() { Some(landing_views::search(&value, limits)?) } else { None };
            let hold = landing_views::hold(&value.phase, value.read_hold, limits);
            let train = match value.train {
                Some(train) => Some(landing_views::train(&train, limits)?),
                None => None,
            };
            let pull = match (value.round.pull, value.members.first()) {
                (Some(number), Some(member)) => Some(w::Pull {
                    number,
                    address: pull_address(member.forge, &member.owner, &member.name, number, forges),
                }),
                (Some(_), None) | (None, _) => None,
            };
            let round = if value.round.branch.is_empty() {
                None
            } else {
                Some(w::LandingRound {
                    search,
                    number: value.preparation,
                    branch: value.round.branch,
                    pull,
                    combined: match value.round.combined {
                        Some(value) => Some(maps::combined_to_record(&value)),
                        None => None,
                    },
                    step: maps::landing_step_to_record(&value.step),
                    upstream: value.round.upstream,
                    failure: forge::text::fit(
                        &value.round.failure,
                        skein_lib::usize_of(limits.landing_round_failure),
                        b"...",
                    ),
                })
            };
            let mut members = List::with_capacity(u32::try_from(value.members.len()).ok()?);
            for member in value.members {
                members.push(landing_member(member, forges, limits)?).ok()?;
            }
            let panel = w::LandingPanel {
                waits: match value.waits {
                    Some(wait) => Some(landing_views::wait(&wait, limits)?),
                    None => None,
                },
                task: value.task,
                project: value.project,
                phase: landing_phase_to_record(&value.phase),
                hold,
                members,
                preparation: value.preparation,
                stale_proofs: value.stale_proofs,
                turn: value.turn,
                train,
                round,
            };
            panel.encode(limits).ok()
        }
        forge::Document::Devices(value) => devices(value, limits),
        forge::Document::Findings(value) => findings::document(&value, limits),
        forge::Document::Aways(value) => aways(value, limits),
        forge::Document::Decisions(value) => decisions(value, limits),
        forge::Document::Status(value) => project_status(value, limits),
        forge::Document::Changes(value) => changes_panel(value, limits),
        forge::Document::Drafts(value) => drafts_panel(value, limits),
        forge::Document::Landings(value) => maps::landings_panel_to_record(&value).ok()?.encode(limits).ok(),
    }
}
fn aways(value: forge::AwaysView, limits: &w::Limits) -> Option<Box<[u8]>> {
    maps::aways_to_record(&value).ok()?.encode(limits).ok()
}

pub(crate) fn away_row_to_record(row: &forge::Away) -> w::AwayRow {
    w::AwayRow {
        project: row.project,
        since: convert::wall_to_record(&row.since),
        until: match &row.until {
            Some(value) => Some(convert::wall_to_record(value)),
            None => None,
        },
        landed: row.landed,
        off: match &row.off {
            Some(value) => Some(convert::wall_to_record(value)),
            None => None,
        },
    }
}

fn decisions(value: forge::DecisionsView, limits: &w::Limits) -> Option<Box<[u8]>> {
    maps::decisions_to_record(&value).ok()?.encode(limits).ok()
}

fn landed_draft(view: &forge::DraftView, limits: &w::Limits) -> Option<w::LandedDraft> {
    let record = landed_draft_to_record(view);
    record.measure(limits).ok()?;
    Some(record)
}

pub(crate) fn landed_draft_to_record(view: &forge::DraftView) -> w::LandedDraft {
    let row = &view.row;
    w::LandedDraft {
        forge: row.repository.forge,
        owner: view.owner.clone(),
        name: view.name.clone(),
        path: row.path.clone(),
        branch: row.branch.clone(),
        commit: forge::commit_text(&row.commit),
        title: row.title.clone(),
        writer: row.writer,
        at: convert::wall_to_record(&row.at),
        chat: row.chat,
        goal: row.goal,
        ready: row.ready,
    }
}
fn drafts_panel(value: forge::DraftsView, limits: &w::Limits) -> Option<Box<[u8]>> {
    if value.rows.len() > skein_lib::usize_of(forge::DRAFTS) {
        return None;
    }
    maps::drafts_panel_to_record(&value).ok()?.encode(limits).ok()
}

fn changes_panel(value: forge::ChangesView, limits: &w::Limits) -> Option<Box<[u8]>> {
    if value.queues.len() > skein_lib::usize_of(limits.changes_panel_queues)
        || value.landed.len() > skein_lib::usize_of(limits.changes_panel_landed)
        || value.landed_default.len() > skein_lib::usize_of(limits.changes_panel_landed_default)
    {
        return None;
    }
    for queue in &value.queues {
        if queue.changes.len() > skein_lib::usize_of(limits.changes_queue_changes) {
            return None;
        }
    }
    for rows in [&value.landed, &value.landed_default] {
        for row in rows {
            if row.signed.len() > forge::SIGN_OFFS {
                return None;
            }
        }
    }
    maps::changes_panel_to_record(&value).ok()?.encode(limits).ok()
}

fn landing_member(
    member: forge::LandingMember,
    forges: &crate::forges::Forges,
    limits: &w::Limits,
) -> Option<w::Member> {
    let pull = match member.pull {
        Some(number) => {
            Some(w::Pull { number, address: pull_address(member.forge, &member.owner, &member.name, number, forges) })
        }
        None => None,
    };
    let mut gates = List::with_capacity(u32::try_from(member.gates.len()).ok()?);
    for gate in &member.gates {
        gates.push(maps::gate_proof_to_record(gate)).ok()?;
    }
    {
        let value = w::Member {
            head: member.head,
            left: match member.left {
                Some(value) => Some(landing_views::left(value, limits)?),
                None => None,
            },
            judgement: match member.judgement {
                Some(value) => Some(landing_views::judgement(&value, limits)?),
                None => None,
            },
            change: member.change,
            forge: member.forge,
            owner: member.owner,
            name: member.name,
            branch: member.branch,
            landing: member.landing,
            pull,
            base: member.base,
            candidate: member.candidate,
            landed: member.landed,
            gates,
        };
        value.measure(limits).ok()?;
        Some(value)
    }
}
pub(crate) fn landing_phase_to_record(phase: &forge::LandingPhase) -> w::LandingPhase {
    match phase {
        forge::LandingPhase::Waiting => w::LandingPhase::Waiting,
        forge::LandingPhase::Preparing => w::LandingPhase::Preparing,
        forge::LandingPhase::Proving => w::LandingPhase::Proving,
        forge::LandingPhase::Proved { .. } => w::LandingPhase::Proved,
        forge::LandingPhase::Turn { .. } => w::LandingPhase::Turn,
        forge::LandingPhase::Landed => w::LandingPhase::Landed,
        forge::LandingPhase::Held { .. } => w::LandingPhase::Held,
        forge::LandingPhase::Cancelled => w::LandingPhase::Cancelled,
    }
}

// temper/docs/design/protocol/forge/engine.md, 2: a new forge carries its whole place.
pub(crate) fn forge_ask_to_domain(record: w::ForgeAsk, context: &Context<'_>) -> Result<forge::ForgeAsk, w::Problem> {
    Ok(match record {
        w::ForgeAsk::Github => forge::ForgeAsk::GitHub,
        w::ForgeAsk::Owned => forge::ForgeAsk::Owned,
        value @ (w::ForgeAsk::Managed(_) | w::ForgeAsk::Connected(_)) => {
            let (value, connected) = match value {
                w::ForgeAsk::Managed(value) => (value, false),
                w::ForgeAsk::Connected(value) => (value, true),
                w::ForgeAsk::Github | w::ForgeAsk::Owned => unreachable!("Forgejo place"),
            };

            let git_program = match value.git_program {
                Some(program) => {
                    let Ok(program) = context.directories().join(jig_protocol_config::Directory::Data, &program) else {
                        return Err(w::paths::MANAGED_FORGE_GIT_PROGRAM.problem(Reason::Malformed));
                    };
                    Some(program)
                }
                None => None,
            };
            let place = forge::Place {
                api: forge::Address { host: value.host, port: value.port, tls: value.tls },
                git: value.git,
                web: value.web,
            };
            if connected {
                forge::ForgeAsk::Connected { place, git_program }
            } else {
                forge::ForgeAsk::Managed { place, git_program }
            }
        }
    })
}

fn deployment_answer(deployment: &Deployment, limits: &w::Limits) -> Box<[u8]> {
    maps::deployment_answer_to_record(deployment).encode(limits).expect("checked deployment facts fit the wire")
}

/// The kept row supplies progress and ids; settings supply the scripted model's public view.
/// temper/docs/design/protocol/setup/playground.md.
fn playground_answer(row: &forge::Playground, demo: &crate::settings::Demo, limits: &w::Limits) -> Box<[u8]> {
    let mut cues = List::with_capacity(u32::try_from(demo.cues.len()).expect("bounded playground cues"));
    for cue in &demo.cues {
        cues.push(cue.clone()).expect("bounded playground cues");
    }
    let answer = w::PlaygroundAnswer {
        state: match row.state {
            forge::PlaygroundState::Starting => w::PlaygroundState::Starting,
            forge::PlaygroundState::Running => w::PlaygroundState::Running,
            forge::PlaygroundState::Resetting(_) => w::PlaygroundState::Resetting,
        },
        forge: row.forge,
        project: row.project,
        provider: demo.provider,
        owner: demo.owner.clone(),
        name: demo.name.clone(),
        cues,
    };
    answer.encode(limits).expect("playground checked at startup")
}

/// Encode a typed forge read answer (temper/docs/design/client/forge/records.md).
#[must_use]
pub fn forge_read(answer: root::ForgeReadAnswer, program_environment: &[u8], limits: &w::Limits) -> Option<Box<[u8]>> {
    use forge::settings::Edit;
    match answer {
        root::ForgeReadAnswer::Limits { .. } => None, // The composing service owns the settings and names tables.
        root::ForgeReadAnswer::LimitHolds { rows, next } => {
            let mut encoded = List::with_capacity(limits.limit_holds_answer_rows);
            for row in rows {
                encoded
                    .push(w::LimitHold {
                        task: row.task,
                        limit: row.member,
                        configured: row.configured,
                        needed: row.needed,
                        revision: row.revision,
                    })
                    .ok()?;
            }
            let record = w::LimitHoldsAnswer { rows: encoded, next };
            record.encode(limits).ok()
        }
        root::ForgeReadAnswer::History(value) => history::answer(value, limits),
        root::ForgeReadAnswer::Milestone(value) => milestones::answer(value, limits),
        root::ForgeReadAnswer::Finding(value) => findings::read_answer(value, limits),
        root::ForgeReadAnswer::View(view) => ways::view_answer(&view, limits),
        root::ForgeReadAnswer::ProgramEnvironment => {
            let record = w::ProgramEnvironmentAnswer { environment: program_environment.into() };
            record.encode(limits).ok()
        }
        root::ForgeReadAnswer::Setting { value, revision } => {
            let value = match value {
                Some(value) => Some(temper_engine_smith::kinds::encode(&value.value)),
                None => None,
            };
            let record = w::SettingAnswer { value, revision };
            record.encode(limits).ok()
        }
        root::ForgeReadAnswer::ProposalItems { items: values, owned_approval } => {
            let mut items = List::with_capacity(limits.proposal_items_answer_items);
            for (key, edit) in values {
                let edit = match edit {
                    Edit::Set(value) => {
                        w::SettingEdit::Set(w::SettingValue { value: temper_engine_smith::kinds::encode(&value.value) })
                    }
                    Edit::Remove => w::SettingEdit::Removed,
                    Edit::Reset => w::SettingEdit::Reset,
                };
                items.push(w::ProposalItem { kind: key.kind, scope: key.scope, number: key.number, edit }).ok()?;
            }
            let record = w::ProposalItemsAnswer { items, owned_approval };
            record.encode(limits).ok()
        }
    }
}

fn forge_settings_query(read: w::Read) -> Option<root::ForgeRead> {
    match read {
        w::Read::Limits(value) => Some(root::ForgeRead::Limits { after: value.after }),
        w::Read::LimitHolds(value) => Some(root::ForgeRead::LimitHolds { after: value.after }),
        w::Read::History(value) => Some(root::ForgeRead::History(history::query(value)?)),
        w::Read::Finding(value) => Some(root::ForgeRead::Finding { project: value.project, number: value.number }),
        w::Read::Milestone(value) => Some(root::ForgeRead::Milestone { project: value.project, task: value.task }),
        w::Read::ProgramEnvironment => Some(root::ForgeRead::ProgramEnvironment),
        w::Read::Setting(value) => {
            Some(root::ForgeRead::Setting { kind: value.kind, scope: value.scope, number: value.number })
        }
        w::Read::RevisionSetting(value) => Some(root::ForgeRead::RevisionSetting {
            kind: value.kind,
            scope: value.scope,
            number: value.number,
            revision: value.revision,
        }),
        w::Read::ProposalItems(value) => Some(root::ForgeRead::ProposalItems { key: value.key }),
        w::Read::View(value) => {
            let w::ViewRead { project, address } = value;
            Some(root::ForgeRead::View { project, address })
        }
        w::Read::Playground | w::Read::Deployment => None,
    }
}

fn setup_first(first: w::SetUpFirst) -> forge::SetUpFirst {
    match first {
        w::SetUpFirst::Empty(first) => {
            let w::FirstEmpty { org, repository, playground, sample } = first;
            forge::SetUpFirst::Empty { org, repository, playground, sample }
        }
        w::SetUpFirst::Copies(copies) => {
            let mut addresses = List::with_capacity(copies.addresses.len());
            for address in copies.addresses.into_boxed() {
                let address = match wire::source::parse(&address) {
                    Ok(source) => wire::source::address(&source),
                    Err(_) => address,
                };
                addresses.push(address).expect("decoded copy list");
            }
            forge::SetUpFirst::Copies { addresses: addresses.into_boxed() }
        }
        w::SetUpFirst::Nothing => forge::SetUpFirst::Nothing,
    }
}
fn starts_with(value: &forge::StartsWith, limits: &w::Limits) -> Option<w::StartsWith> {
    if let forge::StartsWith::Copies { addresses } = value
        && addresses.len() > skein_lib::usize_of(wire::FIRST_COPIES)
    {
        return None;
    }
    let record = maps::starts_with_to_record(value).ok()?;
    if let w::StartsWith::Copies(copies) = &record {
        copies.measure(limits).ok()?;
    }
    Some(record)
}

/// The person's whole trials document (temper/docs/design/domain/trial.md, 6).
#[must_use]
pub fn trials(rows: Box<[root::TrialRow]>, limits: &w::Limits) -> Option<Box<[u8]>> {
    let mut encoded = List::with_capacity(64);
    for row in rows {
        encoded
            .push(w::TrialRow {
                run_wait: match row.run_wait {
                    Some(wait) => Some(trial_wait(wait)?),
                    None => None,
                },
                account: row.account,
                task: row.task,
                took: row.took,
                step: maps::trial_step_to_record(&row.step),
                state: maps::trial_state_to_record(&row.state),
            })
            .ok()?;
    }
    let document = w::Trials { rows: encoded };
    document.encode(limits).ok()
}

fn trial_wait(value: jig_domain_core::documents::TaskRunWait) -> Option<Box<[u8]>> {
    let value = jig_protocol_engine_clients::translate::task_wait::waiting(Some(value))?;
    value.encode(&jig_client_protocol_wire::CEILINGS).ok()
}

/// The watcher's approvals document, each row's kind and files-page address from configuration,
/// as a panel's pull request's are (client/forge/records.md; client/forge/README.md, 4.1).
#[must_use]
pub fn approvals(document: root::Approvals, forges: &crate::forges::Forges, limits: &w::Limits) -> Option<Box<[u8]>> {
    let mut rows = List::with_capacity(32);
    for row in document.rows {
        let kind = match forges.get(row.forge) {
            Some(forge) => match forge.kind {
                crate::settings::Kind::GitHub => forge::Kind::GitHub,
                crate::settings::Kind::Forgejo => forge::Kind::Forgejo,
            },
            None => forge::Kind::Forgejo,
        };
        let pull = pull_address(row.forge, &row.owner, &row.name, row.pull, forges);
        let address = if pull.is_empty() || pull.len().saturating_add(6) > 2048 {
            Box::default()
        } else {
            let mut address = Writer::new(pull.len().saturating_add(6));
            address.put(&pull).expect("measured pull address");
            address.put(b"/files").expect("measured files suffix");
            address.finish()
        };
        let mut sets = List::with_capacity(6);
        for set in row.sets {
            sets.push(maps::approval_set_to_record(&set)).ok()?;
        }
        rows.push(w::ApprovalRow {
            task: row.task,
            project: row.project,
            goal: row.goal,
            forge: row.forge,
            kind: maps::kind_to_record(&kind),
            owner: row.owner,
            name: row.name,
            pull: row.pull,
            address,
            title: row.title,
            since: convert::wall_to_record(&row.since),
            scope: row.scope,
            rule: row.rule,
            role: row.role,
            required: row.required,
            counted: row.counted,
            sets,
            whole: row.whole,
            signed: approval_people(row.signed)?,
            can_sign: approval_people(row.can_sign)?,
            can_sign_more: row.can_sign_more,
            yours: maps::approval_yours_to_record(&row.yours),
            size: match &row.size {
                Some(value) => Some(maps::approval_size_to_record(value)),
                None => None,
            },
        })
        .ok()?;
    }
    let document = w::Approvals { rows, more: document.more };
    document.encode(limits).ok()
}

fn approval_people(people: Box<[root::ApprovalPerson]>) -> Option<List<w::ApprovalPerson>> {
    let mut values = List::with_capacity(8);
    for person in people {
        values.push(maps::approval_person_to_record(&person)).ok()?;
    }
    Some(values)
}

fn devices(value: forge::DevicesView, limits: &w::Limits) -> Option<Box<[u8]>> {
    if value.rows.len() > skein_lib::usize_of(forge::DEVICES) {
        return None;
    }
    maps::devices_to_record(&value).ok()?.encode(limits).ok()
}
