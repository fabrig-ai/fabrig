//! The shell's adapter over the temper composition (jig's protocol/service.md, 7).
//! Start refusals name their cause (temper/docs/design/protocol/binaries.md).
use crate::Served;
use skein_io::kernel::{Complete, Exit, Submit};
use skein_lib::{Queue, Time, Wall};
use std::io::{self, Write};
use temper_service::long_pass::{LONG_PASS, LongPass, Part, Watch};
use temper_service::{End, Stop};
use temper_shell::directories::Directories;
use temper_shell::directories::EnvironmentEntry;

pub struct Host<S> {
    pub service: S,
    served: Served<S>,
    upgrade: Option<fn(&S) -> Option<&temper_service::upgrade::Report>>,
    binary: String,
    directories: Directories,
    pub(super) store: Box<[u8]>,
    pub(super) link_file: Option<std::path::PathBuf>,
    output: Box<dyn Write>,
    output_failed: bool,
    wait_check: bool,
    cut_at_listen: Option<std::path::PathBuf>,
    now: Time,
    watch: Watch,
    clock: skein_shell::Clock,
}
impl<S> Host<S> {
    #[must_use]
    pub fn new(service: S, served: Served<S>, directories: Directories) -> Self {
        Self::with_binary("temper", service, served, directories, &[])
    }
    #[must_use]
    pub fn with_binary(
        binary: &str,
        service: S,
        served: Served<S>,
        directories: Directories,
        environment: &[EnvironmentEntry],
    ) -> Self {
        let store = directories.data.clone();
        Self {
            service,
            served,
            upgrade: None,
            binary: binary.into(),
            directories,
            store,
            link_file: None,
            output: Box::new(io::stdout()),
            output_failed: false,
            // QA launches sanitized child environments. Test/debug programs
            // always check; release programs can opt in with the same flag.
            wait_check: wait_check(environment, cfg!(debug_assertions)),
            cut_at_listen: super::restart_fault::marker(environment),
            now: Time::ZERO,
            watch: Watch::new(),
            clock: skein_shell::Clock::new(),
        }
    }
    fn ending(&self) -> (String, u8) {
        (self.served.ending)(&self.service, &self.binary, self.output_failed, &self.directories, &self.store)
    }
    pub(super) fn keep_link(&self, url: &[u8]) {
        if let Some(path) = &self.link_file
            && let Err(why) = super::link_file::keep(path, url, skein_shell::effective_user())
        {
            eprintln!("{} serve: link file {why}", self.binary);
        }
    }
}

impl Host<temper_service::Service> {
    /// Upgrade refusals name the checked directory, not the configured data directory.
    #[must_use]
    pub fn for_upgrade(
        binary: &str,
        service: temper_service::Service,
        served: Served<temper_service::Service>,
        directories: Directories,
        store: Box<[u8]>,
    ) -> Self {
        let mut host = Self::with_binary(binary, service, served, directories, &[]);
        host.store = store;
        host.upgrade = Some(temper_service::Service::upgrade_report);
        host
    }
}

pub(crate) fn wait_check(environment: &[EnvironmentEntry], debug: bool) -> bool {
    debug || crate::WAIT_CHECK.value(environment).is_some()
}

impl<S> skein_shell::Host for Host<S> {
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.now = now;
        self.watch.begin(now);
        for &stage in self.served.stages {
            (self.served.iterate_stage)(&mut self.service, stage, now, wall);
            self.watch.ended(Part::Service(stage), self.clock.now().now);
        }
        (self.served.resolve)(&mut self.service);
        self.watch.ended(Part::Names, self.clock.now().now);
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        (self.served.completions)(&mut self.service)
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        (self.served.submissions)(&mut self.service)
    }
    fn work_pending(&self, now: Time) -> bool {
        (self.served.work_pending)(&self.service, now)
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        (self.served.next_mechanism_deadline)(&self.service)
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        (self.served.next_policy_deadline)(&self.service)
    }
    fn is_empty(&self) -> bool {
        (self.served.is_empty)(&self.service)
    }
    fn operations(&self) -> u32 {
        (self.served.operations)(&self.service)
    }
    fn exit(&self) -> Option<Exit> {
        if !(self.served.is_empty)(&self.service) {
            return None;
        }
        if let Some(report) = self.upgrade.and_then(|report| report(&self.service)) {
            let (lines, code) = crate::upgrade::lines(&self.binary, report, &self.directories, &self.store);
            print!("{lines}");
            return Some(Exit::Code(code));
        }
        if let Some(long) = self.watch.held() {
            eprint!("{}", long_pass_line(&self.binary, long));
        }
        let (lines, code) = self.ending();
        eprint!("{lines}");
        Some(Exit::Code(code))
    }
    fn drain(&mut self) {
        (self.served.handover)(&mut self.service);
        while let Some((lines, clients)) = (self.served.listening)(&mut self.service, &self.binary) {
            eprint!("{lines}");
            if self.wait_check && clients {
                super::restart_fault::cut_at_listen(self.cut_at_listen.as_deref());
            }
        }
        while let Some(lines) = (self.served.note)(&mut self.service, &self.binary) {
            if self.upgrade.and_then(|report| report(&self.service)).is_none() {
                eprint!("{lines}");
            }
        }
        while let Some(link) = (self.served.link)(&mut self.service) {
            if self
                .output
                .write_all(&link)
                .and_then(|()| self.output.write_all(b"\n"))
                .and_then(|()| self.output.flush())
                .is_err()
            {
                self.output_failed = true;
                (self.served.shutdown)(&mut self.service);
            }
        }
        while let Some(link) = (self.served.local_link)(&mut self.service) {
            self.keep_link(&link);
        }
        if self.wait_check {
            (self.served.check_wait_boundary)(&self.service, self.now);
        }
        self.watch.ended(Part::Drain, self.clock.now().now);
        if let Some(long) = self.watch.pass_ended() {
            eprint!("{}", long_pass_line(&self.binary, long));
        }
    }
}

pub fn read_handover(
    read: &temper_service::HandoverRead,
) -> Result<Option<skein_lib::Secret>, temper_service::HandoverUnread> {
    use temper_service::HandoverUnread;
    match skein_shell::read_private_file(super::path(&read.path), read.max, skein_shell::effective_user()) {
        Ok(value) => {
            let bytes = value.expose();
            let bytes = bytes.strip_suffix(b"\n").unwrap_or(bytes);
            Ok(Some(skein_lib::Secret::new(bytes.into())))
        }
        Err(skein_shell::Unread::Absent) => Ok(None),
        Err(skein_shell::Unread::NotAFile) => Err(HandoverUnread::NotAFile),
        Err(skein_shell::Unread::TooLarge { .. }) => Err(HandoverUnread::TooLarge),
        Err(skein_shell::Unread::Unsafe(why)) => Err(HandoverUnread::Unsafe(why)),
        Err(skein_shell::Unread::Kernel(error)) => Err(HandoverUnread::Kernel(super::errno(error))),
    }
}
pub fn remove_handover(remove: &temper_service::HandoverRemoval) -> Result<(), i32> {
    match std::fs::remove_file(super::path(&remove.path)) {
        Ok(()) => Ok(()),
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(why) => Err(why.raw_os_error().expect("unlink returns an errno")),
    }
}

/// The composition's listening note, with the client-listen boundary kept beside its words.
pub fn listening_line(service: &mut temper_service::Service, binary: &str) -> Option<(String, bool)> {
    let note = service.listening().pop()?;
    let clients = matches!(&note, temper_service::Note::Listening { listener: temper_service::Listening::Clients, .. });
    Some((note_line(binary, note), clients))
}
pub fn next_note_line(service: &mut temper_service::Service, binary: &str) -> Option<String> {
    service.notes().pop().map(|note| note_line(binary, note))
}
#[must_use]
pub fn service_ending(
    service: &temper_service::Service,
    binary: &str,
    failed: bool,
    directories: &Directories,
    store: &[u8],
) -> (String, u8) {
    ending_with_binary(binary, service.end(), failed, directories, store, service.unsettled())
}

/// A worker alone has no listening diagnostic; its first drain carries its worker notes.
pub fn worker_listening_line(
    service: &mut temper_service::worker_alone::Service,
    binary: &str,
) -> Option<(String, bool)> {
    worker_note_line(service, binary).map(|line| (line, false))
}
pub fn worker_note_line(service: &mut temper_service::worker_alone::Service, binary: &str) -> Option<String> {
    service.notes().pop().map(|note| worker_note_words(binary, note))
}
#[must_use]
pub fn worker_service_ending(
    service: &temper_service::worker_alone::Service,
    binary: &str,
    failed: bool,
    directories: &Directories,
    store: &[u8],
) -> (String, u8) {
    worker_ending_words(binary, service.end(), failed, directories, store)
}
fn worker_note_words(binary: &str, note: temper_service::worker_alone::Note) -> String {
    note_line(binary, note.into())
}
fn worker_ending_words(
    binary: &str,
    end: Option<&temper_service::worker_alone::End>,
    failed: bool,
    directories: &Directories,
    store: &[u8],
) -> (String, u8) {
    let end = end.cloned().map(Into::into);
    ending_with_binary(binary, end.as_ref(), failed, directories, store, &[])
}

/// The terminal stderr and status shared by the shell and its real-loop host.
#[must_use]
pub fn ending_with_binary(
    binary: &str,
    end: Option<&End>,
    output_failed: bool,
    _directories: &Directories,
    store: &[u8],
    unsettled: &[temper_service::Unsettled],
) -> (String, u8) {
    let mut lines = match end {
        Some(End::Stopped(why)) => match why {
            Stop::Start(found) => super::problems(
                binary,
                &temper_service::startup::Problems::jig(found.clone(), &jig_protocol_config::Path::root()),
            ),
            Stop::Room(short) => {
                format!("{binary} serve: restart refused: {}\n", String::from_utf8_lossy(&short.words()))
            }
            Stop::RestorePage { range, failure } => {
                format!("{binary} serve: restore {}: {}\n", restore_kind(*range), page_failure(failure.as_ref()))
            }
            Stop::StartRefused { phase, failure } => {
                let during = phase.map_or_else(|| "unknown phase".into(), start_phase);
                format!("{binary} serve: restart refused during {during}: {}\n", start_failure(failure))
            }
            Stop::ForgeFit { phase, failure } => {
                let during = phase.map_or_else(|| "unknown phase".into(), start_phase);
                let family = match failure.family {
                    Some(kind) => format!(" {kind:?}"),
                    None => String::new(),
                };
                format!(
                    "{binary} serve: restart refused during {during}: forge{family} {:?}, held {}, additional {}, limit {}\n",
                    failure.set, failure.held, failure.additional, failure.limit
                )
            }
            Stop::Setting(key) => {
                let words = temper_engine_domain_forge::settings::retired_words(*key);
                format!("{binary} serve: restart refused: {}\n", String::from_utf8_lossy(&words))
            }
            Stop::Binding => format!("{binary} serve: deployment namespace binding refused\n"),
            Stop::StoreOpen { why } => {
                let deployment = super::path(store);
                return store_refusal(
                    binary,
                    temper_service::refused_start(why, deployment.display().to_string().as_bytes()),
                );
            }
            Stop::StoreRow { row } => {
                let deployment = super::path(store);
                return store_refusal(
                    binary,
                    temper_service::refused_row_start(row, deployment.display().to_string().as_bytes()),
                );
            }
            Stop::FrameTooLarge { number, task, bytes, limit } => format!(
                "{binary} serve: commit {number} refused: task {task}'s transcript frame, {bytes} bytes, past the store's page of {limit} bytes\n"
            ),
            Stop::Journal { why } => {
                let why = match why {
                    temper_service::JournalStop::Refused { commit, sent, durable } => {
                        format!("the store answered commit {commit}, with {sent} sent and {durable} durable")
                    }
                    temper_service::JournalStop::PastEdge { writes } => {
                        format!("a decision past the store's {writes} writes")
                    }
                    temper_service::JournalStop::Counter => "the commit counter at its end".into(),
                };
                format!("{binary} serve: the engine's journal stopped: {why}\n")
            }
            Stop::CommitFailed { number } => format!("{binary} serve: commit {number} failed\n"),
            Stop::Hooks => format!("{binary} serve: webhooks listener failed\n"),
            Stop::Listener { listener, address, error } => listener_failed_line(binary, *listener, *address, *error),
        },
        None | Some(End::Drained | End::Aborted) => String::new(),
    };
    if end == Some(&End::Drained) {
        use std::fmt::Write as _;
        for write in unsettled {
            let (entry, task) = (write.entry, write.task);
            writeln!(
                lines,
                "{binary} serve: stopped before task {task}'s forge write {entry} settled; the next start settles it"
            )
            .expect("format stop diagnostic");
        }
    }
    (lines, u8::from(output_failed || end != Some(&End::Drained)))
}

