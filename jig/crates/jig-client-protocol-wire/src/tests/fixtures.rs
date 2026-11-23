//! Small codec limits and shared client contract records.
mod delivery_values;
mod read_values;
mod stream;
mod task_values;
use crate::*;
use alloc::boxed::Box;
pub use delivery_values::delivery_values;
pub use read_values::read_values;
use skein_lib::{List, Writer};
pub use stream::*;
pub use task_values::task;

pub fn limits() -> Limits {
    limits_with_activity_bytes(CEILINGS.activity_page_rows)
}

pub fn limits_with_activity_bytes(bound: u32) -> Limits {
    let mut limits = CEILINGS;
    limits.keyed_request = 32;
    limits.spend_page_rows = 2;
    limits.activity_page_rows = bound;
    check_limits(&limits).expect("client test limits below ceilings");
    limits
}

pub fn bytes(n: u32) -> Box<[u8]> {
    skein_lib::bytes::zeroed(skein_lib::length::usize_of(n))
}

pub fn ceiling_text(n: u32) -> Box<[u8]> {
    let mut words = bytes(n);
    words.fill(b'x');
    words
}

pub fn message() -> Message {
    let limits = limits();
    Message { project: 17, task: 19, words: ceiling_text(limits.message_words) }
}

pub fn refused_moved() -> Refused {
    let how = EndingTaskFields { value: 2 };
    let reason = HoldReasonDependencyFields { task: 31, how: 1 };
    let words = DocumentText { bytes: Box::from(b"failed".as_slice()), cut: true };
    let failed = ChipEndingFailedFields { value: words };
    let mut rows = List::with_capacity(3);
    rows.push(Fact::HoldReason(HoldReasonRow { parent: None, value: HoldReasonFields::Dependency(reason) }))
        .expect("three rows");
    rows.push(Fact::Ending(EndingRow { parent: Some(0), value: EndingFields::Task(how) })).expect("three rows");
    rows.push(Fact::ChipEnding(ChipEndingRow { parent: Some(1), value: ChipEndingFields::Failed(failed) }))
        .expect("three rows");
    let rows = piece_items(&rows);
    let reason = HoldReason { rows };
    let reason = CurrentHoldDocument { value: reason };
    let what = MovedHold { reason: Some(CurrentHold::Document(reason)) };
    let now = Revision { value: u64::MAX };
    let moved = RefusalMoved { now, what: Moved::Hold(what) };
    Refused { at: None, refusal: Refusal::Moved(moved) }
}

pub fn encoded<T: Record<Limits = Limits>>(value: &T) -> Box<[u8]> {
    value.encode(&CEILINGS).expect("bounded fixture")
}

pub fn message_bytes() -> Box<[u8]> {
    encoded(&message())
}

pub fn owned() -> Owned {
    let family = Family { number: u16::MAX };
    Owned { owner: Owner::Family(family), version: u32::MAX, record: message_bytes() }
}

pub fn read_none(none: ReadNone) -> ReadAnswerNone {
    ReadAnswerNone { value: none }
}

pub fn encoded_batch_members(members: &List<Member>) -> Box<[u8]> {
    let mut length = 4_u32;
    for member in members {
        length = length.checked_add(member.measure(&CEILINGS).expect("bounded fixture")).expect("small batch");
    }
    let mut writer = Writer::new(skein_lib::length::usize_of(length));
    writer.put(&members.len().to_be_bytes()).expect("room for member count");
    for member in members {
        member.write(&CEILINGS, &mut writer).expect("room for member");
    }
    writer.finish()
}

pub fn funding_rows(count: u32) -> List<FundingRecord> {
    let period = PeriodTimes { began: u64::MAX, next: None };
    let numbers = Numbers { budget: u64::MAX, spent: 11, spent_below: 17, reserved: 23 };
    let source = FunderPeriod { project: 17, period: u64::MAX };
    let row = FundingRecord {
        period: Some(period),
        funder: Funder::Period(source),
        parent: None,
        numbers,
        made: u32::MAX,
        closed: true,
    };
    let mut rows = List::with_capacity(count);
    for _ in 0..count {
        rows.push(row.clone()).expect("reserved funding rows");
    }
    rows
}

pub fn piece_items<T: Record<Limits = Limits>>(rows: &List<T>) -> Box<[u8]> {
    let mut length = 0_u32;
    for row in rows {
        length = length.checked_add(row.measure(&CEILINGS).expect("bounded row")).expect("small fixture row");
        length = length.checked_add(4).expect("small fixture piece");
    }
    let mut out = Writer::new(skein_lib::length::usize_of(length));
    for row in rows {
        let bytes = encoded(row);
        skein_codec::piece::write(&mut out, &bytes).expect("measured piece room");
    }
    out.finish()
}

