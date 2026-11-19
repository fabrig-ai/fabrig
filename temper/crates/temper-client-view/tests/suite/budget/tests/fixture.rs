//! Typed engine records for the two sizes of Work's project.
use super::*;
use jig_client::contract as c;
use jig_client_test_records as records;
use temper_client_protocol_wire::{self as wire, v1 as f};
use temper_client_view_kit::engine::{encoded, owned};

pub const PROJECT: u32 = 1;
pub const PARTY: u64 = 73;
pub const GOALS: u32 = 8;
pub const CHATS: u32 = 7;
pub const MILESTONES: u32 = 9;
pub const NOW: Wall = Wall::from_nanos(1_792_656_000_000_000_000);
pub const RECENT: u64 = 1_792_655_640_000_000_000;

pub fn pieces(records: impl IntoIterator<Item = Box<[u8]>>) -> Box<[u8]> {
    let mut result = Vec::new();
    for record in records {
        result.extend_from_slice(&u32::try_from(record.len()).expect("piece").to_be_bytes());
        result.extend_from_slice(&record);
    }
    result.into_boxed_slice()
}
fn need() -> Box<[u8]> {
    let cursor = w::InboxCursor::Number(
        w::InboxCursorNumber::new(&w::CEILINGS, w::InboxCursorNumberParts { number: 1 }).expect("cursor"),
    );
    let task = w::TaskLink::Stored(
        w::TaskLinkStored::new(&w::CEILINGS, w::TaskLinkStoredParts { number: 100, status: None }).expect("task link"),
    );
    let entry = w::InboxEntry::new(
        &w::CEILINGS,
        w::InboxEntryParts {
            reply: None,
            task,
            project: Some(PROJECT),
            object: w::InboxObject::PersonTask,
            activity: None,
            at: Some(RECENT),
            unread: None,
            cursor,
            projects: List::with_capacity(0),
        },
    )
    .expect("need");
    pieces([encoded(entry.measure(), |w| entry.encode(w))])
}
fn text(value: &str) -> w::DocumentText {
    w::DocumentText::new(&w::CEILINGS, w::DocumentTextParts { bytes: value.as_bytes().into(), cut: false })
        .expect("text")
}
fn template(number: u32, name: &str, label: u32, tracked: bool) -> w::Template {
    let task = records::task_record();
    w::Template::new(
        &w::CEILINGS,
        w::TemplateParts {
            number,
            name: text(name),
            label,
            tracked: tracked.then_some(label),
            executor: w::TemplateExecutor::Charter(
                w::TemplateCharter::new(&w::CEILINGS, w::TemplateCharterParts { value: 1 }).expect("charter"),
            ),
            contract: task.result_contract().clone(),
            flags: task.flags().clone(),
            shallower: 1,
            budget: w::TemplateBudget::AsAllowed(
                w::TemplateAsAllowed::new(&w::CEILINGS, w::TemplateAsAllowedParts { cap: None }).expect("budget"),
            ),
            deadline: w::TemplateDeadline::None,
            grace: None,
            llm: None,
            resources: List::with_capacity(0),
        },
    )
    .expect("template")
}
pub fn me() -> w::Me {
    let mut me = records::frame_record().into_parts();
    me.party = w::MeParty::new(
        &w::CEILINGS,
        w::MePartyParts { number: PARTY, kind: w::PartyKind::Person, login: text("pat"), name: text("Pat") },
    )
    .expect("party");
    me.deployment_role = Some(w::HeldRole::Owner);
    me.retention = Duration::from_secs(3600);
    me.needs_total = 1;
    me.needs =
        w::Needs::new(&w::CEILINGS, w::NeedsParts { questions: 0, proposals: 0, escalations: 0, person_tasks: 1 })
            .expect("needs");
    me.updates = w::Count::new(&w::CEILINGS, w::CountParts { value: 0, limited: false }).expect("count");
    let templates = [
        template(jig_client::shapes::GOAL, "goal", GOALS, true),
        template(jig_client::shapes::CHAT, "chat", CHATS, false),
        template(domain::milestones::TEMPLATE, "milestone", MILESTONES, true),
    ];
    let project = w::MeProject::new(
        &w::CEILINGS,
        w::MeProjectParts {
            left_out: 0,
            project: PROJECT,
            name: text("Shop"),
            role: w::HeldRole::Member,
            pool: None,
            creating: None,
            models: List::with_capacity(0),
            defaults: List::with_capacity(0),
            public: None,
            templates: pieces(templates.iter().map(|record| encoded(record.measure(), |w| record.encode(w)))),
        },
    )
    .expect("project");
    me.projects = pieces([encoded(project.measure(), |w| project.encode(w))]);
    w::Me::new(&w::CEILINGS, me).expect("me")
}