fn store_refusal(binary: &str, (lines, exit): (Box<[u8]>, u8)) -> (String, u8) {
    use std::fmt::Write as _;
    let lines = String::from_utf8(lines.into_vec()).expect("refusal is UTF-8");
    let mut named = String::new();
    for line in lines.lines() {
        writeln!(named, "{binary} serve: {}", line.strip_prefix("temper serve: ").expect("service refusal prefix"))
            .expect("writing into a string");
    }
    (named, exit)
}

fn restore_kind(kind: temper_service::RestoreRangeKind) -> &'static str {
    use temper_service::RestoreRangeKind as Kind;
    match kind {
        Kind::Deployment => "deployment",
        Kind::Settings => "settings",
        Kind::Accounts => "accounts",
        Kind::Tasks => "tasks",
        Kind::RunProofs => "run proofs",
        Kind::People => "people",
        Kind::Calls => "calls",
        Kind::EndedRead => "ended read position",
        Kind::TaskResult => "task result",
        Kind::RequesterWord => "requester word",
        Kind::Turns => "turns",
        Kind::TaskTranscript => "task transcript",
        Kind::Notes => "notes",
        Kind::ProposalDecision => "proposal decision",
        Kind::TaskProposal => "task proposal",
        Kind::EscalationDecision => "escalation decision",
        Kind::Census => "census",
        Kind::Forge => "forge",
    }
}
fn start_failure(failure: &jig_domain_core::start::Failure) -> String {
    use jig_domain_core::start::Failure;
    match failure {
        Failure::Settings { failure, changed } => {
            let source = if *changed { "changed" } else { "default" };
            format!("{source} setting {:?}: {:?}", failure.setting, failure.fault)
        }
        Failure::ProjectPolicy { project } => format!("project {project}: policy refused"),
        Failure::Application { kind, scope, number, why } => {
            format!("application setting {kind}/{scope}/{number}: {why:?}")
        }
        Failure::HeldPastLimit { which, held, limit } => format!("{which:?}, held {held}, limit {limit}"),
        Failure::AccountsPastLimit { failure } => {
            format!("accounts {:?}, key {:?}, limit {}", failure.set, failure.key, failure.limit)
        }
        Failure::LiveSet { set, key, held, limit } => {
            format!("{set:?}, key {key:?}, held {held}, limit {limit}")
        }
        Failure::Accounting { key } => format!("accounting refused, key {key:?}"),
        Failure::LlmChoice { key } => format!("LLM choice refused, key {key:?}"),
        Failure::Link { record, linked } => {
            use jig_domain_core::start::Linked;
            let target = match linked {
                Linked::Record(key) => format!("{key:?}"),
                Linked::Charter { charter, revision } => format!("charter {charter} at revision {revision}"),
                Linked::Project(project) => format!("project {project}"),
                Linked::Connector(connector) => format!("connector {connector}"),
                Linked::SettingsProposal { proposer, proposal } => format!("settings proposal {proposer:?}/{proposal}"),
                Linked::Holder(resource) => format!("hold owner for {resource:?}"),
            };
            format!("{record:?} links missing {target}")
        }
    }
}
fn page_failure(failure: Option<&jig_protocol_store::restore::Failure>) -> String {
    use std::fmt::Write as _;
    let Some(failure) = failure else { return "store load failed".into() };
    let mut line = format!("{:?}", failure.cause);
    if let Some(row) = &failure.row {
        if let Some(family) = row.family {
            write!(line, ", family {family:?}").expect("format restore family");
        }
        write!(line, ", key {:02x?}", row.key).expect("format restore key");
        if row.truncated {
            write!(line, " (truncated, {} bytes total)", row.key_bytes).expect("format restore key size");
        }
    }
    line
}

fn long_pass_line(binary: &str, long: LongPass) -> String {
    use std::fmt::Write as _;
    let mut line = format!(
        "{binary} serve: long pass: {} ms, {} {} ms of it, past the {} ms bound",
        long.took.as_millis(),
        long.part.name(),
        long.part_took.as_millis(),
        LONG_PASS.as_millis(),
    );
    if long.more != 0 {
        write!(line, "; {} more since the last line", long.more).expect("format long pass count");
    }
    line.push('\n');
    line
}

#[expect(clippy::too_many_lines, reason = "one exhaustive note match keeps each operator line at the host")]
#[must_use]
pub fn note_line(binary: &str, note: temper_service::Note) -> String {
    match note {
        temper_service::Note::CoreWaiting { why } => {
            format!("{binary} serve: the core is not answering: {}\n", String::from_utf8_lossy(&why.words()))
        }
        temper_service::Note::CoreAnswers => format!("{binary} serve: the core answers again\n"),
        temper_service::Note::StartupWarning { line } => {
            format!("{binary} serve: warning: {}\n", String::from_utf8_lossy(&line))
        }
        temper_service::Note::ForgeLink { forge, note } => {
            let words = match note {
                temper_engine_domain_forge::LinkNote::Gone => "their account is gone, the link kept",
                temper_engine_domain_forge::LinkNote::Writer => "refused, temper's writer",
                temper_engine_domain_forge::LinkNote::Replaced => "replaced",
                temper_engine_domain_forge::LinkNote::Cleared => "cleared",
            };
            format!("forge {forge}'s person: {words}\n")
        }
        temper_service::Note::RunFailed(diagnostic) => run_failure_line(
            binary,
            diagnostic.task,
            diagnostic.run,
            diagnostic.attempt,
            diagnostic.failure,
            &diagnostic.detail,
        ),
        temper_service::Note::RunRefused(diagnostic) => run_refusal_line(binary, diagnostic),
        temper_service::Note::JobAnswered(diagnostic) => job_line(binary, diagnostic),
        temper_service::Note::WorkerJob { run, attempt, kind, verb, ending, why } => {
            let verb = verb.escape_ascii();
            let prefix = format!("{binary} worker: run {run} attempt {attempt} job {kind} {verb} ");
            let (class, _) = worker_job_ending(&ending);
            run_detail_line(format!("{prefix}{class}: "), &why)
        }
        temper_service::Note::HelloRefused { channel, text } => {
            format!("{binary} serve: hello refused channel={channel}: {}\n", String::from_utf8_lossy(&text))
        }
        temper_service::Note::ReadRefused { task, attempt, tool, layer, reason, findings, error, job } => {
            let job = match job {
                Some(job) => format!(
                    " head={} run={} job={} job_attempt={}",
                    String::from_utf8_lossy(&temper_engine_domain_forge::commit_text(&job.head)),
                    job.run,
                    job.job,
                    job.attempt
                ),
                None => String::new(),
            };
            format!(
                "{binary} serve: read refused task={task} attempt={attempt} tool={} layer={layer} reason={reason}{job} error={error:?} findings={findings:?}\n",
                String::from_utf8_lossy(&tool)
            )
        }
        temper_service::Note::BriefReadRefused { task, section, stage, reason, heads, error } => {
            let (bound, error) = match error {
                Some(skein_forge::api::Error::TooLarge(excess)) => {
                    let bound = temper_engine_domain_forge::bound_words(excess);
                    let bound = String::from_utf8_lossy(&bound);
                    (format!(" {bound}"), format!(" error=TooLarge: past {bound}"))
                }
                Some(error) => (String::new(), format!(" error={error:?}")),
                None => (String::new(), String::new()),
            };
            let heads = match heads {
                Some((pinned, head)) => format!(
                    " pinned={} head={}",
                    String::from_utf8_lossy(&temper_engine_domain_forge::commit_text(&pinned)),
                    String::from_utf8_lossy(&temper_engine_domain_forge::commit_text(&head)),
                ),
                None => String::new(),
            };
            format!(
                "{binary} serve: brief read refused task={task} section={} stage={stage} reason={reason}{bound}{heads}{error}\n",
                String::from_utf8_lossy(section)
            )
        }
        temper_service::Note::BriefFailed { task, counts, reason } => {
            format!(
                "{binary} serve: brief failed task={task} counts={counts} reason={}\n",
                String::from_utf8_lossy(&reason)
            )
        }
        temper_service::Note::CallAdmissionRefused { task, attempt, tool, layer, reason } => {
            format!(
                "{binary} serve: call admission refused task={task} attempt={attempt} tool={} layer={layer} reason={reason}\n",
                String::from_utf8_lossy(&tool)
            )
        }
        temper_service::Note::Patched { of, patch, counts } => {
            let binary = if binary == "temper" { "temper-box" } else { binary };
            format!(
                "{binary} serve: patched {} {patch}: {} records in {} commits\n",
                crate::upgrade::owner(of),
                counts.records,
                counts.commits
            )
        }
        temper_service::Note::Listening { listener, addr } => {
            let name = match listener {
                temper_service::Listening::Clients => "clients",
                temper_service::Listening::Workers => "workers",
                temper_service::Listening::Webhooks => "webhooks",
            };
            format!("listening {name} {addr}\n")
        }
        temper_service::Note::Linked { worker, address } => {
            format!("linked {} {address}\n", String::from_utf8_lossy(&worker))
        }
        temper_service::Note::Lost { worker, address, why } => {
            format!("lost {} {address} {}\n", String::from_utf8_lossy(&worker), worker_loss(why))
        }
        temper_service::Note::WorkerLost { worker, channel, why } => {
            format!(
                "{binary} serve: worker lost worker={} channel={}: {}\n",
                worker_name(worker.as_deref()),
                channel.raw(),
                engine_worker_loss(why)
            )
        }
        temper_service::Note::WorkerUnsent { worker, channel, unsent } => {
            use jig_protocol_engine_workers::Bound;
            use jig_protocol_worker_wire::Kind;
            let record = match unsent.kind {
                Kind::Hello => "hello",
                Kind::Turn => "turn",
                Kind::Call => "call",
                Kind::Answer => "answer",
                Kind::Fact => "fact",
                Kind::Unauthorized => "unauthorized",
                Kind::Limited => "limited",
                Kind::AccountRefused => "account-refused",
                Kind::Assignment => "assignment",
                Kind::Message => "message",
                Kind::Relayed => "relayed",
                Kind::TurnKept => "turn-kept",
                Kind::TurnBusy => "turn-busy",
                Kind::Acknowledge => "acknowledge",
                Kind::Cancel => "cancel",
                Kind::Grant => "grant",
                Kind::Sources => "sources",
                Kind::Check => "check",
                Kind::ReadList => "read-list",
                Kind::ListRead => "list-read",
                Kind::Draining => "draining",
                Kind::SignOut => "sign-out",
                Kind::Logins => "logins",
                Kind::SignedOut => "signed-out",
                Kind::Job => "job",
                Kind::JobAnswer => "job-answer",
            };
            let bound = match unsent.bound {
                Bound::Room(room) => format!("past the room of {room} bytes"),
                Bound::Channel(why) => {
                    let reason = match why {
                        skein_channel::Unsent::WrongDirection => "wrong-direction",
                        skein_channel::Unsent::PeerDoesNotTake => "not-taken",
                        skein_channel::Unsent::TooLarge => "too-large",
                        skein_channel::Unsent::Closed => "closed",
                        skein_channel::Unsent::Full => "full",
                    };
                    format!("refused: {reason}")
                }
            };
            format!(
                "{binary} serve: worker unsent worker={} channel={}: {record} of {} bytes {bound}\n",
                worker_name(Some(&worker)),
                channel.raw(),
                unsent.bytes
            )
        }
        temper_service::Note::WorkerOpeningEnded { worker, why } => {
            format!(
                "{binary} serve: worker opening ended worker={}: {}\n",
                worker_name(worker.as_deref()),
                engine_worker_loss(why)
            )
        }
        temper_service::Note::WorkerListenerFailed { address, error } => {
            listener_failed_line(binary, temper_service::Listening::Workers, address, error)
        }
        temper_service::Note::Refused { worker, address, reason } => {
            let reason = refusal(reason);
            format!("refused {} {address} {reason}\n", String::from_utf8_lossy(&worker))
        }
        temper_service::Note::ForgeLacking { number, names } => {
            format!("{binary} serve: forge {number}: App lacks {}\n", String::from_utf8_lossy(&names))
        }
        temper_service::Note::SetUpRefused { number, why } => set_up_refused_line(binary, number, why),
        temper_service::Note::Playground { line } | temper_service::Note::BoxLink { line } => {
            format!("{}\n", String::from_utf8_lossy(&line))
        }
        temper_service::Note::ForgeStopped { number, why } => stopped_line(binary, number, why),
        temper_service::Note::ForgeSetupStopped { number, why } => setup_stopped_line(binary, number, why),
        temper_service::Note::ForgeNotConnected { number, why } => {
            format!("{binary} serve: forge {number}: not connected: {why}\n")
        }

        temper_service::Note::ForgeSignIn { number, client, why } => {
            use temper_service::clients::forge_sign_in::Unserved;
            let why = match why {
                Unserved::NotKept => "no such forge is kept".into(),
                Unserved::NotForgejo => "it is not a Forgejo".into(),
                Unserved::Origin { origin } => format!(
                    "its browser origin {} is not client {}'s redirect_uri's",
                    String::from_utf8_lossy(&origin),
                    String::from_utf8_lossy(&client)
                ),
                Unserved::NoSource => "the temper sign-in source is not ready".into(),
                Unserved::NoRoom => "no room for its meta within clients.bundle_bytes".into(),
            };
            format!("{binary} serve: forge {number}: no sign-in through {binary}: {why}\n")
        }
        temper_service::Note::ForgeOrigin { number, origin } => origin_line(binary, number, &origin),
        temper_service::Note::ForgeUnkept { number, installation } => {
            format!("{binary} serve: forge {number}: installation {installation}: not kept\n")
        }
    }
}
// temper/docs/design/domain/worker/README.md, 4.3: one bounded, escaped operator line.
fn run_refusal_line(binary: &str, diagnostic: temper_service::RunRefusalDiagnostic) -> String {
    let temper_service::RunRefusalDiagnostic { task, run, attempt, reason } = diagnostic;
    let (class, detail) = match &reason {
        jig_domain_host::Refusal::Busy => (String::from("Busy"), b"".as_slice()),
        jig_domain_host::Refusal::Invalid(jig_domain_host::Invalid::Start { cause, why }) => {
            (format!("Invalid(Start({cause:?}))"), why.as_ref())
        }
        jig_domain_host::Refusal::Invalid(
            jig_domain_host::Invalid::Charter
            | jig_domain_host::Invalid::Transcript
            | jig_domain_host::Invalid::Grants
            | jig_domain_host::Invalid::Messages,
        ) => (format!("{reason:?}"), b"".as_slice()),
    };
    run_detail_line(format!("{binary} serve: task {task} run {run} attempt {attempt} refused {class}: "), detail)
}

