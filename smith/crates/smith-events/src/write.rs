//! Sized deterministic encoding; capture removes content fields.
//! Contract: protocol/events.md, sections 5.3 and 6.

use crate::records::{
    Agent, AgentExit, AnswerFailure, Block, Budget, BudgetStanding, Capture, CheckCompleted, CheckExit, CheckStarted,
    ChildCompleted, CommandExit, Completion, CompletionFailure, ConversationClosed, ConversationOpened, Count, Cpu,
    EffectiveLimit, Event, Family, Field, Form, JobEnded, LimitValue, Loss, Message, MessageRead, MessageReceived,
    MessageUnread, Mode, Model, Notice, NoticeKind, PeakRss, Prompt, Quantity, Record, Requested, Reserve,
    ResponseCompleted, ResponseStarted, RunCompleted, RunResult, RunStarted, SessionEnded, SessionStarted, Share,
    TextDelta, ToolCompleted, ToolStarted, Tools, Usage, Versions,
};
use crate::{Error, Limits};
use alloc::boxed::Box;
use skein_json::writer::Encoder;
use skein_lib::{List, Writer};

/// Encode one line, or omit live text when capture does not request it.
pub fn write(record: &Record, capture: &Capture, limits: &Limits) -> Result<Option<Box<[u8]>>, Error> {
    let capacity = crate::limits::largest_size(&record.event, capture, limits).ok_or(Error::Limits)?;
    write_with_ceiling(record, capture, limits, capacity)
}

/// Encode within the owner's joint line ceiling, including the newline;
/// independent string, content and list caps still validate the whole record.
pub fn write_with_ceiling(
    record: &Record,
    capture: &Capture,
    limits: &Limits,
    encoded_line: u32,
) -> Result<Option<Box<[u8]>>, Error> {
    let bounded = skein_json::writer::Limits { depth: 16, length: encoded_line.checked_sub(1).ok_or(Error::Limits)? };
    match &record.event {
        Event::TextDelta(_) if *capture != Capture::Everything => return Ok(None),
        Event::Budget(_)
        | Event::SessionStarted(_)
        | Event::SessionEnded(_)
        | Event::RunStarted(_)
        | Event::RunCompleted(_)
        | Event::ConversationOpened(_)
        | Event::ConversationClosed(_)
        | Event::ResponseStarted(_)
        | Event::ResponseCompleted(_)
        | Event::TextDelta(_)
        | Event::ToolStarted(_)
        | Event::JobEnded(_)
        | Event::ToolCompleted(_)
        | Event::CheckStarted(_)
        | Event::CheckCompleted(_)
        | Event::MessageReceived(_)
        | Event::MessageRead(_)
        | Event::MessageUnread(_)
        | Event::Notice(_) => {}
    }
    validate_record(record, limits)?;
    let mut measure = Encoder::measure(&bounded);
    encode(&mut measure, record, capture);
    let length = match measure.measured() {
        Ok(length) => length,
        Err(refusal) => return Err(map_refusal(refusal)),
    };
    let mut encoder = Encoder::write(length, &bounded);
    encode(&mut encoder, record, capture);
    let body = encoder.finish();
    let mut line = Writer::new(body.len().checked_add(1).ok_or(Error::Limits)?);
    line.put(&body).expect("measured line room");
    line.put(b"\n").expect("newline room reserved");
    Ok(Some(line.finish()))
}

fn map_refusal(refusal: skein_json::writer::Refusal) -> Error {
    match refusal {
        skein_json::writer::Refusal::Text => Error::Text,
        skein_json::writer::Refusal::TooLong
        | skein_json::writer::Refusal::TooDeep
        | skein_json::writer::Refusal::Number => Error::Limits,
    }
}

fn encode(json: &mut Encoder, record: &Record, capture: &Capture) {
    json.object_start();
    json.key(b"v");
    json.unsigned(crate::VERSION);
    json.key(b"type");
    match &record.event {
        Event::Budget(_) => json.string(b"budget"),
        Event::SessionStarted(_) => json.string(b"session.started"),
        Event::SessionEnded(_) => json.string(b"session.ended"),
        Event::RunStarted(_) => json.string(b"run.started"),
        Event::RunCompleted(_) => json.string(b"run.completed"),
        Event::ConversationOpened(_) => json.string(b"conversation.opened"),
        Event::ConversationClosed(_) => json.string(b"conversation.closed"),
        Event::ResponseStarted(_) => json.string(b"response.started"),
        Event::ResponseCompleted(_) => json.string(b"response.completed"),
        Event::TextDelta(_) => json.string(b"text.delta"),
        Event::ToolStarted(_) => json.string(b"tool.started"),
        Event::JobEnded(_) => json.string(b"job.ended"),
        Event::ToolCompleted(_) => json.string(b"tool.completed"),
        Event::CheckStarted(_) => json.string(b"check.started"),
        Event::CheckCompleted(_) => json.string(b"check.completed"),
        Event::MessageReceived(_) => json.string(b"message.received"),
        Event::MessageRead(_) => json.string(b"message.read"),
        Event::MessageUnread(_) => json.string(b"message.unread"),
        Event::Notice(_) => json.string(b"notice"),
    }
    json.key(b"t_ms");
    json.unsigned(record.t_ms);
    match &record.event {
        Event::Budget(value) => encode_budget_standing(json, value),
        Event::SessionStarted(value) => encode_session_started(json, value),
        Event::SessionEnded(value) => encode_session_ended(json, value),
        Event::RunStarted(value) => encode_run_started(json, value),
        Event::RunCompleted(value) => encode_run_completed(json, value, capture),
        Event::ConversationOpened(value) => encode_conversation_opened(json, value),
        Event::ConversationClosed(value) => encode_conversation_closed(json, value, capture),
        Event::ResponseStarted(value) => encode_response_started(json, value, capture),
        Event::ResponseCompleted(value) => encode_response_completed(json, value, capture),
        Event::TextDelta(value) => encode_text_delta(json, value),
        Event::ToolStarted(value) => encode_tool_started(json, value, capture),
        Event::JobEnded(value) => encode_job_ended(json, value),
        Event::ToolCompleted(value) => encode_tool_completed(json, value),
        Event::CheckStarted(value) => encode_check_started(json, value),
        Event::CheckCompleted(value) => encode_check_completed(json, value),
        Event::MessageReceived(value) => encode_message_received(json, value, capture),
        Event::MessageRead(value) => encode_message_read(json, value),
        Event::MessageUnread(value) => encode_message_unread(json, value),
        Event::Notice(value) => encode_notice(json, value),
    }
    json.object_end();
}

