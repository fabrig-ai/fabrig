//! Typed fixture values for frozen event goldens.

use skein_lib::List;
use smith_events::{
    AccountState, Agent, AgentExit, AnswerClass, AnswerFailure, Backend, Block, Budget, BudgetStage, BudgetStanding,
    Capture, CheckCompleted, CheckExit, CheckStarted, ChildCompleted, ChildStop, Clamp, CommandExit, Completion,
    CompletionFailure, ConversationClosed, ConversationEnd, ConversationKind, ConversationOpened, Count, Cpu,
    Delivered, Delivery, Effect, EffectiveLimit, Event, Evidence, FailureClass, FailureLimit, FailurePhase, Family,
    Field, Forced, Form, JobEnded, Level, LimitValue, Loss, Message, MessageRead, MessageReceived, MessageRefusal,
    MessageUnread, Mode, Model, Notice, NoticeKind, Outcome, PeakRss, Prompt, Quantity, Record, Requested, Reserve,
    ResponseCompleted, ResponseStarted, Role, RunCompleted, RunResult, RunStarted, SessionEnded, SessionStarted, Share,
    ShareBound, SharePart, Source, Status, Stop, TextDelta, ToolChoice, ToolCompleted, ToolStarted, Tools, Usage,
    Verdict, Versions,
};

#[derive(Clone, Debug)]
struct Fill {
    limits: Option<smith_events::Limits>,
    containment: smith_events::Containment,
    capture: Capture,
    delivery: Delivery,
    mode: Mode,
    family: Family,
    form: Form,
    status: Status,
    conversation_kind: ConversationKind,
    conversation_end: ConversationEnd,
    outcome: Outcome,
    stop: Stop,
    source: Source,
    effect: Effect,
    verdict: Verdict,
    delivered: Delivered,
    failure_class: FailureClass,
    evidence: Evidence,
    answer_class: AnswerClass,
    level: Level,
    notice_kind: NoticeKind,
    role: Role,
}

impl Fill {
    fn maximum(&self) -> bool {
        self.limits.is_some()
    }

    fn count(&self, sample: u32) -> u32 {
        self.limits.map_or(sample, |limits| limits.items)
    }

    fn text(&self) -> Box<[u8]> {
        self.metadata("sample\n\"🌍".as_bytes())
    }

    fn content(&self) -> Box<[u8]> {
        self.captured("sample\n\"🌍".as_bytes())
    }

    fn metadata(&self, sample: &[u8]) -> Box<[u8]> {
        self.limits.map_or_else(|| Box::from(sample), |limits| payload(limits.string))
    }

    fn captured(&self, sample: &[u8]) -> Box<[u8]> {
        self.limits.map_or_else(|| Box::from(sample), |limits| payload(limits.content))
    }

    fn largest(limits: &smith_events::Limits) -> Fill {
        Fill {
            limits: Some(*limits),
            containment: smith_events::Containment::Unknown(payload(limits.string)),
            capture: Capture::Unknown(payload(limits.string)),
            delivery: Delivery::Unknown(payload(limits.string)),
            mode: Mode::Unknown(payload(limits.string)),
            family: Family::Unknown(payload(limits.string)),
            form: Form::Unknown(payload(limits.string)),
            status: Status::Unknown(payload(limits.string)),
            conversation_kind: ConversationKind::Unknown(payload(limits.string)),
            conversation_end: ConversationEnd::Unknown(payload(limits.string)),
            outcome: Outcome::Unknown(payload(limits.string)),
            stop: Stop::Unknown(payload(limits.string)),
            source: Source::Unknown(payload(limits.string)),
            effect: Effect::Unknown(payload(limits.string)),
            verdict: Verdict::Unknown(payload(limits.string)),
            delivered: Delivered::Unknown(payload(limits.string)),
            failure_class: FailureClass::Unknown(payload(limits.string)),
            evidence: Evidence::Unknown(payload(limits.string)),
            answer_class: AnswerClass::Unknown(payload(limits.string)),
            level: Level::Unknown(payload(limits.string)),
            notice_kind: NoticeKind::Unknown(payload(limits.string)),
            role: Role::Unknown(payload(limits.string)),
        }
    }

    const DEFAULT: Fill = Fill {
        limits: None,
        containment: smith_events::Containment::Groups,
        capture: Capture::None,
        delivery: Delivery::Complete,
        mode: Mode::Agent,
        family: Family::Inspect,
        form: Form::Report,
        status: Status::Accepted,
        conversation_kind: ConversationKind::Main,
        conversation_end: ConversationEnd::Closed,
        outcome: Outcome::Completed,
        stop: Stop::End,
        source: Source::Workspace,
        effect: Effect::Read,
        verdict: Verdict::Read,
        delivered: Delivered::Delivered,
        failure_class: FailureClass::Overloaded,
        evidence: Evidence::Unsent,
        answer_class: AnswerClass::Model,
        level: Level::Info,
        notice_kind: NoticeKind::CredentialUnauthorized,
        role: Role::User,
    };

    fn containment(value: smith_events::Containment) -> Fill {
        Fill { containment: value, ..Self::DEFAULT }
    }

    fn capture(value: Capture) -> Fill {
        Fill { capture: value, ..Self::DEFAULT }
    }

    fn delivery(value: Delivery) -> Fill {
        Fill { delivery: value, ..Self::DEFAULT }
    }

    fn mode(value: Mode) -> Fill {
        Fill { mode: value, ..Self::DEFAULT }
    }

    fn family(value: Family) -> Fill {
        Fill { family: value, ..Self::DEFAULT }
    }

    fn form(value: Form) -> Fill {
        Fill { form: value, ..Self::DEFAULT }
    }

    fn status(value: Status) -> Fill {
        Fill { status: value, ..Self::DEFAULT }
    }

    fn conversation_kind(value: ConversationKind) -> Fill {
        Fill { conversation_kind: value, ..Self::DEFAULT }
    }

    fn conversation_end(value: ConversationEnd) -> Fill {
        Fill { conversation_end: value, ..Self::DEFAULT }
    }

    fn outcome(value: Outcome) -> Fill {
        Fill { outcome: value, ..Self::DEFAULT }
    }

    fn stop(value: Stop) -> Fill {
        Fill { stop: value, ..Self::DEFAULT }
    }

    fn source(value: Source) -> Fill {
        Fill { source: value, ..Self::DEFAULT }
    }

    fn effect(value: Effect) -> Fill {
        Fill { effect: value, ..Self::DEFAULT }
    }

    fn verdict(value: Verdict) -> Fill {
        Fill { verdict: value, ..Self::DEFAULT }
    }

    fn delivered(value: Delivered) -> Fill {
        Fill { delivered: value, ..Self::DEFAULT }
    }

    fn failure_class(value: FailureClass) -> Fill {
        Fill { failure_class: value, ..Self::DEFAULT }
    }

    fn evidence(value: Evidence) -> Fill {
        Fill { evidence: value, ..Self::DEFAULT }
    }

    fn answer_class(value: AnswerClass) -> Fill {
        Fill { answer_class: value, ..Self::DEFAULT }
    }

    fn level(value: Level) -> Fill {
        Fill { level: value, ..Self::DEFAULT }
    }

    fn notice_kind(value: NoticeKind) -> Fill {
        Fill { notice_kind: value, ..Self::DEFAULT }
    }

    fn role(value: Role) -> Fill {
        Fill { role: value, ..Self::DEFAULT }
    }
}

fn value_agent(fill: &Fill) -> Agent {
    Agent { name: fill.text(), version: fill.text() }
}

fn value_versions(fill: &Fill) -> Versions {
    Versions {
        events: u64::MAX,
        channel: u64::MAX,
        charter: u64::MAX,
        transcript: u64::MAX,
        chat_store: if fill.maximum() { Some(u64::MAX) } else { None },
    }
}

fn value_loss() -> Loss {
    Loss { events: u64::MAX, channel: Some(u64::MAX) }
}

fn value_cpu() -> Cpu {
    Cpu { user: u64::MAX, system: u64::MAX, children_user: u64::MAX, children_system: u64::MAX }
}