fn run_failure_line(
    binary: &str,
    task: u64,
    run: u64,
    attempt: u64,
    failure: jig_domain_host::Failure,
    detail: &[u8],
) -> String {
    run_detail_line(format!("{binary} serve: task {task} run {run} attempt {attempt} failed {failure:?}: "), detail)
}

// temper/docs/design/domain/worker/jobs.md: the job's kind stays in the run line's detail.
fn job_line(binary: &str, diagnostic: temper_service::JobDiagnostic) -> String {
    let temper_service::JobDiagnostic { task, run, attempt, kind, ending } = diagnostic;
    let prefix = format!("{binary} serve: task {task} run {run} attempt {attempt} ");
    let (class, detail) = job_ending(&ending);
    run_detail_line(format!("{prefix}{class}: job {kind}: "), detail)
}

fn job_ending(ending: &temper_service::JobDiagnosticEnding) -> (String, &[u8]) {
    use jig_domain_host::{JobInvalid, JobRefusal};
    use temper_service::JobDiagnosticEnding;
    match ending {
        JobDiagnosticEnding::Failed { failure, detail } => (format!("failed {failure:?}"), detail),
        JobDiagnosticEnding::Refused(JobRefusal::Invalid(JobInvalid::Start { why })) => {
            (String::from("refused Invalid(Start)"), why)
        }
        JobDiagnosticEnding::Refused(
            reason @ (JobRefusal::Busy
            | JobRefusal::Invalid(
                JobInvalid::Kind | JobInvalid::Spec | JobInvalid::Deadline | JobInvalid::Grants,
            )),
        ) => (format!("refused {reason:?}"), b""),
    }
}

fn worker_job_ending(ending: &temper_service::WorkerJobEnding) -> (String, &[u8]) {
    use jig_domain_host::{JobInvalid, JobRefusal};
    use temper_service::WorkerJobEnding;
    match ending {
        WorkerJobEnding::Failed { failure, detail } => (format!("failed {failure:?}"), detail),
        WorkerJobEnding::Refused(JobRefusal::Invalid(JobInvalid::Start { why })) => {
            (String::from("refused Invalid(Start)"), why)
        }
        WorkerJobEnding::Refused(
            reason @ (JobRefusal::Busy
            | JobRefusal::Invalid(
                JobInvalid::Kind | JobInvalid::Spec | JobInvalid::Deadline | JobInvalid::Grants,
            )),
        ) => (format!("refused {reason:?}"), b""),
    }
}

fn run_detail_line(mut line: String, detail: &[u8]) -> String {
    use std::fmt::Write as _;
    let from = detail.len().saturating_sub(4096);
    for byte in detail.get(from..).expect("bounded diagnostic tail") {
        for escaped in core::ascii::escape_default(*byte) {
            line.write_char(char::from(escaped)).expect("format escaped diagnostic");
        }
    }
    line.push('\n');
    line
}

fn origin_line(binary: &str, number: u16, origin: &skein_forge::github::redirect::Origin) -> String {
    let scheme = if origin.tls { "https" } else { "http" };
    format!(
        "{binary} serve: forge {number}: refused job-log origin {scheme}://{}:{}\n",
        String::from_utf8_lossy(&origin.host),
        origin.port
    )
}
fn set_up_refused_line(binary: &str, number: u16, why: temper_engine_domain_forge::SetUpRefusal) -> String {
    use temper_engine_domain_forge::SetUpRefusal;
    let why = match why {
        SetUpRefusal::NotUnderTheBox => "not under the box",
        SetUpRefusal::NotOffered => "not offered",
        SetUpRefusal::Busy => "busy",
        SetUpRefusal::Invalid => "invalid: the organization, repository or project name is empty or too long",
    };
    format!("{binary} serve: forge {number}: set up refused: {why}\n")
}

fn setup_stopped_line(binary: &str, number: u16, why: temper_service::SetupStopped) -> String {
    use temper_service::{SetupPlan, SetupStopped, SetupWhy};
    let why = match why {
        SetupStopped::NotPrepared => format!("prepare it with `{binary} forge prepare`"),
        SetupStopped::Missing => "setup host missing".into(),
        SetupStopped::Busy => "setup host busy".into(),
        SetupStopped::Removal { path, errno } => {
            format!("removing the handover: {}: errno {errno}", super::path(&path).display())
        }
        SetupStopped::Unreadable { path, why } => {
            format!("the handover is unreadable: {}: {why:?}", super::path(&path).display())
        }
        SetupStopped::Stopped { plan, step } => {
            let reason = match step.why {
                SetupWhy::Refused(status) => format!("refused {status}"),
                why @ (SetupWhy::Missing
                | SetupWhy::NotAdmin
                | SetupWhy::BotIsAdmin
                | SetupWhy::Unmarked
                | SetupWhy::OtherUser
                | SetupWhy::OtherApp
                | SetupWhy::KeyForm
                | SetupWhy::Unknown
                | SetupWhy::Busy
                | SetupWhy::Crowded
                | SetupWhy::Down
                | SetupWhy::Taken
                | SetupWhy::Empty
                | SetupWhy::TimedOut
                | SetupWhy::Unexpected) => format!("{why:?}"),
            };
            let action = match plan {
                SetupPlan::Reconcile => "reconciling",
                SetupPlan::TakeOver => "taking the handover",
                SetupPlan::Adopt => "adopting",
                SetupPlan::Create => "making",
            };
            format!("{action} stopped at {:?}: {reason}", step.stage)
        }
    };
    format!("{binary} serve: forge {number}: Not connected: {why}\n")
}

fn listener_failed_line(
    binary: &str,
    listener: temper_service::Listening,
    address: Option<skein_io::Addr>,
    error: Option<skein_io::Error>,
) -> String {
    let name = match listener {
        temper_service::Listening::Clients => "client",
        temper_service::Listening::Workers => "worker",
        temper_service::Listening::Webhooks => "webhooks",
    };
    let address = match address {
        Some(skein_io::Addr::Ip(socket)) => socket.to_string(),
        Some(skein_io::Addr::InProcess(name)) => String::from_utf8_lossy(name.as_bytes()).into_owned(),
        None => "unknown".into(),
    };
    let why = match error {
        Some(skein_io::Error::Refused) => "refused",
        Some(skein_io::Error::Unreachable) => "unreachable",
        Some(skein_io::Error::TimedOut) => "timed out",
        Some(skein_io::Error::Reset) => "reset",
        Some(skein_io::Error::Busy) => "busy",
        Some(skein_io::Error::Other) => "other",
        None => "closed",
    };
    format!("{binary} serve: {name} listener {address} failed: {why}\n")
}

fn loss_refusal(code: u16) -> String {
    refusal(match code {
        2 => temper_service::Refusal::Terms,
        3 => temper_service::Refusal::Malformed,
        code => temper_service::Refusal::Peer(code),
    })
}

fn worker_name(worker: Option<&[u8]>) -> String {
    match worker {
        Some(name) => String::from_utf8_lossy(name).into_owned(),
        None => "-".into(),
    }
}

