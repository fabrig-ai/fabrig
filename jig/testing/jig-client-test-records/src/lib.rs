//! Shared literals for client document codec tests (testing-strategy.md, 8).
/// Template numbers used by the literal wire fixtures.
pub const CHAT: u32 = 1;
pub const GOAL: u32 = 2;
use jig_client_protocol_wire as w;
use jig_client_protocol_wire::Record as _;
use skein_lib::{Duration, List, Writer};
mod owner;
pub use owner::*;

/// The stream's first record, with its implicit me delivery (protocol/client/wire.md, 2).
#[must_use]
pub fn opened(hello: w::Hello, me: w::Me) -> w::StreamEvent {
    opened_with_layouts(hello, me, &[])
}

#[must_use]
pub fn opened_with_layouts(hello: w::Hello, me: w::Me, applications: &[w::Layout]) -> w::StreamEvent {
    let value = w::DeliveryValue::Me(me);
    let record = value.encode(&w::CEILINGS).expect("fixture me fits");
    let me = w::Owned { owner: w::Owner::Jig, version: w::VERSION, record };
    w::StreamEvent::Opened(w::Opened { hello, layouts: layouts(applications), me })
}

#[must_use]
pub fn layouts(applications: &[w::Layout]) -> List<w::Layout> {
    let mut layouts = List::with_capacity(
        u32::try_from(applications.len()).expect("bounded fixture").checked_add(1).expect("jig included"),
    );
    layouts.push(w::Layout { owner: w::Owner::Jig, layout: w::LAYOUT }).expect("jig layout");
    for layout in applications {
        layouts.push(layout.clone()).expect("fixture layouts");
    }
    layouts
}

#[must_use]
pub const fn family_layout(number: u16, layout: [u8; 32]) -> w::Layout {
    w::Layout { owner: w::Owner::Family(w::Family { number }), layout }
}

#[must_use]
pub fn limits() -> w::Limits {
    w::Limits {
        document_text_bytes: 32,
        tasks_rows: 4096,
        tree_rows: 4096,
        tree_row_dependencies: 4096,
        inbox_needs: 4096,
        inbox_updates: 4096,
        inbox_selection_some: 4096,
        deployment_hosts: 4096,
        deployment_attempts: 4096,
        deployment_accounts: 4096,
        accounts_accounts: 4096,
        accounts_signing: 4096,
        account_line_sources: 4096,
        account_sources: 4096,
        host_runs: 4096,
        account_runs: 4096,
        account_waiting: 4096,
        run_text_text: 32,
        current_call_input: 32,
        current_call_tool: 32,
        call_identity_id: 32,
        // Leave 64 bytes of room plus the cursor's 32-byte layout.
        cursor_value: 96,
        inbox_selection_goals: 2,
        me_projects: 512,
        document_specification_parameters: 2,
        document_specification_inputs: 2,
        part_key_path_value: 2,
        part_key_path_value_item: 32,
        ..w::CEILINGS
    }
}
#[must_use]
pub fn wire_text(value: &[u8]) -> w::DocumentText {
    w::DocumentText { bytes: Box::from(value), cut: false }
}
#[must_use]
pub fn chip_record() -> w::Chip {
    owner_waiting_chip_record()
}