fn value_peak_rss() -> PeakRss {
    PeakRss { self_bytes: u64::MAX, children: u64::MAX }
}

fn value_model(fill: &Fill) -> Model {
    Model {
        account: if fill.maximum() { Some(u32::MAX) } else { None },
        backend: if fill.maximum() { Backend::Unknown(fill.text()) } else { Backend::Direct },
        endpoint: fill.text(),
        model: fill.text(),
        window_tokens: u64::MAX,
        output_tokens: u64::MAX,
        effort: Some(fill.text()),
    }
}

fn value_usage() -> Usage {
    Usage {
        input_tokens: Some(u64::MAX),
        cache_read_tokens: Some(u64::MAX),
        cache_write_tokens: Some(u64::MAX),
        output_tokens: Some(u64::MAX),
        reasoning_tokens: Some(u64::MAX),
    }
}

fn value_budget() -> Budget {
    Budget {
        turns: u64::MAX,
        spend: u64::MAX,
        time_ms: u64::MAX,
        reserve: Reserve { turns: u64::MAX, time_ms: u64::MAX, spend: u64::MAX },
    }
}

fn value_tools(fill: &Fill) -> Tools {
    Tools {
        withheld: if fill.maximum() { list_family(fill) } else { skein_lib::List::with_capacity(0) },
        families: list_family(fill),
        wait: !fill.maximum(),
        deliver: !fill.maximum(),
        host: list_text(fill),
    }
}

fn value_completion_failure(fill: &Fill) -> CompletionFailure {
    CompletionFailure {
        field: if fill.maximum() { Some(fill.text()) } else { None },
        code: if fill.maximum() { Some(fill.text()) } else { None },
        request_id: if fill.maximum() { Some(fill.text()) } else { None },
        phase: if fill.maximum() { Some(FailurePhase::Unknown(fill.text())) } else { None },
        limit: if fill.maximum() { Some(FailureLimit { which: fill.text(), bound: u64::MAX }) } else { None },
        status: if fill.maximum() { Some(u64::MAX) } else { None },
        detail: if fill.maximum() { Some(fill.content()) } else { None },
        class: fill.failure_class.clone(),
        evidence: fill.evidence.clone(),
        retry_after_ms: Some(u64::MAX),
    }
}

fn value_answer_failure(fill: &Fill) -> AnswerFailure {
    AnswerFailure {
        state: if fill.maximum() { Some(AccountState::Unknown(fill.text())) } else { None },
        code: if fill.maximum() { Some(fill.text()) } else { None },
        program: if fill.maximum() { Some(fill.text()) } else { None },
        exit: if fill.maximum() { Some(AgentExit::Signal(u64::MAX)) } else { None },
        detail: if fill.maximum() { Some(fill.content()) } else { None },
        reset_ms: if fill.maximum() { Some(u64::MAX) } else { None },
        used: if fill.maximum() { Some(u64::MAX) } else { None },
        bound: if fill.maximum() { Some(u64::MAX) } else { None },
        dropped: if fill.maximum() || fill.answer_class == AnswerClass::Cancelled { Some(false) } else { None },
        ended: if fill.maximum() { Some(AgentExit::Code(i64::MIN)) } else { None },
        turn: if fill.maximum() { Some(u64::MAX) } else { None },
        endpoint: if fill.maximum() { Some(fill.text()) } else { None },
        model: if fill.maximum() { Some(fill.text()) } else { None },
        attempts: if fill.maximum() { Some(u64::MAX) } else { None },
        class: fill.answer_class.clone(),
        completion: Some(value_completion_failure(fill)),
        account: Some(u32::MAX),
        which: Some(fill.text()),
        reason: Some(fill.text()),
    }
}

fn value_field(fill: &Fill) -> Field {
    Field { name: fill.text(), text: fill.content() }
}

fn value_run_result(fill: &Fill) -> RunResult {
    RunResult {
        form: fill.form.clone(),
        label: Some(fill.text()),
        text: Some(fill.content()),
        fields: Some(list_field(fill)),
    }
}

fn value_message(fill: &Fill) -> Message {
    Message { role: fill.role.clone(), blocks: list_block(fill) }
}

fn value_prompt(fill: &Fill) -> Prompt {
    Prompt { system: Some(fill.content()), tools: Some(list_text(fill)), messages: list_message(fill) }
}

fn value_completion(fill: &Fill) -> Completion {
    Completion { blocks: list_block(fill) }
}

fn value_command_exit(fill: &Fill) -> CommandExit {
    if fill.maximum() { CommandExit::Signal { signal: u64::MAX } } else { CommandExit::Code { code: i64::MIN } }
}

fn value_session_started(fill: &Fill) -> SessionStarted {
    SessionStarted {
        chat: if fill.maximum() {
            Some(fill.text())
        } else if fill.mode == Mode::Chat {
            Some(b"main".as_slice().into())
        } else {
            None
        },
        limits: {
            let mut values = List::with_capacity(fill.count(0));
            for index in 0..fill.count(0) {
                let mut name = fill.text();
                *name.last_mut().expect("max metadata has a name") = u8::try_from(index).expect("small fixture");
                values
                    .push(EffectiveLimit {
                        name,
                        value: LimitValue::Fraction { numerator: u64::MAX, denominator: u64::MAX },
                    })
                    .expect("configured count");
            }
            values
        },
        wall_ms: u64::MAX,
        agent: value_agent(fill),
        mode: fill.mode.clone(),
        containment: fill.containment.clone(),
        pid: u64::MAX,
        profile: fill.text(),
        capture: fill.capture.clone(),
        delivery: fill.delivery.clone(),
        versions: value_versions(fill),
    }
}

fn value_session_ended(fill: &Fill) -> SessionEnded {
    SessionEnded {
        exit: i64::MIN,
        answered: !fill.maximum(),
        aborted: false,
        teardown_ms: Some(u64::MAX),
        emitted: list_count(fill),
        loss: value_loss(),
        cpu_ms: Some(value_cpu()),
        peak_rss_bytes: Some(value_peak_rss()),
    }
}

fn value_run_started(fill: &Fill) -> RunStarted {
    RunStarted {
        run: u64::MAX,
        resumed: !fill.maximum(),
        messages: u64::MAX,
        main: value_model(fill),
        budget: value_budget(),
        tools: value_tools(fill),
        contract: list_form(fill),
    }
}

fn value_run_completed(fill: &Fill) -> RunCompleted {
    RunCompleted {
        run: u64::MAX,
        wind_down: false,
        status: fill.status.clone(),
        failure: Some(value_answer_failure(fill)),
        result: Some(value_run_result(fill)),
        turns: u64::MAX,
        spent: u64::MAX,
        usage: value_usage(),
        read: Some(u64::MAX),
        unread: u64::MAX,
        duration_ms: u64::MAX,
    }
}

fn value_conversation_opened(fill: &Fill) -> ConversationOpened {
    ConversationOpened {
        run: u64::MAX,
        conversation: u64::MAX,
        kind: fill.conversation_kind.clone(),
        parent: Some(u64::MAX),
        call: Some(fill.text()),
        thread: u64::MAX,
        model: value_model(fill),
        requested: if fill.maximum() {
            Some(Requested { turns: Some(u64::MAX), seconds: Some(u64::MAX) })
        } else {
            None
        },
        share: if fill.maximum() { Some(Share { turns: u64::MAX, time_ms: u64::MAX, spend: u64::MAX }) } else { None },
        clamped: if fill.maximum() {
            {
                let mut values = List::with_capacity(fill.count(0));
                for _ in 0..fill.count(0) {
                    values
                        .push(Clamp { part: SharePart::Unknown(fill.text()), by: ShareBound::Unknown(fill.text()) })
                        .expect("configured clamp item room");
                }
                values
            }
        } else {
            List::with_capacity(0)
        },
    }
}

fn value_conversation_closed(fill: &Fill) -> ConversationClosed {
    ConversationClosed {
        run: u64::MAX,
        conversation: u64::MAX,
        end: fill.conversation_end.clone(),
        which: Some(fill.text()),
        failure: Some(value_completion_failure(fill)),
        turns: u64::MAX,
        usage: value_usage(),
        spent: u64::MAX,
        duration_ms: u64::MAX,
    }
}