fn write_agent(json: &mut Encoder, value: &Agent) {
    json.object_start();
    json.key(b"name");
    json.string(&value.name);
    json.key(b"version");
    json.string(&value.version);
    json.object_end();
}

fn write_versions(json: &mut Encoder, value: &Versions) {
    json.object_start();
    json.key(b"events");
    json.unsigned(value.events);
    json.key(b"channel");
    json.unsigned(value.channel);
    json.key(b"charter");
    json.unsigned(value.charter);
    json.key(b"transcript");
    json.unsigned(value.transcript);
    if let Some(version) = value.chat_store {
        json.key(b"chat_store");
        json.unsigned(version);
    }
    json.object_end();
}

fn write_loss(json: &mut Encoder, value: &Loss) {
    json.object_start();
    json.key(b"events");
    json.unsigned(value.events);
    json.key(b"channel");
    put_nullable_unsigned(json, value.channel);
    json.object_end();
}

fn write_cpu(json: &mut Encoder, value: &Cpu) {
    json.object_start();
    json.key(b"user");
    json.unsigned(value.user);
    json.key(b"system");
    json.unsigned(value.system);
    json.key(b"children_user");
    json.unsigned(value.children_user);
    json.key(b"children_system");
    json.unsigned(value.children_system);
    json.object_end();
}

fn write_peak_rss(json: &mut Encoder, value: &PeakRss) {
    json.object_start();
    json.key(b"self");
    json.unsigned(value.self_bytes);
    json.key(b"children");
    json.unsigned(value.children);
    json.object_end();
}

fn write_model(json: &mut Encoder, value: &Model) {
    json.object_start();
    json.key(b"endpoint");
    json.string(&value.endpoint);
    json.key(b"model");
    json.string(&value.model);
    json.key(b"window_tokens");
    json.unsigned(value.window_tokens);
    json.key(b"output_tokens");
    json.unsigned(value.output_tokens);
    json.key(b"effort");
    match &value.effort {
        Some(item) => {
            json.string(item);
        }
        None => json.null(),
    }
    json.key(b"account");
    match value.account {
        Some(account) => json.unsigned(u64::from(account)),
        None => json.null(),
    }
    json.key(b"backend");
    json.string(value.backend.spelled().0);
    json.object_end();
}

fn write_usage(json: &mut Encoder, value: &Usage) {
    json.object_start();
    json.key(b"input_tokens");
    put_nullable_unsigned(json, value.input_tokens);
    json.key(b"cache_read_tokens");
    put_nullable_unsigned(json, value.cache_read_tokens);
    json.key(b"cache_write_tokens");
    put_nullable_unsigned(json, value.cache_write_tokens);
    json.key(b"output_tokens");
    put_nullable_unsigned(json, value.output_tokens);
    json.key(b"reasoning_tokens");
    put_nullable_unsigned(json, value.reasoning_tokens);
    json.object_end();
}

fn write_budget(json: &mut Encoder, value: &Budget) {
    json.object_start();
    json.key(b"turns");
    json.unsigned(value.turns);
    json.key(b"spend");
    json.unsigned(value.spend);
    json.key(b"time_ms");
    json.unsigned(value.time_ms);
    json.key(b"reserve");
    write_reserve(json, &value.reserve);
    json.object_end();
}

fn write_tools(json: &mut Encoder, value: &Tools) {
    json.object_start();
    json.key(b"families");
    write_list_family(json, &value.families);
    json.key(b"withheld");
    write_list_family(json, &value.withheld);
    json.key(b"wait");
    json.boolean(value.wait);
    json.key(b"deliver");
    json.boolean(value.deliver);
    json.key(b"host");
    write_list_text(json, &value.host);
    json.object_end();
}

fn write_completion_failure(json: &mut Encoder, value: &CompletionFailure, capture: &Capture) {
    json.object_start();
    json.key(b"class");
    json.string(value.class.spelled().0);
    json.key(b"evidence");
    json.string(value.evidence.spelled().0);
    if let Some(item) = &value.phase {
        json.key(b"phase");
        json.string(item.spelled().0);
    }
    if let Some(item) = &value.limit {
        json.key(b"limit");
        json.object_start();
        json.key(b"which");
        json.string(&item.which);
        json.key(b"bound");
        json.unsigned(item.bound);
        json.object_end();
    }
    if let Some(item) = value.status {
        json.key(b"status");
        json.unsigned(item);
    }
    put_optional_text(json, b"field", value.field.as_deref());
    put_optional_text(json, b"code", value.code.as_deref());
    put_optional_text(json, b"request_id", value.request_id.as_deref());
    match capture {
        Capture::None | Capture::Unknown(_) => {}
        Capture::Calls | Capture::Everything => {
            put_optional_text(json, b"detail", value.detail.as_deref());
        }
    }
    put_optional_unsigned(json, b"retry_after_ms", value.retry_after_ms);
    json.object_end();
}

fn write_answer_failure(json: &mut Encoder, value: &AnswerFailure, capture: &Capture) {
    json.object_start();
    if let Some(state) = &value.state {
        json.key(b"state");
        json.string(state.spelled().0);
    }
    if let Some(code) = &value.code {
        json.key(b"code");
        json.string(code);
    }
    if let Some(program) = &value.program {
        json.key(b"program");
        json.string(program);
    }
    if let Some(exit) = value.exit {
        json.key(b"exit");
        write_agent_exit(json, exit);
    }
    match capture {
        Capture::None | Capture::Unknown(_) => {}
        Capture::Calls | Capture::Everything => {
            if let Some(detail) = &value.detail {
                json.key(b"detail");
                json.string(detail);
            }
        }
    }
    if let Some(item) = value.reset_ms {
        json.key(b"reset_ms");
        json.unsigned(item);
    }
    if let Some(item) = value.used {
        json.key(b"used");
        json.unsigned(item);
    }
    if let Some(item) = value.bound {
        json.key(b"bound");
        json.unsigned(item);
    }
    json.key(b"class");
    json.string(value.class.spelled().0);
    put_optional_text(json, b"endpoint", value.endpoint.as_deref());
    put_optional_text(json, b"model", value.model.as_deref());
    if let Some(item) = value.attempts {
        json.key(b"attempts");
        json.unsigned(item);
    }
    if let Some(item) = &value.completion {
        json.key(b"completion");
        write_completion_failure(json, item, capture);
    }
    if let Some(item) = &value.account {
        json.key(b"account");
        json.unsigned(u64::from(*item));
    }
    put_optional_text(json, b"which", value.which.as_deref());
    put_optional_text(json, b"reason", value.reason.as_deref());
    if let Some(item) = value.dropped {
        json.key(b"dropped");
        json.boolean(item);
    }
    if let Some(item) = &value.ended {
        json.key(b"ended");
        write_agent_exit(json, *item);
    }
    if let Some(item) = value.turn {
        json.key(b"turn");
        json.unsigned(item);
    }
    json.object_end();
}