#[must_use]
pub fn chip_pieces(count: u32) -> Box<[u8]> {
    let row = chip_record();
    let mut encoded = Writer::new(skein_lib::length::usize_of(row.measure(&limits()).expect("bounded fixture")));
    row.write(&limits(), &mut encoded).expect("literal fixture fits");
    let encoded = encoded.finish();
    let mut pieces = Writer::new(
        encoded
            .len()
            .checked_add(4)
            .expect("literal fixture fits")
            .checked_mul(skein_lib::length::usize_of(count))
            .expect("literal fixture fits"),
    );
    for _ in 0..count {
        skein_codec::piece::write(&mut pieces, &encoded).expect("literal fixture fits");
    }
    pieces.finish()
}
#[must_use]
pub fn frame_record() -> w::Me {
    w::Me {
        providers: Box::new([]),
        providers_cut: false,
        key_bytes: 256,
        redirect_bytes: 4096,
        words_bytes: 8192,
        sign_in_expires: 86_400_000_000_000,
        party: w::MeParty {
            number: 7,
            kind: w::PartyKind::Person,
            login: w::DocumentText { bytes: Box::from([]), cut: false },
            name: w::DocumentText { bytes: Box::from([]), cut: false },
        },
        projects: Box::from([]),
        needs: w::Needs { questions: 7, proposals: 7, escalations: 7, person_tasks: 7 },
        needs_total: 7,
        updates: w::Count { value: 7, limited: false },
        results_seen: 7,
        cut: false,
        retention: Duration::from_nanos(19),
        deployment_role: None,
    }
}
#[must_use]
pub fn task_record() -> w::Task {
    w::Task {
        concluded: None,
        talk: None,
        session: None,
        dependency_facts: List::with_capacity(0),
        unread: List::with_capacity(0),
        kept_account: None,
        run_wait: None,
        settings: Some(w::TaskSettings { task: 1, run: None, newest: 1 }),
        llm: None,
        label: 0,
        flags: w::TaskFlags {
            ended_by: w::EndedBy::Result,
            conversation: false,
            listed: false,
            retention: w::Retention::Kept,
        },
        last_activity: 0,
        made: 7,
        tasks_left: 5,
        waits_on: w::WaitsOn::Nothing,
        wait: None,
        nearest: None,
        waits: List::with_capacity(0),
        wait_count: 0,
        chip: chip_record(),
        goal: None,
        priority: None,
        lineage_chips: Box::from([]),
        inputs: Box::from([]),
        project: 7,
        requester: w::Requester::Task(w::RequesterTask { value: 7 }),
        lineage: List::with_capacity(0),
        spec: wire_specification_document(),
        spec_revision: w::Revision { value: 7 },
        person_revision: None,
        taken_by: None,
        instructions: None,
        wake: w::WakePolicy {
            words: w::WakeRule::Never,
            notices: w::WakeRule::Never,
            news: w::WakeRule::Never,
            results: w::ResultsWake::Never,
            questions: false,
            answers: false,
            timers: false,
        },
        subscriptions: w::Subscriptions { entries: List::with_capacity(0) },
        result_contract: w::TaskContractDocument::Approval(w::TaskContractDocumentApproval {
            state: Box::from([]),
            part: w::Part {
                connector: 7,
                kind: w::PartKind::Resource,
                key: w::PartKey::Path(w::PartKeyPath { value: List::with_capacity(0) }),
            },
        }),
        dependencies: Box::from([]),
        references: Box::from([]),
        cut: false,
        resources: Box::from([]),
        budget: w::Budget {
            numbers: w::Numbers { budget: 7, spent: 7, spent_below: 7, reserved: 7 },
            left: 7,
            funder: w::Funder::Task(w::FunderTask { value: 7 }),
            deadline: None,
        },
        authority: wire_owner_authority_document(),
        hold: None,
        proposals: List::with_capacity(0),
        activity: 7,
        changed: 7,
        acts: Box::from([]),
        result: None,
        delegates: Box::from([]),
        context: None,
    }
}
#[must_use]
pub fn wire_specification_document() -> w::SpecificationDocument {
    w::SpecificationDocument {
        specification: w::DocumentSpecification {
            asks: false,
            words: w::DocumentText { bytes: Box::from([]), cut: false },
            parameters: List::with_capacity(0),
            inputs: List::with_capacity(0),
        },
        cut: false,
    }
}
#[must_use]
pub fn wire_owner_authority_document() -> w::AuthorityDocument {
    w::AuthorityDocument {
        cut: false,
        authority: w::Authority {
            models: Some(w::ModelSet::All),
            tools: 7,
            grants: Box::from([]),
            delegation: w::Delegation { kinds: List::with_capacity(0), tasks: 7, depth: 7 },
            spend: 7,
            deadline: None,
            notes: 7,
            note_resources: Box::from([]),
        },
    }
}
#[must_use]
pub fn chip_root_fact() -> w::Fact {
    w::Fact::Chip(w::ChipRow {
        parent: None,
        value: w::ChipFields {
            task: 7,
            title: wire_text(b""),
            executor: w::ChipExecutor::Agent(w::ChipExecutorAgent { charter: 7 }),
            phase: w::Phase::Waiting,
            details: 1,
        },
    })
}
#[must_use]
pub fn wire_owner_chip_details_row() -> w::ChipDetailsRow {
    w::ChipDetailsRow {
        parent: Some(1),
        value: w::ChipDetailsFields {
            dependencies: List::with_capacity(0),
            project: 7,
            executor: w::Executor::Agent(w::ExecutorAgent { charter: 7 }),
            phase_since: 7,
            run_since: None,
            attempt: None,
            retry: None,
            wake: w::WakePolicy {
                words: w::WakeRule::Never,
                notices: w::WakeRule::Never,
                news: w::WakeRule::Never,
                results: w::ResultsWake::Never,
                questions: false,
                answers: false,
                timers: false,
            },
            person_wait: None,
            hold: None,
            budget: w::Budget {
                numbers: w::Numbers { budget: 7, spent: 7, spent_below: 7, reserved: 7 },
                left: 7,
                funder: w::Funder::Task(w::FunderTask { value: 7 }),
                deadline: None,
            },
            activity: 7,
            resources: Box::from([]),
            resources_cut: false,
            resources_observed: false,
            procedure: None,
            live_delegates: 7,
            closing_connectors: None,
            ending: None,
            ended_at: None,
            observation: None,
        },
    }
}
#[must_use]
pub fn chip_from_rows(rows: &[w::Fact]) -> w::Chip {
    let mut length = 0_usize;
    for row in rows {
        length = length
            .checked_add(skein_lib::length::usize_of(row.measure(&w::CEILINGS).expect("bounded fixture")))
            .expect("literal fixture fits")
            .checked_add(4)
            .expect("literal fixture fits");
    }
    let mut bytes = Writer::new(length);
    for row in rows {
        bytes.put(&fact_bytes(row)).expect("literal fixture fits");
    }
    w::Chip { rows: bytes.finish() }
}
#[must_use]
pub fn fact_bytes(value: &w::Fact) -> Box<[u8]> {
    let mut row = Writer::new(skein_lib::length::usize_of(value.measure(&w::CEILINGS).expect("bounded fixture")));
    value.write(&w::CEILINGS, &mut row).expect("literal fixture fits");
    let mut piece = Writer::new(skein_lib::length::usize_of(
        value.measure(&w::CEILINGS).expect("bounded fixture").checked_add(4).expect("literal fixture fits"),
    ));
    skein_codec::piece::write(&mut piece, &row.finish()).expect("literal fixture fits");
    piece.finish()
}

mod live;
pub use live::*;
mod accounts;
mod live_values;
pub use accounts::*;
pub use live_values::*;

mod objects;
pub use objects::*;

mod object_reads;
pub use object_reads::*;

mod pages;
pub use pages::*;

mod page_reads;
pub use page_reads::*;

mod activity_reads;
pub use activity_reads::*;

pub mod settings;
pub mod settings_proposals;

mod waits;
pub use waits::structured_waits;
