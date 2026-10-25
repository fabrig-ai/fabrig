//! Provisional writing, bounded per live run (`client/domain/contract.md`, 7.3;
//! `QUESTIONS.md`, item 41). Retention starts with a report and ends with the run,
//! independently of its followers. Opening resumes a matching retained position
//! or starts at the current retained beginning.
//! Only the current turn's suffix and call remain in memory. Each watch keeps a
//! turn-qualified acknowledged offset, and one separate stream in flight.
//! Stale turns are ignored; a gap replaces the suffix and marks the turn cut.
//! An acknowledged earlier turn never advances a later turn's offset.
//! Text reports and made calls are bounded by `text_bytes` at ingress;
//! input appends and trims its oldest bytes within the same call allowance.
//! a gap or trimming the retained suffix marks the current turn cut.
//!
//! ```text
//! stream state  event                 next       emits
//! Idle          new text/call         Sending    Stream from its own offset
//! Sending       new text/call         Sending    none (run retains bounded suffix)
//! Sending       StreamDelivered       Idle       none, if caught up
//! Sending       StreamDelivered       Sending    Stream of current suffix
//! Sending       close                 Closing    none
//! Closing       StreamDelivered       Closed     Ended, once document also ends
//! any           RunEnded              same       none (drop provisional writing)
//! ```
use crate::watch::{self, State, Watcher};
use crate::{Call, Domain, Limits, Request, TurnName, WritingCall};
use alloc::boxed::Box;
use skein_lib::bytes::copy_of;
use skein_lib::{Env, Id, List, Queue, Token, Wall};

/// One opaque live run's current bounded writing, watched or not.
#[derive(Debug)]
pub struct RetainedRun {
    attempt: u64,
    turn: Option<TurnName>,
    from: u32,
    to: u32,
    text: Box<[u8]>,
    call: Option<CallSlot>,
    revision: u64,
    committed: bool,
    cut: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum CallSlot {
    Made(Call),
    Writing(WritingCall),
}

fn made(slot: Option<&CallSlot>) -> Option<&Call> {
    match slot {
        Some(CallSlot::Made(call)) => Some(call),
        Some(CallSlot::Writing(_)) | None => None,
    }
}
fn writing_call(slot: Option<&CallSlot>) -> Option<&WritingCall> {
    match slot {
        Some(CallSlot::Writing(call)) => Some(call),
        Some(CallSlot::Made(_)) | None => None,
    }
}

impl RetainedRun {
    /// Pure attempt, optional writing position and committed mark.
    #[must_use]
    pub fn metadata(&self) -> (u64, Option<TurnName>, bool) {
        (self.attempt, self.turn, self.committed)
    }
}

/// A stream terminal advances only this named turn's position.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Flight {
    turn: TurnName,
    to: u32,
    revision: u64,
}

#[expect(clippy::too_many_arguments, reason = "a host's writing report carries its named turn, offset, text and call")]
pub(crate) fn written(
    domain: &mut Domain,
    env: &Env<Limits>,
    token: Token,
    turn: TurnName,
    offset: u32,
    text: &[u8],
    call: Option<Call>,
    out: &mut Queue<Request>,
) {
    let limits = &env.limits;
    if let Some(call) = &call
        && !fits(call, limits.text_bytes)
    {
        return;
    }
    let Ok(length) = u32::try_from(text.len()) else {
        return;
    };
    if length > limits.text_bytes {
        return;
    }
    let Some(end) = offset.checked_add(length) else {
        return;
    };
    let Some(new_turn) = prepare(domain, limits, token, turn) else {
        return;
    };
    let run = domain.runs.get_mut(&token).expect("admitted live run");
    if new_turn {
        run.from = offset;
        run.to = offset;
    }
    let gap = !new_turn && offset > run.to;
    if gap {
        run.cut = true;
        run.from = offset;
        run.to = offset;
        run.text = Box::new([]);
    }
    // Duplicates may overlap retained bytes; inconsistent overlap is fenced.
    for position in offset.max(run.from)..end.min(run.to) {
        let kept = usize::try_from(position.checked_sub(run.from).expect("retained offset")).expect("byte index");
        let incoming = usize::try_from(position.checked_sub(offset).expect("incoming offset")).expect("byte index");
        if run.text.get(kept) != text.get(incoming) {
            return;
        }
    }
    let call = match call {
        Some(call) => Some(CallSlot::Made(timed(call, made(run.call.as_ref()), env.wall))),
        None => run.call.clone(),
    };
    if !new_turn && !gap && end <= run.to && call == run.call {
        return;
    }
    if end > run.to {
        let from = run.from.max(end.saturating_sub(limits.text_bytes));
        let mut suffix = List::with_capacity(limits.text_bytes);
        for position in from..run.to {
            let index = usize::try_from(position.checked_sub(run.from).expect("retained offset")).expect("byte index");
            suffix.push(*run.text.get(index).expect("retained byte")).expect("bounded suffix");
        }
        for position in from.max(run.to)..end {
            let index = usize::try_from(position.checked_sub(offset).expect("incoming offset")).expect("byte index");
            suffix.push(*text.get(index).expect("incoming byte")).expect("bounded suffix");
        }
        run.cut = run.cut || from > run.from;
        run.text = suffix.into_boxed();
        run.from = from;
        run.to = end;
    }
    run.call = call;
    run.revision = run.revision.checked_add(1).expect("provisional revision space");
    for (_, id) in &domain.names {
        let watch = domain.watchers.get_mut(*id).expect("named watch");
        if let Some(follow) = watch.follow
            && follow.run == token
        {
            offer(watch, run, out);
        }
    }
}