fn write_field(json: &mut Encoder, value: &Field) {
    json.object_start();
    json.key(b"name");
    json.string(&value.name);
    json.key(b"text");
    json.string(&value.text);
    json.object_end();
}

fn write_run_result(json: &mut Encoder, value: &RunResult, capture: &Capture) {
    json.object_start();
    json.key(b"form");
    json.string(value.form.spelled().0);
    put_optional_text(json, b"label", value.label.as_deref());
    if *capture == Capture::Everything
        && let Some(item) = &value.text
    {
        json.key(b"text");
        json.string(item);
    }
    if *capture == Capture::Everything
        && let Some(item) = &value.fields
    {
        json.key(b"fields");
        write_list_field(json, item);
    }
    json.object_end();
}

fn write_message(json: &mut Encoder, value: &Message) {
    json.object_start();
    json.key(b"role");
    json.string(value.role.spelled().0);
    json.key(b"blocks");
    write_list_block(json, &value.blocks);
    json.object_end();
}

fn write_prompt(json: &mut Encoder, value: &Prompt) {
    json.object_start();
    put_optional_text(json, b"system", value.system.as_deref());
    if let Some(item) = &value.tools {
        json.key(b"tools");
        write_list_text(json, item);
    }
    json.key(b"messages");
    write_list_message(json, &value.messages);
    json.object_end();
}

fn write_completion(json: &mut Encoder, value: &Completion) {
    json.object_start();
    json.key(b"blocks");
    write_list_block(json, &value.blocks);
    json.object_end();
}

fn write_command_exit(json: &mut Encoder, value: &CommandExit) {
    json.object_start();
    match value {
        CommandExit::Code { code } => {
            json.key(b"code");
            json.signed(*code);
        }
        CommandExit::Signal { signal } => {
            json.key(b"signal");
            json.unsigned(*signal);
        }
    }
    json.object_end();
}

fn write_check_exit(json: &mut Encoder, value: &CheckExit) {
    json.object_start();
    json.key(b"kind");
    match value {
        CheckExit::Code { code } => {
            json.string(b"code");
            json.key(b"value");
            json.signed(*code);
        }
        CheckExit::Signal { signal } => {
            json.string(b"signal");
            json.key(b"value");
            json.unsigned(*signal);
        }
        CheckExit::TimedOut => json.string(b"timed_out"),
        CheckExit::Unstarted => json.string(b"unstarted"),
        CheckExit::Cancelled => json.string(b"cancelled"),
        CheckExit::Unknown(kind) => json.string(kind),
    }
    json.object_end();
}

fn write_block(json: &mut Encoder, value: &Block) {
    json.object_start();
    json.key(b"type");
    match value {
        Block::Unknown { kind } => json.string(kind),
        Block::Text { text } => {
            json.string(b"text");
            json.key(b"text");
            json.string(text);
        }
        Block::Refusal { text } => {
            json.string(b"refusal");
            json.key(b"text");
            json.string(text);
        }
        Block::Call { call, tool, input } => {
            json.string(b"call");
            json.key(b"call");
            json.string(call);
            json.key(b"tool");
            json.string(tool);
            json.key(b"input");
            json.string(input);
        }
        Block::Oversized { call, tool, bytes } => {
            json.string(b"oversized");
            json.key(b"call");
            json.string(call);
            json.key(b"tool");
            json.string(tool);
            json.key(b"bytes");
            json.unsigned(*bytes);
        }
        Block::Cut { call, tool, input } => {
            json.string(b"cut");
            json.key(b"call");
            json.string(call);
            json.key(b"tool");
            json.string(tool);
            json.key(b"input");
            json.string(input);
        }
        Block::Result { call, error, text } => {
            json.string(b"result");
            json.key(b"call");
            json.string(call);
            json.key(b"error");
            json.boolean(*error);
            json.key(b"text");
            json.string(text);
        }
        Block::Opaque { bytes } => {
            json.string(b"opaque");
            json.key(b"bytes");
            json.unsigned(*bytes);
        }
    }
    json.object_end();
}

fn write_list_block(json: &mut Encoder, values: &List<Block>) {
    json.array_start();
    for value in values {
        write_block(json, value);
    }
    json.array_end();
}

fn write_list_count(json: &mut Encoder, values: &List<Count>) {
    json.object_start();
    for value in values {
        json.key(&value.record);
        json.unsigned(value.count);
    }
    json.object_end();
}

fn write_list_family(json: &mut Encoder, values: &List<Family>) {
    json.array_start();
    for value in values {
        json.string(value.spelled().0);
    }
    json.array_end();
}

fn write_list_field(json: &mut Encoder, values: &List<Field>) {
    json.array_start();
    for value in values {
        write_field(json, value);
    }
    json.array_end();
}

fn write_list_form(json: &mut Encoder, values: &List<Form>) {
    json.array_start();
    for value in values {
        json.string(value.spelled().0);
    }
    json.array_end();
}

fn write_list_message(json: &mut Encoder, values: &List<Message>) {
    json.array_start();
    for value in values {
        write_message(json, value);
    }
    json.array_end();
}

fn write_list_text(json: &mut Encoder, values: &List<Box<[u8]>>) {
    json.array_start();
    for value in values {
        json.string(value);
    }
    json.array_end();
}

fn check_text(text: &[u8], capacity: u32) -> Result<(), Error> {
    if text.len() > usize::try_from(capacity).expect("u32 fits") {
        return Err(Error::Limits);
    }
    Ok(())
}