pub fn activity_line(turn: Box<[u8]>) -> DecodedActivityLine {
    let name = TurnName { attempt: u64::MAX, number: u32::MAX };
    let source = ActivityLine {
        task: u64::MAX,
        position: u64::MAX,
        at: u64::MAX,
        line: Activity::Turn(ActivityTurn { reply: 0, attempt: u64::MAX, turn: u32::MAX, read: None }),
    };
    let decoded = ArchivedTurn {
        task: u64::MAX,
        name,
        cumulative: 0,
        read: None,
        at: u64::MAX,
        turn: TurnBytes::Carried(TurnCarried { bytes: turn }),
    };
    DecodedActivityLine { source, decoded: Some(decoded) }
}

pub fn activity_rows(count: u32, width: u32) -> List<DecodedActivityLine> {
    let row = activity_line(bytes(width));
    let mut rows = List::with_capacity(count);
    for _ in 0..count {
        rows.push(row.clone()).expect("reserved activity rows");
    }
    rows
}

pub fn activity_page(rows: Box<[u8]>) -> ActivityPage {
    ActivityPage { cut: true, rows, before: None, after: None }
}

pub fn encoded_activity_page(rows: &List<DecodedActivityLine>) -> Box<[u8]> {
    let rows = piece_items(rows);
    encoded(&activity_page(rows))
}

fn next_activity_row(
    limits: &Limits,
    reader: &mut skein_lib::Reader<'_>,
) -> Result<Option<DecodedActivityLine>, Problem> {
    let bytes = match skein_codec::piece::split(reader) {
        Ok(bytes) => bytes,
        Err(reason) => return Err(Problem { path: paths::ACTIVITY_PAGE_ROWS, reason }),
    };
    let Some(bytes) = bytes else { return Ok(None) };
    let row = DecodedActivityLine::decode(limits, &mut skein_lib::Reader::new(&bytes))?;
    Ok(Some(row))
}

pub fn decode_activity_page(
    limits: &Limits,
    bytes: &[u8],
    row_limit: u32,
) -> Result<List<DecodedActivityLine>, Problem> {
    let page = ActivityPage::decode(limits, &mut skein_lib::Reader::new(bytes))?;
    let mut reader = skein_lib::Reader::new(page.rows.as_ref());
    let mut rows = List::with_capacity(row_limit);
    while let Some(row) = next_activity_row(limits, &mut reader)? {
        if rows.push(row).is_err() {
            return Err(Problem { path: paths::ACTIVITY_PAGE_ROWS, reason: skein_codec::Reason::Bound });
        }
    }
    Ok(rows)
}

pub fn malformed_activity_pieces() -> [Box<[u8]>; 3] {
    let rows = activity_rows(1, 8);
    let valid = piece_items(&rows);
    let mut out = Writer::new(valid.len().checked_add(1).expect("small trailing fragment"));
    out.put(&valid).expect("room for valid row");
    out.put(&[255]).expect("room for trailing fragment");
    [Box::from([0, 0, 0]), Box::from([0, 0, 0, 5, 1, 2]), out.finish()]
}

// Shared task and run fixtures retain supplied values independently of wire order.
pub fn document_limits() -> Limits {
    let mut limits = CEILINGS;
    limits.document_text_bytes = 4;
    limits.me_projects = 512;
    check_limits(&limits).expect("document fixtures below ceilings");
    limits
}

pub fn repeated<T: Clone + core::fmt::Debug>(value: T, count: u32) -> List<T> {
    let mut values = List::with_capacity(count);
    for _ in 0..count {
        values.push(value.clone()).expect("reserved fixture list");
    }
    values
}

pub fn document_text(limits: &Limits) -> DocumentText {
    DocumentText { bytes: ceiling_text(limits.document_text_bytes), cut: true }
}

pub fn numbers() -> Numbers {
    Numbers { budget: u64::MAX, spent: 11, spent_below: 17, reserved: 23 }
}

pub fn funder_period() -> FunderPeriod {
    FunderPeriod { project: u32::MAX, period: u64::MAX }
}

pub fn budget() -> Budget {
    Budget { numbers: numbers(), left: u64::MAX, funder: Funder::Period(funder_period()), deadline: Some(u64::MAX) }
}

pub fn pool_ledger() -> PoolLedger {
    PoolLedger {
        source: Funder::Period(funder_period()),
        parent: Some(Funder::Task(funder_task())),
        numbers: numbers(),
        left: u64::MAX,
        closed: true,
        began: Some(u64::MAX),
        next: None,
    }
}

pub fn funder_task() -> FunderTask {
    FunderTask { value: u64::MAX }
}

pub fn held_role_policy() -> HeldRolePolicy {
    HeldRolePolicy { role: u32::MAX }
}

pub fn project_role(limits: &Limits) -> MeProject {
    MeProject {
        left_out: 0,
        defaults: List::with_capacity(0),
        public: None,
        models: List::with_capacity(0),
        templates: Box::from([]),
        creating: None,
        project: u32::MAX,
        name: document_text(limits),
        role: HeldRole::Policy(held_role_policy()),
        pool: Some(pool_ledger()),
    }
}

pub fn frame_party(limits: &Limits) -> MeParty {
    MeParty { number: u64::MAX, kind: PartyKind::Service, login: document_text(limits), name: document_text(limits) }
}