fn engine_worker_loss(why: temper_service::EngineWorkerLoss) -> String {
    use temper_service::EngineWorkerLoss as Loss;
    match why {
        Loss::Silence => "silence".into(),
        Loss::Stall => "stall".into(),
        Loss::Full => "full".into(),
        Loss::Deadline => "deadline".into(),
        Loss::Stream => "stream".into(),
        Loss::Framing => "framing".into(),
        Loss::Refused { reason } => format!("refused {}", loss_refusal(reason)),
        Loss::OutputRoom { engine, worker } => {
            format!(
                "engine io.output {engine} bytes differs from worker io.output {worker} bytes; change io.output to agree"
            )
        }
        Loss::RefusedPeer { reason } => format!("refused-by-peer {}", loss_refusal(reason)),
        Loss::NoRoom => "no-room".into(),
        Loss::Stopping => "stopping".into(),
    }
}

fn worker_loss(why: temper_service::WorkerLoss) -> String {
    use temper_service::{WorkerClosed as Closed, WorkerLoss as Loss};
    match why {
        Loss::Closed(Closed::Stream) => "stream".into(),
        Loss::Closed(Closed::Truncated) => "truncated".into(),
        Loss::Closed(Closed::Framing) => "framing".into(),
        Loss::Closed(Closed::Owner) => "owner".into(),
        Loss::Closed(Closed::RefusedHere(reason)) => format!("refused {}", loss_refusal(reason)),
        Loss::Closed(Closed::RefusedPeer(reason)) => format!("refused-by-peer {}", loss_refusal(reason)),
        Loss::Silence => "silence".into(),
        Loss::Stall => "stall".into(),
        Loss::Handshake => "handshake".into(),
        Loss::Full => "full".into(),
        Loss::Busy => "busy".into(),
        Loss::Fault => "fault".into(),
        Loss::Unreachable => "unreachable".into(),
    }
}

fn refusal(reason: temper_service::Refusal) -> String {
    use temper_service::WorkerRefusalReason as Reason;
    match reason {
        temper_service::Refusal::Terms => "terms".into(),
        temper_service::Refusal::Malformed => "malformed".into(),
        temper_service::Refusal::Peer(code) => match Reason::from_code(code) {
            Some(Reason::Unauthorized) => "unauthorized".into(),
            Some(Reason::Application) => "application".into(),
            Some(Reason::Busy) => "busy".into(),
            Some(Reason::Replaced) => "replaced".into(),
            Some(Reason::NotAdmitted) => "not-admitted".into(),
            Some(Reason::Stopping) => "stopping".into(),
            None => format!("code-{code}"),
        },
    }
}
fn stopped_line(binary: &str, number: u16, why: skein_forge_connection::Stopped) -> String {
    use skein_forge_connection::{StartRead, Stopped, Value};
    match why {
        Stopped::Invalid { value } => {
            let member = match value {
                Value::Credential | Value::Token => "api_token",
                Value::User => "user",
                Value::Key => "app_key",
                Value::Secret => "webhook_secret",
            };
            format!("{binary} serve: forge {number}: {member}: invalid\n")
        }
        Stopped::KeyRefused => format!("{binary} serve: forge {number}: app_key: refused\n"),
        Stopped::TokenRefused { installation, status } => {
            format!(
                "{binary} serve: forge {number}: /app/installations/{installation}/access_tokens: status {status}\n"
            )
        }
        Stopped::OtherUser { user } => {
            format!("{binary} serve: forge {number}: start answered as user {user}\n")
        }
        Stopped::Refused { read, status } => {
            let path = match read {
                StartRead::Installations => "/app/installations",
                StartRead::Mint => "/app/installations/{id}/access_tokens",
                StartRead::Viewer => "/graphql viewer",
                StartRead::User => "/api/v1/user",
                StartRead::ApiSettings => "/api/v1/settings/api",
            };
            format!("{binary} serve: forge {number}: {path}: status {status}\n")
        }
    }
}

#[test]
fn github_start_refusals_name_their_paths() {
    use skein_forge_connection::{StartRead, Stopped};
    for (why, words) in [
        (Stopped::KeyRefused, "app_key: refused"),
        (Stopped::TokenRefused { installation: 7, status: 403 }, "/app/installations/7/access_tokens: status 403"),
        (Stopped::Refused { read: StartRead::Installations, status: 401 }, "/app/installations: status 401"),
        (Stopped::Refused { read: StartRead::Mint, status: 401 }, "/app/installations/{id}/access_tokens: status 401"),
        (Stopped::Refused { read: StartRead::Viewer, status: 401 }, "/graphql viewer: status 401"),
    ] {
        assert_eq!(stopped_line("temper", 1, why), format!("temper serve: forge 1: {words}\n"));
    }
}

#[test]
fn records_and_installation_notes_name_the_forge_and_reason() {
    use temper_service::{NotConnected, Note};
    assert_eq!(
        note_line("temper", Note::ForgeNotConnected { number: 7, why: NotConnected::Unreadable }),
        "temper serve: forge 7: not connected: its records are unreadable\n"
    );
    assert_eq!(
        note_line(
            "temper",
            Note::ForgeNotConnected {
                number: 7,
                why: NotConnected::OtherForge {
                    kind: skein_forge_connection::settings::Kind::Forgejo,
                    api: Box::from("fake.test:443")
                }
            }
        ),
        "temper serve: forge 7: not connected: its records were made for forgejo at fake.test:443\n"
    );

    assert_eq!(
        note_line("temper", Note::ForgeUnkept { number: 7, installation: 93 }),
        "temper serve: forge 7: installation 93: not kept\n"
    );
}

#[test]
fn forge_sign_in_notes_name_each_unserved_reason() {
    use temper_service::{Note, clients::forge_sign_in::Unserved};
    for (why, words) in [
        (Unserved::NotKept, "no such forge is kept"),
        (Unserved::NotForgejo, "it is not a Forgejo"),
        (
            Unserved::Origin { origin: b"https://git.example.com".as_slice().into() },
            "its browser origin https://git.example.com is not client forgejo's redirect_uri's",
        ),
        (Unserved::NoRoom, "no room for its meta within clients.bundle_bytes"),
    ] {
        assert_eq!(
            note_line("temper", Note::ForgeSignIn { number: 7, client: b"forgejo".as_slice().into(), why }),
            format!("temper serve: forge 7: no sign-in through temper: {words}\n")
        );
    }
}

#[test]
fn older_store_refusals_use_jigs_recovery_words_and_the_store_path() {
    use jig_protocol_store::{Family, Older, OpenRefusal};
    let arguments =
        [b"--config".as_slice(), b"/configuration", b"--data", b"/data", b"--cache", b"/cache"].map(Box::from);
    let (directories, _) = temper_shell::directories::resolve(&arguments, &[], b"/").expect("resolved directories");
    for older in [Older::Unleveled, Older::Unrecorded, Older::Layout(Family::Application), Older::Directory] {
        let why = OpenRefusal::OlderBuild { why: older };
        for store in ["/data/deployment", "/elsewhere/store"] {
            let expected = format!("temper serve: {}\n", why.words().join(store));
            let (line, code) = ending_with_binary(
                "temper",
                Some(&End::Stopped(Stop::StoreOpen { why: why.clone() })),
                false,
                &directories,
                store.as_bytes(),
                &[],
            );
            assert_eq!(line, expected);
            assert!(line.contains("this start has changed nothing"));
            assert!(line.contains("together"));
            assert!(line.contains("every client signs in again"));
            assert_eq!(code, 1);
        }
    }
}

#[test]
fn other_endings_keep_their_exit_behavior() {
    let arguments =
        [b"--config".as_slice(), b"/configuration", b"--data", b"/data", b"--cache", b"/cache"].map(Box::from);
    let (directories, _) = temper_shell::directories::resolve(&arguments, &[], b"/").expect("resolved directories");
    let mut problems = jig_protocol_config::Problems::empty();
    problems.push(jig_protocol_config::Path::root().member(b"deployment"), jig_protocol_config::What::Refers);
    for (end, output_failed, expected, code) in [
        (
            Some(End::Stopped(Stop::RestorePage { range: temper_service::RestoreRangeKind::Census, failure: None })),
            false,
            "temper serve: restore census: store load failed\n",
            1,
        ),
        (Some(End::Stopped(Stop::Start(problems))), false, "temper serve: deployment: unknown reference\n", 1),
        (
            Some(End::Stopped(Stop::Room(temper_engine_domain::room::Short {
                charter: 3,
                room: 2_047,
                bound: 51_717,
            }))),
            false,
            "temper serve: restart refused: charter 3 leaves its brief 2,047 bytes of its host part's 51,717; it needs at least 2,048\n",
            1,
        ),
        (Some(End::Stopped(Stop::Binding)), false, "temper serve: deployment namespace binding refused\n", 1),
        (Some(End::Stopped(Stop::CommitFailed { number: 7 })), false, "temper serve: commit 7 failed\n", 1),
        (
            Some(End::Stopped(Stop::Journal {
                why: temper_service::JournalStop::Refused { commit: 7, sent: 9, durable: 8 },
            })),
            false,
            "temper serve: the engine's journal stopped: the store answered commit 7, with 9 sent and 8 durable\n",
            1,
        ),
        (
            Some(End::Stopped(Stop::Journal { why: temper_service::JournalStop::PastEdge { writes: 32 } })),
            false,
            "temper serve: the engine's journal stopped: a decision past the store's 32 writes\n",
            1,
        ),
        (
            Some(End::Stopped(Stop::Journal { why: temper_service::JournalStop::Counter })),
            false,
            "temper serve: the engine's journal stopped: the commit counter at its end\n",
            1,
        ),
        (
            Some(End::Stopped(Stop::FrameTooLarge { number: 7, task: 11, bytes: 79, limit: 78 })),
            false,
            "temper serve: commit 7 refused: task 11's transcript frame, 79 bytes, past the store's page of 78 bytes\n",
            1,
        ),
        (Some(End::Stopped(Stop::Hooks)), false, "temper serve: webhooks listener failed\n", 1),
        (
            Some(End::Stopped(Stop::Listener {
                listener: temper_service::Listening::Clients,
                address: Some(skein_io::Addr::Ip(([127, 0, 0, 1], 7100).into())),
                error: Some(skein_io::Error::Other),
            })),
            false,
            "temper serve: client listener 127.0.0.1:7100 failed: other\n",
            1,
        ),
        (
            Some(End::Stopped(Stop::Listener {
                listener: temper_service::Listening::Workers,
                address: Some(skein_io::Addr::InProcess(
                    skein_io::Name::new(b"workers").expect("configured worker listener"),
                )),
                error: None,
            })),
            false,
            "temper serve: worker listener workers failed: closed\n",
            1,
        ),
        (Some(End::Drained), false, "", 0),
        (Some(End::Drained), true, "", 1),
        (Some(End::Aborted), false, "", 1),
        (None, false, "", 1),
    ] {
        assert_eq!(
            ending_with_binary("temper", end.as_ref(), output_failed, &directories, b"/data/store", &[]),
            (expected.into(), code),
            "{end:?}, output failed {output_failed}"
        );
    }
}

#[test]
fn listening_notes_print_each_listener_and_socket_address_exactly() {
    use temper_service::{Listening, Note};
    for (listener, name) in
        [(Listening::Clients, "clients"), (Listening::Workers, "workers"), (Listening::Webhooks, "webhooks")]
    {
        for (addr, printed) in [
            (([127, 0, 0, 1], 41733).into(), "127.0.0.1:41733"),
            (([0, 0, 0, 0, 0, 0, 0, 1], 41733).into(), "[::1]:41733"),
        ] {
            assert_eq!(
                note_line("temper", Note::Listening { listener, addr }),
                format!("listening {name} {printed}\n")
            );
        }
    }
}