fn validate_record(record: &Record, limits: &Limits) -> Result<(), Error> {
    match &record.event {
        Event::Budget(value) => {
            validate_word(value.stage.spelled(), limits)?;
            if let Some(call) = &value.call {
                check_text(call, limits.string)?;
            }
            Ok(())
        }
        Event::SessionStarted(value) => validate_session_started(value, limits),
        Event::SessionEnded(value) => validate_session_ended(value, limits),
        Event::RunStarted(value) => validate_run_started(value, limits),
        Event::RunCompleted(value) => validate_run_completed(value, limits),
        Event::ConversationOpened(value) => validate_conversation_opened(value, limits),
        Event::ConversationClosed(value) => validate_conversation_closed(value, limits),
        Event::ResponseStarted(value) => validate_response_started(value, limits),
        Event::ResponseCompleted(value) => validate_response_completed(value, limits),
        Event::TextDelta(value) => validate_text_delta(value, limits),
        Event::ToolStarted(value) => validate_tool_started(value, limits),
        Event::JobEnded(value) => {
            check_text(&value.call, limits.string)?;
            validate_word(value.verdict.spelled(), limits)
        }
        Event::ToolCompleted(value) => validate_tool_completed(value, limits),
        Event::CheckCompleted(value) => validate_check_exit(&value.exit, limits),
        Event::CheckStarted(_) | Event::MessageRead(_) | Event::MessageUnread(_) => Ok(()),
        Event::MessageReceived(value) => validate_message_received(value, limits),
        Event::Notice(value) => validate_notice(value, limits),
    }
}

fn validate_check_exit(value: &CheckExit, limits: &Limits) -> Result<(), Error> {
    match value {
        CheckExit::Code { .. }
        | CheckExit::Signal { .. }
        | CheckExit::TimedOut
        | CheckExit::Unstarted
        | CheckExit::Cancelled => Ok(()),
        CheckExit::Unknown(kind) => check_text(kind, limits.string),
    }
}

fn validate_agent(value: &Agent, limits: &Limits) -> Result<(), Error> {
    check_text(&value.name, limits.string)?;
    check_text(&value.version, limits.string)?;
    Ok(())
}

fn validate_count(value: &Count, limits: &Limits) -> Result<(), Error> {
    check_text(&value.record, limits.string)?;

    Ok(())
}

fn validate_model(value: &Model, limits: &Limits) -> Result<(), Error> {
    check_text(&value.endpoint, limits.string)?;
    check_text(&value.model, limits.string)?;
    validate_word(value.backend.spelled(), limits)?;

    if let Some(item) = &value.effort {
        check_text(item, limits.string)?;
    }
    Ok(())
}

fn validate_tools(value: &Tools, limits: &Limits) -> Result<(), Error> {
    validate_list_family(&value.families, limits)?;
    validate_list_family(&value.withheld, limits)?;

    validate_list_text(&value.host, limits)?;
    Ok(())
}

fn validate_completion_failure(value: &CompletionFailure, limits: &Limits) -> Result<(), Error> {
    validate_word(value.class.spelled(), limits)?;
    for item in [&value.field, &value.code, &value.request_id].into_iter().flatten() {
        check_text(item, limits.string)?;
    }
    validate_word(value.evidence.spelled(), limits)?;
    if let Some(phase) = &value.phase {
        validate_word(phase.spelled(), limits)?;
    }
    if let Some(limit) = &value.limit {
        check_text(&limit.which, limits.string)?;
    }
    if let Some(detail) = &value.detail {
        check_text(detail, limits.content)?;
    }
    Ok(())
}

fn validate_answer_failure(value: &AnswerFailure, limits: &Limits) -> Result<(), Error> {
    validate_word(value.class.spelled(), limits)?;
    if let Some(endpoint) = &value.endpoint {
        check_text(endpoint, limits.string)?;
    }
    if let Some(model) = &value.model {
        check_text(model, limits.string)?;
    }
    if let Some(item) = &value.completion {
        validate_completion_failure(item, limits)?;
    }
    if let Some(item) = &value.which {
        check_text(item, limits.string)?;
    }
    if let Some(item) = &value.reason {
        check_text(item, limits.string)?;
    }
    if let Some(state) = &value.state {
        validate_word(state.spelled(), limits)?;
    }
    if let Some(code) = &value.code {
        check_text(code, limits.string)?;
    }
    if let Some(program) = &value.program {
        check_text(program, limits.string)?;
    }
    if let Some(detail) = &value.detail {
        check_text(detail, limits.content)?;
    }
    Ok(())
}

fn validate_field(value: &Field, limits: &Limits) -> Result<(), Error> {
    check_text(&value.name, limits.string)?;
    check_text(&value.text, limits.content)?;
    Ok(())
}

fn validate_run_result(value: &RunResult, limits: &Limits) -> Result<(), Error> {
    validate_word(value.form.spelled(), limits)?;
    if let Some(item) = &value.label {
        check_text(item, limits.string)?;
    }
    if let Some(item) = &value.text {
        check_text(item, limits.content)?;
    }
    if let Some(item) = &value.fields {
        validate_list_field(item, limits)?;
    }
    Ok(())
}

fn validate_message(value: &Message, limits: &Limits) -> Result<(), Error> {
    validate_word(value.role.spelled(), limits)?;
    validate_list_block(&value.blocks, limits)?;
    Ok(())
}

fn validate_prompt(value: &Prompt, limits: &Limits) -> Result<(), Error> {
    if let Some(item) = &value.system {
        check_text(item, limits.content)?;
    }
    if let Some(item) = &value.tools {
        validate_list_text(item, limits)?;
    }
    validate_list_message(&value.messages, limits)?;
    Ok(())
}

fn validate_completion(value: &Completion, limits: &Limits) -> Result<(), Error> {
    validate_list_block(&value.blocks, limits)?;
    Ok(())
}

fn validate_session_started(value: &SessionStarted, limits: &Limits) -> Result<(), Error> {
    validate_agent(&value.agent, limits)?;
    validate_word(value.mode.spelled(), limits)?;
    match &value.mode {
        Mode::Agent if value.chat.is_some() => return Err(Error::Shape),
        Mode::Agent | Mode::Exec | Mode::Chat | Mode::Unknown(_) => {}
    }
    if let Some(chat) = &value.chat {
        check_text(chat, limits.string)?;
    }
    validate_word(value.containment.spelled(), limits)?;

    check_text(&value.profile, limits.string)?;
    validate_word(value.capture.spelled(), limits)?;
    validate_word(value.delivery.spelled(), limits)?;
    validate_effective_limits(&value.limits, limits)?;
    Ok(())
}

