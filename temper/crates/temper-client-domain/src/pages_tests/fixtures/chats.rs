use super::*;
use jig_client::contract::*;
const EMPTY: u32 = 0;
const ONE: u32 = 1;
const BUDGET: u64 = 100;
const MAXIMUM: u64 = 500;
const CHARTER: u32 = 1;
const TEMPLATE: u32 = 1;
const DEPTH: u32 = 2;
const WORDS: u32 = 32;
const LABEL: u32 = 7;
const GRACE: Duration = Duration::from_secs(5);
pub fn me() -> Me {
    let WatchEvent::Delivered(Delivery::Me(me)) = frame(PARTY) else { panic!("me") };
    let mut me = *me;
    let mut templates = List::with_capacity(ONE);
    templates
        .push(Template {
            resources: List::with_capacity(0),
            number: TEMPLATE,
            name: DocumentText::complete(acts::text(b"Chat")),
            executor: TemplateExecutor::Charter(CHARTER),
            contract: TaskContractDocument::Report { words: WORDS },
            shallower: ONE,
            budget: TemplateBudget::AsAllowed { cap: None },
            deadline: TemplateDeadline::None,
            tracked: None,
            flags: TaskFlags {
                ended_by: EndedBy::Result,
                conversation: true,
                listed: false,
                retention: Retention::Kept,
            },
            label: LABEL,
            grace: Some(GRACE),
            llm: None,
        })
        .expect("one template");
    let ReadValue::Escalation(held) = acts::held(ActKind::Release) else { panic!("held") };
    let mut authority = held.task.authority;
    authority.authority.delegation.depth = DEPTH;
    let mut charters = List::with_capacity(ONE);
    charters.push(CHARTER).expect("one charter");
    let creating = Creating {
        within: CreateBounds {
            authority: Box::new(authority),
            pool_left: BUDGET,
            charters,
            procedures: List::with_capacity(EMPTY),
            latest: None,
            budget: BUDGET,
            maximum_budget: MAXIMUM,
            maximum_deadline: None,
        },
        beyond: Beyond::Refused(Refusal::NotOffered),
    };
    me.projects = List::with_capacity(ONE);
    me.projects
        .push(MeProject {
            left_out: 0,
            defaults: List::with_capacity(0),
            public: None,
            project: PROJECT,
            name: DocumentText::complete(Text::new(b"Project 1".as_slice().into(), 64).expect("project name")),
            role: HeldRole::Owner,
            pool: None,
            templates,
            creating: Some(creating),
            models: List::with_capacity(EMPTY),
        })
        .expect("project");
    me
}

use jig_client::{Event, Watchable};
const ASKING: u32 = 5;
// The stock chat, custom conversation and Ask temper have separate templates and charters.
const TEMPLATES: u32 = 3;
pub fn asking_me() -> Me {
    let mut me = me();
    let project = me.projects.get_mut(0).expect("project");
    let stock = project.templates.get(0).expect("stock chat").clone();
    let mut asking = stock.clone();
    asking.number = ASKING;
    asking.executor = TemplateExecutor::Charter(ASKING);
    project.templates = List::with_capacity(TEMPLATES);
    let mut custom = stock.clone();
    custom.number = 7;
    custom.executor = TemplateExecutor::Charter(38);
    project.templates.push(stock).unwrap();
    project.templates.push(custom).unwrap();
    project.templates.push(asking).unwrap();
    let charters = &mut project.creating.as_mut().expect("creating").within.charters;
    *charters = List::with_capacity(TEMPLATES);
    charters.push(1).unwrap();
    charters.push(38).unwrap();
    charters.push(ASKING).unwrap();
    me
}
pub fn chat_line() -> TaskLine {
    let task = chat::task();
    let ChipPhaseFacts { details, .. } = task.chip.details.as_ref();
    let budget = details.budget.clone();
    TaskLine {
        chip: task.chip,
        project: PROJECT,
        requester: Requester::Person(PARTY),
        tracked: None,
        label: chat::LABEL,
        flags: TaskFlags { ended_by: EndedBy::Result, conversation: true, listed: false, retention: Retention::Kept },
        changed: Wall::EPOCH,
        last_activity: Wall::EPOCH,
        ended_at: None,
        budget,
        waits_on: WaitsOn::Nothing,
        phases: PhaseCounts::ZERO,
        delegates_done: 0,
        delegates_total: 0,
        held_below: 0,
        conversation: None,
    }
}
pub struct Page {
    pub pages: crate::pages::Pages,
}
impl Page {
    pub fn new(me: Me, address: crate::Address) -> Self {
        let limits = limits();
        let mut client = jig_client::Client::unlinked(&limits.client, SEED).expect("client");
        let mut emitted = Queue::uncapped();
        for event in [
            Event::StartUnlinked { kept: List::with_capacity(0) },
            Event::SignIn { with: jig_client::With::Transport },
            Event::Up { up: jig_client::Up::Served { served: true, versions: None } },
            Event::Watch { handle: crate::pages::me_handle(), subject: Watchable::Jig(Subject::Me) },
        ] {
            client.step_unlinked(Time::ZERO, Wall::EPOCH, event, &mut emitted);
        }
        let mut watch = None;
        while let Some(request) = emitted.pop() {
            if let jig_client::UnlinkedRequest::Down {
                down: jig_client::Down::Open { watch: token, subject: Watchable::Jig(Subject::Me) },
            } = request
            {
                watch = Some(token);
            }
        }
        client.step_unlinked(
            Time::ZERO,
            Wall::EPOCH,
            Event::Up {
                up: jig_client::Up::Watched {
                    watch: watch.expect("me watch"),
                    event: WatchEvent::Delivered(Delivery::Me(Box::new(me))),
                },
            },
            &mut emitted,
        );
        let mut pages = crate::pages::Pages::new();
        crate::pages::project_changed(&mut pages, &client);
        crate::pages::step(
            &mut pages,
            &env(&limits),
            &client,
            crate::pages::Event::Went { address },
            &mut Queue::uncapped(),
        );
        Self { pages }
    }
}