fn value_response_started(fill: &Fill) -> ResponseStarted {
    ResponseStarted {
        run: u64::MAX,
        conversation: u64::MAX,
        response: u64::MAX,
        turn: u64::MAX,
        attempt: u64::MAX,
        window: u64::MAX,
        model: value_model(fill),
        messages: u64::MAX,
        output_tokens: u64::MAX,
        tool_choice: if fill.maximum() { ToolChoice::Unknown(fill.text()) } else { ToolChoice::Auto },
        prompt: Some(value_prompt(fill)),
    }
}

fn value_response_completed(fill: &Fill) -> ResponseCompleted {
    ResponseCompleted {
        run: u64::MAX,
        conversation: u64::MAX,
        response: u64::MAX,
        outcome: fill.outcome.clone(),
        stop: Some(fill.stop.clone()),
        blocks: u64::MAX,
        calls: u64::MAX,
        invalid: u64::MAX,
        oversized: u64::MAX,
        cut: u64::MAX,
        usage: value_usage(),
        spent: u64::MAX,
        request_bytes: Some(u64::MAX),
        first_byte_ms: Some(u64::MAX),
        largest_gap_ms: Some(u64::MAX),
        duration_ms: u64::MAX,
        failure: Some(value_completion_failure(fill)),
        retry_ms: Some(u64::MAX),
        completion: Some(value_completion(fill)),
    }
}

fn value_text_delta(fill: &Fill) -> TextDelta {
    TextDelta { run: u64::MAX, conversation: u64::MAX, response: u64::MAX, block: u64::MAX, text: fill.content() }
}

fn value_tool_started(fill: &Fill) -> ToolStarted {
    ToolStarted {
        run: u64::MAX,
        conversation: u64::MAX,
        call: fill.text(),
        tool: fill.text(),
        source: fill.source.clone(),
        effect: fill.effect.clone(),
        deadline_ms: Some(u64::MAX),
        input_bytes: u64::MAX,
        input: Some(fill.content()),
    }
}

fn value_tool_completed(fill: &Fill) -> ToolCompleted {
    ToolCompleted {
        run: u64::MAX,
        conversation: u64::MAX,
        call: fill.text(),
        tool: fill.text(),
        verdict: fill.verdict.clone(),
        exit: Some(value_command_exit(fill)),
        job: if fill.maximum() { Some(u64::MAX) } else { None },
        bytes: u64::MAX,
        omitted_bytes: u64::MAX,
        clamped: true,
        duration_ms: u64::MAX,
        child: if fill.maximum() {
            Some(ChildCompleted {
                conversation: u64::MAX,
                stop: ChildStop::Unknown(fill.text()),
                turns: u64::MAX,
                time_ms: u64::MAX,
                spent: u64::MAX,
                usage: value_usage(),
            })
        } else {
            None
        },
        delivery: Some(fill.delivered.clone()),
    }
}

fn value_check_started() -> CheckStarted {
    CheckStarted { run: u64::MAX, deadline_ms: Some(u64::MAX) }
}

fn value_check_completed(fill: &Fill) -> CheckCompleted {
    CheckCompleted {
        run: u64::MAX,
        exit: if fill.maximum() { CheckExit::Unknown(fill.text()) } else { CheckExit::Code { code: i64::MIN } },
        passed: false,
        duration_ms: u64::MAX,
    }
}

fn value_notice(fill: &Fill) -> Notice {
    Notice {
        code: if fill.maximum() { Some(fill.text()) } else { None },
        level: fill.level.clone(),
        kind: fill.notice_kind.clone(),
        run: Some(u64::MAX),
        account: Some(u32::MAX),
        bytes: if fill.maximum() { Some(u64::MAX) } else { None },
        reset_ms: Some(u64::MAX),
        exit: if fill.maximum() { Some(AgentExit::Signal(u64::MAX)) } else { None },
        forced: if fill.maximum() { Some(Forced::Unknown(fill.text())) } else { None },
    }
}

fn list_family(fill: &Fill) -> List<Family> {
    let mut values = List::with_capacity(fill.count(1));
    for _ in 0..fill.count(1) {
        values.push(fill.family.clone()).expect("fixture room");
    }
    values
}

fn list_field(fill: &Fill) -> List<Field> {
    let mut values = List::with_capacity(fill.count(1));
    for _ in 0..fill.count(1) {
        values.push(value_field(fill)).expect("fixture room");
    }
    values
}

fn list_form(fill: &Fill) -> List<Form> {
    let mut values = List::with_capacity(fill.count(1));
    for _ in 0..fill.count(1) {
        values.push(fill.form.clone()).expect("fixture room");
    }
    values
}

fn list_message(fill: &Fill) -> List<Message> {
    let mut values = List::with_capacity(fill.count(1));
    for _ in 0..fill.count(1) {
        values.push(value_message(fill)).expect("fixture room");
    }
    values
}

fn list_text(fill: &Fill) -> List<Box<[u8]>> {
    let mut values = List::with_capacity(fill.count(1));
    for _ in 0..fill.count(1) {
        values.push(fill.text()).expect("fixture room");
    }
    values
}

fn value_block(fill: &Fill) -> Block {
    Block::Call { call: fill.text(), tool: fill.text(), input: fill.content() }
}

fn list_block(fill: &Fill) -> List<Block> {
    if fill.maximum() {
        let mut values = List::with_capacity(fill.count(7));
        for _ in 0..fill.count(7) {
            values.push(value_block(fill)).expect("block room");
        }
        values
    } else {
        let mut values = List::with_capacity(7);
        values.push(Block::Text { text: fill.content() }).expect("block room");
        values.push(Block::Refusal { text: fill.content() }).expect("block room");
        values.push(value_block(fill)).expect("block room");
        values.push(Block::Oversized { call: fill.text(), tool: fill.text(), bytes: u64::MAX }).expect("block room");
        values.push(Block::Cut { call: fill.text(), tool: fill.text(), input: fill.content() }).expect("block room");
        values.push(Block::Result { call: fill.text(), error: true, text: fill.content() }).expect("block room");
        values.push(Block::Opaque { bytes: u64::MAX }).expect("block room");
        values
    }
}

fn list_count(fill: &Fill) -> List<Count> {
    let mut values = List::with_capacity(fill.count(1));
    for index in 0..fill.count(1) {
        let record = if fill.maximum() {
            let mut name = fill.text();
            *name.first_mut().expect("nonempty metadata limit") =
                u8::try_from(index).expect("small fixture").saturating_add(32);
            name
        } else {
            Box::from(b"notice".as_slice())
        };
        values.push(Count { record, count: u64::MAX }).expect("count room");
    }
    values
}

fn session_started_samples() -> Vec<(&'static str, Record, Capture)> {
    let mut values = Vec::new();
    for (name, words, capture) in [
        ("session_started_none", Fill::capture(Capture::None), Capture::None),
        ("session_started_calls", Fill::capture(Capture::Calls), Capture::Calls),
        ("session_started_everything", Fill::capture(Capture::Everything), Capture::Everything),
        ("capture_none", Fill::capture(Capture::None), Capture::Everything),
        ("capture_calls", Fill::capture(Capture::Calls), Capture::Everything),
        ("capture_everything", Fill::capture(Capture::Everything), Capture::Everything),
        ("delivery_complete", Fill::delivery(Delivery::Complete), Capture::Everything),
        ("delivery_best_effort", Fill::delivery(Delivery::BestEffort), Capture::Everything),
        ("mode_agent", Fill::mode(Mode::Agent), Capture::Everything),
        ("mode_exec", Fill::mode(Mode::Exec), Capture::Everything),
        ("mode_chat", Fill::mode(Mode::Chat), Capture::Everything),
    ] {
        values.push((name, recorded(Event::SessionStarted(value_session_started(&words))), capture));
    }
    values
}

fn session_ended_samples() -> Vec<(&'static str, Record, Capture)> {
    let mut values = Vec::new();
    for (name, capture) in [
        ("session_ended_none", Capture::None),
        ("session_ended_calls", Capture::Calls),
        ("session_ended_everything", Capture::Everything),
    ] {
        values.push((name, recorded(Event::SessionEnded(value_session_ended(&Fill::DEFAULT))), capture));
    }
    values
}