fn validate_effective_limits(values: &List<EffectiveLimit>, limits: &Limits) -> Result<(), Error> {
    if values.len() > limits.items {
        return Err(Error::Limits);
    }
    for (index, value) in values.as_slice().iter().enumerate() {
        if value.name.is_empty() {
            return Err(Error::Shape);
        }
        check_text(&value.name, limits.string)?;
        for previous in values.as_slice().get(..index).ok_or(Error::Shape)? {
            if previous.name == value.name {
                return Err(Error::Shape);
            }
        }
        match value.value {
            LimitValue::Integer(_) => {}
            LimitValue::Fraction { denominator, .. } => {
                if denominator == 0 {
                    return Err(Error::Shape);
                }
            }
        }
    }
    Ok(())
}

fn validate_session_ended(value: &SessionEnded, limits: &Limits) -> Result<(), Error> {
    validate_list_count(&value.emitted, limits)?;
    Ok(())
}

fn validate_run_started(value: &RunStarted, limits: &Limits) -> Result<(), Error> {
    validate_model(&value.main, limits)?;
    validate_tools(&value.tools, limits)?;
    validate_list_form(&value.contract, limits)?;
    Ok(())
}

fn validate_run_completed(value: &RunCompleted, limits: &Limits) -> Result<(), Error> {
    validate_word(value.status.spelled(), limits)?;
    if let Some(item) = &value.failure {
        validate_answer_failure(item, limits)?;
    }
    if let Some(item) = &value.result {
        validate_run_result(item, limits)?;
    }

    Ok(())
}

fn validate_conversation_opened(value: &ConversationOpened, limits: &Limits) -> Result<(), Error> {
    validate_word(value.kind.spelled(), limits)?;
    if let Some(item) = &value.call {
        check_text(item, limits.string)?;
    }
    validate_model(&value.model, limits)?;
    if value.clamped.len() > limits.items {
        return Err(Error::Limits);
    }
    for clamp in &value.clamped {
        validate_word(clamp.part.spelled(), limits)?;
        validate_word(clamp.by.spelled(), limits)?;
    }
    Ok(())
}

fn validate_conversation_closed(value: &ConversationClosed, limits: &Limits) -> Result<(), Error> {
    validate_word(value.end.spelled(), limits)?;
    if let Some(item) = &value.which {
        check_text(item, limits.string)?;
    }
    if let Some(item) = &value.failure {
        validate_completion_failure(item, limits)?;
    }

    Ok(())
}

fn validate_response_started(value: &ResponseStarted, limits: &Limits) -> Result<(), Error> {
    validate_model(&value.model, limits)?;
    validate_word(value.tool_choice.spelled(), limits)?;

    if let Some(item) = &value.prompt {
        validate_prompt(item, limits)?;
    }
    Ok(())
}

fn validate_response_completed(value: &ResponseCompleted, limits: &Limits) -> Result<(), Error> {
    validate_word(value.outcome.spelled(), limits)?;
    if let Some(item) = &value.stop {
        validate_word(item.spelled(), limits)?;
    }

    if let Some(item) = &value.failure {
        validate_completion_failure(item, limits)?;
    }
    if let Some(item) = &value.completion {
        validate_completion(item, limits)?;
    }
    Ok(())
}

fn validate_text_delta(value: &TextDelta, limits: &Limits) -> Result<(), Error> {
    check_text(&value.text, limits.content)?;
    Ok(())
}

fn validate_tool_started(value: &ToolStarted, limits: &Limits) -> Result<(), Error> {
    check_text(&value.call, limits.string)?;
    check_text(&value.tool, limits.string)?;
    validate_word(value.source.spelled(), limits)?;
    validate_word(value.effect.spelled(), limits)?;

    if let Some(item) = &value.input {
        check_text(item, limits.content)?;
    }
    Ok(())
}

fn validate_tool_completed(value: &ToolCompleted, limits: &Limits) -> Result<(), Error> {
    check_text(&value.call, limits.string)?;
    check_text(&value.tool, limits.string)?;
    validate_word(value.verdict.spelled(), limits)?;

    if let Some(item) = &value.child {
        validate_word(item.stop.spelled(), limits)?;
    }
    if let Some(item) = &value.delivery {
        validate_word(item.spelled(), limits)?;
    }
    Ok(())
}

fn validate_message_received(value: &MessageReceived, limits: &Limits) -> Result<(), Error> {
    if let Some(refusal) = &value.refused {
        validate_word(refusal.spelled(), limits)?;
    }
    if let Some(label) = &value.label {
        check_text(label, limits.string)?;
    }
    if let Some(text) = &value.text {
        check_text(text, limits.content)?;
    }
    Ok(())
}

fn validate_notice(value: &Notice, limits: &Limits) -> Result<(), Error> {
    if let Some(code) = &value.code {
        check_text(code, limits.string)?;
    }
    validate_word(value.level.spelled(), limits)?;
    validate_word(value.kind.spelled(), limits)?;
    if let Some(forced) = &value.forced {
        validate_word(forced.spelled(), limits)?;
    }
    match value.kind {
        NoticeKind::ForcedExit if value.forced.is_none() => Err(Error::Shape),
        NoticeKind::AccountRefused if value.code.is_none() => Err(Error::Shape),
        NoticeKind::ReasoningDropped if value.bytes.is_none() => Err(Error::Shape),
        NoticeKind::CredentialUnauthorized | NoticeKind::AccountLimited if value.account.is_none() => Err(Error::Shape),
        NoticeKind::ReasoningDropped
        | NoticeKind::CredentialUnauthorized
        | NoticeKind::AccountLimited
        | NoticeKind::AccountRefused
        | NoticeKind::TeardownAborted
        | NoticeKind::ForcedExit
        | NoticeKind::Unknown(_) => Ok(()),
    }
}

fn validate_block(value: &Block, limits: &Limits) -> Result<(), Error> {
    match value {
        Block::Unknown { kind } => check_text(kind, limits.string),
        Block::Text { text } | Block::Refusal { text } => {
            check_text(text, limits.content)?;
            Ok(())
        }
        Block::Oversized { call, tool, bytes: _ } => {
            check_text(call, limits.string)?;
            check_text(tool, limits.string)?;

            Ok(())
        }
        Block::Call { call, tool, input } | Block::Cut { call, tool, input } => {
            check_text(call, limits.string)?;
            check_text(tool, limits.string)?;
            check_text(input, limits.content)?;
            Ok(())
        }
        Block::Result { call, error: _, text } => {
            check_text(call, limits.string)?;

            check_text(text, limits.content)?;
            Ok(())
        }
        Block::Opaque { bytes: _ } => Ok(()),
    }
}

