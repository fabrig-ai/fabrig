//! The forge family's records at the clients' boundary (client/engine.md).
//! Trials: temper/docs/design/domain/trial.md, 6.
use alloc::boxed::Box;
use jig_protocol_engine_clients as kit;
use skein_lib::{List, Token, Wall, Writer};
use temper_client_protocol_wire::{self as wire, v1 as w};
use temper_engine_domain as root;
use temper_engine_domain_forge as forge;
use temper_engine_domain_forge_change as change;
mod refusals;
mod ways;
pub use ways::ways_panel;

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
        kit::ToApplication::Request { reply_to, sign_in, key, family, version, record, digest, scope, carried } => {
            let request = match wire::owner(family) {
                Some(wire::Owner::Forge) if version == wire::CLIENT_VERSION => wire::decode_requesting(&record, limits),
                Some(wire::Owner::Forge | wire::Owner::Pages | wire::Owner::Runner) | None => None,
            };
            let request = match request {
                Some(request) => request.into_parts().request,
                None => {
                    return Err(Box::new(kit::Event::Application(kit::FromApplication::Undecodable { to: reply_to })));
                }
            };
            let request = match request {
                w::Request::FollowAct(value) => {
                    let value = value.into_parts();
                    let act = match value.act {
                        w::FollowAct::Start(v) => forge::follows::FollowAct::Start { after: *v.after() },
                        w::FollowAct::Hold(v) => {
                            let p = v.into_parts();
                            forge::follows::FollowAct::Hold { after: p.after, reason: p.reason }
                        }
                        w::FollowAct::Skip(v) => {
                            let p = v.into_parts();
                            forge::follows::FollowAct::Skip { after: p.after, reason: p.reason }
                        }
                        w::FollowAct::Merge(v) => {
                            forge::follows::FollowAct::Merge { afters: v.into_parts().afters.into_boxed() }
                        }
                    };
                    return Ok(root::Event::Party(root::PartyInput::Application {
                        scope,
                        reply_to,
                        sign_in,
                        key,
                        digest,
                        carried,
                        request: root::Application::Follow { project: value.project, follow: value.follow, act },
                    }));
                }
                w::Request::SetUp(value) => {
                    let value = value.into_parts();
                    let Some(forge) = making(value.forge, directories) else {
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
                            let value = value.into_parts();
                            let how = match value.how {
                                w::How::ForgejoAdmin(admin) => {
                                    let first = match admin.into_parts().first {
                                        Some(first) => {
                                            let first = first.into_parts();
                                            Some(forge::FirstAsk {
                                                org: first.org,
                                                repository: first.repository,
                                                label: first.label,
                                            })
                                        }
                                        None => None,
                                    };
                                    forge::How::ForgejoAdmin { first }
                                }
                                w::How::ForgejoTokens(_) => forge::How::ForgejoTokens,
                                w::How::GithubApp(app) => forge::How::GitHubApp {
                                    app: core::num::NonZeroU64::new(app.id()).expect("decoded nonzero App"),
                                    read_only: app.read_only().unwrap_or(false),
                                },
                                w::How::GithubManifest(manifest) => {
                                    let back = manifest.into_parts().back;
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
                                forge: match making(value.forge, directories) {
                                    Some(forge) => forge,
                                    None => {
                                        return Err(Box::new(kit::Event::Application(
                                            kit::FromApplication::Undecodable { to: reply_to },
                                        )));
                                    }
                                },
                                how,
                            }
                        }
                        w::Request::Replace(value) => forge::Ask::Replace { forge: value.forge() },
                        w::Request::Accept(value) => {
                            let value = value.into_parts();
                            let Some(installation) = core::num::NonZeroU64::new(value.installation) else {
                                return Err(Box::new(kit::Event::Application(kit::FromApplication::Undecodable {
                                    to: reply_to,
                                })));
                            };
                            forge::Ask::Accept { forge: value.forge, installation }
                        }
                        w::Request::Install(value) => {
                            let value = value.into_parts();
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
                        | w::Request::Adopt(_)
                        | w::Request::SetUp(_)
                        | w::Request::TryAccount(_)
                        | w::Request::Dismiss(_)
                        | w::Request::SetAway(_)
                        | w::Request::Overturn(_)
                        | w::Request::Looked(_)
                        | w::Request::RemoveForge(_)
                        | w::Request::StartPlayground
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
                        request: root::Application::Forge(forge::Ask::RemoveForge { forge: value.forge() }),
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
                    let value = value.into_parts();
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
                    let value = value.into_parts();
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
                    let value = value.into_parts();
                    let from = match value.from {
                        w::MakeFrom::Empty => forge::MakeFrom::Empty,
                        w::MakeFrom::Copy(copy) => {
                            let copy = copy.into_parts();
                            let Ok(source) = wire::source::parse(&copy.address) else {
                                return Err(Box::new(kit::Event::Application(kit::FromApplication::Undecodable {
                                    to: reply_to,
                                })));
                            };
                            forge::MakeFrom::Copy { address: wire::source::address(&source) }
                        }
                    };
                    let into = match value.into {
                        Some(into) => {
                            let into = into.into_parts();
                            Some(forge::MakeInto {
                                project: into.project,
                                home: into.home,
                                host: match forges.get(value.forge) {
                                    Some(forge) => host(&forge.git),
                                    None => Box::default(),
                                },
                            })
                        }
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
                        request: root::Application::Try { account: value.account(), again: value.again() },
                    }));
                }
                w::Request::Dismiss(value) => {
                    let what = match value.into_parts().what {
                        w::PersonFact::KeepAsking(value) => forge::PersonFact::KeepAsking {
                            project: value.project(),
                            subject: temper_protocol_reaches::subject_from(value.subject()),
                            at: value.at(),
                        },
                        w::PersonFact::ToBegin(value) => forge::PersonFact::ToBegin { project: value.project() },
                        w::PersonFact::GoalFrom(value) => forge::PersonFact::GoalFrom {
                            project: value.project(),
                            chat: value.chat(),
                            goal: value.goal(),
                            at: value.at(),
                        },
                    };
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
                        request: root::Application::Looked { project: value.project() },
                    }));
                }
                w::Request::SetAway(value) => {
                    let value = value.into_parts();
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
                                Some(until) => Some(Wall::from_nanos(until)),
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
                        request: root::Application::Overturn { project: value.project(), number: value.number() },
                    }));
                }
                w::Request::Adopt(value) => value,
            };
            let a = request.into_parts();
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
                    role: role(a.role),
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
                let query = match reading.into_parts().read {
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
                    w::Read::View(read) => {
                        if scope == jig_domain_people::ApplicationScope::Project(read.project()) {
                            forge_settings_query(w::Read::View(read))
                        } else {
                            None
                        }
                    }
                    read @ (w::Read::Setting(_)
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
                let subject = match watching.into_parts().watch {
                    w::Watch::Forges => jig_domain_core::documents::Watched::Application {
                        scope: jig_domain_people::ApplicationScope::Deployment,
                        connector: wire::FORGE,
                        subject: Token::new(0),
                    },
                    w::Watch::Change(subject) if subject.task() > 0 => {
                        jig_domain_core::documents::Watched::Application {
                            scope,
                            connector: wire::FORGE,
                            subject: Token::new(subject.task()),
                        }
                    }
                    w::Watch::Landing(subject) if subject.task() > 0 => {
                        jig_domain_core::documents::Watched::Application {
                            scope,
                            connector: wire::FORGE,
                            subject: Token::new(subject.task()),
                        }
                    }
                    w::Watch::Follows(subject) => jig_domain_core::documents::Watched::Application {
                        scope,
                        connector: wire::FORGE,
                        subject: Token::new(forge::follows_subject(subject.project())),
                    },
                    w::Watch::Landings(subject) => jig_domain_core::documents::Watched::Application {
                        scope,
                        connector: wire::FORGE,
                        subject: Token::new(forge::landings_subject(subject.project(), subject.ended())),
                    },
                    w::Watch::Changes(subject) => jig_domain_core::documents::Watched::Application {
                        scope,
                        connector: wire::FORGE,
                        subject: Token::new(forge::changes_subject(subject.project())),
                    },
                    w::Watch::Ways(subject) => jig_domain_core::documents::Watched::Application {
                        scope: jig_domain_people::ApplicationScope::Project(subject.project()),
                        connector: wire::FORGE,
                        subject: Token::new(root::ways::subject(subject.project())),
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
                        subject: Token::new(forge::status_subject(subject.project())),
                    },
                    w::Watch::Away => jig_domain_core::documents::Watched::Application {
                        scope: jig_domain_people::ApplicationScope::Own,
                        connector: wire::FORGE,
                        subject: Token::new(forge::AWAY_SUBJECT),
                    },
                    w::Watch::Decided(subject) => jig_domain_core::documents::Watched::Application {
                        scope: jig_domain_people::ApplicationScope::Project(subject.project()),
                        connector: wire::FORGE,
                        subject: Token::new(forge::decided_subject(subject.project())),
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
    let pull = match value.pull {
        Some(number) => {
            Some(w::Pull::new(limits, w::PullParts { number, address: address(&value, number, forges) }).ok()?)
        }
        None => None,
    };
    let gates = panel_gates(value.gates, limits)?;
    let approvals = panel_approvals(value.approvals, limits)?;
    let unmapped_reviews = panel_unmapped(value.unmapped_reviews, limits)?;
    let record = w::ChangePanel::new(
        limits,
        w::ChangePanelParts {
            way: match value.way {
                change::Way::Queue => w::Way::Queue,
                change::Way::Turn => w::Way::Turn,
            },
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
            hold: panel_hold(value.hold),
            runs: value.runs,
            ci_required: value.ci_required,
            ci: match value.ci {
                change::Status::Unknown => w::Ci::Unknown,
                change::Status::Pending => w::Ci::Pending,
                change::Status::Passed => w::Ci::Passed,
                change::Status::Failed => w::Ci::Failed,
            },
            gates,
            approvals,
            unmapped_reviews,
            repairs: used(value.repairs, limits)?,
            resolutions: used(value.resolutions, limits)?,
            updates: used(value.updates, limits)?,
            questions: panel_questions(value.questions, limits)?,
        },
    )
    .ok()?;
    Some(wire::change_panel(&record))
}

fn panel_gates(value: Box<[forge::PanelGate]>, limits: &w::Limits) -> Option<List<w::Gate>> {
    let mut gates = List::with_capacity(forge::PANEL_GATES);
    for gate in value {
        gates
            .push(
                w::Gate::new(
                    limits,
                    w::GateParts {
                        number: gate.number,
                        kind: match gate.kind {
                            change::GateKind::Agent => w::GateKind::Review,
                            change::GateKind::Person => w::GateKind::Approval,
                            change::GateKind::Check | change::GateKind::Follow => w::GateKind::Check,
                        },
                        blocks: gate.blocking,
                        verdict: match gate.verdict {
                            change::Status::Unknown => w::Verdict::None,
                            change::Status::Pending => w::Verdict::Pending,
                            change::Status::Passed => w::Verdict::Approved,
                            change::Status::Failed => w::Verdict::Changes,
                        },
                    },
                )
                .ok()?,
            )
            .ok()?;
    }
    Some(gates)
}
fn panel_approvals(value: Box<[forge::PanelApproval]>, limits: &w::Limits) -> Option<List<w::Approval>> {
    let mut approvals = List::with_capacity(limits.change_panel_approvals);
    for line in value {
        let mut counted = List::with_capacity(limits.approval_counted);
        for review in line.counted {
            counted
                .push(
                    w::CountedReview::new(limits, w::CountedReviewParts { person: review.person, head: review.head })
                        .ok()?,
                )
                .ok()?;
        }
        approvals
            .push(
                w::Approval::new(
                    limits,
                    w::ApprovalParts {
                        scope: line.scope,
                        number: line.rule,
                        role: line.role,
                        freshness: match line.freshness {
                            forge::JudgeFreshness::Exact => w::RuleFreshness::Exact,
                            forge::JudgeFreshness::Clean => w::RuleFreshness::Clean,
                        },
                        required: line.required,
                        read: match line.read {
                            forge::ReviewRead::Unread => w::ReviewRead::Unread,
                            forge::ReviewRead::Incomplete => w::ReviewRead::Incomplete,
                            forge::ReviewRead::Complete => w::ReviewRead::Complete,
                        },
                        counted,
                        remaining: line.remaining,
                        refused: line.refused,
                    },
                )
                .ok()?,
            )
            .ok()?;
    }
    Some(approvals)
}
fn panel_unmapped(value: Box<[forge::UnmappedReview]>, limits: &w::Limits) -> Option<List<w::UnmappedReview>> {
    let mut reviews = List::with_capacity(limits.change_panel_unmapped_reviews);
    for review in value {
        reviews
            .push(
                w::UnmappedReview::new(
                    limits,
                    w::UnmappedReviewParts {
                        account: review.account,
                        head: review.head,
                        verdict: match review.verdict {
                            skein_forge::api::Verdict::Approve => w::Verdict::Approved,
                            skein_forge::api::Verdict::Comment => w::Verdict::None,
                            skein_forge::api::Verdict::RequestChanges => w::Verdict::Changes,
                        },
                    },
                )
                .ok()?,
            )
            .ok()?;
    }
    Some(reviews)
}

fn panel_questions(value: forge::PanelQuestions, limits: &w::Limits) -> Option<w::Questions> {
    let mut open = List::with_capacity(8);
    for question in value.open {
        open.push(
            w::Question::new(
                limits,
                w::QuestionParts {
                    asker: question.asker,
                    number: question.number,
                    line: question.line,
                    stands: match question.stands {
                        forge::PanelStands::Taken => w::Stands::Taken,
                        forge::PanelStands::Foreman(task) => {
                            w::Stands::Foreman(w::Foreman::new(limits, w::ForemanParts { task }).ok()?)
                        }
                        forge::PanelStands::Asked => w::Stands::Asked,
                        forge::PanelStands::Held => w::Stands::Held,
                    },
                },
            )
            .ok()?,
        )
        .ok()?;
    }
    w::Questions::new(
        limits,
        w::QuestionsParts {
            route: match value.route {
                change::Route::None => w::Route::None,
                change::Route::Foreman => w::Route::Foreman,
                change::Route::Up => w::Route::Up,
            },
            taken: used(value.taken, limits)?,
            open,
        },
    )
    .ok()
}

fn used(value: forge::PanelUsed, limits: &w::Limits) -> Option<w::Used> {
    w::Used::new(limits, w::UsedParts { used: value.used, limit: value.limit }).ok()
}

fn panel_step(value: forge::PanelStep, limits: &w::Limits) -> Option<w::Step> {
    Some(match value {
        forge::PanelStep::Ended => w::Step::Ended,
        forge::PanelStep::Ready { landing } => {
            w::Step::Ready(w::InLanding::new(limits, w::InLandingParts { landing }).ok()?)
        }
        forge::PanelStep::Preparing { landing } => {
            w::Step::Preparing(w::InLanding::new(limits, w::InLandingParts { landing }).ok()?)
        }
        forge::PanelStep::Producing => w::Step::Producing,
        forge::PanelStep::Opening => w::Step::Opening,
        forge::PanelStep::Recreating => w::Step::Recreating,
        forge::PanelStep::Reopening => w::Step::Reopening,
        forge::PanelStep::Checking => w::Step::Checking,
        forge::PanelStep::Gating => w::Step::Gating,
        forge::PanelStep::Queued { place, ahead } => {
            let mut tasks = List::with_capacity(forge::PANEL_AHEAD);
            for task in ahead {
                tasks.push(task).ok()?;
            }
            w::Step::Queued(w::Queued::new(limits, w::QueuedParts { place, ahead: tasks }).ok()?)
        }
        forge::PanelStep::First => w::Step::First,
        forge::PanelStep::Updating => w::Step::Updating,
        forge::PanelStep::Resolving => w::Step::Resolving,
        forge::PanelStep::Repairing(reason) => w::Step::Repairing(
            w::Repairing::new(
                limits,
                w::RepairingParts {
                    reason: match reason {
                        change::Repair::Ci => w::Reason::Ci,
                        change::Repair::Review { .. } => w::Reason::Review,
                        change::Repair::Gate(gate) => {
                            w::Reason::Gate(w::GateReason::new(limits, w::GateReasonParts { gate }).ok()?)
                        }
                        change::Repair::Semantic => w::Reason::Semantic,
                        // landing.md 3: the top's task
                        change::Repair::Landing { .. } => return None,
                    },
                },
            )
            .ok()?,
        ),
        forge::PanelStep::Landing => w::Step::Landing,
        forge::PanelStep::Landed { merge } => w::Step::Landed(w::Landed::new(limits, w::LandedParts { merge }).ok()?),
        forge::PanelStep::Verifying { merge, tiers } => {
            w::Step::Verifying(w::Verifying::new(limits, w::VerifyingParts { merge, tiers: panel_tiers(tiers) }).ok()?)
        }
        forge::PanelStep::Red { merge, tiers } => {
            w::Step::Red(w::Red::new(limits, w::RedParts { merge, tiers: panel_tiers(tiers) }).ok()?)
        }
        forge::PanelStep::Cleared { merge, tiers, uncleared } => w::Step::Cleared(
            w::Cleared::new(limits, w::ClearedParts { merge, tiers: panel_tiers(tiers), uncleared }).ok()?,
        ),
    })
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
        change::Hold::Failed => w::Hold::Failed,
        change::Hold::Rejected => w::Hold::Rejected,
        change::Hold::Authority(cause) => w::Hold::Authority(
            w::AuthorityHold::new(&w::CEILINGS, w::AuthorityHoldParts { reason: change::authority_words(cause) })
                .expect("bounded authority hold"),
        ),
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
fn role(role: w::Role) -> forge::Role {
    match role {
        w::Role::Owned => forge::Role::Owned,
        w::Role::Adopted => forge::Role::Adopted,
        w::Role::Fork => forge::Role::Fork,
        w::Role::Context => forge::Role::Context,
    }
}
fn encoded_role(role: forge::Role) -> w::Role {
    match role {
        forge::Role::Owned => w::Role::Owned,
        forge::Role::Adopted => w::Role::Adopted,
        forge::Role::Fork => w::Role::Fork,
        forge::Role::Context => w::Role::Context,
    }
}
/// Encode one durable terminal, or the unknown terminal past its retention.
#[must_use]
#[expect(clippy::too_many_lines, reason = "one exhaustive family outcome translation")]
pub fn answer(
    outcome: Option<temper_engine_domain::ApplicationAnswer>,
    limits: &w::Limits,
    jig: &jig_client_protocol_wire::v19::Limits,
) -> Option<Box<[u8]>> {
    let outcome = match outcome {
        Some(root::ApplicationAnswer::Reachable(forge::ReachableAnswer::Page(page))) => {
            let mut rows = List::with_capacity(limits.page_rows);
            for row in page.rows {
                rows.push(
                    w::Reached::new(
                        limits,
                        w::ReachedParts {
                            id: row.id,
                            owner: w::Account::new(limits, w::AccountParts { id: row.owner_id, login: row.owner })
                                .ok()?,
                            name: row.name,
                            private: row.private,
                        },
                    )
                    .ok()?,
                )
                .ok()?;
            }
            w::Outcome::Page(w::Page::new(limits, w::PageParts { rows, more: page.more }).ok()?)
        }
        Some(
            root::ApplicationAnswer::Follow(refusal)
            | root::ApplicationAnswer::Adopt(forge::ApplicationOutcome::Follow(refusal)),
        ) => w::Outcome::Follow(w::FollowOutcome::new(limits, w::FollowOutcomeParts { refusal }).ok()?),
        Some(root::ApplicationAnswer::Make(answer)) => match answer {
            forge::MakeAnswer::Kept => w::Outcome::MakeKept,
            forge::MakeAnswer::Refused(why) => {
                let why = match why {
                    forge::MakeRefusal::NoSuchForge => w::MakeRefusal::NoSuchForge,
                    forge::MakeRefusal::NotOffered => w::MakeRefusal::NotOffered,
                    forge::MakeRefusal::NotOwner => w::MakeRefusal::NotOwner,
                    forge::MakeRefusal::Invalid => w::MakeRefusal::Invalid,
                    forge::MakeRefusal::HomeHeld => w::MakeRefusal::HomeHeld,
                    forge::MakeRefusal::Full => w::MakeRefusal::Full,
                    forge::MakeRefusal::NotFailed => w::MakeRefusal::NotFailed,
                };
                w::Outcome::MakeRefused(w::MakeRefused::new(limits, w::MakeRefusedParts { why }).ok()?)
            }
        },
        Some(temper_engine_domain::ApplicationAnswer::Adopt(
            forge::ApplicationOutcome::MakeKept | forge::ApplicationOutcome::MakeRefused(_),
        )) => return None,
        Some(root::ApplicationAnswer::Try(root::TryAnswer::Tried)) => w::Outcome::Tried,
        Some(root::ApplicationAnswer::Try(root::TryAnswer::NotTheirs)) => w::Outcome::NotTheirs,
        Some(root::ApplicationAnswer::Dismiss(forge::DismissAnswer::Dismissed)) => w::Outcome::Dismissed,
        Some(root::ApplicationAnswer::Looked(forge::LookedAnswer::Looked)) => w::Outcome::Looked,
        Some(root::ApplicationAnswer::SetAway(forge::SetAwayAnswer::Away)) => w::Outcome::Away,
        Some(root::ApplicationAnswer::Overturn(answer)) => match answer {
            forge::OverturnAnswer::Overturned => w::Outcome::Overturned,
            forge::OverturnAnswer::Already => w::Outcome::OverturnRefused(
                w::OverturnRefused::new(limits, w::OverturnRefusedParts { reason: w::OverturnRefusal::Already })
                    .ok()?,
            ),
            forge::OverturnAnswer::Unknown => w::Outcome::OverturnRefused(
                w::OverturnRefused::new(limits, w::OverturnRefusedParts { reason: w::OverturnRefusal::Unknown })
                    .ok()?,
            ),
        },
        Some(
            root::ApplicationAnswer::Dismiss(forge::DismissAnswer::TooLarge)
            | root::ApplicationAnswer::Looked(forge::LookedAnswer::TooLarge)
            | root::ApplicationAnswer::SetAway(forge::SetAwayAnswer::TooLarge),
        ) => w::Outcome::ForgeRefused(
            w::ForgeRefused::new(limits, w::ForgeRefusedParts { reason: w::ForgeReason::TooLarge }).ok()?,
        ),
        Some(temper_engine_domain::ApplicationAnswer::Adopt(forge::ApplicationOutcome::Adopted(a))) => {
            w::Outcome::Adopted(
                w::Adopted::new(
                    limits,
                    w::AdoptedParts {
                        forge: a.repository.forge,
                        repository: a.repository.repository,
                        owner: a.owner,
                        name: a.name,
                        role: encoded_role(a.role),
                        home: a.home,
                        kinds: w::Kinds::new(
                            limits,
                            w::KindsParts {
                                skein_bools: [
                                    a.kinds.read,
                                    a.kinds.push,
                                    a.kinds.open,
                                    a.kinds.land,
                                    a.kinds.review,
                                    a.kinds.status,
                                    a.kinds.comment,
                                    a.kinds.issue,
                                    a.kinds.branch,
                                ],
                            },
                        )
                        .ok()?,
                        protection: protection(a.protection, limits)?,
                    },
                )
                .ok()?,
            )
        }
        Some(temper_engine_domain::ApplicationAnswer::Adopt(forge::ApplicationOutcome::CoreRefused(value))) => {
            let value = refusals::refusal(value, jig).ok()?;
            let mut bytes = Writer::new(usize::try_from(value.measure()).ok()?);
            value.encode(&mut bytes).ok()?;
            w::Outcome::CoreRefused(w::CoreRefused::new(limits, w::CoreRefusedParts { refusal: bytes.finish() }).ok()?)
        }
        Some(
            temper_engine_domain::ApplicationAnswer::Adopt(forge::ApplicationOutcome::ForgeRefused(reason))
            | root::ApplicationAnswer::Reachable(forge::ReachableAnswer::ForgeRefused(reason))
            | root::ApplicationAnswer::Accept(forge::AcceptAnswer::ForgeRefused(reason)),
        ) => {
            w::Outcome::ForgeRefused(
                w::ForgeRefused::new(
                    limits,
                    w::ForgeRefusedParts {
                        reason: match reason {
                            forge::AdoptionRefusal::NoSuchForge => w::ForgeReason::NoSuchForge,
                            forge::AdoptionRefusal::NotFound => w::ForgeReason::NotFound,
                            // The client wire has no not-installed reason (client/forge/family.md, 6).
                            forge::AdoptionRefusal::NotInstalled | forge::AdoptionRefusal::NoAccess => {
                                w::ForgeReason::NoAccess
                            }
                            forge::AdoptionRefusal::Refused => w::ForgeReason::Refused,
                            forge::AdoptionRefusal::InFlight => w::ForgeReason::InFlight,
                            forge::AdoptionRefusal::TooLarge => w::ForgeReason::TooLarge,
                            forge::AdoptionRefusal::Unreachable => w::ForgeReason::Unreachable,
                            forge::AdoptionRefusal::InvalidAnswer => w::ForgeReason::InvalidAnswer,
                            forge::AdoptionRefusal::AnotherProject => w::ForgeReason::AnotherProject,
                            forge::AdoptionRefusal::HomeHeld => w::ForgeReason::HomeHeld,
                            forge::AdoptionRefusal::PrefixOccupied => w::ForgeReason::PrefixOccupied,
                        },
                    },
                )
                .ok()?,
            )
        }
        Some(root::ApplicationAnswer::Accept(forge::AcceptAnswer::Accepted { installation })) => w::Outcome::Accepted(
            w::AcceptedInstallation::new(limits, w::AcceptedInstallationParts { installation }).ok()?,
        ),
        Some(
            root::ApplicationAnswer::Connect(forge::ConnectAnswer::Connecting)
            | root::ApplicationAnswer::Install(forge::InstallAnswer::Connecting),
        ) => w::Outcome::Connecting,
        Some(
            root::ApplicationAnswer::Install(forge::InstallAnswer::Link(address))
            | root::ApplicationAnswer::Connect(forge::ConnectAnswer::Link(address)),
        ) => w::Outcome::Link(w::Link::new(limits, w::LinkParts { address }).ok()?),
        Some(
            root::ApplicationAnswer::Accept(forge::AcceptAnswer::Refused(why))
            | root::ApplicationAnswer::Reachable(forge::ReachableAnswer::Refused(why))
            | root::ApplicationAnswer::Connect(forge::ConnectAnswer::Refused(why))
            | root::ApplicationAnswer::Install(forge::InstallAnswer::Refused(why)),
        ) => w::Outcome::ConnectRefused(
            w::ConnectRefused::new(limits, w::ConnectRefusedParts { why: forge_refusal(why) }).ok()?,
        ),
        Some(root::ApplicationAnswer::StartPlayground(answer)) => start_answer(answer, limits)?,
        Some(root::ApplicationAnswer::RemoveForge(forge::RemoveAnswer::Removed)) => w::Outcome::Removed,
        Some(root::ApplicationAnswer::RemoveForge(forge::RemoveAnswer::Refused(why))) => w::Outcome::RemoveRefused(
            w::RemoveRefused::new(limits, w::RemoveRefusedParts { why: forge_refusal(why) }).ok()?,
        ),
        Some(root::ApplicationAnswer::EndPlayground(forge::EndAnswer::Reset)) => w::Outcome::Reset,
        Some(root::ApplicationAnswer::EndPlayground(forge::EndAnswer::NotOffered)) => w::Outcome::EndRefused(
            w::EndRefused::new(limits, w::EndRefusedParts { why: w::EndRefusal::NotOffered }).ok()?,
        ),
        None => w::Outcome::Unknown,
        Some(temper_engine_domain::ApplicationAnswer::SetUp(forge::SetUpAnswer::SettingUp)) => w::Outcome::SettingUp,
        Some(temper_engine_domain::ApplicationAnswer::SetUp(forge::SetUpAnswer::Refused(why))) => {
            w::Outcome::SetUpRefused(
                w::SetUpRefused::new(
                    limits,
                    w::SetUpRefusedParts {
                        why: match why {
                            forge::SetUpRefusal::NotOffered => w::SetUpRefusal::NotOffered,
                            forge::SetUpRefusal::NotUnderTheBox => w::SetUpRefusal::NotUnderTheBox,
                            forge::SetUpRefusal::Busy => w::SetUpRefusal::Busy,
                            forge::SetUpRefusal::Invalid => w::SetUpRefusal::Invalid,
                        },
                    },
                )
                .ok()?,
            )
        }
    };
    let answer = w::Answer::new(limits, w::AnswerParts { outcome }).ok()?;
    Some(wire::answer(&answer))
}
fn start_answer(answer: forge::StartAnswer, limits: &w::Limits) -> Option<w::Outcome> {
    Some(match answer {
        forge::StartAnswer::Running => w::Outcome::Running,
        forge::StartAnswer::NotOffered => w::Outcome::StartRefused(
            w::StartRefused::new(limits, w::StartRefusedParts { why: w::StartRefusal::NotOffered }).ok()?,
        ),
        forge::StartAnswer::Failed(why) => w::Outcome::StartFailed(
            w::StartFailed::new(
                limits,
                w::StartFailedParts {
                    why: match why {
                        forge::StartFailure::Group { why } => {
                            w::StartFailure::Group(w::GroupFailed::new(limits, w::GroupFailedParts { why }).ok()?)
                        }
                        forge::StartFailure::Account => w::StartFailure::Account,
                        forge::StartFailure::Adoption => w::StartFailure::Adoption,
                        forge::StartFailure::NotConnected => w::StartFailure::NotConnected,
                    },
                },
            )
            .ok()?,
        ),
    })
}
fn protection(value: forge::Protection, limits: &w::Limits) -> Option<w::Protection> {
    Some(match value {
        forge::Protection::Unknown => w::Protection::Unknown,
        forge::Protection::Absent => w::Protection::Absent,
        forge::Protection::Rule(rule) => {
            let mut contexts = List::with_capacity(limits.rule_contexts);
            for context in rule.contexts {
                contexts.push(context).ok()?;
            }
            w::Protection::Rule(
                w::Rule::new(
                    limits,
                    w::RuleParts {
                        branch: rule.branch,
                        contexts,
                        approvals: rule.approvals,
                        dismiss_stale: rule.dismiss_stale,
                    },
                )
                .ok()?,
            )
        }
    })
}

/// Encode the connector's whole document without adding policy from settings.
#[must_use]
pub fn forges(value: forge::Forges, linked: bool, limits: &w::Limits) -> Option<Box<[u8]>> {
    let mut forges = List::with_capacity(limits.forges_forges);
    for row in value.rows {
        forges.push(forge_row(row, limits)?).ok()?;
    }
    Some(wire::forges(&w::Forges::new(limits, w::ForgesParts { forges, linked }).ok()?))
}
fn forge_row(row: forge::Row, limits: &w::Limits) -> Option<w::ForgeRow> {
    let ended = match row.ended {
        Some(why) => Some(w::ConnectStopped::new(limits, w::ConnectStoppedParts { why }).ok()?),
        None => None,
    };
    let mut makings = List::with_capacity(limits.forge_row_makings);
    for row in row.makings {
        makings.push(repository_making(row, limits)?).ok()?;
    }
    w::ForgeRow::new(
        limits,
        w::ForgeRowParts {
            number: row.number,
            host: row.host,
            makings,
            read_only: row.read_only,
            kind: forge_kind(row.kind),
            mode: forge_mode(row.mode),
            connection: connection(row.connection, row.failed, row.fetched, limits)?,
            offer: match row.offer {
                forge::Offer::NotOffered => w::Offer::NotOffered,
                forge::Offer::NotUnderTheBox => w::Offer::NotUnderTheBox,
                forge::Offer::Offered => w::Offer::Offered,
            },
            first: match row.first {
                Some(first) => Some(first_repository(first, limits)?),
                None => None,
            },
            starts_with: match row.starts_with {
                forge::StartsWith::Repository => w::StartsWith::Repository,
                forge::StartsWith::Copies { addresses } => w::StartsWith::Copies(copy_addresses(addresses, limits)?),
                forge::StartsWith::Nothing => w::StartsWith::Nothing,
            },
            installations: forge_installations(row.installations, limits)?,
            ended,
            app: match row.app {
                Some(forge::AppWay::Manifest) => Some(w::AppWay::Manifest),
                Some(forge::AppWay::Pasted) => Some(w::AppWay::Pasted),
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
        },
    )
    .ok()
}
fn repository_making(row: forge::RepositoryMaking, limits: &w::Limits) -> Option<w::RepositoryMaking> {
    let from = match row.from {
        forge::MakeFrom::Empty => w::MakeFrom::Empty,
        forge::MakeFrom::Copy { address } => {
            w::MakeFrom::Copy(w::MakeCopy::new(limits, w::MakeCopyParts { address }).ok()?)
        }
    };
    let into = match row.into {
        Some(into) => Some(w::MakeInto::new(limits, w::MakeIntoParts { project: into.project, home: into.home }).ok()?),
        None => None,
    };
    let state = match row.state {
        forge::MakingState::Waiting => w::MakingState::Waiting,
        forge::MakingState::Making => w::MakingState::Making,
        forge::MakingState::Made { id, branch } => {
            w::MakingState::Made(w::MakingMade::new(limits, w::MakingMadeParts { id, branch }).ok()?)
        }
        forge::MakingState::Adopting { id, branch } => {
            w::MakingState::Adopting(w::MakingMade::new(limits, w::MakingMadeParts { id, branch }).ok()?)
        }
        forge::MakingState::Failed(failed) => {
            let step = match failed.step {
                forge::MakeStep::Admin => w::MakeStep::Admin,
                forge::MakeStep::Org => w::MakeStep::Org,
                forge::MakeStep::Repository => w::MakeStep::Repository,
                forge::MakeStep::File => w::MakeStep::File,
                forge::MakeStep::Adoption => w::MakeStep::Adoption,
            };
            let reason = match failed.reason {
                forge::MakeFailure::Taken => w::MakeFailure::Taken,
                forge::MakeFailure::Empty => w::MakeFailure::Empty,
                forge::MakeFailure::TimedOut => w::MakeFailure::TimedOut,
                forge::MakeFailure::Refused { status } => {
                    w::MakeFailure::Refused(w::Refused::new(limits, w::RefusedParts { status }).ok()?)
                }
                forge::MakeFailure::NotAllowed => w::MakeFailure::NotAllowed,
                forge::MakeFailure::Other => w::MakeFailure::Other,
            };
            w::MakingState::Failed(
                w::MakingFailed::new(limits, w::MakingFailedParts { step, reason, words: failed.words }).ok()?,
            )
        }
    };
    w::RepositoryMaking::new(
        limits,
        w::RepositoryMakingParts { org: row.key.org, name: row.key.name, from, into, state },
    )
    .ok()
}
fn forge_adoptions(values: Box<[forge::AdoptedRow]>, limits: &w::Limits) -> Option<List<w::Adoption>> {
    let mut repositories = List::with_capacity(limits.forge_row_repositories);
    for adopted in values {
        repositories
            .push(
                w::Adoption::new(
                    limits,
                    w::AdoptionParts { owner: adopted.owner, name: adopted.name, project: adopted.project },
                )
                .ok()?,
            )
            .ok()?;
    }
    Some(repositories)
}
fn forge_installations(values: Box<[(u64, forge::Installation)]>, limits: &w::Limits) -> Option<List<w::Installation>> {
    let mut installations = List::with_capacity(limits.forge_row_installations);
    for (id, installation) in values {
        let account = match installation.account {
            Some((id, login)) => Some(w::Account::new(limits, w::AccountParts { id, login }).ok()?),
            None => None,
        };
        installations
            .push(
                w::Installation::new(
                    limits,
                    w::InstallationParts { id, account, suspended: installation.suspended, stray: installation.stray },
                )
                .ok()?,
            )
            .ok()?;
    }
    Some(installations)
}
fn owned_forge(owned: Option<forge::Owned>, limits: &w::Limits) -> Option<w::OwnedForge> {
    match owned {
        Some(owned) => w::OwnedForge::new(
            limits,
            w::OwnedForgeParts { forgejo: owned.forgejo, runner: owned.runner, password: owned.password },
        )
        .ok(),
        None => None,
    }
}
fn forge_kind(kind: forge::Kind) -> w::Kind {
    match kind {
        forge::Kind::Forgejo => w::Kind::Forgejo,
        forge::Kind::GitHub => w::Kind::Github,
    }
}
fn forge_mode(mode: forge::Mode) -> w::Mode {
    match mode {
        forge::Mode::Connected => w::Mode::Connected,
        forge::Mode::Managed => w::Mode::Managed,
        forge::Mode::Owned => w::Mode::Owned,
        forge::Mode::Fake => w::Mode::Fake,
    }
}

fn connection(
    value: forge::Connection,
    failed: Option<forge::FailedStep>,
    fetched: Option<forge::SetUpFetched>,
    limits: &w::Limits,
) -> Option<w::Connection> {
    Some(match value {
        forge::Connection::NotConnected => w::Connection::NotConnected,
        forge::Connection::NotSetUp => w::Connection::NotSetUp,
        forge::Connection::SettingUp { step } => w::Connection::SettingUp(
            w::SettingUp::new(
                limits,
                w::SettingUpParts {
                    fetched: match fetched {
                        None => None,
                        Some(fetched) => Some(
                            w::SetUpFetched::new(
                                limits,
                                w::SetUpFetchedParts {
                                    binary: match fetched.binary {
                                        forge::SetUpBinary::Forgejo => w::SetUpBinary::Forgejo,
                                        forge::SetUpBinary::Runner => w::SetUpBinary::Runner,
                                    },
                                    done: fetched.done,
                                    total: fetched.total,
                                },
                            )
                            .ok()?,
                        ),
                    },
                    step: match step {
                        None => None,
                        Some(forge::SetUpStep::Fetch) => Some(w::SetUpStep::Fetch),
                        Some(forge::SetUpStep::Prepare) => Some(w::SetUpStep::Prepare),
                        Some(forge::SetUpStep::Run) => Some(w::SetUpStep::Run),
                        Some(forge::SetUpStep::Source) => Some(w::SetUpStep::Source),
                    },
                },
            )
            .ok()?,
        ),
        forge::Connection::SetUpFailed { why } => w::Connection::SetUpFailed(
            w::SetUpFailed::new(
                limits,
                w::SetUpFailedParts {
                    why,
                    at: match failed {
                        Some(forge::FailedStep::Fetch) => Some(w::FailedStep::Fetch),
                        Some(forge::FailedStep::Prepare) => Some(w::FailedStep::Prepare),
                        Some(forge::FailedStep::Run) => Some(w::FailedStep::Run),
                        Some(forge::FailedStep::Connect) => Some(w::FailedStep::Connect),
                        None => None,
                    },
                },
            )
            .ok()?,
        ),
        forge::Connection::Connecting => w::Connection::Connecting,
        forge::Connection::Connected { id, login } => {
            w::Connection::Connected(w::Connected::new(limits, w::ConnectedParts { id, login }).ok()?)
        }
        forge::Connection::Stopped { reason } => {
            w::Connection::Stopped(w::Stopped::new(limits, w::StoppedParts { reason }).ok()?)
        }
    })
}

fn first_repository(first: forge::First, limits: &w::Limits) -> Option<w::FirstRepository> {
    let state = match first.state {
        forge::FirstState::Making => w::FirstState::Making,
        forge::FirstState::Made { id } => w::FirstState::Made(w::Made::new(limits, w::MadeParts { id }).ok()?),
        forge::FirstState::Failed { step } => w::FirstState::Failed(
            w::CreateFailed::new(
                limits,
                w::CreateFailedParts {
                    step: match step {
                        forge::CreateStep::Admin => w::CreateStep::Admin,
                        forge::CreateStep::Org => w::CreateStep::Org,
                        forge::CreateStep::Repository => w::CreateStep::Repository,
                        forge::CreateStep::File => w::CreateStep::File,
                    },
                },
            )
            .ok()?,
        ),
    };
    w::FirstRepository::new(
        limits,
        w::FirstRepositoryParts {
            playground: first.playground,
            org: first.org,
            repository: first.repository,
            state,
            label: first.label,
        },
    )
    .ok()
}

/// A bounded path on this clients listener, checked before root admission (client/engine.md).
pub(crate) fn return_path(path: &[u8]) -> bool {
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
    Some(match stop {
        forge::AdminStop::NotAdmin => w::AdminStop::NotAdmin,
        forge::AdminStop::Unmarked => w::AdminStop::Unmarked,
        forge::AdminStop::Other => w::AdminStop::Other,
        forge::AdminStop::Refused { status, stage } => w::AdminStop::Refused(
            w::AdminRefused::new(
                limits,
                w::AdminRefusedParts {
                    status,
                    stage: match stage {
                        forge::SetupStage::Admin => w::SetupStage::Admin,
                        forge::SetupStage::Bot => w::SetupStage::Bot,
                        forge::SetupStage::Token => w::SetupStage::Token,
                        forge::SetupStage::Org => w::SetupStage::Org,
                        forge::SetupStage::Repository => w::SetupStage::Repository,
                        forge::SetupStage::File => w::SetupStage::File,
                        forge::SetupStage::App => w::SetupStage::App,
                        forge::SetupStage::Installation => w::SetupStage::Installation,
                        forge::SetupStage::State => w::SetupStage::State,
                    },
                },
            )
            .ok()?,
        ),
    })
}

fn project_status(value: forge::StatusView, limits: &w::Limits) -> Option<Box<[u8]>> {
    let brief = match value.brief {
        Some(brief) => Some(
            w::StatusBrief::new(
                limits,
                w::StatusBriefParts {
                    sentence: brief.sentence,
                    lede: brief.lede,
                    at: brief.at.as_nanos(),
                    landed: brief.landed,
                },
            )
            .ok()?,
        ),
        None => None,
    };
    let document = w::ProjectStatus::new(
        limits,
        w::ProjectStatusParts {
            project: value.project,
            landed: value.landed,
            last_landed: match value.last_landed {
                Some(at) => Some(at.as_nanos()),
                None => None,
            },
            brief,
            previous: match value.previous {
                Some(at) => Some(at.as_nanos()),
                None => None,
            },
            seen: value.seen,
            queued: value.queued,
            red: value.red,
            branch: value.branch,
        },
    )
    .ok()?;
    Some(wire::project_status(&document))
}

fn follow_state(value: forge::FollowState, limits: &w::Limits) -> Option<w::FollowState> {
    Some(match value {
        forge::FollowState::NotMade => w::FollowState::NotMade,
        forge::FollowState::WaitsWord => w::FollowState::WaitsWord,
        forge::FollowState::Producing => w::FollowState::Producing,
        forge::FollowState::WaitsFor(change) => {
            w::FollowState::WaitsFor(w::FollowWait::new(limits, w::FollowWaitParts { change }).ok()?)
        }
        forge::FollowState::HeldByOwner(words) => {
            w::FollowState::HeldByOwner(w::FollowWords::new(limits, w::FollowWordsParts { words }).ok()?)
        }
        forge::FollowState::Held(words) => {
            w::FollowState::Held(w::FollowWords::new(limits, w::FollowWordsParts { words }).ok()?)
        }
        forge::FollowState::Ended(words) => {
            w::FollowState::Ended(w::FollowWords::new(limits, w::FollowWordsParts { words }).ok()?)
        }
    })
}
fn follows_document(value: forge::FollowsView, limits: &w::Limits) -> Option<Box<[u8]>> {
    let mut follows = List::with_capacity(limits.follows_panel_follows);
    for row in value.follows {
        let mut entries = List::with_capacity(limits.follow_view_entries);
        for entry in row.entries {
            entries
                .push(
                    w::FollowEntry::new(
                        limits,
                        w::FollowEntryParts {
                            after: entry.after,
                            change: entry.change,
                            state: follow_state(entry.state, limits)?,
                        },
                    )
                    .ok()?,
                )
                .ok()?;
        }
        let mut skips = List::with_capacity(limits.follow_view_skips);
        for skip in row.skips {
            skips
                .push(w::FollowSkip::new(limits, w::FollowSkipParts { after: skip.after, reason: skip.reason }).ok()?)
                .ok()?;
        }
        let parity = match row.parity {
            Some(parity) => Some(
                w::FollowParity::new(
                    limits,
                    w::FollowParityParts {
                        source: parity.source,
                        head: parity.head,
                        run: parity.run,
                        verdict: match parity.verdict {
                            forge::FollowVerdict::Passed => w::FollowVerdict::Passed,
                            forge::FollowVerdict::Failing(count) => w::FollowVerdict::Failing(
                                w::FollowFailing::new(limits, w::FollowFailingParts { count }).ok()?,
                            ),
                            forge::FollowVerdict::Many => w::FollowVerdict::Many,
                        },
                    },
                )
                .ok()?,
            ),
            None => None,
        };
        follows
            .push(
                w::FollowView::new(
                    limits,
                    w::FollowViewParts {
                        number: row.number,
                        setting: forge::settings::follow_record(&row.setting)?,
                        source: row.source,
                        source_branch: row.source_branch,
                        target: row.target,
                        target_branch: row.target_branch,
                        task: row.task,
                        stopped: row.stopped,
                        lag: row.lag,
                        entries,
                        skips,
                        parity,
                    },
                )
                .ok()?,
            )
            .ok()?;
    }
    Some(wire::follows_panel(
        &w::FollowsPanel::new(limits, w::FollowsPanelParts { project: value.project, follows }).ok()?,
    ))
}

/// Encode a whole forge document, with deployment addresses at this boundary.
#[must_use]
#[expect(clippy::too_many_lines, reason = "one exhaustive encoding map for forge documents")]
pub fn document(value: forge::Document, forges: &crate::forges::Forges, limits: &w::Limits) -> Option<Box<[u8]>> {
    match value {
        forge::Document::Change(value) => panel(value, forges, limits),
        forge::Document::Follows(value) => follows_document(value, limits),
        forge::Document::Dismissals(rows) => {
            let mut dismissed = List::with_capacity(u32::try_from(rows.len()).ok()?);
            for row in rows {
                let value = match row {
                    forge::PersonFact::KeepAsking { project, subject, at } => w::PersonFact::KeepAsking(
                        w::KeepAsking::new(
                            limits,
                            w::KeepAskingParts {
                                project,
                                subject: temper_protocol_reaches::subject_record(subject),
                                at,
                            },
                        )
                        .ok()?,
                    ),
                    forge::PersonFact::ToBegin { project } => {
                        w::PersonFact::ToBegin(w::ToBegin::new(limits, w::ToBeginParts { project }).ok()?)
                    }
                    forge::PersonFact::GoalFrom { project, chat, goal, at } => w::PersonFact::GoalFrom(
                        w::GoalFrom::new(limits, w::GoalFromParts { project, chat, goal, at }).ok()?,
                    ),
                };
                dismissed.push(value).ok()?;
            }
            let document = w::Dismissals::new(limits, w::DismissalsParts { dismissed }).ok()?;
            Some(wire::dismissals(&document))
        }
        forge::Document::Landing(value) => {
            let mut members = List::with_capacity(u32::try_from(value.members.len()).ok()?);
            for member in value.members {
                members.push(landing_member(member, forges, limits)?).ok()?;
            }
            let hold = match &value.phase {
                forge::LandingPhase::Held { why: forge::LandingHold::Faults { member, gate }, .. } => {
                    Some(w::LandingHold::Faults(
                        w::HeldFaults::new(limits, w::HeldFaultsParts { member: *member, gate: *gate }).ok()?,
                    ))
                }
                forge::LandingPhase::Waiting
                | forge::LandingPhase::Preparing
                | forge::LandingPhase::Proving
                | forge::LandingPhase::Proved { .. }
                | forge::LandingPhase::Turn { .. }
                | forge::LandingPhase::Landed
                | forge::LandingPhase::Cancelled => None,
            };
            let panel = w::LandingPanel::new(
                limits,
                w::LandingPanelParts {
                    task: value.task,
                    project: value.project,
                    phase: landing_phase(&value.phase),
                    hold,
                    members,
                    preparation: value.preparation,
                    stale_proofs: value.stale_proofs,
                    turn: value.turn,
                },
            )
            .ok()?;
            Some(wire::landing_panel(&panel))
        }
        forge::Document::Aways(value) => aways(value, limits),
        forge::Document::Decisions(value) => decisions(value, limits),
        forge::Document::Status(value) => project_status(value, limits),
        forge::Document::Changes(value) => changes_panel(value, limits),
        forge::Document::Landings(value) => {
            let mut landings = List::with_capacity(u32::try_from(value.landings.len()).ok()?);
            for line in value.landings {
                let mut changes = List::with_capacity(u32::try_from(line.changes.len()).ok()?);
                for change in line.changes {
                    changes.push(change).ok()?;
                }
                let mut repositories = List::with_capacity(u32::try_from(line.repositories.len()).ok()?);
                for repository in line.repositories {
                    repositories.push(repository).ok()?;
                }
                let mut landed = List::with_capacity(u32::try_from(line.landed.len()).ok()?);
                for commit in line.landed {
                    landed.push(commit).ok()?;
                }
                landings
                    .push(
                        w::LandingLine::new(
                            limits,
                            w::LandingLineParts {
                                task: line.task,
                                phase: landing_phase(&line.phase),
                                changes,
                                repositories,
                                landed,
                            },
                        )
                        .ok()?,
                    )
                    .ok()?;
            }
            let panel = w::LandingsPanel::new(
                limits,
                w::LandingsPanelParts { project: value.project, turn: value.turn, landings },
            )
            .ok()?;
            Some(wire::landings_panel(&panel))
        }
    }
}
fn aways(value: forge::AwaysView, limits: &w::Limits) -> Option<Box<[u8]>> {
    let mut rows = List::with_capacity(u32::try_from(value.rows.len()).ok()?);
    for row in value.rows {
        rows.push(
            w::AwayRow::new(
                limits,
                w::AwayRowParts {
                    project: row.project,
                    since: row.since.as_nanos(),
                    until: match row.until {
                        Some(at) => Some(at.as_nanos()),
                        None => None,
                    },
                    landed: row.landed,
                    off: match row.off {
                        Some(at) => Some(at.as_nanos()),
                        None => None,
                    },
                },
            )
            .ok()?,
        )
        .ok()?;
    }
    let document = w::Aways::new(limits, w::AwaysParts { rows }).ok()?;
    Some(wire::aways(&document))
}

fn decisions(value: forge::DecisionsView, limits: &w::Limits) -> Option<Box<[u8]>> {
    let mut rows = List::with_capacity(u32::try_from(value.rows.len()).ok()?);
    for row in value.rows {
        rows.push(
            w::Decision::new(
                limits,
                w::DecisionParts {
                    number: row.number,
                    task: row.task,
                    at: row.at.as_nanos(),
                    question: row.question,
                    took: row.took,
                    why: row.why,
                    overturned: row.overturned,
                    reach: temper_protocol_reaches::reach_record(&row.reach),
                },
            )
            .ok()?,
        )
        .ok()?;
    }
    let mut asked = List::with_capacity(u32::try_from(value.asked.len()).ok()?);
    for row in value.asked {
        asked
            .push(
                w::Asked::new(
                    limits,
                    w::AskedParts {
                        number: row.number,
                        task: row.task,
                        at: row.at.as_nanos(),
                        reach: temper_protocol_reaches::reach_record(&row.reach),
                    },
                )
                .ok()?,
            )
            .ok()?;
    }
    let document =
        w::Decisions::new(limits, w::DecisionsParts { project: value.project, rows, asked, cut: value.cut }).ok()?;
    Some(wire::decisions(&document))
}

fn changes_panel(value: forge::ChangesView, limits: &w::Limits) -> Option<Box<[u8]>> {
    let mut queues = List::with_capacity(limits.changes_panel_queues);
    for queue in value.queues {
        let mut changes = List::with_capacity(limits.changes_queue_changes);
        for row in queue.changes {
            changes
                .push(
                    w::ChangesQueued::new(
                        limits,
                        w::ChangesQueuedParts {
                            task: row.task,
                            title: row.title,
                            ready_since: row.ready_since.as_nanos(),
                            checks_since: match row.checks_since {
                                Some(at) => Some(at.as_nanos()),
                                None => None,
                            },
                        },
                    )
                    .ok()?,
                )
                .ok()?;
        }
        queues
            .push(
                w::ChangesQueue::new(
                    limits,
                    w::ChangesQueueParts {
                        repository: queue.repository,
                        branch: queue.branch,
                        red: queue.red,
                        changes,
                    },
                )
                .ok()?,
            )
            .ok()?;
    }
    let landed = changes_landed(value.landed, limits.changes_panel_landed, limits)?;
    let landed_default = changes_landed(value.landed_default, limits.changes_panel_landed_default, limits)?;
    let panel = w::ChangesPanel::new(
        limits,
        w::ChangesPanelParts {
            project: value.project,
            landing: value.landing,
            queues,
            more: value.more,
            landed,
            landed_more: value.landed_more,
            landed_default,
            landed_default_more: value.landed_default_more,
        },
    )
    .ok()?;
    Some(wire::changes_panel(&panel))
}

fn changes_landed(
    rows: Box<[forge::ChangesLanded]>,
    capacity: u32,
    limits: &w::Limits,
) -> Option<List<w::ChangesLanded>> {
    let mut landed = List::with_capacity(capacity);
    for row in rows {
        landed
            .push(
                w::ChangesLanded::new(
                    limits,
                    w::ChangesLandedParts {
                        task: row.task,
                        title: row.title,
                        at: row.at.as_nanos(),
                        commit: row.commit,
                        default_branch: row.default_branch,
                    },
                )
                .ok()?,
            )
            .ok()?;
    }
    Some(landed)
}

fn landing_member(
    member: forge::LandingMember,
    forges: &crate::forges::Forges,
    limits: &w::Limits,
) -> Option<w::Member> {
    let pull = match member.pull {
        Some(number) => Some(
            w::Pull::new(
                limits,
                w::PullParts {
                    number,
                    address: pull_address(member.forge, &member.owner, &member.name, number, forges),
                },
            )
            .ok()?,
        ),
        None => None,
    };
    let mut gates = List::with_capacity(u32::try_from(member.gates.len()).ok()?);
    for gate in member.gates {
        let proof = match gate.proof {
            forge::GateProof::Unkeyed => w::Proof::Unkeyed,
            forge::GateProof::Keyed => w::Proof::Keyed,
            forge::GateProof::Waiting => w::Proof::Waiting,
            forge::GateProof::Dispatched { attempt, run } => {
                w::Proof::Dispatched(w::Dispatched::new(limits, w::DispatchedParts { attempt, run }).ok()?)
            }
            forge::GateProof::Passed(run) => w::Proof::Passed(receipted(run, limits)?),
            forge::GateProof::Failed(run) => w::Proof::Failed(receipted(run, limits)?),
        };
        gates
            .push(
                w::GateProof::new(
                    limits,
                    w::GateProofParts {
                        name: gate.name,
                        blocking: gate.blocking,
                        key: gate.key,
                        proof,
                        faults: gate.faults,
                    },
                )
                .ok()?,
            )
            .ok()?;
    }
    w::Member::new(
        limits,
        w::MemberParts {
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
        },
    )
    .ok()
}
fn receipted(run: forge::ReceiptRun, limits: &w::Limits) -> Option<w::Receipted> {
    w::Receipted::new(limits, w::ReceiptedParts { run: run.id, number: run.number, address: run.url }).ok()
}
fn landing_phase(phase: &forge::LandingPhase) -> w::LandingPhase {
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
fn making(value: w::Making, directories: &jig_protocol_config::Directories) -> Option<forge::Making> {
    Some(match value {
        w::Making::Kept(value) => forge::Making::Kept(value.forge()),
        w::Making::New(value) => forge::Making::New(match value.into_parts().forge {
            w::ForgeAsk::Github => forge::ForgeAsk::GitHub,
            w::ForgeAsk::Owned => forge::ForgeAsk::Owned,
            value @ (w::ForgeAsk::Managed(_) | w::ForgeAsk::Connected(_)) => {
                let (value, connected) = match value {
                    w::ForgeAsk::Managed(value) => (value, false),
                    w::ForgeAsk::Connected(value) => (value, true),
                    w::ForgeAsk::Github | w::ForgeAsk::Owned => unreachable!("Forgejo place"),
                };
                let value = value.into_parts();
                let git_program = match value.git_program {
                    Some(program) => Some(directories.join(jig_protocol_config::Directory::Data, &program).ok()?),
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
        }),
    })
}
fn forge_refusal(value: forge::ForgeRefusal) -> w::ForgeRefusal {
    match value {
        forge::ForgeRefusal::Role => w::ForgeRefusal::Role,
        forge::ForgeRefusal::NoSuchForge => w::ForgeRefusal::NoSuchForge,
        forge::ForgeRefusal::NotTaken => w::ForgeRefusal::NotTaken,
        forge::ForgeRefusal::Connected => w::ForgeRefusal::Connected,
        forge::ForgeRefusal::NotConnected => w::ForgeRefusal::NotConnected,
        forge::ForgeRefusal::Busy => w::ForgeRefusal::Busy,
        forge::ForgeRefusal::NoKey => w::ForgeRefusal::NoKey,
        forge::ForgeRefusal::FirstIsBot => w::ForgeRefusal::FirstIsBot,
    }
}

fn deployment_answer(deployment: &Deployment, limits: &w::Limits) -> Box<[u8]> {
    let answer = w::DeploymentAnswer::new(
        limits,
        w::DeploymentAnswerParts {
            welcome: deployment.welcome,
            owned: deployment.owned,
            program_login: deployment.program_login.clone(),
        },
    )
    .expect("checked deployment facts fit the wire");
    wire::deployment_answer(&answer)
}

/// The kept row supplies progress and ids; settings supply the scripted model's public view.
/// temper/docs/design/protocol/setup/playground.md.
fn playground_answer(row: &forge::Playground, demo: &crate::settings::Demo, limits: &w::Limits) -> Box<[u8]> {
    let mut cues = List::with_capacity(u32::try_from(demo.cues.len()).expect("bounded playground cues"));
    for cue in &demo.cues {
        cues.push(cue.clone()).expect("bounded playground cues");
    }
    let answer = w::PlaygroundAnswer::new(
        limits,
        w::PlaygroundAnswerParts {
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
        },
    )
    .expect("playground checked at startup");
    wire::playground_answer(&answer)
}

fn panel_tiers(lines: Box<[Box<[u8]>]>) -> List<Box<[u8]>> {
    let mut tiers = List::with_capacity(u32::try_from(lines.len()).expect("admitted tiers"));
    for line in lines {
        tiers.push(line).expect("admitted tiers");
    }
    tiers
}

/// Encode a typed forge read answer (temper/docs/design/client/forge/records.md).
#[must_use]
pub fn forge_read(answer: root::ForgeReadAnswer, program_environment: &[u8], limits: &w::Limits) -> Option<Box<[u8]>> {
    use forge::settings::Edit;
    match answer {
        root::ForgeReadAnswer::View(view) => ways::view_answer(*view, limits),
        root::ForgeReadAnswer::ProgramEnvironment => {
            let record = w::ProgramEnvironmentAnswer::new(
                limits,
                w::ProgramEnvironmentAnswerParts { environment: program_environment.into() },
            )
            .ok()?;
            Some(wire::program_environment_answer(&record))
        }
        root::ForgeReadAnswer::Setting(value) => {
            let value = match value {
                Some(value) => Some(temper_engine_smith::kinds::encode(&value.value)),
                None => None,
            };
            let record = w::SettingAnswer::new(limits, w::SettingAnswerParts { value }).ok()?;
            Some(wire::setting_answer(&record))
        }
        root::ForgeReadAnswer::ProposalItems { items: values, owned_approval } => {
            let mut items = List::with_capacity(limits.proposal_items_answer_items);
            for (key, edit) in values {
                let edit = match edit {
                    Edit::Set(value) => w::SettingEdit::Set(
                        w::SettingValue::new(
                            limits,
                            w::SettingValueParts { value: temper_engine_smith::kinds::encode(&value.value) },
                        )
                        .ok()?,
                    ),
                    Edit::Remove => w::SettingEdit::Removed,
                    Edit::Reset => w::SettingEdit::Reset,
                };
                items
                    .push(
                        w::ProposalItem::new(
                            limits,
                            w::ProposalItemParts { kind: key.kind, scope: key.scope, number: key.number, edit },
                        )
                        .ok()?,
                    )
                    .ok()?;
            }
            let record =
                w::ProposalItemsAnswer::new(limits, w::ProposalItemsAnswerParts { items, owned_approval }).ok()?;
            Some(wire::proposal_items_answer(&record))
        }
    }
}

fn forge_settings_query(read: w::Read) -> Option<root::ForgeRead> {
    match read {
        w::Read::ProgramEnvironment => Some(root::ForgeRead::ProgramEnvironment),
        w::Read::Setting(value) => {
            Some(root::ForgeRead::Setting { kind: value.kind(), scope: value.scope(), number: value.number() })
        }
        w::Read::RevisionSetting(value) => Some(root::ForgeRead::RevisionSetting {
            kind: value.kind(),
            scope: value.scope(),
            number: value.number(),
            revision: value.revision(),
        }),
        w::Read::ProposalItems(value) => Some(root::ForgeRead::ProposalItems { key: value.key() }),
        w::Read::View(value) => {
            let w::ViewReadParts { project, address } = value.into_parts();
            Some(root::ForgeRead::View { project, address })
        }
        w::Read::Playground | w::Read::Deployment => None,
    }
}

fn setup_first(first: w::SetUpFirst) -> forge::SetUpFirst {
    match first {
        w::SetUpFirst::Empty(first) => {
            let w::FirstEmptyParts { org, repository, playground } = first.into_parts();
            forge::SetUpFirst::Empty { org, repository, playground }
        }
        w::SetUpFirst::Copies(copies) => {
            let mut addresses = List::with_capacity(copies.addresses().len());
            for address in copies.into_parts().addresses.into_boxed() {
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
fn copy_addresses(addresses: Box<[Box<[u8]>]>, limits: &w::Limits) -> Option<w::CopyAddresses> {
    let mut list = List::with_capacity(wire::FIRST_COPIES);
    for address in addresses {
        list.push(address).ok()?;
    }
    w::CopyAddresses::new(limits, w::CopyAddressesParts { addresses: list }).ok()
}

/// The person's whole trials document (temper/docs/design/domain/trial.md, 6).
#[must_use]
pub fn trials(rows: Box<[temper_engine_domain_trials::Row]>, limits: &w::Limits) -> Option<Box<[u8]>> {
    use temper_engine_domain_trials::{Class, State, Step, Why};
    let mut encoded = List::with_capacity(64);
    for row in rows {
        let state = match row.state {
            State::Running => w::TrialState::Running,
            State::Answered { seconds } => {
                w::TrialState::Answered(w::TrialAnswered::new(limits, w::TrialAnsweredParts { seconds }).ok()?)
            }
            State::Failed { why } => {
                let why = match why {
                    Why::Refused => w::TrialWhy::Refused,
                    Why::Waited => w::TrialWhy::Waited,
                    Why::Ended { class } => w::TrialWhy::Ended(
                        w::TrialEnded::new(
                            limits,
                            w::TrialEndedParts {
                                class: match class {
                                    Class::Transient => w::TrialClass::Transient,
                                    Class::Permanent => w::TrialClass::Permanent,
                                    Class::Run => w::TrialClass::Run,
                                    Class::Agent => w::TrialClass::Agent,
                                    Class::Lost => w::TrialClass::Lost,
                                    Class::Invalid => w::TrialClass::Invalid,
                                    Class::Refused => w::TrialClass::Refused,
                                },
                            },
                        )
                        .ok()?,
                    ),
                };
                w::TrialState::Failed(w::TrialFailed::new(limits, w::TrialFailedParts { why }).ok()?)
            }
        };
        encoded
            .push(
                w::TrialRow::new(
                    limits,
                    w::TrialRowParts {
                        account: row.account,
                        task: row.task,
                        took: row.took,
                        step: match row.step {
                            Step::Starts => w::TrialStep::Starts,
                            Step::Answers => w::TrialStep::Answers,
                        },
                        state,
                    },
                )
                .ok()?,
            )
            .ok()?;
    }
    let document = w::Trials::new(limits, w::TrialsParts { rows: encoded }).ok()?;
    let mut writer = Writer::new(usize::try_from(document.measure()).ok()?);
    document.encode(&mut writer).ok()?;
    Some(writer.finish())
}

/// The watcher's approvals document, each row's kind and files-page address from configuration,
/// as a panel's pull request's are (client/forge/records.md; client/forge/README.md, 4.1).
#[must_use]
pub fn approvals(document: root::Approvals, forges: &crate::forges::Forges, limits: &w::Limits) -> Option<Box<[u8]>> {
    let mut rows = List::with_capacity(32);
    for row in document.rows {
        let kind = match forges.get(row.forge) {
            Some(forge) => match forge.kind {
                crate::settings::Kind::GitHub => w::Kind::Github,
                crate::settings::Kind::Forgejo => w::Kind::Forgejo,
            },
            None => w::Kind::Forgejo,
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
            sets.push(w::ApprovalSet::new(limits, w::ApprovalSetParts { number: set.number, name: set.name }).ok()?)
                .ok()?;
        }
        rows.push(
            w::ApprovalRow::new(
                limits,
                w::ApprovalRowParts {
                    task: row.task,
                    project: row.project,
                    goal: row.goal,
                    forge: row.forge,
                    kind,
                    owner: row.owner,
                    name: row.name,
                    pull: row.pull,
                    address,
                    title: row.title,
                    since: row.since.as_nanos(),
                    scope: row.scope,
                    rule: row.rule,
                    role: row.role,
                    required: row.required,
                    counted: row.counted,
                    sets,
                    whole: row.whole,
                    yours: match row.yours {
                        root::YourReview::Counts => w::ApprovalYours::Counts,
                        root::YourReview::NoAccount => w::ApprovalYours::NoAccount,
                        root::YourReview::TooFew => w::ApprovalYours::TooFew,
                        root::YourReview::OtherRole => w::ApprovalYours::OtherRole,
                    },
                    size: match row.size {
                        Some(size) => Some(
                            w::ApprovalSize::new(
                                limits,
                                w::ApprovalSizeParts {
                                    files: size.files,
                                    additions: size.additions,
                                    deletions: size.deletions,
                                    cut: size.cut,
                                },
                            )
                            .ok()?,
                        ),
                        None => None,
                    },
                },
            )
            .ok()?,
        )
        .ok()?;
    }
    let document = w::Approvals::new(limits, w::ApprovalsParts { rows, more: document.more }).ok()?;
    Some(wire::approvals(&document))
}