#[test]
fn startup_warnings_print_the_configuration_path_and_figures() {
    let line = b"engine.charters[1].reserve_time_ms: needs checks within 60000 ms; the agent's profile.policy.check_timeout is 90000 ms; raise charter 2's reserve_time (temper help settings) or lower the timeout";
    assert_eq!(
        note_line("temper", temper_service::Note::StartupWarning { line: line.as_slice().into() }),
        "temper serve: warning: engine.charters[1].reserve_time_ms: needs checks within 60000 ms; the agent's profile.policy.check_timeout is 90000 ms; raise charter 2's reserve_time (temper help settings) or lower the timeout\n"
    );
}

#[test]
fn hello_refusal_notes_print_the_channel_and_words_exactly() {
    assert_eq!(
        note_line(
            "temper",
            temper_service::Note::HelloRefused {
                channel: 7,
                text: b"charter 2: needs checks within 60000 ms; the agent's profile.policy.check_timeout is 90000 ms; raise charter 2's reserve_time (temper help settings) or lower the timeout".as_slice().into(),
            }
        ),
        "temper serve: hello refused channel=7: charter 2: needs checks within 60000 ms; the agent's profile.policy.check_timeout is 90000 ms; raise charter 2's reserve_time (temper help settings) or lower the timeout\n",
    );
}

#[test]
fn worker_link_notes_print_each_address_and_refusal_exactly() {
    use temper_service::{Note, Refusal};
    for (address, printed) in
        [(([127, 0, 0, 1], 7100).into(), "127.0.0.1:7100"), (([0, 0, 0, 0, 0, 0, 0, 1], 7100).into(), "[::1]:7100")]
    {
        assert_eq!(
            note_line("temper", Note::Linked { worker: b"worker-a".as_slice().into(), address: Box::new(address) }),
            format!("linked worker-a {printed}\n")
        );
        for (reason, expected) in [
            (Refusal::Peer(256), "unauthorized"),
            (Refusal::Peer(257), "application"),
            (Refusal::Peer(258), "busy"),
            (Refusal::Peer(259), "replaced"),
            (Refusal::Peer(260), "not-admitted"),
            (Refusal::Peer(261), "stopping"),
            (Refusal::Terms, "terms"),
            (Refusal::Malformed, "malformed"),
            (Refusal::Peer(42), "code-42"),
        ] {
            assert_eq!(
                note_line(
                    "temper",
                    Note::Refused { worker: b"worker-a".as_slice().into(), address: Box::new(address), reason }
                ),
                format!("refused worker-a {printed} {expected}\n")
            );
        }
    }
}

#[test]
fn worker_loss_lines_name_each_cause_and_refusal_exactly() {
    use temper_service::{Note, WorkerClosed as Closed, WorkerLoss as Loss};
    for (why, expected) in [
        (Loss::Closed(Closed::Stream), "lost worker-a [::1]:7100 stream\n"),
        (Loss::Closed(Closed::Truncated), "lost worker-a [::1]:7100 truncated\n"),
        (Loss::Closed(Closed::Framing), "lost worker-a [::1]:7100 framing\n"),
        (Loss::Closed(Closed::Owner), "lost worker-a [::1]:7100 owner\n"),
        (Loss::Silence, "lost worker-a [::1]:7100 silence\n"),
        (Loss::Stall, "lost worker-a [::1]:7100 stall\n"),
        (Loss::Handshake, "lost worker-a [::1]:7100 handshake\n"),
        (Loss::Full, "lost worker-a [::1]:7100 full\n"),
        (Loss::Busy, "lost worker-a [::1]:7100 busy\n"),
        (Loss::Fault, "lost worker-a [::1]:7100 fault\n"),
        (Loss::Unreachable, "lost worker-a [::1]:7100 unreachable\n"),
        (Loss::Closed(Closed::RefusedHere(2)), "lost worker-a [::1]:7100 refused terms\n"),
        (Loss::Closed(Closed::RefusedHere(3)), "lost worker-a [::1]:7100 refused malformed\n"),
        (Loss::Closed(Closed::RefusedHere(256)), "lost worker-a [::1]:7100 refused unauthorized\n"),
        (Loss::Closed(Closed::RefusedHere(257)), "lost worker-a [::1]:7100 refused application\n"),
        (Loss::Closed(Closed::RefusedHere(258)), "lost worker-a [::1]:7100 refused busy\n"),
        (Loss::Closed(Closed::RefusedHere(259)), "lost worker-a [::1]:7100 refused replaced\n"),
        (Loss::Closed(Closed::RefusedHere(260)), "lost worker-a [::1]:7100 refused not-admitted\n"),
        (Loss::Closed(Closed::RefusedHere(261)), "lost worker-a [::1]:7100 refused stopping\n"),
        (Loss::Closed(Closed::RefusedHere(42)), "lost worker-a [::1]:7100 refused code-42\n"),
        (Loss::Closed(Closed::RefusedPeer(2)), "lost worker-a [::1]:7100 refused-by-peer terms\n"),
        (Loss::Closed(Closed::RefusedPeer(3)), "lost worker-a [::1]:7100 refused-by-peer malformed\n"),
        (Loss::Closed(Closed::RefusedPeer(256)), "lost worker-a [::1]:7100 refused-by-peer unauthorized\n"),
        (Loss::Closed(Closed::RefusedPeer(257)), "lost worker-a [::1]:7100 refused-by-peer application\n"),
        (Loss::Closed(Closed::RefusedPeer(258)), "lost worker-a [::1]:7100 refused-by-peer busy\n"),
        (Loss::Closed(Closed::RefusedPeer(259)), "lost worker-a [::1]:7100 refused-by-peer replaced\n"),
        (Loss::Closed(Closed::RefusedPeer(260)), "lost worker-a [::1]:7100 refused-by-peer not-admitted\n"),
        (Loss::Closed(Closed::RefusedPeer(261)), "lost worker-a [::1]:7100 refused-by-peer stopping\n"),
        (Loss::Closed(Closed::RefusedPeer(42)), "lost worker-a [::1]:7100 refused-by-peer code-42\n"),
    ] {
        assert_eq!(
            note_line(
                "temper",
                Note::Lost {
                    worker: b"worker-a".as_slice().into(),
                    address: Box::new(([0, 0, 0, 0, 0, 0, 0, 1], 7100).into()),
                    why,
                }
            ),
            expected,
            "{why:?}"
        );
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one table pins every engine loss and refusal for both a lost channel and an opening"
)]
fn engine_worker_loss_lines_name_each_cause_and_refusal_exactly() {
    use temper_service::{EngineWorkerLoss as Loss, Note};
    for (why, lost, opening) in [
        (
            Loss::OutputRoom { engine: 1_048_576, worker: 18_432_634 },
            "temper serve: worker lost worker=worker-a channel=7: engine io.output 1048576 bytes differs from worker io.output 18432634 bytes; change io.output to agree\n",
            "temper serve: worker opening ended worker=-: engine io.output 1048576 bytes differs from worker io.output 18432634 bytes; change io.output to agree\n",
        ),
        (
            Loss::Silence,
            "temper serve: worker lost worker=worker-a channel=7: silence\n",
            "temper serve: worker opening ended worker=-: silence\n",
        ),
        (
            Loss::Stall,
            "temper serve: worker lost worker=worker-a channel=7: stall\n",
            "temper serve: worker opening ended worker=-: stall\n",
        ),
        (
            Loss::Full,
            "temper serve: worker lost worker=worker-a channel=7: full\n",
            "temper serve: worker opening ended worker=-: full\n",
        ),
        (
            Loss::Deadline,
            "temper serve: worker lost worker=worker-a channel=7: deadline\n",
            "temper serve: worker opening ended worker=-: deadline\n",
        ),
        (
            Loss::Stream,
            "temper serve: worker lost worker=worker-a channel=7: stream\n",
            "temper serve: worker opening ended worker=-: stream\n",
        ),
        (
            Loss::Framing,
            "temper serve: worker lost worker=worker-a channel=7: framing\n",
            "temper serve: worker opening ended worker=-: framing\n",
        ),
        (
            Loss::NoRoom,
            "temper serve: worker lost worker=worker-a channel=7: no-room\n",
            "temper serve: worker opening ended worker=-: no-room\n",
        ),
        (
            Loss::Stopping,
            "temper serve: worker lost worker=worker-a channel=7: stopping\n",
            "temper serve: worker opening ended worker=-: stopping\n",
        ),
        (
            Loss::Refused { reason: 2 },
            "temper serve: worker lost worker=worker-a channel=7: refused terms\n",
            "temper serve: worker opening ended worker=-: refused terms\n",
        ),
        (
            Loss::Refused { reason: 3 },
            "temper serve: worker lost worker=worker-a channel=7: refused malformed\n",
            "temper serve: worker opening ended worker=-: refused malformed\n",
        ),
        (
            Loss::Refused { reason: 256 },
            "temper serve: worker lost worker=worker-a channel=7: refused unauthorized\n",
            "temper serve: worker opening ended worker=-: refused unauthorized\n",
        ),
        (
            Loss::Refused { reason: 257 },
            "temper serve: worker lost worker=worker-a channel=7: refused application\n",
            "temper serve: worker opening ended worker=-: refused application\n",
        ),
        (
            Loss::Refused { reason: 258 },
            "temper serve: worker lost worker=worker-a channel=7: refused busy\n",
            "temper serve: worker opening ended worker=-: refused busy\n",
        ),
        (
            Loss::Refused { reason: 259 },
            "temper serve: worker lost worker=worker-a channel=7: refused replaced\n",
            "temper serve: worker opening ended worker=-: refused replaced\n",
        ),
        (
            Loss::Refused { reason: 260 },
            "temper serve: worker lost worker=worker-a channel=7: refused not-admitted\n",
            "temper serve: worker opening ended worker=-: refused not-admitted\n",
        ),
        (
            Loss::Refused { reason: 261 },
            "temper serve: worker lost worker=worker-a channel=7: refused stopping\n",
            "temper serve: worker opening ended worker=-: refused stopping\n",
        ),
        (
            Loss::Refused { reason: 42 },
            "temper serve: worker lost worker=worker-a channel=7: refused code-42\n",
            "temper serve: worker opening ended worker=-: refused code-42\n",
        ),
        (
            Loss::RefusedPeer { reason: 2 },
            "temper serve: worker lost worker=worker-a channel=7: refused-by-peer terms\n",
            "temper serve: worker opening ended worker=-: refused-by-peer terms\n",
        ),
        (
            Loss::RefusedPeer { reason: 3 },
            "temper serve: worker lost worker=worker-a channel=7: refused-by-peer malformed\n",
            "temper serve: worker opening ended worker=-: refused-by-peer malformed\n",
        ),
        (
            Loss::RefusedPeer { reason: 256 },
            "temper serve: worker lost worker=worker-a channel=7: refused-by-peer unauthorized\n",
            "temper serve: worker opening ended worker=-: refused-by-peer unauthorized\n",
        ),
        (
            Loss::RefusedPeer { reason: 257 },
            "temper serve: worker lost worker=worker-a channel=7: refused-by-peer application\n",
            "temper serve: worker opening ended worker=-: refused-by-peer application\n",
        ),
        (
            Loss::RefusedPeer { reason: 258 },
            "temper serve: worker lost worker=worker-a channel=7: refused-by-peer busy\n",
            "temper serve: worker opening ended worker=-: refused-by-peer busy\n",
        ),
        (
            Loss::RefusedPeer { reason: 259 },
            "temper serve: worker lost worker=worker-a channel=7: refused-by-peer replaced\n",
            "temper serve: worker opening ended worker=-: refused-by-peer replaced\n",
        ),
        (
            Loss::RefusedPeer { reason: 260 },
            "temper serve: worker lost worker=worker-a channel=7: refused-by-peer not-admitted\n",
            "temper serve: worker opening ended worker=-: refused-by-peer not-admitted\n",
        ),
        (
            Loss::RefusedPeer { reason: 261 },
            "temper serve: worker lost worker=worker-a channel=7: refused-by-peer stopping\n",
            "temper serve: worker opening ended worker=-: refused-by-peer stopping\n",
        ),
        (
            Loss::RefusedPeer { reason: 42 },
            "temper serve: worker lost worker=worker-a channel=7: refused-by-peer code-42\n",
            "temper serve: worker opening ended worker=-: refused-by-peer code-42\n",
        ),
    ] {
        assert_eq!(
            note_line(
                "temper",
                Note::WorkerLost {
                    worker: Some(b"worker-a".as_slice().into()),
                    channel: skein_lib::Token::new(7),
                    why,
                }
            ),
            lost,
            "{why:?}"
        );
        assert_eq!(note_line("temper", Note::WorkerOpeningEnded { worker: None, why }), opening, "{why:?}");
    }
    assert_eq!(
        note_line(
            "temper",
            Note::WorkerOpeningEnded { worker: Some(b"worker-a".as_slice().into()), why: Loss::Silence }
        ),
        "temper serve: worker opening ended worker=worker-a: silence\n"
    );
}