fn validate_list_block(values: &List<Block>, limits: &Limits) -> Result<(), Error> {
    if values.len() > limits.items {
        return Err(Error::Limits);
    }
    for value in values {
        validate_block(value, limits)?;
    }
    Ok(())
}

fn validate_list_count(values: &List<Count>, limits: &Limits) -> Result<(), Error> {
    if values.len() > limits.items {
        return Err(Error::Limits);
    }
    for value in values {
        validate_count(value, limits)?;
    }
    for first in 0..values.len() {
        let first_value = values.get(first).expect("bounded counter list");
        for second in first.saturating_add(1)..values.len() {
            let second_value = values.get(second).expect("bounded counter list");
            if first_value.record == second_value.record {
                return Err(Error::Shape);
            }
        }
    }
    Ok(())
}

fn validate_list_family(values: &List<Family>, limits: &Limits) -> Result<(), Error> {
    if values.len() > limits.items {
        return Err(Error::Limits);
    }
    for value in values {
        validate_word(value.spelled(), limits)?;
    }
    Ok(())
}

fn validate_list_field(values: &List<Field>, limits: &Limits) -> Result<(), Error> {
    if values.len() > limits.items {
        return Err(Error::Limits);
    }
    for value in values {
        validate_field(value, limits)?;
    }
    Ok(())
}

fn validate_list_form(values: &List<Form>, limits: &Limits) -> Result<(), Error> {
    if values.len() > limits.items {
        return Err(Error::Limits);
    }
    for value in values {
        validate_word(value.spelled(), limits)?;
    }
    Ok(())
}

fn validate_list_message(values: &List<Message>, limits: &Limits) -> Result<(), Error> {
    if values.len() > limits.items {
        return Err(Error::Limits);
    }
    for value in values {
        validate_message(value, limits)?;
    }
    Ok(())
}

fn validate_list_text(values: &List<Box<[u8]>>, limits: &Limits) -> Result<(), Error> {
    if values.len() > limits.items {
        return Err(Error::Limits);
    }
    for value in values {
        check_text(value, limits.string)?;
    }
    Ok(())
}

fn encode_session_started(json: &mut Encoder, value: &SessionStarted) {
    json.key(b"wall_ms");
    json.unsigned(value.wall_ms);
    json.key(b"agent");
    write_agent(json, &value.agent);
    json.key(b"mode");
    json.string(value.mode.spelled().0);
    if let Some(chat) = &value.chat {
        json.key(b"chat");
        json.string(chat);
    }
    json.key(b"containment");
    json.string(value.containment.spelled().0);
    json.key(b"pid");
    json.unsigned(value.pid);
    json.key(b"profile");
    json.string(&value.profile);
    json.key(b"capture");
    json.string(value.capture.spelled().0);
    json.key(b"delivery");
    json.string(value.delivery.spelled().0);
    json.key(b"versions");
    write_versions(json, &value.versions);
    json.key(b"limits");
    json.object_start();
    for limit in &value.limits {
        json.key(&limit.name);
        match limit.value {
            LimitValue::Integer(value) => json.unsigned(value),
            LimitValue::Fraction { numerator, denominator } => {
                json.object_start();
                json.key(b"numerator");
                json.unsigned(numerator);
                json.key(b"denominator");
                json.unsigned(denominator);
                json.object_end();
            }
        }
    }
    json.object_end();
}

fn encode_session_ended(json: &mut Encoder, value: &SessionEnded) {
    json.key(b"exit");
    json.signed(value.exit);
    json.key(b"answered");
    json.boolean(value.answered);
    json.key(b"aborted");
    json.boolean(value.aborted);
    json.key(b"teardown_ms");
    put_nullable_unsigned(json, value.teardown_ms);
    json.key(b"emitted");
    write_list_count(json, &value.emitted);
    json.key(b"loss");
    write_loss(json, &value.loss);
    json.key(b"cpu_ms");
    match &value.cpu_ms {
        Some(cpu) => write_cpu(json, cpu),
        None => json.null(),
    }
    json.key(b"peak_rss_bytes");
    match &value.peak_rss_bytes {
        Some(peak_rss) => write_peak_rss(json, peak_rss),
        None => json.null(),
    }
}

fn encode_run_started(json: &mut Encoder, value: &RunStarted) {
    json.key(b"run");
    json.unsigned(value.run);
    json.key(b"resumed");
    json.boolean(value.resumed);
    json.key(b"messages");
    json.unsigned(value.messages);
    json.key(b"main");
    write_model(json, &value.main);
    json.key(b"budget");
    write_budget(json, &value.budget);
    json.key(b"tools");
    write_tools(json, &value.tools);
    json.key(b"contract");
    write_list_form(json, &value.contract);
}

fn encode_run_completed(json: &mut Encoder, value: &RunCompleted, capture: &Capture) {
    json.key(b"wind_down");
    json.boolean(value.wind_down);
    json.key(b"run");
    json.unsigned(value.run);
    json.key(b"status");
    json.string(value.status.spelled().0);
    if let Some(item) = &value.failure {
        json.key(b"failure");
        write_answer_failure(json, item, capture);
    }
    if let Some(item) = &value.result {
        json.key(b"result");
        write_run_result(json, item, capture);
    }
    json.key(b"turns");
    json.unsigned(value.turns);
    json.key(b"spent");
    json.unsigned(value.spent);
    json.key(b"usage");
    write_usage(json, &value.usage);
    json.key(b"read");
    match value.read {
        Some(name) => json.unsigned(name),
        None => json.null(),
    }
    json.key(b"unread");
    json.unsigned(value.unread);
    json.key(b"duration_ms");
    json.unsigned(value.duration_ms);
}

fn encode_conversation_opened(json: &mut Encoder, value: &ConversationOpened) {
    json.key(b"run");
    json.unsigned(value.run);
    json.key(b"conversation");
    json.unsigned(value.conversation);
    json.key(b"kind");
    json.string(value.kind.spelled().0);
    put_optional_unsigned(json, b"parent", value.parent);
    put_optional_text(json, b"call", value.call.as_deref());
    json.key(b"thread");
    json.unsigned(value.thread);
    json.key(b"model");
    write_model(json, &value.model);
    if let Some(requested) = &value.requested {
        json.key(b"requested");
        write_requested(json, requested);
    }
    if let Some(share) = &value.share {
        json.key(b"share");
        write_share(json, share);
    }
    if !value.clamped.is_empty() {
        json.key(b"clamped");
        json.array_start();
        for clamp in &value.clamped {
            json.object_start();
            json.key(b"part");
            json.string(clamp.part.spelled().0);
            json.key(b"by");
            json.string(clamp.by.spelled().0);
            json.object_end();
        }
        json.array_end();
    }
}

