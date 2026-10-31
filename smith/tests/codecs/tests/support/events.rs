//! Typed fixture values for frozen event goldens.

use skein_lib::List;
use smith_events::{
    Agent, AgentExit, AnswerClass, AnswerFailure, Block, Budget, Capture, CheckCompleted, CheckExit, CheckStarted,
    CommandExit, Completion, CompletionFailure, ConversationClosed, ConversationEnd, ConversationKind,
    ConversationOpened, Count, Cpu, Delivered, Delivery, Effect, Event, Evidence, FailureClass, FailureLimit,
    FailurePhase, Family, Field, Form, Level, Loss, Message, Mode, Model, Notice, NoticeKind, Outcome, PeakRss, Prompt,
    Record, ResponseCompleted, ResponseStarted, Role, RunCompleted, RunResult, RunStarted, SessionEnded,
    SessionStarted, Source, Status, Stop, TextDelta, ToolChoice, ToolCompleted, ToolStarted, Tools, Usage, Verdict,
    Versions,
};

#[derive(Clone, Debug)]
struct Words {
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

impl Words {
    const DEFAULT: Words = Words {
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

    fn containment(value: smith_events::Containment) -> Words {
        Words { containment: value, ..Self::DEFAULT }
    }

    fn capture(value: Capture) -> Words {
        Words { capture: value, ..Self::DEFAULT }
    }

    fn delivery(value: Delivery) -> Words {
        Words { delivery: value, ..Self::DEFAULT }
    }

    fn mode(value: Mode) -> Words {
        Words { mode: value, ..Self::DEFAULT }
    }

    fn family(value: Family) -> Words {
        Words { family: value, ..Self::DEFAULT }
    }

    fn form(value: Form) -> Words {
        Words { form: value, ..Self::DEFAULT }
    }

    fn status(value: Status) -> Words {
        Words { status: value, ..Self::DEFAULT }
    }

    fn conversation_kind(value: ConversationKind) -> Words {
        Words { conversation_kind: value, ..Self::DEFAULT }
    }

    fn conversation_end(value: ConversationEnd) -> Words {
        Words { conversation_end: value, ..Self::DEFAULT }
    }

    fn outcome(value: Outcome) -> Words {
        Words { outcome: value, ..Self::DEFAULT }
    }

    fn stop(value: Stop) -> Words {
        Words { stop: value, ..Self::DEFAULT }
    }

    fn source(value: Source) -> Words {
        Words { source: value, ..Self::DEFAULT }
    }

    fn effect(value: Effect) -> Words {
        Words { effect: value, ..Self::DEFAULT }
    }

    fn verdict(value: Verdict) -> Words {
        Words { verdict: value, ..Self::DEFAULT }
    }

    fn delivered(value: Delivered) -> Words {
        Words { delivered: value, ..Self::DEFAULT }
    }

    fn failure_class(value: FailureClass) -> Words {
        Words { failure_class: value, ..Self::DEFAULT }
    }

    fn evidence(value: Evidence) -> Words {
        Words { evidence: value, ..Self::DEFAULT }
    }

    fn answer_class(value: AnswerClass) -> Words {
        Words { answer_class: value, ..Self::DEFAULT }
    }

    fn level(value: Level) -> Words {
        Words { level: value, ..Self::DEFAULT }
    }

    fn notice_kind(value: NoticeKind) -> Words {
        Words { notice_kind: value, ..Self::DEFAULT }
    }

    fn role(value: Role) -> Words {
        Words { role: value, ..Self::DEFAULT }
    }
}

fn value_agent() -> Agent {
    Agent { name: Box::from("sample\n\"🌍".as_bytes()), version: Box::from("sample\n\"🌍".as_bytes()) }
}

fn value_versions() -> Versions {
    Versions { events: u64::MAX, channel: u64::MAX, charter: u64::MAX, transcript: u64::MAX, chat_store: None }
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

fn value_model() -> Model {
    Model {
        account: None,
        backend: smith_events::Backend::Direct,
        endpoint: Box::from("sample\n\"🌍".as_bytes()),
        model: Box::from("sample\n\"🌍".as_bytes()),
        window_tokens: u64::MAX,
        output_tokens: u64::MAX,
        effort: Some(Box::from("sample\n\"🌍".as_bytes())),
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
        reserve: smith_events::Reserve { turns: u64::MAX, time_ms: u64::MAX, spend: u64::MAX },
    }
}

fn value_tools(words: &Words) -> Tools {
    Tools {
        withheld: skein_lib::List::with_capacity(0),
        families: list_family(words),
        wait: true,
        deliver: true,
        host: list_text(),
    }
}

fn value_completion_failure(words: &Words) -> CompletionFailure {
    CompletionFailure {
        field: None,
        code: None,
        request_id: None,
        phase: None,
        limit: None,
        status: None,
        detail: None,
        class: words.failure_class.clone(),
        evidence: words.evidence.clone(),
        retry_after_ms: Some(u64::MAX),
    }
}

fn value_answer_failure(words: &Words) -> AnswerFailure {
    AnswerFailure {
        state: None,
        code: None,
        program: None,
        exit: None,
        detail: None,
        reset_ms: None,
        used: None,
        bound: None,
        dropped: if words.answer_class == AnswerClass::Cancelled { Some(false) } else { None },
        ended: None,
        turn: None,
        endpoint: None,
        model: None,
        attempts: None,
        class: words.answer_class.clone(),
        completion: Some(value_completion_failure(words)),
        account: Some(u32::MAX),
        which: Some(Box::from("sample\n\"🌍".as_bytes())),
        reason: Some(Box::from("sample\n\"🌍".as_bytes())),
    }
}

fn value_field() -> Field {
    Field { name: Box::from("sample\n\"🌍".as_bytes()), text: Box::from("sample\n\"🌍".as_bytes()) }
}

fn value_run_result(words: &Words) -> RunResult {
    RunResult {
        form: words.form.clone(),
        label: Some(Box::from("sample\n\"🌍".as_bytes())),
        text: Some(Box::from("sample\n\"🌍".as_bytes())),
        fields: Some(list_field()),
    }
}

fn value_message(words: &Words) -> Message {
    Message { role: words.role.clone(), blocks: list_block() }
}

fn value_prompt(words: &Words) -> Prompt {
    Prompt {
        system: Some(Box::from("sample\n\"🌍".as_bytes())),
        tools: Some(list_text()),
        messages: list_message(words),
    }
}

fn value_completion() -> Completion {
    Completion { blocks: list_block() }
}

fn value_command_exit() -> CommandExit {
    CommandExit::Code { code: i64::MIN }
}

fn value_session_started(words: &Words) -> SessionStarted {
    SessionStarted {
        chat: if words.mode == Mode::Chat { Some(b"main".as_slice().into()) } else { None },
        limits: List::with_capacity(0),
        wall_ms: u64::MAX,
        agent: value_agent(),
        mode: words.mode.clone(),
        containment: words.containment.clone(),
        pid: u64::MAX,
        profile: Box::from("sample\n\"🌍".as_bytes()),
        capture: words.capture.clone(),
        delivery: words.delivery.clone(),
        versions: value_versions(),
    }
}

fn value_session_ended() -> SessionEnded {
    SessionEnded {
        exit: i64::MIN,
        answered: true,
        aborted: false,
        teardown_ms: Some(u64::MAX),
        emitted: list_count(),
        loss: value_loss(),
        cpu_ms: Some(value_cpu()),
        peak_rss_bytes: Some(value_peak_rss()),
    }
}

fn value_run_started(words: &Words) -> RunStarted {
    RunStarted {
        run: u64::MAX,
        resumed: true,
        messages: u64::MAX,
        main: value_model(),
        budget: value_budget(),
        tools: value_tools(words),
        contract: list_form(words),
    }
}

fn value_run_completed(words: &Words) -> RunCompleted {
    RunCompleted {
        run: u64::MAX,
        wind_down: false,
        status: words.status.clone(),
        failure: Some(value_answer_failure(words)),
        result: Some(value_run_result(words)),
        turns: u64::MAX,
        spent: u64::MAX,
        usage: value_usage(),
        read: Some(u64::MAX),
        unread: u64::MAX,
        duration_ms: u64::MAX,
    }
}

fn value_conversation_opened(words: &Words) -> ConversationOpened {
    ConversationOpened {
        run: u64::MAX,
        conversation: u64::MAX,
        kind: words.conversation_kind.clone(),
        parent: Some(u64::MAX),
        call: Some(Box::from("sample\n\"🌍".as_bytes())),
        thread: u64::MAX,
        model: value_model(),
        requested: None,
        share: None,
        clamped: List::with_capacity(0),
    }
}

fn value_conversation_closed(words: &Words) -> ConversationClosed {
    ConversationClosed {
        run: u64::MAX,
        conversation: u64::MAX,
        end: words.conversation_end.clone(),
        which: Some(Box::from("sample\n\"🌍".as_bytes())),
        failure: Some(value_completion_failure(words)),
        turns: u64::MAX,
        usage: value_usage(),
        spent: u64::MAX,
        duration_ms: u64::MAX,
    }
}

fn value_response_started(words: &Words) -> ResponseStarted {
    ResponseStarted {
        run: u64::MAX,
        conversation: u64::MAX,
        response: u64::MAX,
        turn: u64::MAX,
        attempt: u64::MAX,
        window: u64::MAX,
        model: value_model(),
        messages: u64::MAX,
        output_tokens: u64::MAX,
        tool_choice: ToolChoice::Auto,
        prompt: Some(value_prompt(words)),
    }
}

fn value_response_completed(words: &Words) -> ResponseCompleted {
    ResponseCompleted {
        run: u64::MAX,
        conversation: u64::MAX,
        response: u64::MAX,
        outcome: words.outcome.clone(),
        stop: Some(words.stop.clone()),
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
        failure: Some(value_completion_failure(words)),
        retry_ms: Some(u64::MAX),
        completion: Some(value_completion()),
    }
}

fn value_text_delta() -> TextDelta {
    TextDelta {
        run: u64::MAX,
        conversation: u64::MAX,
        response: u64::MAX,
        block: u64::MAX,
        text: Box::from("sample\n\"🌍".as_bytes()),
    }
}

fn value_tool_started(words: &Words) -> ToolStarted {
    ToolStarted {
        run: u64::MAX,
        conversation: u64::MAX,
        call: Box::from("sample\n\"🌍".as_bytes()),
        tool: Box::from("sample\n\"🌍".as_bytes()),
        source: words.source.clone(),
        effect: words.effect.clone(),
        deadline_ms: Some(u64::MAX),
        input_bytes: u64::MAX,
        input: Some(Box::from("sample\n\"🌍".as_bytes())),
    }
}

fn value_tool_completed(words: &Words) -> ToolCompleted {
    ToolCompleted {
        run: u64::MAX,
        conversation: u64::MAX,
        call: Box::from("sample\n\"🌍".as_bytes()),
        tool: Box::from("sample\n\"🌍".as_bytes()),
        verdict: words.verdict.clone(),
        exit: Some(value_command_exit()),
        job: None,
        bytes: u64::MAX,
        omitted_bytes: u64::MAX,
        clamped: true,
        duration_ms: u64::MAX,
        child: None,
        delivery: Some(words.delivered.clone()),
    }
}

fn value_check_started() -> CheckStarted {
    CheckStarted { run: u64::MAX, deadline_ms: Some(u64::MAX) }
}

fn value_check_completed() -> CheckCompleted {
    CheckCompleted { run: u64::MAX, exit: CheckExit::Code { code: i64::MIN }, passed: false, duration_ms: u64::MAX }
}

fn value_notice(words: &Words) -> Notice {
    Notice {
        code: None,
        level: words.level.clone(),
        kind: words.notice_kind.clone(),
        run: Some(u64::MAX),
        account: Some(u32::MAX),
        bytes: None,
        reset_ms: Some(u64::MAX),
        exit: None,
        forced: None,
    }
}

fn list_block() -> List<Block> {
    let mut values = List::with_capacity(7);
    values.push(Block::Text { text: Box::from("sample\n\"🌍".as_bytes()) }).expect("block room");
    values.push(Block::Refusal { text: Box::from("sample\n\"🌍".as_bytes()) }).expect("block room");
    values
        .push(Block::Call {
            call: Box::from("sample\n\"🌍".as_bytes()),
            tool: Box::from("sample\n\"🌍".as_bytes()),
            input: Box::from("sample\n\"🌍".as_bytes()),
        })
        .expect("block room");
    values
        .push(Block::Oversized {
            call: Box::from("sample\n\"🌍".as_bytes()),
            tool: Box::from("sample\n\"🌍".as_bytes()),
            bytes: u64::MAX,
        })
        .expect("block room");
    values
        .push(Block::Cut {
            call: Box::from("sample\n\"🌍".as_bytes()),
            tool: Box::from("sample\n\"🌍".as_bytes()),
            input: Box::from("sample\n\"🌍".as_bytes()),
        })
        .expect("block room");
    values
        .push(Block::Result {
            call: Box::from("sample\n\"🌍".as_bytes()),
            error: true,
            text: Box::from("sample\n\"🌍".as_bytes()),
        })
        .expect("block room");
    values.push(Block::Opaque { bytes: u64::MAX }).expect("block room");
    values
}

fn list_count() -> List<Count> {
    let mut values = List::with_capacity(1);
    values.push(Count { record: Box::from(b"notice".as_slice()), count: u64::MAX }).expect("one count");
    values
}

fn list_family(words: &Words) -> List<Family> {
    let mut values = List::with_capacity(1);
    values.push(words.family.clone()).expect("fixture room");
    values
}

fn list_field() -> List<Field> {
    let mut values = List::with_capacity(1);
    values.push(value_field()).expect("fixture room");
    values
}

fn list_form(words: &Words) -> List<Form> {
    let mut values = List::with_capacity(1);
    values.push(words.form.clone()).expect("fixture room");
    values
}

fn list_message(words: &Words) -> List<Message> {
    let mut values = List::with_capacity(1);
    values.push(value_message(words)).expect("fixture room");
    values
}

fn list_text() -> List<Box<[u8]>> {
    let mut values = List::with_capacity(1);
    values.push(Box::from("sample\n\"🌍".as_bytes())).expect("fixture room");
    values
}

fn samples_0() -> Vec<(&'static str, Record, Capture)> {
    vec![
        (
            "session_started_none",
            recorded(Event::SessionStarted(value_session_started(&Words::capture(Capture::None)))),
            Capture::None,
        ),
        (
            "session_started_calls",
            recorded(Event::SessionStarted(value_session_started(&Words::capture(Capture::Calls)))),
            Capture::Calls,
        ),
        (
            "session_started_everything",
            recorded(Event::SessionStarted(value_session_started(&Words::capture(Capture::Everything)))),
            Capture::Everything,
        ),
        ("session_ended_none", recorded(Event::SessionEnded(value_session_ended())), Capture::None),
        ("session_ended_calls", recorded(Event::SessionEnded(value_session_ended())), Capture::Calls),
        ("session_ended_everything", recorded(Event::SessionEnded(value_session_ended())), Capture::Everything),
        (
            "run_started_none",
            recorded(Event::RunStarted(value_run_started(&Words::capture(Capture::None)))),
            Capture::None,
        ),
        (
            "run_started_calls",
            recorded(Event::RunStarted(value_run_started(&Words::capture(Capture::Calls)))),
            Capture::Calls,
        ),
        (
            "run_started_everything",
            recorded(Event::RunStarted(value_run_started(&Words::capture(Capture::Everything)))),
            Capture::Everything,
        ),
        (
            "run_completed_none",
            recorded(Event::RunCompleted(value_run_completed(&Words::capture(Capture::None)))),
            Capture::None,
        ),
    ]
}

fn samples_1() -> Vec<(&'static str, Record, Capture)> {
    vec![
        (
            "run_completed_calls",
            recorded(Event::RunCompleted(value_run_completed(&Words::capture(Capture::Calls)))),
            Capture::Calls,
        ),
        (
            "run_completed_everything",
            recorded(Event::RunCompleted(value_run_completed(&Words::capture(Capture::Everything)))),
            Capture::Everything,
        ),
        (
            "conversation_opened_none",
            recorded(Event::ConversationOpened(value_conversation_opened(&Words::capture(Capture::None)))),
            Capture::None,
        ),
        (
            "conversation_opened_calls",
            recorded(Event::ConversationOpened(value_conversation_opened(&Words::capture(Capture::Calls)))),
            Capture::Calls,
        ),
        (
            "conversation_opened_everything",
            Record {
                t_ms: u64::MAX,
                event: Event::ConversationOpened(value_conversation_opened(&Words::capture(Capture::Everything))),
            },
            Capture::Everything,
        ),
        (
            "conversation_closed_none",
            recorded(Event::ConversationClosed(value_conversation_closed(&Words::capture(Capture::None)))),
            Capture::None,
        ),
        (
            "conversation_closed_calls",
            recorded(Event::ConversationClosed(value_conversation_closed(&Words::capture(Capture::Calls)))),
            Capture::Calls,
        ),
        (
            "conversation_closed_everything",
            Record {
                t_ms: u64::MAX,
                event: Event::ConversationClosed(value_conversation_closed(&Words::capture(Capture::Everything))),
            },
            Capture::Everything,
        ),
        (
            "response_started_none",
            recorded(Event::ResponseStarted(value_response_started(&Words::capture(Capture::None)))),
            Capture::None,
        ),
        (
            "response_started_calls",
            recorded(Event::ResponseStarted(value_response_started(&Words::capture(Capture::Calls)))),
            Capture::Calls,
        ),
    ]
}

fn samples_2() -> Vec<(&'static str, Record, Capture)> {
    vec![
        (
            "response_started_everything",
            recorded(Event::ResponseStarted(value_response_started(&Words::capture(Capture::Everything)))),
            Capture::Everything,
        ),
        (
            "response_completed_none",
            recorded(Event::ResponseCompleted(value_response_completed(&Words::capture(Capture::None)))),
            Capture::None,
        ),
        (
            "response_completed_calls",
            recorded(Event::ResponseCompleted(value_response_completed(&Words::capture(Capture::Calls)))),
            Capture::Calls,
        ),
        (
            "response_completed_everything",
            Record {
                t_ms: u64::MAX,
                event: Event::ResponseCompleted(value_response_completed(&Words::capture(Capture::Everything))),
            },
            Capture::Everything,
        ),
        ("text_delta_everything", recorded(Event::TextDelta(value_text_delta())), Capture::Everything),
        (
            "tool_started_none",
            recorded(Event::ToolStarted(value_tool_started(&Words::capture(Capture::None)))),
            Capture::None,
        ),
        (
            "tool_started_calls",
            recorded(Event::ToolStarted(value_tool_started(&Words::capture(Capture::Calls)))),
            Capture::Calls,
        ),
        (
            "tool_started_everything",
            recorded(Event::ToolStarted(value_tool_started(&Words::capture(Capture::Everything)))),
            Capture::Everything,
        ),
        (
            "tool_completed_none",
            recorded(Event::ToolCompleted(value_tool_completed(&Words::capture(Capture::None)))),
            Capture::None,
        ),
        (
            "tool_completed_calls",
            recorded(Event::ToolCompleted(value_tool_completed(&Words::capture(Capture::Calls)))),
            Capture::Calls,
        ),
    ]
}

fn samples_3() -> Vec<(&'static str, Record, Capture)> {
    vec![
        (
            "tool_completed_everything",
            recorded(Event::ToolCompleted(value_tool_completed(&Words::capture(Capture::Everything)))),
            Capture::Everything,
        ),
        ("check_started_none", recorded(Event::CheckStarted(value_check_started())), Capture::None),
        ("check_started_calls", recorded(Event::CheckStarted(value_check_started())), Capture::Calls),
        ("check_started_everything", recorded(Event::CheckStarted(value_check_started())), Capture::Everything),
        ("check_completed_none", recorded(Event::CheckCompleted(value_check_completed())), Capture::None),
        ("check_completed_calls", recorded(Event::CheckCompleted(value_check_completed())), Capture::Calls),
        ("check_completed_everything", recorded(Event::CheckCompleted(value_check_completed())), Capture::Everything),
        ("notice_none", recorded(Event::Notice(value_notice(&Words::capture(Capture::None)))), Capture::None),
        ("notice_calls", recorded(Event::Notice(value_notice(&Words::capture(Capture::Calls)))), Capture::Calls),
        (
            "notice_everything",
            recorded(Event::Notice(value_notice(&Words::capture(Capture::Everything)))),
            Capture::Everything,
        ),
    ]
}

fn samples_4() -> Vec<(&'static str, Record, Capture)> {
    vec![
        (
            "capture_none",
            recorded(Event::SessionStarted(value_session_started(&Words::capture(Capture::None)))),
            Capture::Everything,
        ),
        (
            "capture_calls",
            recorded(Event::SessionStarted(value_session_started(&Words::capture(Capture::Calls)))),
            Capture::Everything,
        ),
        (
            "capture_everything",
            recorded(Event::SessionStarted(value_session_started(&Words::capture(Capture::Everything)))),
            Capture::Everything,
        ),
        (
            "delivery_complete",
            recorded(Event::SessionStarted(value_session_started(&Words::delivery(Delivery::Complete)))),
            Capture::Everything,
        ),
        (
            "delivery_best_effort",
            recorded(Event::SessionStarted(value_session_started(&Words::delivery(Delivery::BestEffort)))),
            Capture::Everything,
        ),
        (
            "mode_agent",
            recorded(Event::SessionStarted(value_session_started(&Words::mode(Mode::Agent)))),
            Capture::Everything,
        ),
        (
            "mode_exec",
            recorded(Event::SessionStarted(value_session_started(&Words::mode(Mode::Exec)))),
            Capture::Everything,
        ),
        (
            "mode_chat",
            recorded(Event::SessionStarted(value_session_started(&Words::mode(Mode::Chat)))),
            Capture::Everything,
        ),
        (
            "family_modify",
            recorded(Event::RunStarted(value_run_started(&Words::family(Family::Modify)))),
            Capture::Everything,
        ),
        (
            "family_shell",
            recorded(Event::RunStarted(value_run_started(&Words::family(Family::Shell)))),
            Capture::Everything,
        ),
    ]
}

fn samples_5() -> Vec<(&'static str, Record, Capture)> {
    vec![
        (
            "family_sub_agents",
            recorded(Event::RunStarted(value_run_started(&Words::family(Family::SubAgents)))),
            Capture::Everything,
        ),
        (
            "form_report",
            recorded(Event::RunStarted(value_run_started(&Words::form(Form::Report)))),
            Capture::Everything,
        ),
        (
            "form_verdict",
            recorded(Event::RunStarted(value_run_started(&Words::form(Form::Verdict)))),
            Capture::Everything,
        ),
        (
            "form_change",
            recorded(Event::RunStarted(value_run_started(&Words::form(Form::Change)))),
            Capture::Everything,
        ),
        (
            "form_failure",
            recorded(Event::RunStarted(value_run_started(&Words::form(Form::Failure)))),
            Capture::Everything,
        ),
        (
            "status_accepted",
            recorded(Event::RunCompleted(value_run_completed(&Words::status(Status::Accepted)))),
            Capture::Everything,
        ),
        (
            "status_parked",
            recorded(Event::RunCompleted(value_run_completed(&Words::status(Status::Parked)))),
            Capture::Everything,
        ),
        (
            "status_failed",
            recorded(Event::RunCompleted(value_run_completed(&Words::status(Status::Failed)))),
            Capture::Everything,
        ),
        (
            "status_refused",
            recorded(Event::RunCompleted(value_run_completed(&Words::status(Status::Refused)))),
            Capture::Everything,
        ),
        (
            "conversation_kind_main",
            Record {
                t_ms: u64::MAX,
                event: Event::ConversationOpened(value_conversation_opened(&Words::conversation_kind(
                    ConversationKind::Main,
                ))),
            },
            Capture::Everything,
        ),
    ]
}

fn samples_6() -> Vec<(&'static str, Record, Capture)> {
    vec![
        (
            "conversation_kind_child",
            Record {
                t_ms: u64::MAX,
                event: Event::ConversationOpened(value_conversation_opened(&Words::conversation_kind(
                    ConversationKind::Child,
                ))),
            },
            Capture::Everything,
        ),
        (
            "conversation_kind_compaction",
            Record {
                t_ms: u64::MAX,
                event: Event::ConversationOpened(value_conversation_opened(&Words::conversation_kind(
                    ConversationKind::Compaction,
                ))),
            },
            Capture::Everything,
        ),
        (
            "conversation_end_closed",
            Record {
                t_ms: u64::MAX,
                event: Event::ConversationClosed(value_conversation_closed(&Words::conversation_end(
                    ConversationEnd::Closed,
                ))),
            },
            Capture::Everything,
        ),
        (
            "conversation_end_budget",
            Record {
                t_ms: u64::MAX,
                event: Event::ConversationClosed(value_conversation_closed(&Words::conversation_end(
                    ConversationEnd::Budget,
                ))),
            },
            Capture::Everything,
        ),
        (
            "conversation_end_failed",
            Record {
                t_ms: u64::MAX,
                event: Event::ConversationClosed(value_conversation_closed(&Words::conversation_end(
                    ConversationEnd::Failed,
                ))),
            },
            Capture::Everything,
        ),
        (
            "conversation_end_refused",
            Record {
                t_ms: u64::MAX,
                event: Event::ConversationClosed(value_conversation_closed(&Words::conversation_end(
                    ConversationEnd::Refused,
                ))),
            },
            Capture::Everything,
        ),
        (
            "conversation_end_transcript",
            Record {
                t_ms: u64::MAX,
                event: Event::ConversationClosed(value_conversation_closed(&Words::conversation_end(
                    ConversationEnd::Transcript,
                ))),
            },
            Capture::Everything,
        ),
        (
            "conversation_end_overflow",
            Record {
                t_ms: u64::MAX,
                event: Event::ConversationClosed(value_conversation_closed(&Words::conversation_end(
                    ConversationEnd::Overflow,
                ))),
            },
            Capture::Everything,
        ),
        (
            "outcome_completed",
            Record {
                t_ms: u64::MAX,
                event: Event::ResponseCompleted(value_response_completed(&Words::outcome(Outcome::Completed))),
            },
            Capture::Everything,
        ),
        (
            "outcome_failed",
            recorded(Event::ResponseCompleted(value_response_completed(&Words::outcome(Outcome::Failed)))),
            Capture::Everything,
        ),
    ]
}

fn samples_7() -> Vec<(&'static str, Record, Capture)> {
    vec![
        (
            "outcome_cancelled",
            Record {
                t_ms: u64::MAX,
                event: Event::ResponseCompleted(value_response_completed(&Words::outcome(Outcome::Cancelled))),
            },
            Capture::Everything,
        ),
        (
            "stop_end",
            recorded(Event::ResponseCompleted(value_response_completed(&Words::stop(Stop::End)))),
            Capture::Everything,
        ),
        (
            "stop_tools",
            recorded(Event::ResponseCompleted(value_response_completed(&Words::stop(Stop::Tools)))),
            Capture::Everything,
        ),
        (
            "stop_max_tokens",
            recorded(Event::ResponseCompleted(value_response_completed(&Words::stop(Stop::MaxTokens)))),
            Capture::Everything,
        ),
        (
            "stop_refusal",
            recorded(Event::ResponseCompleted(value_response_completed(&Words::stop(Stop::Refusal)))),
            Capture::Everything,
        ),
        (
            "source_workspace",
            recorded(Event::ToolStarted(value_tool_started(&Words::source(Source::Workspace)))),
            Capture::Everything,
        ),
        (
            "source_run",
            recorded(Event::ToolStarted(value_tool_started(&Words::source(Source::Run)))),
            Capture::Everything,
        ),
        (
            "source_host",
            recorded(Event::ToolStarted(value_tool_started(&Words::source(Source::Host)))),
            Capture::Everything,
        ),
        (
            "effect_read",
            recorded(Event::ToolStarted(value_tool_started(&Words::effect(Effect::Read)))),
            Capture::Everything,
        ),
        (
            "effect_write",
            recorded(Event::ToolStarted(value_tool_started(&Words::effect(Effect::Write)))),
            Capture::Everything,
        ),
    ]
}

fn samples_8() -> Vec<(&'static str, Record, Capture)> {
    vec![
        (
            "verdict_read",
            recorded(Event::ToolCompleted(value_tool_completed(&Words::verdict(Verdict::Read)))),
            Capture::Everything,
        ),
        (
            "verdict_listed",
            recorded(Event::ToolCompleted(value_tool_completed(&Words::verdict(Verdict::Listed)))),
            Capture::Everything,
        ),
        (
            "verdict_found",
            recorded(Event::ToolCompleted(value_tool_completed(&Words::verdict(Verdict::Found)))),
            Capture::Everything,
        ),
        (
            "verdict_written",
            recorded(Event::ToolCompleted(value_tool_completed(&Words::verdict(Verdict::Written)))),
            Capture::Everything,
        ),
        (
            "verdict_edited",
            recorded(Event::ToolCompleted(value_tool_completed(&Words::verdict(Verdict::Edited)))),
            Capture::Everything,
        ),
        (
            "verdict_exited",
            recorded(Event::ToolCompleted(value_tool_completed(&Words::verdict(Verdict::Exited)))),
            Capture::Everything,
        ),
        (
            "verdict_conflict",
            recorded(Event::ToolCompleted(value_tool_completed(&Words::verdict(Verdict::Conflict)))),
            Capture::Everything,
        ),
        (
            "verdict_missing",
            recorded(Event::ToolCompleted(value_tool_completed(&Words::verdict(Verdict::Missing)))),
            Capture::Everything,
        ),
        (
            "verdict_too_large",
            recorded(Event::ToolCompleted(value_tool_completed(&Words::verdict(Verdict::TooLarge)))),
            Capture::Everything,
        ),
        (
            "verdict_timed_out",
            recorded(Event::ToolCompleted(value_tool_completed(&Words::verdict(Verdict::TimedOut)))),
            Capture::Everything,
        ),
    ]
}

fn samples_9() -> Vec<(&'static str, Record, Capture)> {
    vec![
        (
            "verdict_failed",
            recorded(Event::ToolCompleted(value_tool_completed(&Words::verdict(Verdict::Failed)))),
            Capture::Everything,
        ),
        (
            "verdict_cancelled",
            recorded(Event::ToolCompleted(value_tool_completed(&Words::verdict(Verdict::Cancelled)))),
            Capture::Everything,
        ),
        (
            "verdict_ambiguous",
            recorded(Event::ToolCompleted(value_tool_completed(&Words::verdict(Verdict::Ambiguous)))),
            Capture::Everything,
        ),
        (
            "verdict_result",
            recorded(Event::ToolCompleted(value_tool_completed(&Words::verdict(Verdict::Result)))),
            Capture::Everything,
        ),
        (
            "verdict_error",
            recorded(Event::ToolCompleted(value_tool_completed(&Words::verdict(Verdict::Error)))),
            Capture::Everything,
        ),
        (
            "verdict_invalid",
            recorded(Event::ToolCompleted(value_tool_completed(&Words::verdict(Verdict::Invalid)))),
            Capture::Everything,
        ),
        (
            "verdict_not_run",
            recorded(Event::ToolCompleted(value_tool_completed(&Words::verdict(Verdict::NotRun)))),
            Capture::Everything,
        ),
        (
            "verdict_withdrawn",
            recorded(Event::ToolCompleted(value_tool_completed(&Words::verdict(Verdict::Withdrawn)))),
            Capture::Everything,
        ),
        (
            "verdict_not_granted",
            recorded(Event::ToolCompleted(value_tool_completed(&Words::verdict(Verdict::NotGranted)))),
            Capture::Everything,
        ),
        (
            "verdict_outside",
            recorded(Event::ToolCompleted(value_tool_completed(&Words::verdict(Verdict::Outside)))),
            Capture::Everything,
        ),
    ]
}

fn samples_10() -> Vec<(&'static str, Record, Capture)> {
    vec![
        (
            "verdict_read_only",
            recorded(Event::ToolCompleted(value_tool_completed(&Words::verdict(Verdict::ReadOnly)))),
            Capture::Everything,
        ),
        (
            "verdict_too_long",
            recorded(Event::ToolCompleted(value_tool_completed(&Words::verdict(Verdict::TooLong)))),
            Capture::Everything,
        ),
        (
            "verdict_not_found",
            recorded(Event::ToolCompleted(value_tool_completed(&Words::verdict(Verdict::NotFound)))),
            Capture::Everything,
        ),
        (
            "verdict_not_file",
            recorded(Event::ToolCompleted(value_tool_completed(&Words::verdict(Verdict::NotFile)))),
            Capture::Everything,
        ),
        (
            "verdict_linked",
            recorded(Event::ToolCompleted(value_tool_completed(&Words::verdict(Verdict::Linked)))),
            Capture::Everything,
        ),
        (
            "verdict_protected",
            recorded(Event::ToolCompleted(value_tool_completed(&Words::verdict(Verdict::Protected)))),
            Capture::Everything,
        ),
        (
            "verdict_not_directory",
            recorded(Event::ToolCompleted(value_tool_completed(&Words::verdict(Verdict::NotDirectory)))),
            Capture::Everything,
        ),
        (
            "verdict_not_read",
            recorded(Event::ToolCompleted(value_tool_completed(&Words::verdict(Verdict::NotRead)))),
            Capture::Everything,
        ),
        (
            "verdict_stale",
            recorded(Event::ToolCompleted(value_tool_completed(&Words::verdict(Verdict::Stale)))),
            Capture::Everything,
        ),
        (
            "verdict_no_match",
            recorded(Event::ToolCompleted(value_tool_completed(&Words::verdict(Verdict::NoMatch)))),
            Capture::Everything,
        ),
    ]
}

fn samples_11() -> Vec<(&'static str, Record, Capture)> {
    vec![
        (
            "verdict_unchanged",
            recorded(Event::ToolCompleted(value_tool_completed(&Words::verdict(Verdict::Unchanged)))),
            Capture::Everything,
        ),
        (
            "verdict_busy",
            recorded(Event::ToolCompleted(value_tool_completed(&Words::verdict(Verdict::Busy)))),
            Capture::Everything,
        ),
        (
            "verdict_nul_byte",
            recorded(Event::ToolCompleted(value_tool_completed(&Words::verdict(Verdict::NulByte)))),
            Capture::Everything,
        ),
        (
            "delivered_delivered",
            recorded(Event::ToolCompleted(value_tool_completed(&Words::delivered(Delivered::Delivered)))),
            Capture::Everything,
        ),
        (
            "delivered_nothing",
            recorded(Event::ToolCompleted(value_tool_completed(&Words::delivered(Delivered::Nothing)))),
            Capture::Everything,
        ),
        (
            "delivered_refused",
            recorded(Event::ToolCompleted(value_tool_completed(&Words::delivered(Delivered::Refused)))),
            Capture::Everything,
        ),
        (
            "delivered_failed",
            recorded(Event::ToolCompleted(value_tool_completed(&Words::delivered(Delivered::Failed)))),
            Capture::Everything,
        ),
        (
            "delivered_stale",
            recorded(Event::ToolCompleted(value_tool_completed(&Words::delivered(Delivered::Stale)))),
            Capture::Everything,
        ),
        (
            "failure_class_rate_limited",
            recorded(Event::RunCompleted(value_run_completed(&Words::failure_class(FailureClass::RateLimited)))),
            Capture::Everything,
        ),
        (
            "failure_class_exhausted",
            recorded(Event::RunCompleted(value_run_completed(&Words::failure_class(FailureClass::Exhausted)))),
            Capture::Everything,
        ),
    ]
}

fn samples_12() -> Vec<(&'static str, Record, Capture)> {
    vec![
        (
            "failure_class_unavailable",
            recorded(Event::RunCompleted(value_run_completed(&Words::failure_class(FailureClass::Unavailable)))),
            Capture::Everything,
        ),
        (
            "failure_class_timed_out",
            recorded(Event::RunCompleted(value_run_completed(&Words::failure_class(FailureClass::TimedOut)))),
            Capture::Everything,
        ),
        (
            "failure_class_context_too_long",
            Record {
                t_ms: u64::MAX,
                event: Event::RunCompleted(value_run_completed(&Words::failure_class(FailureClass::ContextTooLong))),
            },
            Capture::Everything,
        ),
        (
            "failure_class_invalid",
            recorded(Event::RunCompleted(value_run_completed(&Words::failure_class(FailureClass::Invalid)))),
            Capture::Everything,
        ),
        (
            "failure_class_unauthorized",
            recorded(Event::RunCompleted(value_run_completed(&Words::failure_class(FailureClass::Unauthorized)))),
            Capture::Everything,
        ),
        (
            "failure_class_limit",
            recorded(Event::RunCompleted(value_run_completed(&Words::failure_class(FailureClass::Limit)))),
            Capture::Everything,
        ),
        (
            "failure_class_protocol",
            recorded(Event::RunCompleted(value_run_completed(&Words::failure_class(FailureClass::Protocol)))),
            Capture::Everything,
        ),
        (
            "failure_class_cancelled",
            recorded(Event::RunCompleted(value_run_completed(&Words::failure_class(FailureClass::Cancelled)))),
            Capture::Everything,
        ),
        (
            "evidence_maybe_sent",
            recorded(Event::RunCompleted(value_run_completed(&Words::evidence(Evidence::MaybeSent)))),
            Capture::Everything,
        ),
        (
            "evidence_response",
            recorded(Event::RunCompleted(value_run_completed(&Words::evidence(Evidence::Response)))),
            Capture::Everything,
        ),
    ]
}

fn samples_13() -> Vec<(&'static str, Record, Capture)> {
    vec![
        (
            "answer_class_budget",
            recorded(Event::RunCompleted(value_run_completed(&Words::answer_class(AnswerClass::Budget)))),
            Capture::Everything,
        ),
        (
            "answer_class_policy",
            recorded(Event::RunCompleted(value_run_completed(&Words::answer_class(AnswerClass::Policy)))),
            Capture::Everything,
        ),
        (
            "answer_class_cancelled",
            recorded(Event::RunCompleted(value_run_completed(&Words::answer_class(AnswerClass::Cancelled)))),
            Capture::Everything,
        ),
        (
            "answer_class_stale",
            recorded(Event::RunCompleted(value_run_completed(&Words::answer_class(AnswerClass::Stale)))),
            Capture::Everything,
        ),
        (
            "answer_class_transcript",
            recorded(Event::RunCompleted(value_run_completed(&Words::answer_class(AnswerClass::Transcript)))),
            Capture::Everything,
        ),
        (
            "answer_class_busy",
            recorded(Event::RunCompleted(value_run_completed(&Words::answer_class(AnswerClass::Busy)))),
            Capture::Everything,
        ),
        (
            "answer_class_invalid",
            recorded(Event::RunCompleted(value_run_completed(&Words::answer_class(AnswerClass::Invalid)))),
            Capture::Everything,
        ),
        ("level_info", recorded(Event::Notice(value_notice(&Words::level(Level::Info)))), Capture::Everything),
        ("level_warning", recorded(Event::Notice(value_notice(&Words::level(Level::Warning)))), Capture::Everything),
        (
            "notice_kind_credential_unauthorized",
            recorded(Event::Notice(value_notice(&Words::notice_kind(NoticeKind::CredentialUnauthorized)))),
            Capture::Everything,
        ),
    ]
}

fn samples_14() -> Vec<(&'static str, Record, Capture)> {
    vec![
        (
            "notice_kind_account_limited",
            recorded(Event::Notice(value_notice(&Words::notice_kind(NoticeKind::AccountLimited)))),
            Capture::Everything,
        ),
        (
            "role_assistant",
            recorded(Event::ResponseStarted(value_response_started(&Words::role(Role::Assistant)))),
            Capture::Everything,
        ),
    ]
}

pub fn samples() -> Vec<(&'static str, Record, Capture)> {
    let mut samples = vec![reasoning_dropped_sample()];
    let mut signal = value_tool_completed(&Words::DEFAULT);
    signal.exit = Some(CommandExit::Signal { signal: u64::MAX });
    samples.push((
        "tool_completed_signal",
        Record { t_ms: 0, event: Event::ToolCompleted(signal) },
        Capture::Everything,
    ));
    let mut zero = value_tool_completed(&Words::DEFAULT);
    zero.omitted_bytes = 0;
    zero.clamped = false;
    samples.push((
        "tool_completed_unclamped",
        Record { t_ms: 0, event: Event::ToolCompleted(zero) },
        Capture::Everything,
    ));
    let mut check = value_check_completed();
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
        let mut check = value_check_completed();
        check.exit = exit;
        check.passed = passed;
        samples.push((name, Record { t_ms: 0, event: Event::CheckCompleted(check) }, Capture::Everything));
    }
    for (name, choice) in [
        ("tool_choice_none", ToolChoice::None),
        ("tool_choice_only", ToolChoice::Only),
        ("tool_choice_unknown", ToolChoice::Unknown("future🌍".as_bytes().into())),
    ] {
        let mut response = value_response_started(&Words::capture(Capture::Everything));
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
    samples.extend(samples_0());
    samples.extend(samples_1());
    samples.extend(samples_2());
    samples.extend(samples_3());
    samples.extend(samples_4());
    samples.extend(samples_5());
    samples.extend(samples_6());
    samples.extend(samples_7());
    samples.extend(samples_8());
    samples.extend(samples_9());
    samples.extend(samples_10());
    samples.extend(samples_11());
    samples.extend(samples_12());
    samples.extend(samples_13());
    samples.extend(samples_14());
    samples.extend(null_samples());
    for (name, capture) in [
        ("session_ended_unread_none", Capture::None),
        ("session_ended_unread_calls", Capture::Calls),
        ("session_ended_unread_everything", Capture::Everything),
    ] {
        let mut end = value_session_ended();
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
            Record { t_ms: 0, event: Event::SessionStarted(value_session_started(&Words::containment(containment))) },
            Capture::Everything,
        ));
    }
    samples
}

fn null_samples() -> Vec<(&'static str, Record, Capture)> {
    let mut response = value_response_completed(&Words::DEFAULT);
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
    let mut started = value_session_started(&Words::DEFAULT);
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
    use smith_events::{
        BudgetStage, BudgetStanding, Clamp, Quantity, Requested, Reserve, Share, ShareBound, SharePart,
    };
    let mut samples = Vec::new();
    samples.push((
        "budget_ordinary",
        Record {
            t_ms: 750,
            event: Event::Budget(BudgetStanding {
                call: None,
                run: 1,
                stage: BudgetStage::Ordinary,
                turns: Quantity { used: 2, limit: 10 },
                spend: Quantity { used: 3, limit: 100 },
                time_ms: Quantity { used: 750, limit: 1999 },
                reserve: Reserve { turns: 1, time_ms: 750, spend: 10 },
            }),
        },
        Capture::Everything,
    ));
    samples.push((
        "budget_told",
        Record {
            t_ms: 750,
            event: Event::Budget(BudgetStanding {
                run: 1,
                stage: BudgetStage::Ordinary,
                turns: Quantity { used: 2, limit: 10 },
                spend: Quantity { used: 3, limit: 100 },
                time_ms: Quantity { used: 750, limit: 1999 },
                reserve: Reserve { turns: 1, time_ms: 750, spend: 10 },
                call: Some(Box::from(b"child-7".as_slice())),
            }),
        },
        Capture::Everything,
    ));
    samples.push((
        "budget_wind_down",
        Record {
            t_ms: 750,
            event: Event::Budget(BudgetStanding {
                call: None,
                run: 1,
                stage: BudgetStage::WindDown,
                turns: Quantity { used: 2, limit: 10 },
                spend: Quantity { used: 3, limit: 100 },
                time_ms: Quantity { used: 750, limit: 1999 },
                reserve: Reserve { turns: 1, time_ms: 750, spend: 10 },
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
    use smith_events::{MessageRead, MessageReceived, MessageRefusal, MessageUnread};
    let mut samples = Vec::new();
    for (name, capture) in [
        ("message_received_none", Capture::None),
        ("message_received_calls", Capture::Calls),
        ("message_received_everything", Capture::Everything),
    ] {
        samples.push((
            name,
            Record {
                t_ms: u64::MAX,
                event: Event::MessageReceived(MessageReceived {
                    run: u64::MAX,
                    name: u64::MAX,
                    bytes: u64::MAX,
                    refused: None,
                    label: Some(Box::from("label\n\"🌍".as_bytes())),
                    text: Some(Box::from("text\n\"🌍".as_bytes())),
                }),
            },
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
                event: Event::MessageReceived(MessageReceived {
                    run: u64::MAX,
                    name: u64::MAX,
                    bytes: u64::MAX,
                    refused: Some(refused),
                    label: None,
                    text: None,
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
        samples.push((
            name,
            Record {
                t_ms: u64::MAX,
                event: Event::MessageRead(MessageRead { run: u64::MAX, name: u64::MAX, turn: u64::MAX }),
            },
            capture,
        ));
    }
    for (name, capture) in [
        ("message_unread_none", Capture::None),
        ("message_unread_calls", Capture::Calls),
        ("message_unread_everything", Capture::Everything),
    ] {
        samples.push((name, recorded(Event::MessageUnread(MessageUnread { run: u64::MAX, name: u64::MAX })), capture));
    }
    let mut completed = value_run_completed(&Words::DEFAULT);
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
        let mut response = value_response_completed(&Words::outcome(Outcome::Failed));
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
        let mut run = value_run_completed(&Words::status(Status::Failed));
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
    let mut aborted = value_session_ended();
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
        let mut run = value_run_completed(&Words::status(Status::Failed));
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
        let mut completed = value_tool_completed(&Words::verdict(verdict));
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
                event: Event::JobEnded(smith_events::JobEnded {
                    run: u64::MAX,
                    conversation: u64::MAX,
                    job: u64::MAX,
                    call: Box::from("sample\n\"🌍".as_bytes()),
                    verdict,
                    exit,
                    unread_bytes: u64::MAX,
                    duration_ms: u64::MAX,
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
        let mut value = value_response_started(&Words::DEFAULT);
        value.model.account = sample.account;
        value.model.backend = sample.backend;
        samples.push((sample.name, Record { t_ms: 0, event: Event::ResponseStarted(value) }, Capture::Everything));
    }
    let mut value = value_run_started(&Words::DEFAULT);
    value.tools.withheld = List::with_capacity(1);
    value.tools.withheld.push(Family::SubAgents).expect("withheld family fits");
    samples.push(("tools_withheld", Record { t_ms: 0, event: Event::RunStarted(value) }, Capture::Everything));
    for sample in [
        FailureSample { name: "failure_class_refused", class: FailureClass::Refused },
        FailureSample { name: "failure_class_unsupported", class: FailureClass::Unsupported },
    ] {
        let mut value = value_response_completed(&Words::DEFAULT);
        let mut failure = value_completion_failure(&Words::DEFAULT);
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
