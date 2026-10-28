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
    let mut authority = held.steering.authority;
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