pub fn needs() -> Needs {
    Needs { questions: u32::MAX, proposals: u32::MAX, escalations: u32::MAX, person_tasks: u32::MAX }
}

pub fn count() -> Count {
    Count { value: u32::MAX, limited: true }
}

pub fn run_state() -> RunState {
    RunState { task: u64::MAX, attempt: Some(u64::MAX), committed_turn: u32::MAX, writing: None, call: None }
}

pub fn revision() -> Revision {
    Revision { value: u64::MAX }
}

pub fn wake_rule_batch() -> WakeRuleBatch {
    WakeRuleBatch { count: u32::MAX, age: skein_lib::Duration::from_nanos(u64::MAX) }
}

pub fn wake_policy() -> WakePolicy {
    WakePolicy {
        words: WakeRule::Batch(wake_rule_batch()),
        notices: WakeRule::Immediate,
        news: WakeRule::Paced(WakeRulePaced { span: skein_lib::Duration::from_nanos(u64::MAX) }),
        results: ResultsWake::LastOrFailure,
        questions: true,
        answers: false,
        timers: true,
    }
}

pub fn chip_executor_agent() -> ChipExecutorAgent {
    ChipExecutorAgent { charter: u32::MAX }
}

pub fn chip_phase_owner_fields() -> ChipPhaseOwnerFields {
    ChipPhaseOwnerFields { phase: ChipPhase::Ended, details: 2 }
}

pub fn executor_agent() -> ExecutorAgent {
    ExecutorAgent { charter: u32::MAX }
}

pub fn chip_details_fields() -> ChipDetailsFields {
    ChipDetailsFields {
        dependencies: List::with_capacity(0),
        project: u32::MAX,
        executor: Executor::Agent(executor_agent()),
        phase_since: u64::MAX,
        run_since: None,
        attempt: Some(u64::MAX),
        retry: None,
        wake: wake_policy(),
        person_wait: None,
        hold: None,
        budget: budget(),
        activity: u64::MAX,
        resources: Box::from([]),
        resources_cut: true,
        resources_observed: false,
        procedure: None,
        live_delegates: u32::MAX,
        closing_connectors: Some(List::with_capacity(0)),
        ending: None,
        ended_at: None,
        observation: None,
    }
}

pub fn frame_at_ceilings(limits: &Limits) -> Me {
    Me {
        providers: Box::new([]),
        providers_cut: false,
        key_bytes: 256,
        redirect_bytes: 4096,
        words_bytes: 8192,
        sign_in_expires: u64::MAX,
        party: frame_party(limits),
        projects: piece_items(&repeated(project_role(limits), 2)),
        needs: needs(),
        needs_total: u32::MAX,
        updates: count(),
        results_seen: u64::MAX,
        cut: true,
        retention: skein_lib::Duration::from_nanos(u64::MAX),
        deployment_role: Some(HeldRole::Owner),
    }
}

pub fn chip(rows: &List<Fact>) -> Chip {
    let rows = piece_items(rows);
    Chip { rows }
}

pub fn decode_chip_facts(limits: &Limits, bytes: &[u8]) -> List<Fact> {
    let chip = Chip::decode(limits, &mut skein_lib::Reader::new(bytes)).expect("whole generated chip");
    let mut reader = skein_lib::Reader::new(chip.rows.as_ref());
    let mut rows = List::with_capacity(3);
    while let Some(bytes) = skein_codec::piece::split(&mut reader).expect("exact chip pieces") {
        let fact = Fact::decode(limits, &mut skein_lib::Reader::new(&bytes)).expect("whole generated fact");
        rows.push(fact).expect("bounded fact rows");
    }
    rows
}

pub fn not_served() -> NotServed {
    let mut served = List::with_capacity(1);
    served.push(Served { owner: Owner::Jig, first: 28, last: 28 }).expect("range slot");
    NotServed { owner: Owner::Jig, served }
}
mod operations;
pub use operations::merged_targets;

mod compound;
pub use compound::{COMPOUND_PROJECT, compound_operations};

// The previous 111-byte minimum plus an empty name: four length bytes and its cut flag.

pub fn chip_phases() -> [ChipPhaseOwnerFields; 1] {
    [chip_phase_owner_fields()]
}
pub fn chip_facts(limits: &Limits, phase: ChipPhaseOwnerFields) -> List<Fact> {
    let mut rows = List::with_capacity(3);
    rows.push(Fact::Chip(ChipRow {
        parent: None,
        value: ChipFields {
            task: u64::MAX,
            title: document_text(limits),
            executor: ChipExecutor::Agent(chip_executor_agent()),
            phase: Phase::Ended,
            details: 1,
        },
    }))
    .expect("root row");
    rows.push(Fact::ChipPhase(ChipPhaseRow { parent: Some(0), value: phase })).expect("phase");
    rows.push(Fact::ChipDetails(Box::new(ChipDetailsRow { parent: Some(1), value: chip_details_fields() })))
        .expect("details");
    rows
}