fn run_started_samples() -> Vec<(&'static str, Record, Capture)> {
    let mut values = Vec::new();
    for (name, words, capture) in [
        ("run_started_none", Fill::capture(Capture::None), Capture::None),
        ("run_started_calls", Fill::capture(Capture::Calls), Capture::Calls),
        ("run_started_everything", Fill::capture(Capture::Everything), Capture::Everything),
        ("family_modify", Fill::family(Family::Modify), Capture::Everything),
        ("family_shell", Fill::family(Family::Shell), Capture::Everything),
        ("family_sub_agents", Fill::family(Family::SubAgents), Capture::Everything),
        ("form_report", Fill::form(Form::Report), Capture::Everything),
        ("form_verdict", Fill::form(Form::Verdict), Capture::Everything),
        ("form_change", Fill::form(Form::Change), Capture::Everything),
        ("form_failure", Fill::form(Form::Failure), Capture::Everything),
    ] {
        values.push((name, recorded(Event::RunStarted(value_run_started(&words))), capture));
    }
    values
}

fn run_completed_samples() -> Vec<(&'static str, Record, Capture)> {
    let mut values = Vec::new();
    for (name, words, capture) in [
        ("run_completed_none", Fill::capture(Capture::None), Capture::None),
        ("run_completed_calls", Fill::capture(Capture::Calls), Capture::Calls),
        ("run_completed_everything", Fill::capture(Capture::Everything), Capture::Everything),
        ("status_accepted", Fill::status(Status::Accepted), Capture::Everything),
        ("status_parked", Fill::status(Status::Parked), Capture::Everything),
        ("status_failed", Fill::status(Status::Failed), Capture::Everything),
        ("status_refused", Fill::status(Status::Refused), Capture::Everything),
        ("failure_class_rate_limited", Fill::failure_class(FailureClass::RateLimited), Capture::Everything),
        ("failure_class_exhausted", Fill::failure_class(FailureClass::Exhausted), Capture::Everything),
        ("failure_class_unavailable", Fill::failure_class(FailureClass::Unavailable), Capture::Everything),
        ("failure_class_timed_out", Fill::failure_class(FailureClass::TimedOut), Capture::Everything),
        ("failure_class_context_too_long", Fill::failure_class(FailureClass::ContextTooLong), Capture::Everything),
        ("failure_class_invalid", Fill::failure_class(FailureClass::Invalid), Capture::Everything),
        ("failure_class_unauthorized", Fill::failure_class(FailureClass::Unauthorized), Capture::Everything),
        ("failure_class_limit", Fill::failure_class(FailureClass::Limit), Capture::Everything),
        ("failure_class_protocol", Fill::failure_class(FailureClass::Protocol), Capture::Everything),
        ("failure_class_cancelled", Fill::failure_class(FailureClass::Cancelled), Capture::Everything),
        ("evidence_maybe_sent", Fill::evidence(Evidence::MaybeSent), Capture::Everything),
        ("evidence_response", Fill::evidence(Evidence::Response), Capture::Everything),
        ("answer_class_budget", Fill::answer_class(AnswerClass::Budget), Capture::Everything),
        ("answer_class_policy", Fill::answer_class(AnswerClass::Policy), Capture::Everything),
        ("answer_class_cancelled", Fill::answer_class(AnswerClass::Cancelled), Capture::Everything),
        ("answer_class_stale", Fill::answer_class(AnswerClass::Stale), Capture::Everything),
        ("answer_class_transcript", Fill::answer_class(AnswerClass::Transcript), Capture::Everything),
        ("answer_class_busy", Fill::answer_class(AnswerClass::Busy), Capture::Everything),
        ("answer_class_invalid", Fill::answer_class(AnswerClass::Invalid), Capture::Everything),
    ] {
        values.push((name, recorded(Event::RunCompleted(value_run_completed(&words))), capture));
    }
    values
}

fn conversation_opened_samples() -> Vec<(&'static str, Record, Capture)> {
    let mut values = Vec::new();
    for (name, words, capture) in [
        ("conversation_opened_none", Fill::capture(Capture::None), Capture::None),
        ("conversation_opened_calls", Fill::capture(Capture::Calls), Capture::Calls),
        ("conversation_opened_everything", Fill::capture(Capture::Everything), Capture::Everything),
        ("conversation_kind_main", Fill::conversation_kind(ConversationKind::Main), Capture::Everything),
        ("conversation_kind_child", Fill::conversation_kind(ConversationKind::Child), Capture::Everything),
        ("conversation_kind_compaction", Fill::conversation_kind(ConversationKind::Compaction), Capture::Everything),
    ] {
        values.push((name, recorded(Event::ConversationOpened(value_conversation_opened(&words))), capture));
    }
    values
}

fn conversation_closed_samples() -> Vec<(&'static str, Record, Capture)> {
    let mut values = Vec::new();
    for (name, words, capture) in [
        ("conversation_closed_none", Fill::capture(Capture::None), Capture::None),
        ("conversation_closed_calls", Fill::capture(Capture::Calls), Capture::Calls),
        ("conversation_closed_everything", Fill::capture(Capture::Everything), Capture::Everything),
        ("conversation_end_closed", Fill::conversation_end(ConversationEnd::Closed), Capture::Everything),
        ("conversation_end_budget", Fill::conversation_end(ConversationEnd::Budget), Capture::Everything),
        ("conversation_end_failed", Fill::conversation_end(ConversationEnd::Failed), Capture::Everything),
        ("conversation_end_refused", Fill::conversation_end(ConversationEnd::Refused), Capture::Everything),
        ("conversation_end_transcript", Fill::conversation_end(ConversationEnd::Transcript), Capture::Everything),
        ("conversation_end_overflow", Fill::conversation_end(ConversationEnd::Overflow), Capture::Everything),
    ] {
        values.push((name, recorded(Event::ConversationClosed(value_conversation_closed(&words))), capture));
    }
    values
}

fn response_started_samples() -> Vec<(&'static str, Record, Capture)> {
    let mut values = Vec::new();
    for (name, words, capture) in [
        ("response_started_none", Fill::capture(Capture::None), Capture::None),
        ("response_started_calls", Fill::capture(Capture::Calls), Capture::Calls),
        ("response_started_everything", Fill::capture(Capture::Everything), Capture::Everything),
        ("role_assistant", Fill::role(Role::Assistant), Capture::Everything),
    ] {
        values.push((name, recorded(Event::ResponseStarted(value_response_started(&words))), capture));
    }
    values
}

fn response_completed_samples() -> Vec<(&'static str, Record, Capture)> {
    let mut values = Vec::new();
    for (name, words, capture) in [
        ("response_completed_none", Fill::capture(Capture::None), Capture::None),
        ("response_completed_calls", Fill::capture(Capture::Calls), Capture::Calls),
        ("response_completed_everything", Fill::capture(Capture::Everything), Capture::Everything),
        ("outcome_completed", Fill::outcome(Outcome::Completed), Capture::Everything),
        ("outcome_failed", Fill::outcome(Outcome::Failed), Capture::Everything),
        ("outcome_cancelled", Fill::outcome(Outcome::Cancelled), Capture::Everything),
        ("stop_end", Fill::stop(Stop::End), Capture::Everything),
        ("stop_tools", Fill::stop(Stop::Tools), Capture::Everything),
        ("stop_max_tokens", Fill::stop(Stop::MaxTokens), Capture::Everything),
        ("stop_refusal", Fill::stop(Stop::Refusal), Capture::Everything),
    ] {
        values.push((name, recorded(Event::ResponseCompleted(value_response_completed(&words))), capture));
    }
    values
}

fn text_delta_samples() -> Vec<(&'static str, Record, Capture)> {
    vec![("text_delta_everything", recorded(Event::TextDelta(value_text_delta(&Fill::DEFAULT))), Capture::Everything)]
}