fn encode_conversation_closed(json: &mut Encoder, value: &ConversationClosed, capture: &Capture) {
    json.key(b"run");
    json.unsigned(value.run);
    json.key(b"conversation");
    json.unsigned(value.conversation);
    json.key(b"end");
    json.string(value.end.spelled().0);
    put_optional_text(json, b"which", value.which.as_deref());
    if let Some(item) = &value.failure {
        json.key(b"failure");
        write_completion_failure(json, item, capture);
    }
    json.key(b"turns");
    json.unsigned(value.turns);
    json.key(b"usage");
    write_usage(json, &value.usage);
    json.key(b"spent");
    json.unsigned(value.spent);
    json.key(b"duration_ms");
    json.unsigned(value.duration_ms);
}

fn encode_response_started(json: &mut Encoder, value: &ResponseStarted, capture: &Capture) {
    json.key(b"run");
    json.unsigned(value.run);
    json.key(b"conversation");
    json.unsigned(value.conversation);
    json.key(b"response");
    json.unsigned(value.response);
    json.key(b"turn");
    json.unsigned(value.turn);
    json.key(b"attempt");
    json.unsigned(value.attempt);
    json.key(b"window");
    json.unsigned(value.window);
    json.key(b"model");
    write_model(json, &value.model);
    json.key(b"messages");
    json.unsigned(value.messages);
    json.key(b"output_tokens");
    json.unsigned(value.output_tokens);
    json.key(b"tool_choice");
    json.string(value.tool_choice.spelled().0);
    if *capture == Capture::Everything
        && let Some(item) = &value.prompt
    {
        json.key(b"prompt");
        write_prompt(json, item);
    }
}

fn encode_response_completed(json: &mut Encoder, value: &ResponseCompleted, capture: &Capture) {
    json.key(b"run");
    json.unsigned(value.run);
    json.key(b"conversation");
    json.unsigned(value.conversation);
    json.key(b"response");
    json.unsigned(value.response);
    json.key(b"outcome");
    json.string(value.outcome.spelled().0);
    if let Some(item) = &value.stop {
        json.key(b"stop");
        json.string(item.spelled().0);
    }
    json.key(b"blocks");
    json.unsigned(value.blocks);
    json.key(b"calls");
    json.unsigned(value.calls);
    json.key(b"invalid");
    json.unsigned(value.invalid);
    json.key(b"oversized");
    json.unsigned(value.oversized);
    json.key(b"cut");
    json.unsigned(value.cut);
    json.key(b"usage");
    write_usage(json, &value.usage);
    json.key(b"spent");
    json.unsigned(value.spent);
    json.key(b"request_bytes");
    put_nullable_unsigned(json, value.request_bytes);
    json.key(b"first_byte_ms");
    put_nullable_unsigned(json, value.first_byte_ms);
    json.key(b"largest_gap_ms");
    put_nullable_unsigned(json, value.largest_gap_ms);
    json.key(b"duration_ms");
    json.unsigned(value.duration_ms);
    if let Some(item) = &value.failure {
        json.key(b"failure");
        write_completion_failure(json, item, capture);
    }
    put_optional_unsigned(json, b"retry_ms", value.retry_ms);
    if *capture == Capture::Everything
        && let Some(item) = &value.completion
    {
        json.key(b"completion");
        write_completion(json, item);
    }
}

fn encode_text_delta(json: &mut Encoder, value: &TextDelta) {
    json.key(b"run");
    json.unsigned(value.run);
    json.key(b"conversation");
    json.unsigned(value.conversation);
    json.key(b"response");
    json.unsigned(value.response);
    json.key(b"block");
    json.unsigned(value.block);
    json.key(b"text");
    json.string(&value.text);
}

fn encode_tool_started(json: &mut Encoder, value: &ToolStarted, capture: &Capture) {
    json.key(b"run");
    json.unsigned(value.run);
    json.key(b"conversation");
    json.unsigned(value.conversation);
    json.key(b"call");
    json.string(&value.call);
    json.key(b"tool");
    json.string(&value.tool);
    json.key(b"source");
    json.string(value.source.spelled().0);
    json.key(b"effect");
    json.string(value.effect.spelled().0);
    json.key(b"deadline_ms");
    put_nullable_unsigned(json, value.deadline_ms);
    json.key(b"input_bytes");
    json.unsigned(value.input_bytes);
    match capture {
        Capture::None | Capture::Unknown(_) => {}
        Capture::Calls | Capture::Everything => {
            put_optional_text(json, b"input", value.input.as_deref());
        }
    }
}

fn write_child_completed(json: &mut Encoder, value: &ChildCompleted) {
    json.object_start();
    json.key(b"conversation");
    json.unsigned(value.conversation);
    json.key(b"stop");
    json.string(value.stop.spelled().0);
    json.key(b"turns");
    json.unsigned(value.turns);
    json.key(b"time_ms");
    json.unsigned(value.time_ms);
    json.key(b"spent");
    json.unsigned(value.spent);
    json.key(b"usage");
    write_usage(json, &value.usage);
    json.object_end();
}

fn encode_tool_completed(json: &mut Encoder, value: &ToolCompleted) {
    json.key(b"run");
    json.unsigned(value.run);
    json.key(b"conversation");
    json.unsigned(value.conversation);
    json.key(b"call");
    json.string(&value.call);
    json.key(b"tool");
    json.string(&value.tool);
    json.key(b"verdict");
    json.string(value.verdict.spelled().0);
    if let Some(item) = &value.exit {
        json.key(b"exit");
        write_command_exit(json, item);
    }
    if let Some(job) = value.job {
        json.key(b"job");
        json.unsigned(job);
    }
    json.key(b"bytes");
    json.unsigned(value.bytes);
    json.key(b"omitted_bytes");
    json.unsigned(value.omitted_bytes);
    json.key(b"clamped");
    json.boolean(value.clamped);
    json.key(b"duration_ms");
    json.unsigned(value.duration_ms);
    if let Some(item) = &value.child {
        json.key(b"child");
        write_child_completed(json, item);
    }
    if let Some(item) = &value.delivery {
        json.key(b"delivery");
        json.string(item.spelled().0);
    }
}