#[test]
fn worker_unsent_lines_name_the_record_bytes_and_bound() {
    use jig_protocol_engine_workers::{Bound, Unsent};
    use jig_protocol_worker_wire::Kind;
    for (kind, bytes, bound, expected) in [
        (
            Kind::Assignment,
            1_300_000,
            Bound::Room(1_048_576),
            "temper serve: worker unsent worker=worker-a channel=7: assignment of 1300000 bytes past the room of 1048576 bytes\n",
        ),
        (
            Kind::Message,
            4_200,
            Bound::Channel(skein_channel::Unsent::TooLarge),
            "temper serve: worker unsent worker=worker-a channel=7: message of 4200 bytes refused: too-large\n",
        ),
    ] {
        assert_eq!(
            note_line(
                "temper",
                temper_service::Note::WorkerUnsent {
                    worker: b"worker-a".as_slice().into(),
                    channel: skein_lib::Token::new(7),
                    unsent: Unsent { kind, bytes, bound },
                }
            ),
            expected
        );
    }
}

#[test]
fn worker_listener_failures_print_the_exit_lines_words_at_once() {
    use temper_service::Note;
    for (address, error, expected) in [
        (
            Some(skein_io::Addr::Ip(([127, 0, 0, 1], 7100).into())),
            Some(skein_io::Error::Unreachable),
            "temper serve: worker listener 127.0.0.1:7100 failed: unreachable\n",
        ),
        (
            Some(skein_io::Addr::InProcess(skein_io::Name::new(b"workers").unwrap())),
            None,
            "temper serve: worker listener workers failed: closed\n",
        ),
    ] {
        assert_eq!(note_line("temper", Note::WorkerListenerFailed { address, error }), expected);
    }
}

#[test]
fn setup_stops_print_the_forge_plan_and_typed_step_without_a_subject() {
    use temper_service::{Note, SetupPlan, SetupStage, SetupStep, SetupStopped, SetupWhy};
    for (why, line) in [
        (SetupStopped::NotPrepared, "prepare it with `temper forge prepare`"),
        (SetupStopped::Missing, "setup host missing"),
        (SetupStopped::Busy, "setup host busy"),
        (
            SetupStopped::Stopped {
                plan: SetupPlan::Reconcile,
                step: SetupStep { stage: SetupStage::Repository, why: SetupWhy::Refused(403) },
            },
            "reconciling stopped at Repository: refused 403",
        ),
    ] {
        assert_eq!(
            note_line("temper", Note::ForgeSetupStopped { number: 3, why }),
            format!("temper serve: forge 3: Not connected: {line}\n")
        );
    }
}

#[test]
fn the_shell_removes_only_the_named_handover_and_reports_unlink_failure() {
    use std::os::unix::ffi::OsStrExt;
    let directory =
        std::env::temp_dir().join(format!("temper-handover-{}", skein_shell::seed().expect("scratch name")));
    std::fs::create_dir(&directory).expect("handover fixture");
    let path = directory.join("admin_token");
    let removal = temper_service::HandoverRemoval {
        forge: 1,
        run: skein_lib::Token::new(7),
        path: path.as_os_str().as_bytes().into(),
    };
    assert!(remove_handover(&removal).is_ok(), "an absent handover counts as removed");
    std::fs::create_dir(&path).expect("handover fixture");
    std::fs::write(path.join("held"), b"").expect("handover fixture");
    let errno = remove_handover(&removal).expect_err("an unlink failure is never acknowledged as removed");
    let line = note_line(
        "temper",
        temper_service::Note::ForgeSetupStopped {
            number: removal.forge,
            why: temper_service::SetupStopped::Removal { path: removal.path.clone(), errno },
        },
    );
    assert!(line.contains("removing the handover:") && line.contains(path.to_str().expect("handover fixture")));
    std::fs::remove_dir_all(&path).expect("handover fixture");
    std::fs::write(&path, b"").expect("handover fixture");
    assert!(remove_handover(&removal).is_ok());
    assert!(!path.exists());
    std::fs::remove_dir(&directory).expect("handover fixture");
}
#[test]
fn the_shell_trims_only_the_handovers_final_newline() {
    use std::os::unix::{ffi::OsStrExt, fs::PermissionsExt};
    let directory =
        std::env::temp_dir().join(format!("temper-handover-read-{}", skein_shell::seed().expect("scratch name")));
    std::fs::create_dir(&directory).expect("handover fixture");
    let path = directory.join("admin_token");
    let read = temper_service::HandoverRead {
        forge: 1,
        run: skein_lib::Token::new(7),
        path: path.as_os_str().as_bytes().into(),
        max: 11,
    };
    for (bytes, expected) in [
        (b"test-admin\n".as_slice(), b"test-admin".as_slice()),
        (b"test-admin".as_slice(), b"test-admin".as_slice()),
        (b"token\r\n".as_slice(), b"token\r".as_slice()),
        (b"one\ntwo".as_slice(), b"one\ntwo".as_slice()),
        (b"".as_slice(), b"".as_slice()),
        (b"12345678901".as_slice(), b"12345678901".as_slice()),
    ] {
        std::fs::write(&path, bytes).expect("handover fixture");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).expect("private fixture");
        let answer = read_handover(&read).expect("within read bound").expect("present fixture");
        assert!(answer.expose() == expected, "only one final LF is removed");
    }
    std::fs::write(&path, b"123456789012").expect("one byte beyond the bound");
    assert!(matches!(read_handover(&read), Err(temper_service::HandoverUnread::TooLarge)));
    std::fs::remove_dir_all(directory).expect("handover fixture");
}

#[test]
fn a_drained_end_prints_each_unsettled_write() {
    let arguments =
        [b"--config".as_slice(), b"/configuration", b"--data", b"/data", b"--cache", b"/cache"].map(Box::from);
    let (directories, _) = temper_shell::directories::resolve(&arguments, &[], b"/").expect("resolved directories");
    let unfinished =
        [temper_service::Unsettled { entry: 17, task: 3 }, temper_service::Unsettled { entry: 22, task: 9 }];
    assert_eq!(ending_with_binary("temper", Some(&End::Drained), false, &directories, b"/data/store", &unfinished), (
        "temper serve: stopped before task 3's forge write 17 settled; the next start settles it\ntemper serve: stopped before task 9's forge write 22 settled; the next start settles it\n".into(), 0
    ));
}

fn start_phase(phase: jig_domain_core::start::Phase) -> String {
    use jig_domain_core::start::Phase;
    match phase {
        Phase::LoadCore => "core restore".into(),
        Phase::LoadInbox => "inbox census".into(),
        Phase::RestoreConnector { connector } => format!("connector {connector} restore"),
        Phase::Link => "link".into(),
        Phase::Sweep => "sweep".into(),
        Phase::Wires => "wires".into(),
        Phase::Switch => "store switch".into(),
        Phase::AdoptRuns => "run adoption".into(),
        Phase::ReadAfresh { connector } => format!("connector {connector} refresh"),
        Phase::SettleOutbox { connector } => format!("connector {connector} outbox settlement"),
        Phase::Open => "opening decisions".into(),
    }
}

#[test]
fn forge_fit_endings_name_phase_family_set_and_measured_limit() {
    use temper_engine_domain_forge::{RecordKind, RestoreFit, RestoreSet};
    let arguments =
        [b"--config".as_slice(), b"/configuration", b"--data", b"/data", b"--cache", b"/cache"].map(Box::from);
    let (directories, _) = temper_shell::directories::resolve(&arguments, &[], b"/").expect("resolved directories");
    for (family, set, held, additional, limit, detail) in [
        (
            Some(RecordKind::Binding),
            RestoreSet::Rows(RecordKind::Binding),
            4,
            1,
            4,
            "forge Binding Rows(Binding), held 4, additional 1, limit 4",
        ),
        (None, RestoreSet::LandingFlights, 9, 0, 8, "forge LandingFlights, held 9, additional 0, limit 8"),
    ] {
        let end = End::Stopped(Stop::ForgeFit {
            phase: Some(jig_domain_core::start::Phase::RestoreConnector { connector: 7 }),
            failure: RestoreFit { family, set, held, additional, limit },
        });
        let (line, code) = ending_with_binary("temper", Some(&end), false, &directories, b"/data/deployment", &[]);
        assert_eq!(line, format!("temper serve: restart refused during connector 7 restore: {detail}\n"));
        assert_eq!(code, 1);
    }
}

#[test]
fn start_refusals_name_every_phase_and_the_refused_project() {
    use jig_domain_core::start::{Failure, Phase};
    let arguments =
        [b"--config".as_slice(), b"/configuration", b"--data", b"/data", b"--cache", b"/cache"].map(Box::from);
    let (directories, _) = temper_shell::directories::resolve(&arguments, &[], b"/").expect("resolved directories");
    for (phase, words) in [
        (Some(Phase::LoadCore), "core restore"),
        (Some(Phase::LoadInbox), "inbox census"),
        (Some(Phase::RestoreConnector { connector: 7 }), "connector 7 restore"),
        (Some(Phase::Link), "link"),
        (Some(Phase::Sweep), "sweep"),
        (Some(Phase::Switch), "store switch"),
        (Some(Phase::Wires), "wires"),
        (Some(Phase::AdoptRuns), "run adoption"),
        (Some(Phase::ReadAfresh { connector: 7 }), "connector 7 refresh"),
        (Some(Phase::SettleOutbox { connector: 7 }), "connector 7 outbox settlement"),
        (Some(Phase::Open), "opening decisions"),
        (None, "unknown phase"),
    ] {
        let end = End::Stopped(Stop::StartRefused { phase, failure: Failure::ProjectPolicy { project: 3 } });
        assert_eq!(
            ending_with_binary("temper", Some(&end), false, &directories, b"/data/deployment", &[]),
            (format!("temper serve: restart refused during {words}: project 3: policy refused\n"), 1)
        );
    }
}