fn tool_started_samples() -> Vec<(&'static str, Record, Capture)> {
    let mut values = Vec::new();
    for (name, words, capture) in [
        ("tool_started_none", Fill::capture(Capture::None), Capture::None),
        ("tool_started_calls", Fill::capture(Capture::Calls), Capture::Calls),
        ("tool_started_everything", Fill::capture(Capture::Everything), Capture::Everything),
        ("source_workspace", Fill::source(Source::Workspace), Capture::Everything),
        ("source_run", Fill::source(Source::Run), Capture::Everything),
        ("source_host", Fill::source(Source::Host), Capture::Everything),
        ("effect_read", Fill::effect(Effect::Read), Capture::Everything),
        ("effect_write", Fill::effect(Effect::Write), Capture::Everything),
    ] {
        values.push((name, recorded(Event::ToolStarted(value_tool_started(&words))), capture));
    }
    values
}

fn tool_completed_samples() -> Vec<(&'static str, Record, Capture)> {
    let mut values = Vec::new();
    for (name, words, capture) in vec![
        ("tool_completed_none", Fill::capture(Capture::None), Capture::None),
        ("tool_completed_calls", Fill::capture(Capture::Calls), Capture::Calls),
        ("tool_completed_everything", Fill::capture(Capture::Everything), Capture::Everything),
        ("verdict_read", Fill::verdict(Verdict::Read), Capture::Everything),
        ("verdict_listed", Fill::verdict(Verdict::Listed), Capture::Everything),
        ("verdict_found", Fill::verdict(Verdict::Found), Capture::Everything),
        ("verdict_written", Fill::verdict(Verdict::Written), Capture::Everything),
        ("verdict_edited", Fill::verdict(Verdict::Edited), Capture::Everything),
        ("verdict_exited", Fill::verdict(Verdict::Exited), Capture::Everything),
        ("verdict_conflict", Fill::verdict(Verdict::Conflict), Capture::Everything),
        ("verdict_missing", Fill::verdict(Verdict::Missing), Capture::Everything),
        ("verdict_too_large", Fill::verdict(Verdict::TooLarge), Capture::Everything),
        ("verdict_timed_out", Fill::verdict(Verdict::TimedOut), Capture::Everything),
        ("verdict_failed", Fill::verdict(Verdict::Failed), Capture::Everything),
        ("verdict_cancelled", Fill::verdict(Verdict::Cancelled), Capture::Everything),
        ("verdict_ambiguous", Fill::verdict(Verdict::Ambiguous), Capture::Everything),
        ("verdict_result", Fill::verdict(Verdict::Result), Capture::Everything),
        ("verdict_error", Fill::verdict(Verdict::Error), Capture::Everything),
        ("verdict_invalid", Fill::verdict(Verdict::Invalid), Capture::Everything),
        ("verdict_not_run", Fill::verdict(Verdict::NotRun), Capture::Everything),
        ("verdict_withdrawn", Fill::verdict(Verdict::Withdrawn), Capture::Everything),
        ("verdict_not_granted", Fill::verdict(Verdict::NotGranted), Capture::Everything),
        ("verdict_outside", Fill::verdict(Verdict::Outside), Capture::Everything),
        ("verdict_read_only", Fill::verdict(Verdict::ReadOnly), Capture::Everything),
        ("verdict_too_long", Fill::verdict(Verdict::TooLong), Capture::Everything),
        ("verdict_not_found", Fill::verdict(Verdict::NotFound), Capture::Everything),
        ("verdict_not_file", Fill::verdict(Verdict::NotFile), Capture::Everything),
        ("verdict_linked", Fill::verdict(Verdict::Linked), Capture::Everything),
        ("verdict_protected", Fill::verdict(Verdict::Protected), Capture::Everything),
        ("verdict_not_directory", Fill::verdict(Verdict::NotDirectory), Capture::Everything),
        ("verdict_not_read", Fill::verdict(Verdict::NotRead), Capture::Everything),
        ("verdict_stale", Fill::verdict(Verdict::Stale), Capture::Everything),
        ("verdict_no_match", Fill::verdict(Verdict::NoMatch), Capture::Everything),
        ("verdict_unchanged", Fill::verdict(Verdict::Unchanged), Capture::Everything),
        ("verdict_busy", Fill::verdict(Verdict::Busy), Capture::Everything),
        ("verdict_nul_byte", Fill::verdict(Verdict::NulByte), Capture::Everything),
        ("delivered_delivered", Fill::delivered(Delivered::Delivered), Capture::Everything),
        ("delivered_nothing", Fill::delivered(Delivered::Nothing), Capture::Everything),
        ("delivered_refused", Fill::delivered(Delivered::Refused), Capture::Everything),
        ("delivered_failed", Fill::delivered(Delivered::Failed), Capture::Everything),
        ("delivered_stale", Fill::delivered(Delivered::Stale), Capture::Everything),
    ] {
        values.push((name, recorded(Event::ToolCompleted(value_tool_completed(&words))), capture));
    }
    values
}

fn check_started_samples() -> Vec<(&'static str, Record, Capture)> {
    let mut values = Vec::new();
    for (name, capture) in [
        ("check_started_none", Capture::None),
        ("check_started_calls", Capture::Calls),
        ("check_started_everything", Capture::Everything),
    ] {
        values.push((name, recorded(Event::CheckStarted(value_check_started())), capture));
    }
    values
}

fn check_completed_samples() -> Vec<(&'static str, Record, Capture)> {
    let mut values = Vec::new();
    for (name, capture) in [
        ("check_completed_none", Capture::None),
        ("check_completed_calls", Capture::Calls),
        ("check_completed_everything", Capture::Everything),
    ] {
        values.push((name, recorded(Event::CheckCompleted(value_check_completed(&Fill::DEFAULT))), capture));
    }
    values
}

fn notice_samples() -> Vec<(&'static str, Record, Capture)> {
    let mut values = Vec::new();
    for (name, words, capture) in [
        ("notice_none", Fill::capture(Capture::None), Capture::None),
        ("notice_calls", Fill::capture(Capture::Calls), Capture::Calls),
        ("notice_everything", Fill::capture(Capture::Everything), Capture::Everything),
        ("level_info", Fill::level(Level::Info), Capture::Everything),
        ("level_warning", Fill::level(Level::Warning), Capture::Everything),
        (
            "notice_kind_credential_unauthorized",
            Fill::notice_kind(NoticeKind::CredentialUnauthorized),
            Capture::Everything,
        ),
        ("notice_kind_account_limited", Fill::notice_kind(NoticeKind::AccountLimited), Capture::Everything),
    ] {
        values.push((name, recorded(Event::Notice(value_notice(&words))), capture));
    }
    values
}

