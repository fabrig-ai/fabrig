//! History's text and JSON (temper/docs/design/domain/landing-history.md, 5.1).
use crate::{jig::{self, render::{self, Text}}, root::Limits};
use alloc::boxed::Box;
use skein_json::writer::{Encoder, Limits as JsonLimits};
use skein_lib::Wall;
use temper_client_protocol_wire::v1 as w;
pub(super) fn page(query: &w::HistoryRead, page: &w::HistoryPage, json: bool, limits: &Limits) -> jig::Printed {
    let stdout = if json {
        let bounds = JsonLimits { depth: 12, length: u32::try_from(limits.tools.result_bytes).expect("result bound") };
        let mut measure = Encoder::measure(&bounds); page_json(page, &mut measure);
        let Ok(size) = measure.measured() else { return render::failure(b"result limit", true, 1) };
        let mut out = Encoder::write(size, &bounds); page_json(page, &mut out); out.finish()
    } else {
        let mut measure = Text::measure(); page_text(query, page, &mut measure);
        if measure.structural() > usize::try_from(limits.tools.text).expect("text bound") { return render::failure(b"result limit", false, 1); }
        let mut out = Text::writing_exact(measure); page_text(query, page, &mut out); out.finish()
    };
    jig::Printed { file: jig::FileEnd::Keep, status: 0, stdout, stderr: Box::default() }
}
fn page_text(query: &w::HistoryRead, page: &w::HistoryPage, out: &mut Text) {
    out.put(b"project "); out.number(u64::from(query.project())); out.put(b": events through "); out.number(page.upper());
    if let Some(at) = page.whole_since() { out.put(b", whole since "); clock(*at, out); }
    out.put(b"\n"); for event in page.events() { event_line(event, out); }
    match page.after() { Some(after) => { command(query, Some(after), out); out.put(b"\n"); }, None => out.put(b"end\n") }
}
fn page_json(page: &w::HistoryPage, out: &mut Encoder) {
    out.object_start(); out.key(b"events"); out.array_start(); for event in page.events() { event_json(event, out); }
    out.array_end(); out.key(b"after"); match page.after() { Some(bytes) => out.string(&hex(bytes)), None => out.null() }
    out.key(b"upper"); out.unsigned(page.upper()); out.key(b"whole_since"); optional_number(*page.whole_since(), out); out.object_end();
}
fn clock(at: u64, out: &mut Text) { render::document::clock::deadline(Wall::from_nanos(at), out); }
fn hex(bytes: &[u8]) -> Box<[u8]> {
    let mut out = skein_lib::Writer::new(bytes.len().checked_mul(2).expect("bounded hexadecimal"));
    for byte in bytes { for nibble in [byte >> 4_u8, byte & 15] { out.put(&[*b"0123456789abcdef".get(usize::from(nibble)).expect("nibble")]).expect("measured hex"); } } out.finish()
}
fn command(query: &w::HistoryRead, after: Option<&[u8; 17]>, out: &mut Text) {
    out.put(b"temper ctl DIR landing history --project "); out.number(u64::from(query.project()));
    match query.of() { w::HistoryOf::Project => {}, w::HistoryOf::Change(task) => { out.put(b" --change "); out.number(task.task()); }, w::HistoryOf::Landing(task) => { out.put(b" --landing "); out.number(task.task()); } }
    for (key, at) in [(b" --since ".as_slice(), query.since()), (b" --until ", query.until())] {
        if let Some(at) = at { out.put(key); render::document::clock::minute(Wall::from_nanos(*at), out); }
    }
    if let Some(after) = after { out.put(b" --after "); out.put(&hex(after)); }
}
pub(super) fn gone(query: &w::HistoryRead, gone: &w::HistoryGone, json: bool) -> jig::Printed {
    let mut measure = Text::measure(); gone_text(query, gone, &mut measure);
    let mut out = Text::writing_exact(measure); gone_text(query, gone, &mut out); render::failure(&out.finish(), json, 2)
}
fn gone_text(query: &w::HistoryRead, gone: &w::HistoryGone, out: &mut Text) {
    out.put(b"history from event "); out.number(gone.from()); out.put(b" is gone; ");
    match gone.first() { Some(first) => { out.put(b"the first event that remains after it is "); out.number(*first); }, None => out.put(b"none remains after it") }
    out.put(b"\nread again: "); command(query, None, out);
}
/// Print one event's complete line and newline. Commit prefixes have twelve digits.
pub fn event_line(event: &w::HistoryEvent, out: &mut Text) {
    out.number(event.number()); out.put(b" "); clock(event.at(), out); out.put(b" "); out.put(kind(event.what())); out.put(b" change "); out.number(event.change());
    if let Some(landing) = event.landing() { out.put(b" landing "); out.number(landing.task()); out.put(b" round "); out.number(u64::from(landing.round())); out.put(b" unit "); out.number(u64::from(landing.unit())); }
    for (key, hash) in [(b" head ".as_slice(), event.head()), (b" base ", event.base())] { if let Some(hash) = hash { out.put(key); short(hash, out); } }
    fields(event.what(), &mut Fields::Text(out)); out.put(b"\n");
}
/// JSON for one event: every row field, with the kind and its fields in `what`.
pub fn event_json(event: &w::HistoryEvent, out: &mut Encoder) {
    out.object_start();
    for (key, n) in [(b"project".as_slice(), u64::from(event.project())), (b"number", event.number()), (b"at", event.at()), (b"change", event.change()), (b"operation", event.operation()), (b"attempt", u64::from(event.attempt()))] { out.key(key); out.unsigned(n); }
    out.key(b"source_at"); optional_number(*event.source_at(), out);
    out.key(b"repository"); out.object_start(); out.key(b"forge"); out.unsigned(u64::from(event.repository().forge())); out.key(b"repository"); out.unsigned(u64::from(event.repository().repository())); out.object_end();
    out.key(b"landing"); match event.landing() { Some(landing) => { out.object_start(); out.key(b"task"); out.unsigned(landing.task()); out.key(b"round"); out.unsigned(u64::from(landing.round())); out.key(b"unit"); out.unsigned(u64::from(landing.unit())); out.object_end(); }, None => out.null() }
    for (key, hash) in [(b"head".as_slice(), event.head()), (b"base", event.base())] { out.key(key); match hash { Some(hash) => out.string(&hex(hash)), None => out.null() } }
    out.key(b"what"); out.object_start(); out.key(b"kind"); out.string(kind(event.what())); fields(event.what(), &mut Fields::Json(out)); out.object_end(); out.object_end();
}
fn optional_number(number: Option<u64>, out: &mut Encoder) { match number { Some(n) => out.unsigned(n), None => out.null() } }
fn short(hash: &[u8; 32], out: &mut Text) { out.put(hex(hash).get(..12).expect("commit prefix")); }
#[derive(Debug)]
enum Fields<'a> { Text(&'a mut Text), Json(&'a mut Encoder) }
impl Fields<'_> {
    fn key(&mut self, key: &[u8]) { match self { Self::Text(out) => { out.put(b" "); out.put(key); out.put(b" "); }, Self::Json(out) => out.key(key) } }
    fn word(&mut self, key: &[u8], value: &[u8]) { self.key(key); match self { Self::Text(out) => { for byte in value { match byte { b'\n' => out.put(b"\\n"), b'\r' => out.put(b"\\r"), b'\t' => out.put(b"\\t"), _ => out.put(&[*byte]) } } }, Self::Json(out) => out.string(value) } }
    fn number(&mut self, key: &[u8], n: u64) { self.key(key); match self { Self::Text(out) => out.number(n), Self::Json(out) => out.unsigned(n) } }
    fn optional(&mut self, key: &[u8], n: Option<u64>) { match n { Some(n) => self.number(key, n), None => { if let Self::Json(out) = self { out.key(key); out.null(); } } } }
    fn boolean(&mut self, key: &[u8], value: bool) { self.key(key); match self { Self::Text(out) => out.put(if value { b"true" } else { b"false" }), Self::Json(out) => out.boolean(value) } }
    fn hash(&mut self, key: &[u8], hash: &[u8; 32]) { self.key(key); match self { Self::Text(out) => { if key == b"key" { out.put(&hex(hash)); } else { short(hash, out); } }, Self::Json(out) => out.string(&hex(hash)) } }
    fn optional_hash(&mut self, key: &[u8], hash: Option<&[u8; 32]>) { match hash { Some(hash) => self.hash(key, hash), None => { if let Self::Json(out) = self { out.key(key); out.null(); } } } }
    fn priority(&mut self, n: Option<u32>) { match n { Some(n) => { self.key(b"priority"); let n = i32::from_be_bytes(n.to_be_bytes()); match self { Self::Text(out) => { if n < 0_i32 { out.put(b"-"); } out.number(u64::from(n.unsigned_abs())); }, Self::Json(out) => out.signed(i64::from(n)) } }, None => { if let Self::Json(out) = self { out.key(b"priority"); out.null(); } } } }
    fn object_start(&mut self) { match self { Self::Text(out) => out.put(b"{"), Self::Json(out) => out.object_start() } }
    fn object_end(&mut self) { match self { Self::Text(out) => out.put(b" }"), Self::Json(out) => out.object_end() } }
    fn array_start(&mut self) { match self { Self::Text(out) => out.put(b"["), Self::Json(out) => out.array_start() } }
    fn array_end(&mut self) { match self { Self::Text(out) => out.put(b"]"), Self::Json(out) => out.array_end() } }
}
fn kind(what: &w::HistoryWhat) -> &'static [u8] {
    match what {
        w::HistoryWhat::Ready => b"ready",
        w::HistoryWhat::Unready(_) => b"unready",
        w::HistoryWhat::Joined(_) => b"joined",
        w::HistoryWhat::Departed(_) => b"departed",
        w::HistoryWhat::Selected => b"selected",
        w::HistoryWhat::Update(_) => b"update",
        w::HistoryWhat::Updated(_) => b"updated",
        w::HistoryWhat::Job(_) => b"job",
        w::HistoryWhat::JobEnded(_) => b"job-ended",
        w::HistoryWhat::Conflict(_) => b"conflict",
        w::HistoryWhat::Asked(_) => b"asked",
        w::HistoryWhat::Answered(_) => b"answered",
        w::HistoryWhat::Pushed => b"pushed",
        w::HistoryWhat::Delivered(_) => b"delivered",
        w::HistoryWhat::Paused(_) => b"paused",
        w::HistoryWhat::Resumed => b"resumed",
        w::HistoryWhat::Formed(_) => b"formed",
        w::HistoryWhat::RoundEnded(_) => b"round-ended",
        w::HistoryWhat::Gate(_) => b"gate",
        w::HistoryWhat::GateEnded(_) => b"gate-ended",
        w::HistoryWhat::Proved(_) => b"proved",
        w::HistoryWhat::Search => b"search",
        w::HistoryWhat::Searched(_) => b"searched",
        w::HistoryWhat::Probe => b"probe",
        w::HistoryWhat::ProbeEnded(_) => b"probe-ended",
        w::HistoryWhat::Turn(_) => b"turn",
        w::HistoryWhat::Landed => b"landed",
        w::HistoryWhat::Recorded(_) => b"recorded",
        w::HistoryWhat::Ended(_) => b"ended",
        w::HistoryWhat::LandingEnded(_) => b"landing-ended",
        w::HistoryWhat::Held(_) => b"held",
        w::HistoryWhat::Released(_) => b"released",
        w::HistoryWhat::After(_) => b"after",
    }
}
fn unready(value: &w::HistoryUnready) -> &'static [u8] {
    match value {
        w::HistoryUnready::Held => b"held",
        w::HistoryUnready::HeadMoved => b"head_moved",
        w::HistoryUnready::Refused => b"refused",
        w::HistoryUnready::Dependency => b"dependency",
        w::HistoryUnready::Approval => b"approval",
        w::HistoryUnready::Conflict => b"conflict",
        w::HistoryUnready::Closed => b"closed",
    }
}
fn line(value: &w::HistoryLine) -> &'static [u8] {
    match value {
        w::HistoryLine::Queue => b"queue",
        w::HistoryLine::Landing => b"landing",
        w::HistoryLine::ForgeQueue => b"forge_queue",
    }
}
fn update_style(value: &w::HistoryUpdateStyle) -> &'static [u8] {
    match value {
        w::HistoryUpdateStyle::Merge => b"merge",
        w::HistoryUpdateStyle::Rebase => b"rebase",
    }
}
fn update_outcome(value: &w::HistoryUpdateOutcome) -> &'static [u8] {
    match value {
        w::HistoryUpdateOutcome::Updated => b"updated",
        w::HistoryUpdateOutcome::Conflict => b"conflict",
        w::HistoryUpdateOutcome::Failed => b"failed",
    }
}
fn job(value: &w::HistoryJob) -> &'static [u8] {
    match value {
        w::HistoryJob::Prepare => b"prepare",
        w::HistoryJob::Merge => b"merge",
    }
}
fn job_outcome(value: &w::HistoryJobOutcome) -> &'static [u8] {
    match value {
        w::HistoryJobOutcome::Done => b"done",
        w::HistoryJobOutcome::Conflict => b"conflict",
        w::HistoryJobOutcome::Failed => b"failed",
        w::HistoryJobOutcome::Lost => b"lost",
    }
}
fn with(value: &w::HistoryWith) -> &'static [u8] {
    match value {
        w::HistoryWith::Branch => b"branch",
        w::HistoryWith::Earlier => b"earlier",
    }
}
fn ask(value: &w::HistoryAsk) -> &'static [u8] {
    match value {
        w::HistoryAsk::Resolution => b"resolution",
        w::HistoryAsk::Repair => b"repair",
        w::HistoryAsk::Judgement => b"judgement",
    }
}
fn delegate_outcome(value: &w::HistoryDelegateOutcome) -> &'static [u8] {
    match value {
        w::HistoryDelegateOutcome::Pushed => b"pushed",
        w::HistoryDelegateOutcome::Passed => b"passed",
        w::HistoryDelegateOutcome::Refused => b"refused",
        w::HistoryDelegateOutcome::Nothing => b"nothing",
        w::HistoryDelegateOutcome::Failed => b"failed",
        w::HistoryDelegateOutcome::Cancelled => b"cancelled",
    }
}
fn pause(value: &w::HistoryPause) -> &'static [u8] {
    match value {
        w::HistoryPause::BranchRed => b"branch_red",
        w::HistoryPause::TierRed => b"tier_red",
    }
}
fn round_cause(value: &w::HistoryRoundCause) -> &'static [u8] {
    match value {
        w::HistoryRoundCause::Turned => b"turned",
        w::HistoryRoundCause::Left => b"left",
        w::HistoryRoundCause::Stale => b"stale",
        w::HistoryRoundCause::Cancelled => b"cancelled",
        w::HistoryRoundCause::Cut => b"cut",
    }
}
fn gate_verdict(value: &w::HistoryGateVerdict) -> &'static [u8] {
    match value {
        w::HistoryGateVerdict::Passed => b"passed",
        w::HistoryGateVerdict::Failed => b"failed",
        w::HistoryGateVerdict::Fault => b"fault",
        w::HistoryGateVerdict::Timeout => b"timeout",
        w::HistoryGateVerdict::Superseded => b"superseded",
    }
}
fn recorded_state(value: &w::HistoryRecordedState) -> &'static [u8] {
    match value {
        w::HistoryRecordedState::Merged => b"merged",
        w::HistoryRecordedState::Closed => b"closed",
    }
}
fn ending(value: &w::HistoryEnding) -> &'static [u8] {
    match value {
        w::HistoryEnding::NoChange => b"no_change",
        w::HistoryEnding::Cancelled => b"cancelled",
        w::HistoryEnding::Failed => b"failed",
    }
}
fn landing_outcome(value: &w::HistoryLandingOutcome) -> &'static [u8] {
    match value {
        w::HistoryLandingOutcome::Done => b"done",
        w::HistoryLandingOutcome::Cancelled => b"cancelled",
        w::HistoryLandingOutcome::Held => b"held",
    }
}
fn on(value: &w::HistoryOn) -> &'static [u8] {
    match value {
        w::HistoryOn::Change => b"change",
        w::HistoryOn::Landing => b"landing",
    }
}
fn after_verdict(value: &w::HistoryAfterVerdict) -> &'static [u8] {
    match value {
        w::HistoryAfterVerdict::Verified => b"verified",
        w::HistoryAfterVerdict::Red => b"red",
        w::HistoryAfterVerdict::Cleared => b"cleared",
        w::HistoryAfterVerdict::Uncleared => b"uncleared",
    }
}
fn fields(what: &w::HistoryWhat, out: &mut Fields<'_>) {
    match what {
        w::HistoryWhat::Ready | w::HistoryWhat::Selected | w::HistoryWhat::Pushed | w::HistoryWhat::Resumed | w::HistoryWhat::Search | w::HistoryWhat::Probe | w::HistoryWhat::Landed => {},
        w::HistoryWhat::Unready(value) => {
            out.word(b"why", unready(value.why()));
        },
        w::HistoryWhat::Joined(value) => {
            out.word(b"line", line(value.line()));
            out.optional(b"place", match value.place() { Some(n) => Some(u64::from(*n)), None => None });
            out.priority(*value.priority());
        },
        w::HistoryWhat::Departed(value) => {
            out.word(b"line", line(value.line()));
            departure(value.cause(), out);
        },
        w::HistoryWhat::Update(value) => {
            out.word(b"style", update_style(value.style()));
        },
        w::HistoryWhat::Updated(value) => {
            out.word(b"outcome", update_outcome(value.outcome()));
        },
        w::HistoryWhat::Job(value) => {
            out.word(b"job", job(value.job()));
            out.number(b"task", value.task());
        },
        w::HistoryWhat::JobEnded(value) => {
            out.word(b"job", job(value.job()));
            out.word(b"outcome", job_outcome(value.outcome()));
        },
        w::HistoryWhat::Conflict(value) => {
            out.word(b"with", with(value.with()));
            out.number(b"files", u64::from(value.files()));
        },
        w::HistoryWhat::Asked(value) => {
            out.word(b"ask", ask(value.ask()));
            out.number(b"task", value.task());
            out.optional(b"gate", match value.gate() { Some(n) => Some(u64::from(*n)), None => None });
        },
        w::HistoryWhat::Answered(value) => {
            out.word(b"ask", ask(value.ask()));
            out.word(b"outcome", delegate_outcome(value.outcome()));
        },
        w::HistoryWhat::Delivered(value) => {
            out.boolean(b"checked", value.checked());
        },
        w::HistoryWhat::Paused(value) => {
            out.word(b"why", pause(value.why()));
        },
        w::HistoryWhat::Formed(value) => {
            out.key(b"members"); out.array_start();
            for member in value.members() { out.object_start(); out.number(b"change", member.change()); out.number(b"unit", u64::from(member.unit())); out.hash(b"head", member.head()); out.object_end(); }
            out.array_end();
            out.number(b"place", u64::from(value.place()));
        },
        w::HistoryWhat::RoundEnded(value) => {
            out.word(b"cause", round_cause(value.cause()));
        },
        w::HistoryWhat::Gate(value) => {
            out.number(b"gate", u64::from(value.gate()));
            out.hash(b"key", value.key());
        },
        w::HistoryWhat::GateEnded(value) => {
            out.number(b"gate", u64::from(value.gate()));
            out.hash(b"key", value.key());
            out.word(b"verdict", gate_verdict(value.verdict()));
        },
        w::HistoryWhat::Proved(value) => {
            out.key(b"proofs"); out.array_start();
            for proof in value.proofs() { out.object_start(); out.number(b"gate", u64::from(proof.gate())); out.number(b"run", proof.run()); out.object_end(); }
            out.array_end();
        },
        w::HistoryWhat::Searched(value) => {
            out.optional_hash(b"green", value.green().as_ref());
            out.hash(b"red", value.red());
            out.number(b"blamed", u64::from(value.blamed()));
        },
        w::HistoryWhat::ProbeEnded(value) => {
            out.boolean(b"green", value.green());
        },
        w::HistoryWhat::Turn(value) => {
            out.number(b"members", u64::from(value.members()));
        },
        w::HistoryWhat::Recorded(value) => {
            out.word(b"state", recorded_state(value.state()));
        },
        w::HistoryWhat::Ended(value) => {
            out.word(b"ending", ending(value.ending()));
        },
        w::HistoryWhat::LandingEnded(value) => {
            out.word(b"outcome", landing_outcome(value.outcome()));
        },
        w::HistoryWhat::Held(value) | w::HistoryWhat::Released(value) => {
            out.word(b"on", on(value.on()));
            out.word(b"reason", value.reason());
            by(value.by().as_ref(), out);
        },
        w::HistoryWhat::After(value) => {
            out.word(b"verdict", after_verdict(value.verdict()));
        },
    }
}
fn by(value: Option<&w::HistoryBy>, out: &mut Fields<'_>) {
    match value { Some(value) => { out.key(b"by"); out.object_start(); match value { w::HistoryBy::Party(n) => out.number(b"party", n.number()), w::HistoryBy::Task(n) => out.number(b"task", n.number()) } out.object_end(); }, None => { if let Fields::Json(out) = out { out.key(b"by"); out.null(); } } }
}
fn departure(value: &w::HistoryDeparture, out: &mut Fields<'_>) {
    out.key(b"cause"); out.object_start();
    match value {
        w::HistoryDeparture::Unready => out.word(b"kind", b"unready"), w::HistoryDeparture::Conflict => out.word(b"kind", b"conflict"), w::HistoryDeparture::Repair => out.word(b"kind", b"repair"), w::HistoryDeparture::Ejected => out.word(b"kind", b"ejected"),
        w::HistoryDeparture::Landing(value) => { out.word(b"kind", b"landing"); out.key(b"cause"); out.object_start(); leave(value.cause(), out); out.object_end(); }
    } out.object_end();
}
fn leave(value: &w::HistoryLeave, out: &mut Fields<'_>) {
    match value {
        w::HistoryLeave::Conflict => out.word(b"kind", b"conflict"), w::HistoryLeave::EarlierConflict => out.word(b"kind", b"earlier_conflict"), w::HistoryLeave::Cancelled => out.word(b"kind", b"cancelled"), w::HistoryLeave::Held => out.word(b"kind", b"held"),
        w::HistoryLeave::HeadMoved(v) => { out.word(b"kind", b"head_moved"); out.hash(b"head", v.commit()); },
        w::HistoryLeave::Refused(v) => { out.word(b"kind", b"refused"); out.number(b"gate", v.gate()); },
        w::HistoryLeave::Failed(v) => { out.word(b"kind", b"failed"); out.number(b"gate", u64::from(v.gate())); out.number(b"run", v.run()); },
        w::HistoryLeave::Unit(v) => { out.word(b"kind", b"unit"); out.number(b"change", v.change()); }, w::HistoryLeave::Dependency(v) => { out.word(b"kind", b"dependency"); out.number(b"change", v.change()); },
        w::HistoryLeave::Blamed(v) => { out.word(b"kind", b"blamed"); out.number(b"gate", u64::from(v.gate())); out.number(b"run", v.run()); out.hash(b"red", v.red()); out.hash(b"green", v.green()); },
    }
}