fn fits(call: &Call, capacity: u32) -> bool {
    let Some(bytes) = call.identity.id.len().checked_add(call.tool.len()) else { return false };
    let Some(bytes) = bytes.checked_add(call.input.len()) else { return false };
    bytes <= usize::try_from(capacity).expect("byte capacity")
}
fn timed(mut call: Call, previous: Option<&Call>, wall: Wall) -> Call {
    call.began = match previous {
        Some(previous) if previous.identity == call.identity => match previous.began {
            Some(began) => Some(began),
            None => Some(wall),
        },
        Some(_) | None => Some(wall),
    };
    call
}

fn empty(attempt: u64) -> RetainedRun {
    RetainedRun {
        attempt,
        turn: None,
        from: 0,
        to: 0,
        text: Box::new([]),
        call: None,
        revision: 0,
        committed: false,
        cut: false,
    }
}

pub(crate) fn called(
    domain: &mut Domain,
    env: &Env<Limits>,
    token: Token,
    attempt: u64,
    call: Call,
    out: &mut Queue<Request>,
) {
    if !fits(&call, env.limits.text_bytes) {
        return;
    }
    for (_, id) in &domain.names {
        let watch = domain.watchers.get(*id).expect("named watch");
        if let Some(follow) = watch.follow
            && follow.run == token
            && let Some(ended) = watch.ended_turn
            && attempt <= ended.attempt
        {
            return;
        }
    }
    if !domain.runs.contains_key(&token) {
        if domain.runs.len() >= env.limits.runs {
            return;
        }
        domain.runs.insert(token, empty(attempt)).expect("retained live run admission checked");
    }
    let run = domain.runs.get_mut(&token).expect("admitted current call");
    if attempt < run.attempt {
        return;
    }
    if attempt != run.attempt {
        *run = empty(attempt);
    }
    let call = CallSlot::Made(timed(call, made(run.call.as_ref()), env.wall));
    if run.call.as_ref() == Some(&call) {
        return;
    }
    run.call = Some(call);
    run.revision = run.revision.checked_add(1).expect("provisional revision space");
    for (_, id) in &domain.names {
        let watch = domain.watchers.get_mut(*id).expect("named watch");
        if let Some(follow) = watch.follow
            && follow.run == token
        {
            offer(watch, run, out);
        }
    }
}

pub(crate) fn committed(domain: &mut Domain, token: Token, attempt: u64, turn: u32) {
    if let Some(run) = domain.runs.get_mut(&token)
        && let Some(name) = run.turn
        && name.attempt == attempt
        && name.turn <= turn
    {
        run.committed = true;
        run.from = 0;
        run.to = 0;
        run.text = Box::new([]);
        if writing_call(run.call.as_ref()).is_some() {
            run.call = None;
        }
        // Accepting text attests no tool ending. The current content call
        // remains independent until a supplied new turn/retry, call or run end.
    }
}

fn bounded_call(call: Option<Call>, capacity: u32) -> Option<Call> {
    let mut call = call?;
    let bound = usize::try_from(capacity).expect("byte capacity");
    let bound = bound.checked_sub(call.identity.id.len())?;
    let tool_length = call.tool.len().min(bound);
    let input_bound = bound.checked_sub(tool_length).expect("bounded tool");
    let input_length = call.input.len().min(input_bound);
    call.cut = call.cut || tool_length < call.tool.len() || input_length < call.input.len();
    call.tool = copy_of(call.tool.get(..tool_length).expect("tool prefix"));
    call.input = copy_of(call.input.get(..input_length).expect("input prefix"));
    Some(call)
}