pub fn samples() -> Vec<(&'static str, Record, Capture)> {
    let mut samples = vec![reasoning_dropped_sample()];
    let mut signal = value_tool_completed(&Fill::DEFAULT);
    signal.exit = Some(CommandExit::Signal { signal: u64::MAX });
    samples.push((
        "tool_completed_signal",
        Record { t_ms: 0, event: Event::ToolCompleted(signal) },
        Capture::Everything,
    ));
    let mut zero = value_tool_completed(&Fill::DEFAULT);
    zero.omitted_bytes = 0;
    zero.clamped = false;
    samples.push((
        "tool_completed_unclamped",
        Record { t_ms: 0, event: Event::ToolCompleted(zero) },
        Capture::Everything,
    ));
    let mut check = value_check_completed(&Fill::DEFAULT);
    check.exit = CheckExit::Signal { signal: u64::MAX };
    samples.push((
        "check_completed_signal",
        Record { t_ms: 0, event: Event::CheckCompleted(check) },
        Capture::Everything,
    ));
    for (name, exit, passed) in [
        ("check_completed_code_zero", CheckExit::Code { code: 0 }, true),
        ("check_completed_timed_out", CheckExit::TimedOut, false),
        ("check_completed_unstarted", CheckExit::Unstarted, false),
        ("check_completed_cancelled", CheckExit::Cancelled, false),
        ("check_completed_unknown", CheckExit::Unknown("future🌍".as_bytes().into()), false),
    ] {
        let mut check = value_check_completed(&Fill::DEFAULT);
        check.exit = exit;
        check.passed = passed;
        samples.push((name, Record { t_ms: 0, event: Event::CheckCompleted(check) }, Capture::Everything));
    }
    for (name, choice) in [
        ("tool_choice_none", ToolChoice::None),
        ("tool_choice_only", ToolChoice::Only),
        ("tool_choice_unknown", ToolChoice::Unknown("future🌍".as_bytes().into())),
    ] {
        let mut response = value_response_started(&Fill::capture(Capture::Everything));
        response.tool_choice = choice;
        samples.push((name, recorded(Event::ResponseStarted(response)), Capture::Everything));
    }
    samples.extend(accounts::samples());
    samples.extend(accounts_samples());
    samples.extend(failure_samples());
    samples.extend(end_samples());
    samples.extend(answer_field_samples());
    samples.extend(budget_samples());
    samples.extend(message_samples());
    samples.extend(session_started_samples());
    samples.extend(session_ended_samples());
    samples.extend(run_started_samples());
    samples.extend(run_completed_samples());
    samples.extend(conversation_opened_samples());
    samples.extend(conversation_closed_samples());
    samples.extend(response_started_samples());
    samples.extend(response_completed_samples());
    samples.extend(text_delta_samples());
    samples.extend(tool_started_samples());
    samples.extend(tool_completed_samples());
    samples.extend(check_started_samples());
    samples.extend(check_completed_samples());
    samples.extend(notice_samples());
    samples.extend(null_samples());
    for (name, capture) in [
        ("session_ended_unread_none", Capture::None),
        ("session_ended_unread_calls", Capture::Calls),
        ("session_ended_unread_everything", Capture::Everything),
    ] {
        let mut end = value_session_ended(&Fill::DEFAULT);
        end.cpu_ms = None;
        end.peak_rss_bytes = None;
        samples.push((name, recorded(Event::SessionEnded(end)), capture));
    }
    samples.push(("session_started_effective_limits", effective_record(), Capture::Everything));
    for (name, containment) in [
        ("containment_groups", smith_events::Containment::Groups),
        ("containment_trees", smith_events::Containment::Trees),
        ("containment_unknown", smith_events::Containment::Unknown("future🌍".as_bytes().into())),
    ] {
        samples.push((
            name,
            Record { t_ms: 0, event: Event::SessionStarted(value_session_started(&Fill::containment(containment))) },
            Capture::Everything,
        ));
    }
    samples
}

fn null_samples() -> Vec<(&'static str, Record, Capture)> {
    let mut response = value_response_completed(&Fill::DEFAULT);
    response.stop = None;
    response.usage = Usage {
        input_tokens: None,
        cache_read_tokens: None,
        cache_write_tokens: None,
        output_tokens: None,
        reasoning_tokens: None,
    };
    response.request_bytes = None;
    response.first_byte_ms = None;
    response.largest_gap_ms = None;
    response.failure = None;
    response.retry_ms = None;
    response.completion = None;
    vec![("response_unknown_measures", recorded(Event::ResponseCompleted(response)), Capture::Everything)]
}

/// Header with exact scoped numbers and a policy share, independent of domain types.
pub fn effective_record() -> Record {
    let mut started = value_session_started(&Fill::DEFAULT);
    let mut values = List::with_capacity(6);
    for (name, value) in [
        ("llm.request[endpoint=7]", smith_events::LimitValue::Integer(u64::MAX)),
        ("llm.request[endpoint=8]", smith_events::LimitValue::Integer(9_007_199_254_740_993)),
        ("declared.window[endpoint=7,model=0]", smith_events::LimitValue::Integer(262_144)),
        ("declared.window[endpoint=7,model=1]", smith_events::LimitValue::Integer(131_072)),
        ("policy.compaction_threshold", smith_events::LimitValue::Fraction { numerator: 4, denominator: 5 }),
        ("declared.memory", smith_events::LimitValue::Integer(2_281_701_376)),
    ] {
        values.push(smith_events::EffectiveLimit { name: name.as_bytes().into(), value }).expect("fixture count");
    }
    started.limits = values;
    Record { t_ms: 0, event: Event::SessionStarted(started) }
}

fn budget_samples() -> Vec<(&'static str, Record, Capture)> {
    use smith_events::{BudgetStage, Clamp, Requested, Share, ShareBound, SharePart};
    let mut samples = Vec::new();
    samples.push((
        "budget_ordinary",
        Record { t_ms: 750, event: Event::Budget(sample_standing(None)) },
        Capture::Everything,
    ));
    samples.push((
        "budget_told",
        Record { t_ms: 750, event: Event::Budget(sample_standing(Some(Box::from(b"child-7".as_slice())))) },
        Capture::Everything,
    ));
    samples.push((
        "budget_wind_down",
        Record {
            t_ms: 750,
            event: Event::Budget({
                let mut value = sample_standing(None);
                value.stage = BudgetStage::WindDown;
                value
            }),
        },
        Capture::Everything,
    ));
    samples.push((
        "child_share_clamped",
        Record {
            t_ms: 0,
            event: Event::ConversationOpened(ConversationOpened {
                run: 1,
                conversation: 2,
                kind: ConversationKind::Child,
                parent: Some(1),
                call: Some(Box::from(b"c".as_slice())),
                thread: 2,
                model: Model {
                    account: None,
                    backend: smith_events::Backend::Direct,
                    endpoint: Box::from(b"e".as_slice()),
                    model: Box::from(b"m".as_slice()),
                    window_tokens: 1000,
                    output_tokens: 100,
                    effort: None,
                },
                requested: Some(Requested { turns: Some(9), seconds: Some(2) }),
                share: Some(Share { turns: 3, time_ms: 750, spend: 100 }),
                clamped: {
                    let mut clamps = List::with_capacity(2);
                    clamps
                        .push(Clamp { part: SharePart::Turns, by: ShareBound::Reserve })
                        .expect("two bounded clamp parts");
                    clamps
                        .push(Clamp { part: SharePart::Time, by: ShareBound::Deadline })
                        .expect("two bounded clamp parts");
                    clamps
                },
            }),
        },
        Capture::Everything,
    ));
    samples.push(child_terminal("child_stop_end", smith_events::ChildStop::End));
    samples.push(child_terminal("child_stop_budget", smith_events::ChildStop::Budget));
    samples.push(child_terminal("child_stop_failed", smith_events::ChildStop::Failed));
    samples.push(child_terminal("child_stop_closed", smith_events::ChildStop::Closed));
    samples.extend(job_samples());
    samples
}

fn child_terminal(name: &'static str, stop: smith_events::ChildStop) -> (&'static str, Record, Capture) {
    use smith_events::ChildCompleted;
    (
        name,
        Record {
            t_ms: 1999,
            event: Event::ToolCompleted(ToolCompleted {
                run: 1,
                conversation: 1,
                call: Box::from(b"c".as_slice()),
                tool: Box::from(b"sub_agent".as_slice()),
                verdict: Verdict::Result,
                exit: None,
                job: None,
                bytes: 1,
                omitted_bytes: 0,
                clamped: false,
                duration_ms: 1999,
                child: Some(ChildCompleted {
                    conversation: 2,
                    stop,
                    turns: 1,
                    time_ms: 1999,
                    spent: 3,
                    usage: Usage {
                        input_tokens: Some(4),
                        cache_read_tokens: None,
                        cache_write_tokens: None,
                        output_tokens: Some(5),
                        reasoning_tokens: None,
                    },
                }),
                delivery: None,
            }),
        },
        Capture::Everything,
    )
}

