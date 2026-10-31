use super::*;

pub fn task(limits: &Limits) -> Task {
    let rows = chip_facts(limits, chip_phase_owner_fields(limits));
    Task::new(
        limits,
        TaskParts {
            dependency_facts: List::with_capacity(0),
            kept_account: None,
            run_wait: None,
            settings: Some(
                TaskSettings::new(limits, TaskSettingsParts { task: 1, run: None, newest: 1 })
                    .expect("fixture settings"),
            ),
            llm: None,
            label: 0,
            flags: TaskFlags::new(
                limits,
                TaskFlagsParts {
                    ended_by: EndedBy::Result,
                    conversation: false,
                    listed: true,
                    retention: Retention::Kept,
                },
            )
            .expect("fixture flags"),
            last_activity: 0,
            waits_on: WaitsOn::Nothing,
            chip: chip(limits, &rows),
            goal: Some(u64::MAX),
            priority: Some(u32::MAX),
            lineage_chips: Box::from([]),
            inputs: Box::from([]),
            project: u32::MAX,
            requester: Requester::Person(requester_person(limits)),
            lineage: List::with_capacity(0),
            spec: specification_document(limits),
            spec_revision: revision(limits),
            person_revision: None,
            taken_by: Some(u64::MAX),
            instructions: Some(document_text(limits)),
            wake: wake_policy(limits),
            subscriptions: subscriptions(limits),
            result_contract: TaskContractDocument::Report(task_contract_document_report(limits)),
            dependencies: Box::from([]),
            references: Box::from([]),
            cut: true,
            resources: Box::from([]),
            budget: budget(limits),
            authority: authority_document(limits),
            hold: None,
            proposals: List::with_capacity(0),
            activity: u64::MAX,
            changed: u64::MAX,
            acts: Box::from([]),
            result: None,
            delegates: Box::from([]),
            context: None,
        },
    )
    .expect("whole task")
}

pub fn document_specification(limits: &Limits) -> DocumentSpecification {
    DocumentSpecification::new(
        limits,
        DocumentSpecificationParts {
            asks: false,
            words: document_text(limits),
            parameters: List::with_capacity(0),
            inputs: List::with_capacity(0),
        },
    )
    .expect("bounded document_specification")
}

pub fn specification_document(limits: &Limits) -> SpecificationDocument {
    SpecificationDocument::new(
        limits,
        SpecificationDocumentParts { specification: document_specification(limits), cut: true },
    )
    .expect("bounded specification_document")
}

pub fn subscriptions(limits: &Limits) -> Subscriptions {
    Subscriptions::new(limits, SubscriptionsParts { entries: List::with_capacity(0) }).expect("bounded subscriptions")
}

pub fn task_contract_document_report(limits: &Limits) -> TaskContractDocumentReport {
    TaskContractDocumentReport::new(limits, TaskContractDocumentReportParts { words: u32::MAX })
        .expect("bounded task_contract_document_report")
}

pub fn delegation(limits: &Limits) -> Delegation {
    Delegation::new(limits, DelegationParts { kinds: List::with_capacity(0), tasks: u32::MAX, depth: u32::MAX })
        .expect("bounded delegation")
}

pub fn authority(limits: &Limits) -> Authority {
    Authority::new(
        limits,
        AuthorityParts {
            models: None,
            tools: u64::MAX,
            grants: Box::from([]),
            delegation: delegation(limits),
            spend: u64::MAX,
            deadline: None,
            notes: u8::MAX,
            note_resources: Box::from([]),
        },
    )
    .expect("bounded authority")
}

pub fn authority_document(limits: &Limits) -> AuthorityDocument {
    AuthorityDocument::new(limits, AuthorityDocumentParts { cut: false, authority: authority(limits) })
        .expect("bounded authority_document")
}

pub fn requester_person(limits: &Limits) -> RequesterPerson {
    RequesterPerson::new(limits, RequesterPersonParts { value: u64::MAX }).expect("bounded requester_person")
}