/// Stock browser/native client bounds, not enlarged to make a batch fit.
pub fn limits() -> domain::Limits {
    let mut limits = kit::limits();
    let client = &mut limits.client.domain;
    client.requests = skein_lib::RequestLimits {
        requests: 16,
        bytes: 16_777_216,
        out: 32,
        first: Duration::from_secs(1),
        most: Duration::from_secs(30),
        margin: Duration::from_secs(1),
    };
    client.ask_bytes = 1_048_576;
    client.handles = 33;
    client.watches = 31;
    client.turn_bytes = 1_048_576;
    client.backoff = jig_client::Backoff { first: Duration::from_secs(1), most: Duration::from_secs(30) };
    client.authority = 22;
    client.heartbeat = Duration::from_secs(30);
    client.reads = 32;
    client.parts = 16;
    client.delegates = 80;
    client.document_bytes = 1_048_576;
    client.records = 16;
    client.kept = 16;
    client.projects = 9;
    client.text = 512;
    client.long_text = 131_072;
    client.turn_text = 131_072;
    client.rows = 16;
    client.fields = 16;
    limits
}

pub struct Project {
    pub goals: Vec<c::TaskLine>,
    pub ended: Vec<c::TaskLine>,
    pub milestones: Vec<c::TaskLine>,
    pub ended_milestones: Vec<c::TaskLine>,
    pub chats: Vec<c::TaskLine>,
    pub closed: Vec<c::TaskLine>,
    pub changes: f::ChangesPanel,
    pub drafts: f::DraftsPanel,
}
fn line(task: u64, title: &str, label: u32, ended: bool) -> c::TaskLine {
    let mut line = kit::chats::line();
    line.chip.task = task;
    line.chip.title = c::DocumentText::complete(c::Text::new(title.as_bytes().into(), 512).expect("title"));
    line.label = label;
    line.tracked = (label != CHATS).then_some(1);
    line.last_activity = Wall::from_nanos(RECENT);
    line.changed = Wall::from_nanos(RECENT);
    line.delegates_done = 2;
    line.delegates_total = 3;
    if ended {
        line.ended_at = Some(Wall::from_nanos(RECENT));
        line.chip.phase = c::Phase::Ended;
        line.chip.details.phase = c::ChipPhase::Ended;
        line.chip.details.details.ended_at = line.ended_at;
        line.chip.details.details.ending = Some(c::ChipEnding::Done(c::ResultValue::Report(line.chip.title.clone())));
    }
    line
}
fn wire_line(line: &c::TaskLine) -> Box<[u8]> {
    let mut record =
        w::TaskLine::decode(&w::CEILINGS, &mut Reader::new(&temper_fixtures::milestones::line_bytes(line)))
            .expect("wire line")
            .into_parts();
    record.budget = w::Budget::new(
        &w::CEILINGS,
        w::BudgetParts {
            numbers: w::Numbers::new(
                &w::CEILINGS,
                w::NumbersParts {
                    budget: line.budget.numbers.budget,
                    spent: line.budget.numbers.spent,
                    spent_below: line.budget.numbers.spent_below,
                    reserved: line.budget.numbers.reserved,
                },
            )
            .expect("numbers"),
            left: line.budget.left,
            deadline: line.budget.deadline.map(Wall::as_nanos),
            funder: w::Funder::Task(
                w::FunderTask::new(&w::CEILINGS, w::FunderTaskParts { value: line.chip.task }).expect("funder"),
            ),
        },
    )
    .expect("row budget");
    record.parts = List::with_capacity(1);
    for part in &line.parts {
        let c::RowPart::Value(value) = part else { panic!("fixture part") };
        record.parts.push(w::RowPart::Value(owned(value.bytes.clone(), Some(value.family)))).expect("part");
    }
    let record = w::TaskLine::new(&w::CEILINGS, record).expect("line");
    encoded(record.measure(), |w| record.encode(w))
}
fn from(task: u64, title: &str) -> f::ChangeFrom {
    f::ChangeFrom { task, title: title.as_bytes().into() }
}
impl Project {
    #[expect(
        clippy::too_many_lines,
        reason = "one complete project fixture names its rows, parts and relationships together"
    )]
    pub fn new(bounded: bool) -> Self {
        // A tasks window is 16 live plus 16 ended; a changes queue/landed window is 32.
        let rows = if bounded { 16 } else { 2 };
        let changes = if bounded { 32 } else { 2 };
        let drafts = if bounded { 64 } else { 1 };
        let mut project = Self {
            goals: Vec::new(),
            ended: Vec::new(),
            milestones: Vec::new(),
            ended_milestones: Vec::new(),
            chats: Vec::new(),
            closed: Vec::new(),
            changes: f::ChangesPanel {
                    project: PROJECT,
                    landing: b"main".as_slice().into(),
                    queues: List::with_capacity(0),
                    more: 0,
                    landed: List::with_capacity(0),
                    landed_more: 0,
                    landed_default: List::with_capacity(0),
                    landed_default_more: 0,
                },
            drafts: f::DraftsPanel { project: PROJECT, drafts: List::with_capacity(drafts) },
        };
        for n in 0..rows {
            let mut goal = line(100_u64.checked_add(n).expect("fixture id"), &format!("Live goal {n}"), GOALS, false);
            goal.parts = List::from_boxed(Box::new([temper_fixtures::client_documents::goal_part(
                Some((format!("Headline live {n}").as_bytes(), NOW)),
                None,
            )]));
            project.goals.push(goal);
            let mut goal = line(200_u64.checked_add(n).expect("fixture id"), &format!("Ended goal {n}"), GOALS, true);
            goal.parts = List::from_boxed(Box::new([temper_fixtures::client_documents::goal_part(
                Some((format!("Headline ended {n}").as_bytes(), NOW)),
                None,
            )]));
            project.ended.push(goal);
            for (ended, list) in [(false, &mut project.milestones), (true, &mut project.ended_milestones)] {
                let task = if ended {
                    2000_u64.checked_add(n).expect("fixture id")
                } else {
                    1000_u64.checked_add(n).expect("fixture id")
                };
                let mut milestone =
                    line(task, &format!("{} milestone {n}", if ended { "Ended" } else { "Live" }), MILESTONES, ended);
                let nested =
                    line(3000_u64.checked_add(n).expect("fixture id"), &format!("Nested goal {n}"), GOALS, false);
                let totals = f::MilestoneTotals {
                        task,
                        done: 2,
                        failed: 0,
                        closed: 0,
                        live: 1,
                        ended: ended.then_some(RECENT),
                        last_landing: None,
                        spent: 31,
                    };
                milestone.parts = List::from_boxed(Box::new([temper_fixtures::milestones::part(
                    ended.then_some(Wall::from_nanos(RECENT)),
                    totals,
                    &[(nested, Some(format!("Nested headline {n}").as_bytes()))],
                    0,
                )]));
                list.push(milestone);
            }
            let mut chat = line(4000_u64.checked_add(n).expect("fixture id"), &format!("Draft chat {n}"), CHATS, false);
            chat.parts = List::from_boxed(Box::new([temper_fixtures::client_documents::chat_part(Some(
                format!("Draft document {n}").as_bytes(),
            ))]));
            project.chats.push(chat);
            let mut closed =
                line(5000_u64.checked_add(n).expect("fixture id"), &format!("Closed chat {n}"), CHATS, true);
            closed.parts = List::from_boxed(Box::new([temper_fixtures::client_documents::chat_part(None)]));
            project.closed.push(closed);
        }
        let mut panel = project.changes.into_parts();
        let mut queued = List::with_capacity(changes);
        let mut landed = List::with_capacity(changes);
        for n in 0..changes {
            queued
                .push(
                    f::ChangesQueued {
                            task: 6000 + u64::from(n),
                            title: format!("Queued change {n}").into_bytes().into(),
                            ready_since: RECENT,
                            checks_since: None,
                            goal: (n == 0).then(|| from(100, "Live goal 0")),
                            chat: (n == 1).then(|| from(4000, "Draft chat 0")),
                            milestone: None,
                        },
                )
                .expect("queue window");
            let signed = [
                f::ChangesSigned {
                        party: PARTY,
                        head: [1; 32],
                        at: RECENT,
                        name: Some(b"Pat".as_slice().into()),
                    },
                f::ChangesSigned {
                        party: 74,
                        head: [1; 32],
                        at: RECENT,
                        name: Some(format!("Signer {n}").into_bytes().into()),
                    },
            ];
            landed
                .push(
                    f::ChangesLanded {
                            task: 7000 + u64::from(n),
                            title: format!("Landed change {n}").into_bytes().into(),
                            at: RECENT,
                            commit: [2; 32],
                            default_branch: true,
                            goal: None,
                            chat: None,
                            milestone: None,
                            signed: List::from_boxed(Box::new(signed)),
                        },
                )
                .expect("landed window");
        }
        panel.queues = List::from_boxed(Box::new([f::ChangesQueue {
                repository: b"shop/widget".as_slice().into(),
                branch: b"main".as_slice().into(),
                red: false,
                changes: queued,
                way: f::Way::Queue,
                in_flight: None,
            }]));
        panel.landed = landed.clone();
        panel.landed_default = landed;
        project.changes = panel;
        let mut panel = project.drafts.into_parts();
        for n in 0..drafts {
            panel
                .drafts
                .push(
                    f::LandedDraft {
                            forge: wire::FORGE,
                            owner: b"shop".as_slice().into(),
                            name: b"widget".as_slice().into(),
                            path: format!("draft-{n}.md").into_bytes().into(),
                            branch: b"main".as_slice().into(),
                            commit: b"abcdef".as_slice().into(),
                            title: format!("Landed draft {n}").into_bytes().into(),
                            writer: PARTY,
                            at: RECENT,
                            chat: None,
                            goal: None,
                            ready: true,
                        },
                )
                .expect("draft window");
        }
        project.drafts = panel;
        project
    }
    fn rows(&self, label: Option<u32>, ended: bool) -> &[c::TaskLine] {
        match (label, ended) {
            (Some(GOALS), false) => &self.goals,
            (Some(GOALS), true) => &self.ended,
            (Some(MILESTONES), false) => &self.milestones,
            (Some(MILESTONES), true) => &self.ended_milestones,
            (Some(CHATS), false) => &self.chats,
            (Some(CHATS), true) => &self.closed,
            (Some(3), false) => &[], // The rail's standing roles, none in this project.
            other => panic!("unknown tasks query: {other:?}"),
        }
    }
    #[expect(
        clippy::too_many_lines,
        reason = "the fake's typed jig replies remain beside its exhaustive subject checks"
    )]
    pub fn answer(&self, subject: &w::Owned, watch: bool) -> w::Owned {
        match subject.owner() {
            w::Owner::Jig if watch => {
                let subject = w::Subject::decode(&w::CEILINGS, &mut Reader::new(subject.record())).expect("subject");
                let value = match subject {
                    w::Subject::Accounts => w::DeliveryValue::Accounts(records::accounts_record(None)),
                    w::Subject::Inbox(_) => w::DeliveryValue::Inbox(
                        w::Inbox::new(
                            &w::CEILINGS,
                            w::InboxParts {
                                party: PARTY,
                                needs: need(),
                                updates: Box::new([]),
                                next: None,
                                cached_needs: 1,
                                cached_updates: w::Count::new(&w::CEILINGS, w::CountParts { value: 0, limited: false })
                                    .expect("updates"),
                                results_seen: 0,
                                updates_time_complete: true,
                                selection: None,
                            },
                        )
                        .expect("session inbox"),
                    ),
                    w::Subject::Settings => w::DeliveryValue::Settings(
                        w::SettingsDocument::new(
                            &w::CEILINGS,
                            w::SettingsDocumentParts {
                                newest: 0,
                                settings: List::with_capacity(0),
                                held: List::with_capacity(0),
                                pending: List::with_capacity(0),
                            },
                        )
                        .expect("settings"),
                    ),
                    w::Subject::Tasks(tasks) => {
                        assert_eq!(tasks.window(), limits().client.domain.rows);
                        assert_eq!(tasks.query().project(), &Some(PROJECT), "the project's tasks");
                        let rows = self.rows(*tasks.query().label(), false);
                        w::DeliveryValue::Tasks(
                            w::Tasks::new(
                                &w::CEILINGS,
                                w::TasksParts { rows: pieces(rows.iter().map(wire_line)), more: false, ended_mark: 0 },
                            )
                            .expect("tasks"),
                        )
                    }
                    other => panic!("unexpected jig watch (no chips or deployment on Work): {other:?}"),
                };
                owned(encoded(value.measure(), |w| value.encode(w)), None)
            }
            w::Owner::Jig => {
                let read = w::Read::decode(&w::CEILINGS, &mut Reader::new(subject.record())).expect("read");
                let value = match read {
                    w::Read::Project(read) => {
                        assert_eq!(read.project(), PROJECT, "the rail's project");
                        let mut project = records::owner_project_record().into_parts();
                        project.number = PROJECT;
                        project.name = text("Shop");
                        w::ReadValue::Project(w::Project::new(&w::CEILINGS, project).expect("project"))
                    }
                    w::Read::Tasks(read) => {
                        assert_eq!(read.query().project(), &Some(PROJECT), "the project's ended tasks");
                        assert_eq!(read.query().phase(), &Some(w::PhaseClass::Ended), "only the ended group is read");
                        let rows = self.rows(*read.query().label(), true);
                        let ended = w::EndedGroup::Rows(
                            w::EndedRows::new(
                                &w::CEILINGS,
                                w::EndedRowsParts { rows: pieces(rows.iter().map(wire_line)), after: None, cut: false },
                            )
                            .expect("ended"),
                        );
                        w::ReadValue::Tasks(
                            w::TaskPage::new(
                                &w::CEILINGS,
                                w::TaskPageParts { live: Box::new([]), live_after: None, ended, ended_mark: 0 },
                            )
                            .expect("tasks page"),
                        )
                    }
                    w::Read::Inbox(read) => {
                        assert_eq!(read.project(), &Some(PROJECT), "the project's needs");
                        let selection = w::InboxSelection::new(
                            &w::CEILINGS,
                            w::InboxSelectionParts {
                                cursor: w::InboxCursor::Number(
                                    w::InboxCursorNumber::new(&w::CEILINGS, w::InboxCursorNumberParts { number: 1 })
                                        .expect("cursor"),
                                ),
                                goals: List::from_boxed(Box::new([100])),
                            },
                        )
                        .expect("selection");
                        w::ReadValue::Inbox(
                            w::InboxPage::new(
                                &w::CEILINGS,
                                w::InboxPageParts {
                                    selection: Some(pieces([encoded(selection.measure(), |w| selection.encode(w))])),
                                    cut: false,
                                    rows: Box::new([]),
                                    before: None,
                                    after: None,
                                },
                            )
                            .expect("inbox"),
                        )
                    }
                    other => panic!("unexpected jig read: {other:?}"),
                };
                owned(encoded(value.measure(), |w| value.encode(w)), None)
            }
            w::Owner::Family(family) => {
                assert_eq!(family.number(), wire::FORGE, "only the forge family is spoken");
                let envelope = w::FamilyRecord::decode(&w::CEILINGS, &mut Reader::new(subject.record()))
                    .expect("family scope and window");
                let record =
                    if watch { self.family_watch(envelope.record()) } else { Self::family_read(envelope.record()) };
                owned(record, Some(wire::FORGE))
            }
        }
    }
    fn family_read(bytes: &[u8]) -> Box<[u8]> {
        let reading = wire::decode_reading(bytes, &f::CEILINGS).expect("family read");
        match reading.read() {
            f::Read::Playground => wire::playground_answer(
                &f::PlaygroundAnswer {
                        forge: wire::FORGE,
                        project: Some(PROJECT),
                        provider: 17,
                        owner: b"playground".as_slice().into(),
                        name: b"widget".as_slice().into(),
                        state: f::PlaygroundState::Running,
                        cues: List::with_capacity(0),
                    },
            ),
            f::Read::Deployment => wire::deployment_answer(
                &f::DeploymentAnswer { welcome: true, owned: true, program_login: None, push: None },
            ),
            f::Read::ProgramEnvironment => wire::program_environment_answer(
                &f::ProgramEnvironmentAnswer { environment: Box::new([]) },
            ),
            f::Read::Setting(_) => wire::setting_answer(
                &f::SettingAnswer { value: None, revision: None },
            ),
            other => panic!("unexpected family read: {other:?}"),
        }
    }
    fn family_watch(&self, bytes: &[u8]) -> Box<[u8]> {
        let watching = wire::decode_watching(bytes, &f::CEILINGS).expect("family watch");
        match watching.watch() {
            f::Watch::Changes(subject) => {
                assert_eq!(subject.project(), PROJECT, "the project's changes");
                wire::changes_panel(&self.changes)
            }
            f::Watch::Drafts(subject) => {
                assert_eq!(subject.project(), PROJECT, "the project's drafts");
                wire::drafts_panel(&self.drafts)
            }
            f::Watch::Dismissed => {
                let fact = f::PersonFact::GoalFrom(
                    f::GoalFrom { project: PROJECT, chat: 4000, goal: 100, at: RECENT },
                );
                wire::dismissals(
                    &f::Dismissals { dismissed: List::from_boxed(Box::new([fact])) },
                )
            }
            f::Watch::Forges => wire::forges(
                &f::Forges { forges: List::with_capacity(0), linked: true },
            ),
            f::Watch::Away => {
                wire::aways(&f::Aways { rows: List::with_capacity(0) })
            }
            f::Watch::Trials => {
                let record =
                    f::Trials { rows: List::with_capacity(0) };
                encoded(record.measure(), |w| record.encode(w))
            }
            f::Watch::Approvals => wire::approvals(
                &f::Approvals { rows: List::with_capacity(0), more: 0 },
            ),
            f::Watch::Devices => wire::devices(
                &f::Devices { rows: List::with_capacity(0) },
            ),
            other => panic!("unexpected family watch: {other:?}"),
        }
    }
}