fn message_samples() -> Vec<(&'static str, Record, Capture)> {
    let mut samples = Vec::new();
    for (name, capture) in [
        ("message_received_none", Capture::None),
        ("message_received_calls", Capture::Calls),
        ("message_received_everything", Capture::Everything),
    ] {
        samples.push((
            name,
            Record { t_ms: u64::MAX, event: Event::MessageReceived(value_message_received(&Fill::DEFAULT)) },
            capture,
        ));
    }
    for (name, refused) in [
        ("message_refusal_too_large", MessageRefusal::TooLarge),
        ("message_refusal_inbox_full", MessageRefusal::InboxFull),
        ("message_refusal_name_in_use", MessageRefusal::NameInUse),
        ("message_refusal_ending", MessageRefusal::Ending),
        ("message_refusal_start_refused", MessageRefusal::StartRefused),
        ("message_refusal_unknown", MessageRefusal::Unknown(Box::from("future🌍".as_bytes()))),
    ] {
        samples.push((
            name,
            Record {
                t_ms: u64::MAX,
                event: Event::MessageReceived({
                    let mut value = value_message_received(&Fill::DEFAULT);
                    value.refused = Some(refused);
                    value.label = None;
                    value.text = None;
                    value
                }),
            },
            Capture::Everything,
        ));
    }
    for (name, capture) in [
        ("message_read_none", Capture::None),
        ("message_read_calls", Capture::Calls),
        ("message_read_everything", Capture::Everything),
    ] {
        samples.push((name, Record { t_ms: u64::MAX, event: Event::MessageRead(value_message_read()) }, capture));
    }
    for (name, capture) in [
        ("message_unread_none", Capture::None),
        ("message_unread_calls", Capture::Calls),
        ("message_unread_everything", Capture::Everything),
    ] {
        samples.push((name, recorded(Event::MessageUnread(value_message_unread())), capture));
    }
    let mut completed = value_run_completed(&Fill::DEFAULT);
    completed.read = None;
    completed.unread = 0;
    samples.push(("run_completed_no_read", recorded(Event::RunCompleted(completed)), Capture::Everything));
    samples
}

fn failure_samples() -> Vec<(&'static str, Record, Capture)> {
    let mut samples = Vec::new();
    for (name, phase) in [
        ("failure_phase_connect", FailurePhase::Connect),
        ("failure_phase_handshake", FailurePhase::Handshake),
        ("failure_phase_head", FailurePhase::Head),
        ("failure_phase_idle", FailurePhase::Idle),
        ("failure_phase_whole", FailurePhase::Whole),
        ("failure_phase_unknown", FailurePhase::Unknown("future🌍".as_bytes().into())),
    ] {
        let mut response = value_response_completed(&Fill::outcome(Outcome::Failed));
        let failure = response.failure.as_mut().expect("failure fixture");
        failure.phase = Some(phase);
        failure.limit = Some(FailureLimit { which: b"response_head".as_slice().into(), bound: u64::MAX });
        failure.status = Some(404);
        failure.detail = Some("unknown model 🌍".as_bytes().into());
        samples.push((name, Record { t_ms: 0, event: Event::ResponseCompleted(response) }, Capture::Everything));
    }
    for (name, capture) in [
        ("run_model_failure_none", Capture::None),
        ("run_model_failure_calls", Capture::Calls),
        ("run_model_failure_everything", Capture::Everything),
    ] {
        let mut run = value_run_completed(&Fill::status(Status::Failed));
        let failure = run.failure.as_mut().expect("run fixture");
        failure.class = AnswerClass::Model;
        failure.endpoint = Some(b"api.provider.test/v1".as_slice().into());
        failure.model = Some(b"missing-model".as_slice().into());
        failure.attempts = Some(2);
        let completion = failure.completion.as_mut().expect("completion fixture");
        completion.class = FailureClass::Invalid;
        completion.status = Some(404);
        completion.detail = Some("unknown model 🌍".as_bytes().into());
        samples.push((name, Record { t_ms: 0, event: Event::RunCompleted(run) }, capture));
    }
    samples
}

fn end_samples() -> Vec<(&'static str, Record, Capture)> {
    let mut samples = Vec::new();
    for (name, class, dropped, ended, turn) in [
        ("end_cancelled_answer", AnswerClass::Cancelled, Some(false), None, None),
        ("end_cancelled_drop", AnswerClass::Cancelled, Some(true), None, None),
        ("end_host_store", AnswerClass::Host, None, None, Some(2)),
        ("end_agent_code", AnswerClass::Agent, None, Some(AgentExit::Code(7)), None),
        ("end_agent_signal", AnswerClass::Agent, None, Some(AgentExit::Signal(15)), None),
    ] {
        let reason = if class == AnswerClass::Host { Some(Box::from(b"store".as_slice())) } else { None };
        samples.push((
            name,
            Record {
                t_ms: 10,
                event: Event::RunCompleted(RunCompleted {
                    run: 1,
                    wind_down: false,
                    read: None,
                    unread: 0,
                    status: Status::Failed,
                    failure: Some(AnswerFailure {
                        state: None,
                        code: None,
                        program: None,
                        exit: None,
                        detail: None,
                        reset_ms: None,
                        used: None,
                        bound: None,
                        endpoint: None,
                        model: None,
                        attempts: None,
                        class,
                        completion: None,
                        account: None,
                        which: None,
                        reason,
                        dropped,
                        ended,
                        turn,
                    }),
                    result: None,
                    turns: 1,
                    spent: 2,
                    usage: Usage {
                        input_tokens: Some(0),
                        cache_read_tokens: Some(0),
                        cache_write_tokens: Some(0),
                        output_tokens: Some(0),
                        reasoning_tokens: Some(0),
                    },
                    duration_ms: 10,
                }),
            },
            Capture::Everything,
        ));
    }
    let mut aborted = value_session_ended(&Fill::DEFAULT);
    aborted.aborted = true;
    samples.push(("session_ended_aborted", recorded(Event::SessionEnded(aborted)), Capture::Everything));
    samples.push((
        "notice_teardown_aborted",
        Record {
            t_ms: 0,
            event: Event::Notice(Notice {
                code: None,
                level: Level::Warning,
                kind: NoticeKind::TeardownAborted,
                run: None,
                account: None,
                reset_ms: None,
                bytes: None,
                exit: None,
                forced: None,
            }),
        },
        Capture::Everything,
    ));
    samples.extend(forced_exit_samples());
    samples
}

fn forced_exit_samples() -> Vec<(&'static str, Record, Capture)> {
    let mut samples = Vec::new();
    for (name, exit, forced) in [
        ("notice_forced_exit", Some(smith_events::AgentExit::Signal(9)), smith_events::Forced::Killed),
        ("notice_forced_terminated", Some(smith_events::AgentExit::Code(7)), smith_events::Forced::Terminated),
        ("notice_forced_no", None, smith_events::Forced::No),
        ("notice_forced_unknown", None, smith_events::Forced::Unknown(b"future".as_slice().into())),
    ] {
        samples.push((
            name,
            Record {
                t_ms: 0,
                event: Event::Notice(Notice {
                    code: None,
                    level: Level::Warning,
                    kind: NoticeKind::ForcedExit,
                    run: Some(1),
                    account: None,
                    reset_ms: None,
                    bytes: None,
                    exit,
                    forced: Some(forced),
                }),
            },
            Capture::Everything,
        ));
    }
    samples
}

fn reasoning_dropped_sample() -> (&'static str, Record, Capture) {
    (
        "notice_kind_reasoning_dropped",
        Record {
            t_ms: u64::MAX,
            event: Event::Notice(Notice {
                code: None,
                level: Level::Warning,
                kind: NoticeKind::ReasoningDropped,
                run: Some(u64::MAX),
                account: None,
                reset_ms: None,
                bytes: Some(u64::MAX),
                exit: None,
                forced: None,
            }),
        },
        Capture::Everything,
    )
}

fn answer_field_samples() -> Vec<(&'static str, Record, Capture)> {
    let mut samples = Vec::new();
    for (name, which, used, bound, account, reset_ms, state) in [
        ("run_account_limited", None, None, None, Some(17), Some(73), Some(smith_events::AccountState::Limited)),
        (
            "run_account_limited_unknown_reset",
            None,
            None,
            None,
            Some(17),
            None,
            Some(smith_events::AccountState::Limited),
        ),
        ("run_budget_turns_fields", Some("turns"), Some(8), Some(8), None, None, None),
        ("run_budget_spend_fields", Some("spend"), Some(19), Some(17), None, None, None),
        ("run_budget_time_fields", Some("time"), Some(1073), Some(1000), None, None, None),
    ] {
        let mut run = value_run_completed(&Fill::status(Status::Failed));
        let failure = run.failure.as_mut().expect("failed fixture");
        failure.class = if account.is_some() { AnswerClass::Model } else { AnswerClass::Budget };
        failure.completion = None;
        failure.account = account;
        failure.reset_ms = reset_ms;
        failure.state = state;
        failure.which = which.map(|which| Box::from(which.as_bytes()));
        failure.reason = None;
        failure.used = used;
        failure.bound = bound;
        samples.push((name, Record { t_ms: 0, event: Event::RunCompleted(run) }, Capture::None));
    }
    samples
}