#[test]
fn start_failures_name_the_set_key_setting_and_measured_limit() {
    use jig_domain_core::{
        HeldLimit, Key, settings,
        start::{Failure, LiveSet},
    };
    let key = Key::Tasks(jig_domain_tasks::Key::Live(17));
    for (failure, words) in [
        (
            Failure::LiveSet {
                set: LiveSet::TasksBound(jig_domain_tasks::RestoreBound::Tasks),
                key: key.clone(),
                held: 5,
                limit: 4,
            },
            "TasksBound(Tasks), key Tasks(Live(17)), held 5, limit 4",
        ),
        (Failure::HeldPastLimit { which: HeldLimit::Revisions, held: 5, limit: 4 }, "Revisions, held 5, limit 4"),
        (Failure::HeldPastLimit { which: HeldLimit::Versions, held: 5, limit: 4 }, "Versions, held 5, limit 4"),
        (
            Failure::Settings {
                failure: settings::Failure {
                    setting: settings::Setting::Policy(1),
                    fault: settings::Fault::Shape,
                    detail: None,
                },
                changed: true,
            },
            "changed setting Policy(1): Shape",
        ),
        (
            Failure::Settings {
                failure: settings::Failure {
                    setting: settings::Setting::Rules,
                    fault: settings::Fault::Reference,
                    detail: None,
                },
                changed: false,
            },
            "default setting Rules: Reference",
        ),
        (Failure::Accounting { key: key.clone() }, "accounting refused, key Tasks(Live(17))"),
        (
            Failure::Link {
                record: Key::Tasks(jig_domain_tasks::Key::Live(199)),
                linked: jig_domain_core::Linked::Record(Key::People(jig_domain_people::Key::Person(102))),
            },
            "Tasks(Live(199)) links missing People(Person(102))",
        ),
        (
            Failure::Link { record: key.clone(), linked: jig_domain_core::Linked::Charter { charter: 3, revision: 2 } },
            "Tasks(Live(17)) links missing charter 3 at revision 2",
        ),
        (
            Failure::Link { record: key.clone(), linked: jig_domain_core::Linked::Project(9) },
            "Tasks(Live(17)) links missing project 9",
        ),
        (
            Failure::Link { record: key.clone(), linked: jig_domain_core::Linked::Connector(9) },
            "Tasks(Live(17)) links missing connector 9",
        ),
        (
            Failure::Link {
                record: key.clone(),
                linked: jig_domain_core::Linked::SettingsProposal {
                    proposer: jig_domain_people::Proposer::Party(1),
                    proposal: 4_294_967_296,
                },
            },
            "Tasks(Live(17)) links missing settings proposal Party(1)/4294967296",
        ),
        (
            Failure::Link {
                record: Key::Tasks(jig_domain_tasks::Key::Writer(1)),
                linked: jig_domain_core::Linked::Holder(jig_domain_tasks::Name { connector: 1, path: Box::new([]) }),
            },
            "Tasks(Writer(1)) links missing hold owner for Name { connector: 1, path: [] }",
        ),
        (Failure::LlmChoice { key }, "LLM choice refused, key Tasks(Live(17))"),
    ] {
        assert_eq!(start_failure(&failure), words);
    }
}

#[test]
fn unreadable_temper_rows_end_with_the_named_recovery_words_and_exit_one() {
    let arguments =
        [b"--config".as_slice(), b"/configuration", b"--data", b"/data", b"--cache", b"/cache"].map(Box::from);
    let (directories, _) = temper_shell::directories::resolve(&arguments, &[], b"/").expect("resolved directories");
    for (tag, name) in [(21, "Flight"), (9, "Landed"), (20, "Setting")] {
        let end = End::Stopped(Stop::StoreRow { row: temper_service::UnreadableRow { key: Box::from([tag]) } });
        for deployment in ["/data/deployment", "/elsewhere/store"] {
            for binary in ["temper", "temper-engine"] {
                assert_eq!(
                    ending_with_binary(binary, Some(&end), false, &directories, deployment.as_bytes(), &[]),
                    (
                        format!(
                            "{binary} serve: The store in {deployment} holds a {name} record, family {tag}, that this build cannot read, and the start stopped there. Start the build that wrote it again, or, to start afresh, stop the service, move `active`, `generations/` and `transcripts/` out of {deployment} together, and start again: nothing is deleted, and the next start makes a new, empty store.\n"
                        ),
                        1
                    )
                );
            }
        }
    }
}

#[test]
fn restore_pages_name_only_the_store_supplied_cause_family_and_key() {
    use jig_protocol_store::{
        Family,
        restore::{Cause, DecodeFailure, Failure, RangeFailure, RowSite},
    };
    let arguments =
        [b"--config".as_slice(), b"/configuration", b"--data", b"/data", b"--cache", b"/cache"].map(Box::from);
    let (directories, _) = temper_shell::directories::resolve(&arguments, &[], b"/").expect("resolved directories");
    for (failure, words) in [
        (None, "store load failed"),
        (Some(Failure { cause: Cause::Range(RangeFailure::Cursor), row: None }), "Range(Cursor)"),
        (
            Some(Failure {
                cause: Cause::Decode(DecodeFailure::Value),
                row: Some(RowSite {
                    family: Some(Family::Tasks),
                    key: Box::from([0x01, 0xab]),
                    key_bytes: 2,
                    truncated: false,
                }),
            }),
            "Decode(Value), family Tasks, key [01, ab]",
        ),
        (
            Some(Failure {
                cause: Cause::Decode(DecodeFailure::Key),
                row: Some(RowSite { family: None, key: Box::from([0xff]), key_bytes: 3, truncated: true }),
            }),
            "Decode(Key), key [ff] (truncated, 3 bytes total)",
        ),
    ] {
        let end = End::Stopped(Stop::RestorePage { range: temper_service::RestoreRangeKind::Tasks, failure });
        assert_eq!(
            ending_with_binary("temper", Some(&end), false, &directories, b"/data/deployment", &[]),
            (format!("temper serve: restore tasks: {words}\n"), 1)
        );
    }
}

#[test]
fn refusal_lines_name_the_task_attempt_tool_layer_and_reason() {
    use temper_engine_domain_forge::LinkNote;
    use temper_service::Note;
    for (note, expected) in [
        (
            Note::CallAdmissionRefused {
                task: 7,
                attempt: 2,
                tool: b"read_file".as_slice().into(),
                layer: "core-call-room",
                reason: "call record room full",
            },
            "temper serve: call admission refused task=7 attempt=2 tool=read_file layer=core-call-room reason=call record room full\n",
        ),
        (
            Note::ForgeLink { forge: 7, note: LinkNote::Gone },
            "forge 7's person: their account is gone, the link kept\n",
        ),
        (Note::ForgeLink { forge: 7, note: LinkNote::Writer }, "forge 7's person: refused, temper's writer\n"),
        (Note::ForgeLink { forge: 7, note: LinkNote::Replaced }, "forge 7's person: replaced\n"),
        (Note::ForgeLink { forge: 7, note: LinkNote::Cleared }, "forge 7's person: cleared\n"),
    ] {
        assert_eq!(note_line("temper", note), expected);
    }
    let line = note_line(
        "temper",
        Note::ReadRefused {
            task: 7,
            attempt: 2,
            tool: b"read_file".as_slice().into(),
            layer: "authority",
            reason: "connector read denied",
            job: None,
            findings: Box::new([]),
            error: None,
        },
    );
    assert!(line.contains("task=7 attempt=2 tool=read_file layer=authority reason=connector read denied"));
    assert!(line.contains("findings=[]"));
}

#[test]
fn read_job_refusal_line_names_the_asked_attempt() {
    let line = note_line(
        "temper",
        temper_service::Note::ReadRefused {
            task: 7,
            attempt: 4,
            tool: Box::from(*b"read_job"),
            layer: "forge-client",
            reason: "forge read failed",
            findings: Box::new([]),
            error: Some(skein_forge::api::Error::MissingJob),
            job: Some(skein_forge::api::JobAttempt { head: [0xab; 32], run: 2, job: 0, attempt: 1 }),
        },
    );
    assert_eq!(
        line,
        format!(
            "temper serve: read refused task=7 attempt=4 tool=read_job layer=forge-client reason=forge read failed head={} run=2 job=0 job_attempt=1 error=Some(MissingJob) findings=[]\n",
            "ab".repeat(32)
        )
    );
}

#[test]
fn brief_read_refusal_line_names_the_task_section_stage_and_error() {
    use skein_forge::api::{Bound, Error, Excess};
    for (stage, reason, heads, error, expected) in [
        (
            "item",
            "the forge's answer did not match its read",
            None,
            Some(Error::InvalidAnswer),
            "temper serve: brief read refused task=7 section=pull stage=item reason=the forge's answer did not match its read error=InvalidAnswer\n",
        ),
        (
            "pull",
            "the change's head moved",
            Some(([1; 32], [99; 32])),
            None,
            "temper serve: brief read refused task=7 section=pull stage=pull reason=the change's head moved pinned=0101010101010101010101010101010101010101010101010101010101010101 head=6363636363636363636363636363636363636363636363636363636363636363\n",
        ),
        (
            "files",
            "the answer is past",
            None,
            Some(Error::TooLarge(Excess::exact(Bound::Rows, 8, 22))),
            "temper serve: brief read refused task=7 section=pull stage=files reason=the answer is past forge.client.rows (8): 22 rows error=TooLarge: past forge.client.rows (8): 22 rows\n",
        ),
    ] {
        assert_eq!(
            note_line(
                "temper",
                temper_service::Note::BriefReadRefused { task: 7, section: b"pull", stage, reason, heads, error }
            ),
            expected
        );
    }
}

#[test]
fn stale_files_refusal_names_the_error_without_heads() {
    assert_eq!(
        note_line(
            "temper",
            temper_service::Note::BriefReadRefused {
                task: 7,
                section: b"pull",
                stage: "files",
                reason: "the change's head moved",
                heads: None,
                error: Some(skein_forge::api::Error::Stale),
            },
        ),
        "temper serve: brief read refused task=7 section=pull stage=files reason=the change's head moved error=Stale\n"
    );
}

#[test]
fn brief_failure_line_names_how_it_counts_and_why() {
    assert_eq!(
        note_line(
            "temper",
            temper_service::Note::BriefFailed {
                task: 7,
                counts: "permanent",
                reason: b"pull: the forge's answer did not match its read".as_slice().into(),
            }
        ),
        "temper serve: brief failed task=7 counts=permanent reason=pull: the forge's answer did not match its read\n"
    );
}

#[test]
fn refused_job_log_origin_note_names_only_the_origin() {
    let line = note_line(
        "temper",
        temper_service::Note::ForgeOrigin {
            number: 7,
            origin: skein_forge::github::redirect::Origin {
                tls: true,
                host: b"unrelated.example.test".as_slice().into(),
                port: 443,
            },
        },
    );
    assert_eq!(line, "temper serve: forge 7: refused job-log origin https://unrelated.example.test:443\n");
}

#[cfg(test)]
fn failed_run_note(detail: &[u8]) -> temper_service::Note {
    temper_service::Note::RunFailed(temper_engine_domain::RunDiagnostic {
        task: 3,
        run: 7,
        attempt: 2,
        failure: jig_domain_host::Failure::Run(jig_domain_host::RunFailure::Model),
        detail: detail.into(),
    })
}

#[test]
fn failed_run_stderr_escapes_control_bytes_on_one_line() {
    assert_eq!(
        note_line("temper", failed_run_note(b"exit 1: bad\nstream\r\t\0\x1b\x7f\xff")),
        "temper serve: task 3 run 7 attempt 2 failed Run(Model): exit 1: bad\\nstream\\r\\t\\x00\\x1b\\x7f\\xff\n"
    );
}