pub(crate) fn offer_one(domain: &mut Domain, id: Id<Watcher>, out: &mut Queue<Request>) {
    let watch = domain.watchers.get_mut(id).expect("named watch");
    if let Some(follow) = watch.follow
        && let Some(run) = domain.runs.get(&follow.run)
    {
        offer(watch, run, out);
    }
}
fn offer(watch: &mut Watcher, run: &RetainedRun, out: &mut Queue<Request>) {
    if run.committed {
        return;
    }
    match watch.state {
        State::ClosingMake { .. } | State::ClosingDelivery { .. } | State::ClosingStream { .. } => return,
        State::Idle | State::Making { .. } | State::Sending { .. } => {}
    }
    if watch.flight.is_some() {
        return;
    }
    let Some(turn) = run.turn else {
        return;
    };
    let follow = watch.follow.expect("a stream binds a run");
    if follow.turn == Some(turn) && watch.revision == Some(run.revision) {
        return;
    }
    let offset =
        if follow.turn == Some(turn) && (run.from..=run.to).contains(&follow.offset) { follow.offset } else { 0 };
    let from = offset.max(run.from);
    let index = usize::try_from(from.checked_sub(run.from).expect("retained offset")).expect("byte index");
    watch.flight = Some(Flight { turn, to: run.to, revision: run.revision });
    out.push(Request::Stream {
        watcher: watch.token,
        turn,
        from,
        to: run.to,
        text: copy_of(run.text.get(index..).expect("retained suffix")),
        cut: run.cut || from > offset,
        call: made(run.call.as_ref()).cloned(),
        writing: writing_call(run.call.as_ref()).cloned(),
    });
}

pub(crate) fn delivered(domain: &mut Domain, token: Token, out: &mut Queue<Request>) {
    let Some(&id) = domain.names.get(&token) else {
        return;
    };
    let watch = domain.watchers.get_mut(id).expect("named watch");
    let Some(flight) = watch.flight.take() else {
        return;
    };
    let follow = watch.follow.as_mut().expect("stream follows a run");
    follow.turn = Some(flight.turn);
    follow.offset = flight.to;
    watch.revision = Some(flight.revision);
    match watch.state {
        State::ClosingStream { end } => watch::finish(domain, id, end, out),
        State::ClosingMake { .. } | State::ClosingDelivery { .. } => {}
        State::Idle | State::Making { .. } | State::Sending { .. } => offer_one(domain, id, out),
    }
}

pub(crate) fn ended(domain: &mut Domain, token: Token) {
    if let Some(run) = domain.runs.remove(&token) {
        for (_, id) in &domain.names {
            let watch = domain.watchers.get_mut(*id).expect("named watch");
            if let Some(follow) = watch.follow
                && follow.run == token
            {
                watch.ended_turn = run.turn;
                watch.revision = None;
            }
        }
    }
}

/// Copy the current run's suffix within the document's independent byte bound.
#[must_use]
pub fn writing(domain: &Domain, token: Token, capacity: u32) -> Option<crate::Writing> {
    let run = domain.runs.get(&token)?;
    if run.committed {
        return None;
    }
    let turn = run.turn?;
    let from = run.from.max(run.to.saturating_sub(capacity));
    let index = usize::try_from(from.checked_sub(run.from).expect("bounded writing offset")).expect("byte index");
    Some(crate::Writing {
        turn,
        from,
        to: run.to,
        text: copy_of(run.text.get(index..).expect("retained suffix")),
        cut: run.cut || from != 0,
        call: bounded_call(made(run.call.as_ref()).cloned(), capacity),
        writing: bounded_writing(writing_call(run.call.as_ref()), capacity),
    })
}

/// Current turn and end offset without copying provisional bytes.
#[must_use]
pub fn position(domain: &Domain, token: Token) -> Option<(TurnName, u32)> {
    let run = domain.runs.get(&token)?;
    Some((run.turn?, run.to))
}

/// Actual attempt retained by the first text or call report, without a guessed turn.
#[must_use]
pub fn retained_attempt(domain: &Domain, token: Token) -> Option<u64> {
    Some(domain.runs.get(&token)?.attempt)
}

/// Current source revision, for a parent to mark only an actual retained change.
#[must_use]
pub fn revision(domain: &Domain, token: Token) -> Option<u64> {
    Some(domain.runs.get(&token)?.revision)
}

/// Whether the actual retained attempt has a current call, without copying bytes.
#[must_use]
pub fn has_current_call(domain: &Domain, token: Token, attempt: u64) -> bool {
    match domain.runs.get(&token) {
        Some(run) => run.attempt == attempt && made(run.call.as_ref()).is_some(),
        None => false,
    }
}