fn job_samples() -> Vec<(&'static str, Record, Capture)> {
    let mut samples = Vec::new();
    for (name, verdict) in [("running", Verdict::Running), ("stopped", Verdict::Stopped)] {
        let mut completed = value_tool_completed(&Fill::verdict(verdict));
        completed.job = Some(u64::MAX);
        samples.push((
            if name == "running" { "verdict_running" } else { "verdict_stopped" },
            recorded(Event::ToolCompleted(completed)),
            Capture::Everything,
        ));
    }
    for (name, verdict, exit) in [
        ("job_ended_exited", Verdict::Exited, Some(CommandExit::Code { code: i64::MIN })),
        ("job_ended_timed_out", Verdict::TimedOut, None),
        ("job_ended_cancelled", Verdict::Cancelled, None),
    ] {
        samples.push((
            name,
            Record {
                t_ms: u64::MAX,
                event: Event::JobEnded({
                    let mut value = value_job_ended(&Fill::verdict(verdict));
                    value.exit = exit;
                    value
                }),
            },
            Capture::Everything,
        ));
    }
    samples
}

fn accounts_samples() -> Vec<(&'static str, Record, Capture)> {
    struct ModelSample {
        name: &'static str,
        account: Option<u32>,
        backend: smith_events::Backend,
    }
    struct FailureSample {
        name: &'static str,
        class: FailureClass,
    }
    let mut samples = Vec::new();
    for sample in [
        ModelSample { name: "model_account", account: Some(u32::MAX), backend: smith_events::Backend::Direct },
        ModelSample { name: "model_claude_code", account: None, backend: smith_events::Backend::ClaudeCode },
        ModelSample {
            name: "model_unknown_backend",
            account: None,
            backend: smith_events::Backend::Unknown(b"future".as_slice().into()),
        },
    ] {
        let mut value = value_response_started(&Fill::DEFAULT);
        value.model.account = sample.account;
        value.model.backend = sample.backend;
        samples.push((sample.name, Record { t_ms: 0, event: Event::ResponseStarted(value) }, Capture::Everything));
    }
    let mut value = value_run_started(&Fill::DEFAULT);
    value.tools.withheld = List::with_capacity(1);
    value.tools.withheld.push(Family::SubAgents).expect("withheld family fits");
    samples.push(("tools_withheld", Record { t_ms: 0, event: Event::RunStarted(value) }, Capture::Everything));
    for sample in [
        FailureSample { name: "failure_class_refused", class: FailureClass::Refused },
        FailureSample { name: "failure_class_unsupported", class: FailureClass::Unsupported },
    ] {
        let mut value = value_response_completed(&Fill::DEFAULT);
        let mut failure = value_completion_failure(&Fill::DEFAULT);
        failure.class = sample.class;
        failure.retry_after_ms = None;
        failure.field = Some(b"effort".as_slice().into());
        failure.code = Some(b"not_allowed".as_slice().into());
        failure.request_id = Some(b"request-7".as_slice().into());
        value.outcome = Outcome::Failed;
        value.failure = Some(failure);
        samples.push((sample.name, Record { t_ms: 0, event: Event::ResponseCompleted(value) }, Capture::Everything));
    }
    samples
}

#[path = "events/accounts.rs"]
mod accounts;

fn recorded(event: Event) -> Record {
    Record { t_ms: u64::MAX, event }
}

fn payload(length: u32) -> Box<[u8]> {
    vec![1; usize::try_from(length).expect("fixture length")].into_boxed_slice()
}

fn value_budget_standing(fill: &Fill) -> BudgetStanding {
    BudgetStanding {
        call: Some(fill.text()),
        run: u64::MAX,
        stage: BudgetStage::Unknown(fill.text()),
        turns: Quantity { used: u64::MAX, limit: u64::MAX },
        spend: Quantity { used: u64::MAX, limit: u64::MAX },
        time_ms: Quantity { used: u64::MAX, limit: u64::MAX },
        reserve: Reserve { turns: u64::MAX, time_ms: u64::MAX, spend: u64::MAX },
    }
}

fn value_job_ended(fill: &Fill) -> JobEnded {
    JobEnded {
        run: u64::MAX,
        conversation: u64::MAX,
        job: u64::MAX,
        call: fill.text(),
        verdict: fill.verdict.clone(),
        exit: Some(value_command_exit(fill)),
        unread_bytes: u64::MAX,
        duration_ms: u64::MAX,
    }
}

fn value_message_received(fill: &Fill) -> MessageReceived {
    MessageReceived {
        run: u64::MAX,
        name: u64::MAX,
        bytes: u64::MAX,
        refused: if fill.maximum() { Some(MessageRefusal::Unknown(fill.text())) } else { None },
        label: Some(fill.metadata("label\n\"🌍".as_bytes())),
        text: Some(fill.captured("text\n\"🌍".as_bytes())),
    }
}

fn value_message_read() -> MessageRead {
    MessageRead { run: u64::MAX, name: u64::MAX, turn: u64::MAX }
}

fn value_message_unread() -> MessageUnread {
    MessageUnread { run: u64::MAX, name: u64::MAX }
}

pub fn largest_samples(limits: &smith_events::Limits) -> Vec<Record> {
    let fill = &Fill::largest(limits);
    vec![
        Record { t_ms: u64::MAX, event: Event::Budget(value_budget_standing(fill)) },
        Record { t_ms: u64::MAX, event: Event::SessionStarted(value_session_started(fill)) },
        Record { t_ms: u64::MAX, event: Event::SessionEnded(value_session_ended(fill)) },
        Record { t_ms: u64::MAX, event: Event::RunStarted(value_run_started(fill)) },
        Record { t_ms: u64::MAX, event: Event::RunCompleted(value_run_completed(fill)) },
        Record { t_ms: u64::MAX, event: Event::ConversationOpened(value_conversation_opened(fill)) },
        Record { t_ms: u64::MAX, event: Event::ConversationClosed(value_conversation_closed(fill)) },
        Record { t_ms: u64::MAX, event: Event::ResponseStarted(value_response_started(fill)) },
        Record { t_ms: u64::MAX, event: Event::ResponseCompleted(value_response_completed(fill)) },
        Record { t_ms: u64::MAX, event: Event::TextDelta(value_text_delta(fill)) },
        Record { t_ms: u64::MAX, event: Event::ToolStarted(value_tool_started(fill)) },
        Record { t_ms: u64::MAX, event: Event::ToolCompleted(value_tool_completed(fill)) },
        Record { t_ms: u64::MAX, event: Event::JobEnded(value_job_ended(fill)) },
        Record { t_ms: u64::MAX, event: Event::CheckStarted(value_check_started()) },
        Record { t_ms: u64::MAX, event: Event::CheckCompleted(value_check_completed(fill)) },
        Record { t_ms: u64::MAX, event: Event::Notice(value_notice(fill)) },
        Record { t_ms: u64::MAX, event: Event::MessageReceived(value_message_received(fill)) },
        Record { t_ms: u64::MAX, event: Event::MessageRead(value_message_read()) },
        Record { t_ms: u64::MAX, event: Event::MessageUnread(value_message_unread()) },
    ]
}

fn sample_standing(call: Option<Box<[u8]>>) -> BudgetStanding {
    let mut value = value_budget_standing(&Fill::DEFAULT);
    value.run = 1;
    value.stage = BudgetStage::Ordinary;
    value.call = call;
    value.turns = Quantity { used: 2, limit: 10 };
    value.spend = Quantity { used: 3, limit: 100 };
    value.time_ms = Quantity { used: 750, limit: 1999 };
    value.reserve = Reserve { turns: 1, time_ms: 750, spend: 10 };
    value
}