fn encode_job_ended(json: &mut Encoder, value: &JobEnded) {
    json.key(b"run");
    json.unsigned(value.run);
    json.key(b"conversation");
    json.unsigned(value.conversation);
    json.key(b"job");
    json.unsigned(value.job);
    json.key(b"call");
    json.string(&value.call);
    json.key(b"verdict");
    json.string(value.verdict.spelled().0);
    if let Some(exit) = &value.exit {
        json.key(b"exit");
        write_command_exit(json, exit);
    }
    json.key(b"unread_bytes");
    json.unsigned(value.unread_bytes);
    json.key(b"duration_ms");
    json.unsigned(value.duration_ms);
}

fn encode_check_started(json: &mut Encoder, value: &CheckStarted) {
    json.key(b"run");
    json.unsigned(value.run);
    json.key(b"deadline_ms");
    put_nullable_unsigned(json, value.deadline_ms);
}

fn encode_check_completed(json: &mut Encoder, value: &CheckCompleted) {
    json.key(b"run");
    json.unsigned(value.run);
    json.key(b"exit");
    write_check_exit(json, &value.exit);
    json.key(b"passed");
    json.boolean(value.passed);
    json.key(b"duration_ms");
    json.unsigned(value.duration_ms);
}

fn encode_message_received(json: &mut Encoder, value: &MessageReceived, capture: &Capture) {
    json.key(b"run");
    json.unsigned(value.run);
    json.key(b"name");
    json.unsigned(value.name);
    json.key(b"bytes");
    json.unsigned(value.bytes);
    if let Some(refusal) = &value.refused {
        json.key(b"refused");
        json.string(refusal.spelled().0);
    }
    if *capture == Capture::Everything {
        if let Some(label) = &value.label {
            json.key(b"label");
            json.string(label);
        }
        if let Some(text) = &value.text {
            json.key(b"text");
            json.string(text);
        }
    }
}

fn encode_message_read(json: &mut Encoder, value: &MessageRead) {
    json.key(b"run");
    json.unsigned(value.run);
    json.key(b"name");
    json.unsigned(value.name);
    json.key(b"turn");
    json.unsigned(value.turn);
}

fn encode_message_unread(json: &mut Encoder, value: &MessageUnread) {
    json.key(b"run");
    json.unsigned(value.run);
    json.key(b"name");
    json.unsigned(value.name);
}

fn write_agent_exit(json: &mut Encoder, exit: AgentExit) {
    json.object_start();
    match exit {
        AgentExit::Code(code) => {
            json.key(b"code");
            json.signed(code);
        }
        AgentExit::Signal(signal) => {
            json.key(b"signal");
            json.unsigned(signal);
        }
    }
    json.object_end();
}

fn encode_notice(json: &mut Encoder, value: &Notice) {
    json.key(b"level");
    json.string(value.level.spelled().0);
    json.key(b"kind");
    json.string(value.kind.spelled().0);
    match value.exit {
        Some(exit) => {
            json.key(b"exit");
            write_agent_exit(json, exit);
        }
        None => match value.kind {
            NoticeKind::ForcedExit => {
                json.key(b"exit");
                json.null();
            }
            NoticeKind::ReasoningDropped
            | NoticeKind::CredentialUnauthorized
            | NoticeKind::AccountLimited
            | NoticeKind::AccountRefused
            | NoticeKind::TeardownAborted
            | NoticeKind::Unknown(_) => {}
        },
    }
    if let Some(forced) = &value.forced {
        json.key(b"forced");
        json.string(forced.spelled().0);
    }
    put_optional_unsigned(json, b"run", value.run);
    if let Some(account) = value.account {
        json.key(b"account");
        json.unsigned(u64::from(account));
    }
    if let Some(bytes) = value.bytes {
        json.key(b"bytes");
        json.unsigned(bytes);
    }
    put_optional_unsigned(json, b"reset_ms", value.reset_ms);
    if let Some(code) = &value.code {
        json.key(b"code");
        json.string(code);
    }
}

fn write_requested(json: &mut Encoder, value: &Requested) {
    json.object_start();
    if let Some(turns) = value.turns {
        json.key(b"turns");
        json.unsigned(turns);
    }
    if let Some(seconds) = value.seconds {
        json.key(b"seconds");
        json.unsigned(seconds);
    }
    json.object_end();
}

fn write_share(json: &mut Encoder, value: &Share) {
    json.object_start();
    json.key(b"turns");
    json.unsigned(value.turns);
    json.key(b"time_ms");
    json.unsigned(value.time_ms);
    json.key(b"spend");
    json.unsigned(value.spend);
    json.object_end();
}

fn write_quantity(json: &mut Encoder, value: &Quantity) {
    json.object_start();
    json.key(b"used");
    json.unsigned(value.used);
    json.key(b"limit");
    json.unsigned(value.limit);
    json.object_end();
}

fn write_reserve(json: &mut Encoder, value: &Reserve) {
    json.object_start();
    json.key(b"turns");
    json.unsigned(value.turns);
    json.key(b"time_ms");
    json.unsigned(value.time_ms);
    json.key(b"spend");
    json.unsigned(value.spend);
    json.object_end();
}

fn encode_budget_standing(json: &mut Encoder, value: &BudgetStanding) {
    json.key(b"run");
    json.unsigned(value.run);
    json.key(b"stage");
    json.string(value.stage.spelled().0);
    json.key(b"turns");
    write_quantity(json, &value.turns);
    json.key(b"spend");
    write_quantity(json, &value.spend);
    json.key(b"time_ms");
    write_quantity(json, &value.time_ms);
    json.key(b"reserve");
    write_reserve(json, &value.reserve);
    if let Some(call) = &value.call {
        json.key(b"call");
        json.string(call);
    }
}

fn put_optional_text(json: &mut Encoder, name: &[u8], value: Option<&[u8]>) {
    if let Some(item) = value {
        json.key(name);
        json.string(item);
    }
}

fn put_optional_unsigned(json: &mut Encoder, name: &[u8], value: Option<u64>) {
    if let Some(item) = value {
        json.key(name);
        json.unsigned(item);
    }
}

fn put_nullable_unsigned(json: &mut Encoder, value: Option<u64>) {
    match value {
        Some(item) => json.unsigned(item),
        None => json.null(),
    }
}

fn validate_word((text, forward): (&[u8], bool), limits: &Limits) -> Result<(), Error> {
    if forward { check_text(text, limits.string) } else { Ok(()) }
}