/// A current call is independent of a main completion's provisional text.
/// Its whole identity is preserved and descriptions use the same document bound.
#[must_use]
pub fn current_call(domain: &Domain, token: Token, attempt: u64, capacity: u32) -> Option<Call> {
    let run = domain.runs.get(&token)?;
    if run.attempt != attempt {
        return None;
    }
    bounded_call(made(run.call.as_ref()).cloned(), capacity)
}

fn prepare(domain: &mut Domain, limits: &Limits, token: Token, turn: TurnName) -> Option<bool> {
    // The core fences ended attempts. Retain the same fence for watches whose
    // stream terminal can arrive after the run's end and before its successor.
    for (_, id) in &domain.names {
        let watch = domain.watchers.get(*id).expect("named watch");
        if let Some(follow) = watch.follow
            && follow.run == token
            && let Some(ended) = watch.ended_turn
            && turn <= ended
        {
            return None;
        }
    }
    if !domain.runs.contains_key(&token) {
        if domain.runs.len() >= limits.runs {
            return None;
        }
        domain.runs.insert(token, empty(turn.attempt)).expect("retained live run admission checked");
    }
    let run = domain.runs.get_mut(&token).expect("admitted live run");
    if turn.attempt < run.attempt {
        return None;
    }
    let new_turn = match run.turn {
        Some(current) => {
            if turn < current || (turn == current && run.committed) {
                return None;
            }
            turn != current
        }
        None => true,
    };
    if new_turn {
        run.cut = false;
        run.attempt = turn.attempt;
        run.turn = Some(turn);
        run.committed = false;
        run.from = 0;
        run.to = 0;
        run.text = Box::new([]);
        run.call = None;
    }
    Some(new_turn)
}

#[expect(clippy::too_many_arguments, reason = "a live input report carries its turn, identity and byte offset")]
pub(crate) fn input(
    domain: &mut Domain,
    env: &Env<Limits>,
    token: Token,
    turn: TurnName,
    offset: u32,
    id: Box<[u8]>,
    tool: Box<[u8]>,
    input: &[u8],
    out: &mut Queue<Request>,
) {
    let Some(identity) = id.len().checked_add(tool.len()) else {
        return;
    };
    let Ok(identity) = u32::try_from(identity) else {
        return;
    };
    let Some(capacity) = env.limits.text_bytes.checked_sub(identity) else {
        return;
    };
    let Ok(length) = u32::try_from(input.len()) else {
        return;
    };
    let Some(end) = offset.checked_add(length) else {
        return;
    };
    if prepare(domain, &env.limits, token, turn).is_none() {
        return;
    }
    let run = domain.runs.get_mut(&token).expect("admitted input run");
    let mut call = match writing_call(run.call.as_ref()) {
        Some(previous) if previous.id == id && previous.tool == tool => previous.clone(),
        Some(_) | None => {
            WritingCall { id, tool, from: offset, through: offset, input: Box::new([]), cut: offset != 0 }
        }
    };
    if offset > call.through {
        call.from = offset;
        call.through = offset;
        call.input = Box::new([]);
        call.cut = true;
    }
    if end > call.through {
        let from = call.from.max(end.saturating_sub(capacity));
        let mut suffix = List::with_capacity(capacity);
        for position in from..call.through {
            let index = usize::try_from(position.checked_sub(call.from).expect("input offset")).expect("input index");
            suffix.push(*call.input.get(index).expect("retained input")).expect("bounded input");
        }
        for position in from.max(call.through)..end {
            let index = usize::try_from(position.checked_sub(offset).expect("input offset")).expect("input index");
            suffix.push(*input.get(index).expect("incoming input")).expect("bounded input");
        }
        call.cut = call.cut || from > call.from;
        call.from = from;
        call.through = end;
        call.input = suffix.into_boxed();
    }
    let slot = Some(CallSlot::Writing(call));
    if run.call == slot {
        return;
    }
    run.call = slot;
    run.revision = run.revision.checked_add(1).expect("provisional revision space");
    for (_, id) in &domain.names {
        let watch = domain.watchers.get_mut(*id).expect("named watch");
        if let Some(follow) = watch.follow
            && follow.run == token
        {
            offer(watch, run, out);
        }
    }
}

fn bounded_writing(value: Option<&WritingCall>, capacity: u32) -> Option<WritingCall> {
    let value = value?;
    let identity = u32::try_from(value.id.len().checked_add(value.tool.len())?).ok()?;
    let bound = capacity.checked_sub(identity)?;
    let from = value.from.max(value.through.saturating_sub(bound));
    let index = usize::try_from(from.checked_sub(value.from)?).ok()?;
    Some(WritingCall {
        id: copy_of(&value.id),
        tool: copy_of(&value.tool),
        from,
        through: value.through,
        input: copy_of(value.input.get(index..)?),
        cut: value.cut || from > value.from,
    })
}