#[test]
fn failed_run_stderr_keeps_a_bounded_tail() {
    let mut detail = vec![b'x'; 4097];
    detail[0] = b'y';
    let line = note_line("temper", failed_run_note(&detail));
    let prefix = "temper serve: task 3 run 7 attempt 2 failed Run(Model): ";
    assert_eq!(line, format!("{prefix}{}\n", "x".repeat(4096)));
}

#[cfg(test)]
fn refused_run_note(reason: jig_domain_host::Refusal) -> temper_service::Note {
    temper_service::Note::RunRefused(temper_service::RunRefusalDiagnostic { task: 3, run: 7, attempt: 2, reason })
}

#[test]
fn refused_run_stderr_names_reason_and_bounds_start_words() {
    use jig_domain_host::{Invalid, Refusal, StartCause};
    let prefix = "temper serve: task 3 run 7 attempt 2 refused ";
    for (reason, class) in [
        (Refusal::Busy, "Busy"),
        (Refusal::Invalid(Invalid::Charter), "Invalid(Charter)"),
        (Refusal::Invalid(Invalid::Transcript), "Invalid(Transcript)"),
        (Refusal::Invalid(Invalid::Grants), "Invalid(Grants)"),
        (Refusal::Invalid(Invalid::Messages), "Invalid(Messages)"),
    ] {
        assert_eq!(note_line("temper", refused_run_note(reason)), format!("{prefix}{class}: \n"));
    }
    for (cause, class) in [(StartCause::Other, "Other"), (StartCause::Grants, "Grants")] {
        let reason =
            Refusal::Invalid(Invalid::Start { cause, why: b"charter\n\r\t\0\x1b\x7f\xff\\\"".as_slice().into() });
        assert_eq!(
            note_line("temper", refused_run_note(reason)),
            format!("{prefix}Invalid(Start({class})): charter\\n\\r\\t\\x00\\x1b\\x7f\\xff\\\\\\\"\n")
        );
    }
    let mut why = vec![b'x'; 4097];
    why[0] = b'y';
    assert_eq!(
        note_line(
            "temper",
            refused_run_note(Refusal::Invalid(Invalid::Start { cause: StartCause::Other, why: why.into() }))
        ),
        format!("{prefix}Invalid(Start(Other)): {}\n", "x".repeat(4096))
    );
}

#[test]
fn job_stderr_forms_escape_and_bound_their_why() {
    use jig_domain_host::{JobFailure, JobInvalid, JobRefusal, Preparation};
    use temper_service::{JobDiagnostic, JobDiagnosticEnding};
    // domain/worker/jobs.md: all four forms retain the last 4,096 input bytes.
    const TAIL: usize = 4096;
    for (worker, refused, prefix) in [
        (false, false, "temper serve: task 3 run 7 attempt 2 failed Unprepared(Transient): job 3: "),
        (false, true, "temper serve: task 3 run 7 attempt 2 refused Invalid(Start): job 3: "),
        (true, false, "temper worker: run 7 attempt 2 job 3 merge failed Unprepared(Transient): "),
        (true, true, "temper worker: run 7 attempt 2 job 3 merge refused Invalid(Start): "),
    ] {
        for (why, expected) in [
            (b"fetch base: Broken\n\xff".to_vec(), "fetch base: Broken\\n\\xff".to_owned()),
            (vec![b'x'; TAIL], "x".repeat(TAIL)),
            ([b"discard".as_slice(), vec![b'x'; TAIL].as_slice()].concat(), "x".repeat(TAIL)),
        ] {
            let ending = if refused {
                JobDiagnosticEnding::Refused(JobRefusal::Invalid(JobInvalid::Start { why: why.clone().into() }))
            } else {
                JobDiagnosticEnding::Failed {
                    failure: JobFailure::Unprepared(Preparation::Transient),
                    detail: why.clone().into(),
                }
            };
            let note = if worker {
                temper_service::Note::WorkerJob {
                    run: 7,
                    attempt: 2,
                    kind: 3,
                    verb: b"merge".as_slice().into(),
                    ending: match ending {
                        JobDiagnosticEnding::Failed { failure, detail } => {
                            temper_service::WorkerJobEnding::Failed { failure, detail }
                        }
                        JobDiagnosticEnding::Refused(reason) => temper_service::WorkerJobEnding::Refused(reason),
                    },
                    why: why.into(),
                }
            } else {
                temper_service::Note::JobAnswered(JobDiagnostic { task: 3, run: 7, attempt: 2, kind: 3, ending })
            };
            assert_eq!(note_line("temper", note), format!("{prefix}{expected}\n"));
        }
    }
    for (failure, class) in [
        (JobFailure::Deadline, "Deadline"),
        (JobFailure::Undelivered(jig_domain_host::DeliveryOutcome::Failed), "Undelivered(Failed)"),
    ] {
        let note = temper_service::Note::JobAnswered(JobDiagnostic {
            task: 3,
            run: 7,
            attempt: 2,
            kind: 3,
            ending: JobDiagnosticEnding::Failed { failure, detail: Box::new([]) },
        });
        assert_eq!(
            note_line("temper", note),
            format!("temper serve: task 3 run 7 attempt 2 failed {class}: job 3: \n")
        );
    }
}

#[test]
fn long_pass_line_names_the_pass_its_longest_part_and_the_bound() {
    let long = LongPass {
        took: skein_lib::Duration::from_millis(1250),
        part: Part::Service(temper_service::Stage::Engine),
        part_took: skein_lib::Duration::from_millis(1200),
        more: 0,
    };
    assert_eq!(
        long_pass_line("temper", long),
        "temper serve: long pass: 1250 ms, engine 1200 ms of it, past the 1000 ms bound\n"
    );
}

#[test]
fn long_pass_line_counts_the_other_long_passes() {
    let long = LongPass {
        took: skein_lib::Duration::from_millis(1250),
        part: Part::Service(temper_service::Stage::Engine),
        part_took: skein_lib::Duration::from_millis(1200),
        more: 3,
    };
    assert_eq!(
        long_pass_line("temper", long),
        "temper serve: long pass: 1250 ms, engine 1200 ms of it, past the 1000 ms bound; 3 more since the last line\n"
    );
}

#[test]
fn core_wait_notes_name_the_reason_and_recovery() {
    use temper_service::{CoreWait, Note};
    for (binary, note, expected) in [
        (
            "temper",
            Note::CoreWaiting { why: CoreWait::Starting { step: None } },
            "temper serve: the core is not answering: starting: loading the core's records\n",
        ),
        ("temper", Note::CoreAnswers, "temper serve: the core answers again\n"),
        ("temper-engine", Note::CoreAnswers, "temper-engine serve: the core answers again\n"),
    ] {
        assert_eq!(note_line(binary, note), expected);
        assert!(expected.len() <= 240);
    }
}

/// The box's ending, preserving its public call path.
#[must_use]
pub fn ending(
    end: Option<&End>,
    output_failed: bool,
    directories: &Directories,
    store: &[u8],
    unsettled: &[temper_service::Unsettled],
) -> (String, u8) {
    ending_with_binary("temper", end, output_failed, directories, store, unsettled)
}

#[test]
fn retired_settings_name_the_ctl_setting_at_start() {
    use temper_engine_domain_forge::settings::{SettingKey, SettingSlot};
    let arguments =
        [b"--config".as_slice(), b"/configuration", b"--data", b"/data", b"--cache", b"/cache"].map(Box::from);
    let (directories, _) = temper_shell::directories::resolve(&arguments, &[], b"/").expect("directories");
    for (kind, scope, number, words) in [(6, 2, 1, "project-questions 2"), (3, 0, 3, "forge-charter 3")] {
        let end = End::Stopped(Stop::Setting(SettingKey { kind, scope, number, slot: SettingSlot::Change }));
        assert_eq!(
            ending_with_binary("temper", Some(&end), false, &directories, b"/data/deployment", &[]),
            (format!("temper serve: restart refused: {words}: retired stored setting\n"), 1)
        );
    }
}
#[cfg(test)]
pub(super) mod adapter_tests {
    use super::*;
    use skein_shell::Host as _;

    pub(crate) struct OneStage {
        stages: u32,
        at: Option<Time>,
        stopped: bool,
        completions: Queue<Complete>,
        submissions: Queue<Submit>,
        links: Queue<Box<[u8]>>,
    }
    impl OneStage {
        fn new() -> Self {
            Self {
                stages: 0,
                at: None,
                stopped: false,
                completions: Queue::uncapped(),
                submissions: Queue::uncapped(),
                links: Queue::uncapped(),
            }
        }
    }
    pub(crate) const SERVED: Served<OneStage> = Served {
        stages: &[temper_service::Stage::Route],
        iterate_stage: |service, stage, now, wall| {
            assert_eq!(stage, temper_service::Stage::Route);
            service.at = Some(now);
            assert_eq!(wall, Wall::EPOCH);
            service.stages += 1;
        },
        resolve: |_| {},
        completions: |service| &mut service.completions,
        submissions: |service| &mut service.submissions,
        operations: |_| 7,
        work_pending: |service, _| service.stages == 0,
        next_mechanism_deadline: |_| Some(Time::ZERO),
        next_policy_deadline: |_| None,
        is_empty: |service| service.stages == 1,
        shutdown: |service| service.stopped = true,
        ending: |service, binary, failed, _, store| {
            assert_eq!(service.stages, 1);
            assert_eq!(binary, "own-service");
            assert_eq!(store, b"/d");
            (format!("{binary}: one stage ended\n"), u8::from(failed))
        },
        handover: |_| {},
        listening: |_, _| None,
        note: |_, _| None,
        link: |service| service.links.pop(),
        local_link: |_| None,
        check_wait_boundary: |service, _| assert_eq!(service.stages, 1),
        adopt_listener: |_, _, _| Ok(()),
        adopt_signals: |_, _| Ok(()),
    };
    fn host() -> Host<OneStage> {
        let arguments = ["--config", "/c", "--data", "/d", "--cache", "/k"].map(|word| Box::from(word.as_bytes()));
        let (directories, _) = temper_shell::directories::resolve(&arguments, &[], b"/").expect("directories");
        Host::with_binary("own-service", OneStage::new(), SERVED, directories, &[])
    }
    #[test]
    fn supplied_service_stage_and_ending_drive_the_host() {
        let mut host = host();
        assert_eq!(host.exit(), None);
        assert!(host.work_pending(Time::ZERO));
        assert_eq!(host.operations(), 7);
        assert_eq!(host.next_mechanism_deadline(), Some(Time::ZERO));
        assert_eq!(host.next_policy_deadline(), None);
        assert!(host.completions().is_empty());
        assert!(host.submissions().is_empty());
        let now = host.clock.now().now;
        host.iterate(now, Wall::EPOCH);
        host.drain();
        assert_eq!(host.service.at, Some(now));
        assert!(host.is_empty());
        assert!(!host.work_pending(Time::ZERO));
        assert_eq!(host.ending(), ("own-service: one stage ended\n".into(), 0));
        assert_eq!(host.exit(), Some(Exit::Code(0)));
    }
    struct BrokenOutput;
    impl Write for BrokenOutput {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::ErrorKind::BrokenPipe.into())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    #[test]
    fn output_failure_shuts_down_the_supplied_service() {
        let mut host = host();
        host.output = Box::new(BrokenOutput);
        host.service.links.push(b"a link".as_slice().into());
        let now = host.clock.now().now;
        host.iterate(now, Wall::EPOCH);
        host.drain();
        assert!(host.service.stopped, "failed output must ask the supplied service to shut down");
        assert_eq!(host.exit(), Some(Exit::Code(1)));
    }
}

#[cfg(test)]
#[path = "worker_lines_tests.rs"]
mod worker_lines_tests;
