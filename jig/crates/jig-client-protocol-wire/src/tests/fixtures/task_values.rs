use super::*;

pub fn task(limits: &Limits) -> Task {
    let rows = chip_facts(limits, chip_phase_owner_fields());
    Task {
        concluded: None,
        talk: None,
        session: None,
        dependency_facts: List::with_capacity(0),
        unread: List::with_capacity(0),
        kept_account: None,
        run_wait: None,
        settings: Some(TaskSettings { task: 1, run: None, newest: 1 }),
        llm: None,
        label: 0,
        flags: TaskFlags { ended_by: EndedBy::Result, conversation: false, listed: true, retention: Retention::Kept },
        last_activity: 0,
        made: 0,
        tasks_left: 5,
        waits_on: WaitsOn::Nothing,
        chip: chip(&rows),
        goal: Some(u64::MAX),
        priority: Some(u32::MAX),
        lineage_chips: Box::from([]),
        inputs: Box::from([]),
        project: u32::MAX,
        requester: Requester::Person(requester_person()),
        lineage: List::with_capacity(0),
        spec: specification_document(limits),
        spec_revision: revision(),
        person_revision: None,
        taken_by: Some(u64::MAX),
        instructions: Some(document_text(limits)),
        wake: wake_policy(),
        subscriptions: subscriptions(),
        result_contract: TaskContractDocument::Report(task_contract_document_report()),
        dependencies: Box::from([]),
        references: Box::from([]),
        cut: true,
        resources: Box::from([]),
        budget: budget(),
        authority: authority_document(),
        hold: None,
        proposals: List::with_capacity(0),
        activity: u64::MAX,
        changed: u64::MAX,
        acts: Box::from([]),
        result: None,
        delegates: Box::from([]),
        context: None,
    }
}

pub fn document_specification(limits: &Limits) -> DocumentSpecification {
    DocumentSpecification {
        asks: false,
        words: document_text(limits),
        parameters: List::with_capacity(0),
        inputs: List::with_capacity(0),
    }
}

pub fn specification_document(limits: &Limits) -> SpecificationDocument {
    SpecificationDocument { specification: document_specification(limits), cut: true }
}

pub fn subscriptions() -> Subscriptions {
    Subscriptions { entries: List::with_capacity(0) }
}

pub fn task_contract_document_report() -> TaskContractDocumentReport {
    TaskContractDocumentReport { words: u32::MAX }
}

pub fn delegation() -> Delegation {
    Delegation { kinds: List::with_capacity(0), tasks: u32::MAX, depth: u32::MAX }
}

pub fn authority() -> Authority {
    Authority {
        models: None,
        tools: u64::MAX,
        grants: Box::from([]),
        delegation: delegation(),
        spend: u64::MAX,
        deadline: None,
        notes: u8::MAX,
        note_resources: Box::from([]),
    }
}

pub fn authority_document() -> AuthorityDocument {
    AuthorityDocument { cut: false, authority: authority() }
}

pub fn requester_person() -> RequesterPerson {
    RequesterPerson { value: u64::MAX }
}
