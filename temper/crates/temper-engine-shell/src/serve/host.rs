//! The shell's adapter over the temper composition (jig's protocol/service.md, 7).
//! Start refusals name their cause (temper/docs/design/protocol/README.md, 4).
use skein_io::kernel::{Complete, Exit, Submit};
use skein_lib::{Queue, Time, Wall};
use std::io::{self, Write};
use temper_service::long_pass::{LONG_PASS, LongPass, Part, Watch};
use temper_service::{End, Service, Stop};
use temper_shell::directories::Directories;

pub struct Host {
    pub service: Service,
    binary: String,
    directories: Directories,
    pub(super) store: Box<[u8]>,
    pub(super) link_file: Option<std::path::PathBuf>,
    output: Box<dyn Write>,
    output_failed: bool,
    wait_check: bool,
    now: Time,
    watch: Watch,
    clock: skein_shell::Clock,
}
impl Host {
    #[must_use]
    pub fn new(service: Service, directories: Directories) -> Self {
        Self::with_binary("temper", service, directories)
    }
    #[must_use]
    pub fn with_binary(binary: &str, service: Service, directories: Directories) -> Self {
        let store = directories.data.clone();
        Self {
            service,
            binary: binary.into(),
            directories,
            store,
            link_file: None,
            output: Box::new(io::stdout()),
            output_failed: false,
            // QA launches sanitized child environments. Test/debug programs
            // always check; release programs can opt in with the same flag.
            wait_check: cfg!(debug_assertions) || std::env::var_os("TEMPER_R8_WAIT_CHECK").is_some(),
            now: Time::ZERO,
            watch: Watch::new(),
            clock: skein_shell::Clock::new(),
        }
    }
    pub(super) fn keep_link(&self, url: &[u8]) {
        if let Some(path) = &self.link_file
            && let Err(why) = super::link_file::keep(path, url, skein_shell::effective_user())
        {
            eprintln!("{} serve: link file {why}", self.binary);
        }
    }
}
impl skein_shell::Host for Host {
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.now = now;
        self.watch.begin(now);
        for stage in temper_service::STAGES {
            temper_service::iterate_stage(&mut self.service, stage, now, wall);
            self.watch.ended(Part::Service(stage), self.clock.now().now);
        }
        for (forge, name, port) in self.service.unresolved() {
            let address = std::str::from_utf8(&name)
                .ok()
                .and_then(|name| std::net::ToSocketAddrs::to_socket_addrs(&(name, port)).ok()?.next());
            self.service.resolved(forge, address);
        }
        self.watch.ended(Part::Names, self.clock.now().now);
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.service.completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.service.submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.service.work_pending(now)
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.service.next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.service.next_policy_deadline()
    }
    fn is_empty(&self) -> bool {
        self.service.is_empty()
    }
    fn operations(&self) -> u32 {
        self.service.operations()
    }
    fn exit(&self) -> Option<Exit> {
        if !self.service.is_empty() {
            return None;
        }
        if let Some(long) = self.watch.held() {
            eprint!("{}", long_pass_line(&self.binary, long));
        }
        let (lines, code) = ending_with_binary(
            &self.binary,
            self.service.end(),
            self.output_failed,
            &self.directories,
            &self.store,
            self.service.unsettled(),
        );
        eprint!("{lines}");
        Some(Exit::Code(code))
    }
    fn drain(&mut self) {
        while let Some(read) = self.service.take_handover_read() {
            let answer = read_handover(&read);
            self.service.handover_read(read.forge, read.run, answer);
        }
        while let Some(remove) = self.service.take_handover_removal() {
            let removed = remove_handover(&remove);
            self.service.handover_removed(remove.forge, remove.run, removed);
        }
        while let Some(note) = self.service.listening().pop() {
            let clients =
                matches!(&note, temper_service::Note::Listening { listener: temper_service::Listening::Clients, .. });
            let lines = note_line(&self.binary, note);
            eprint!("{lines}");
            if self.wait_check && clients {
                super::restart_fault::cut_at_listen();
            }
        }
        while let Some(note) = self.service.notes().pop() {
            let lines = note_line(&self.binary, note);
            eprint!("{lines}");
        }
        while let Some(link) = self.service.links().pop() {
            if self
                .output
                .write_all(&link)
                .and_then(|()| self.output.write_all(b"\n"))
                .and_then(|()| self.output.flush())
                .is_err()
            {
                self.output_failed = true;
                self.service.shutdown();
            }
        }
        while let Some(link) = self.service.local_links().pop() {
            self.keep_link(&link);
        }
        if self.wait_check {
            self.service.check_wait_boundary(self.now);
        }
        self.watch.ended(Part::Drain, self.clock.now().now);
        if let Some(long) = self.watch.pass_ended() {
            eprint!("{}", long_pass_line(&self.binary, long));
        }
    }
}

pub(crate) fn read_handover(
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
fn remove_handover(remove: &temper_service::HandoverRemoval) -> Result<(), i32> {
    match std::fs::remove_file(super::path(&remove.path)) {
        Ok(()) => Ok(()),
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(why) => Err(why.raw_os_error().expect("unlink returns an errno")),
    }
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
            Stop::Binding => format!("{binary} serve: deployment namespace binding refused\n"),
            Stop::StoreOpen { why } => {
                let deployment = super::path(store);
                let (lines, exit) = temper_service::refused_start(why, deployment.display().to_string().as_bytes());
                let lines = String::from_utf8(lines.into_vec()).expect("refusal is UTF-8");
                let mut named = String::new();
                for line in lines.lines() {
                    use std::fmt::Write as _;
                    writeln!(
                        named,
                        "{binary} serve: {}",
                        line.strip_prefix("temper serve: ").expect("service refusal prefix")
                    )
                    .expect("writing into a string");
                }
                return (named, exit);
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
            Stop::Listener { listener, address, error } => {
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
        temper_service::Note::HelloRefused { channel, text } => {
            format!("{binary} serve: hello refused channel={channel}: {}\n", String::from_utf8_lossy(&text))
        }
        temper_service::Note::ReadRefused { task, attempt, tool, layer, reason, findings, error } => {
            format!(
                "{binary} serve: read refused task={task} attempt={attempt} tool={} layer={layer} reason={reason} error={error:?} findings={findings:?}\n",
                String::from_utf8_lossy(&tool)
            )
        }
        temper_service::Note::BriefReadRefused { task, section, stage, error } => {
            let error = match error {
                Some(error) => format!("{error:?}"),
                None => "None".into(),
            };
            format!(
                "{binary} serve: brief read refused task={task} section={} stage={stage} error={error}\n",
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
            format!("{binary} serve: {of:?} patch {patch}: {} records, {} commits\n", counts.records, counts.commits)
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
// temper/docs/design/domain/worker.md, 4.3: one bounded, escaped operator line.
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
fn hello_refusal_notes_print_the_channel_and_words_exactly() {
    assert_eq!(
        note_line(
            "temper",
            temper_service::Note::HelloRefused {
                channel: 7,
                text: b"charter 3: needs 64 turns; the agent's profile.policy.max_turns allows 32".as_slice().into(),
            }
        ),
        "temper serve: hello refused channel=7: charter 3: needs 64 turns; the agent's profile.policy.max_turns allows 32\n",
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
            findings: Box::new([]),
            error: None,
        },
    );
    assert!(line.contains("task=7 attempt=2 tool=read_file layer=authority reason=connector read denied"));
    assert!(line.contains("findings=[]"));
}

#[test]
fn brief_read_refusal_line_names_the_task_section_stage_and_error() {
    assert_eq!(
        note_line(
            "temper",
            temper_service::Note::BriefReadRefused {
                task: 7,
                section: b"pull",
                stage: "item",
                error: Some(skein_forge::api::Error::InvalidAnswer),
            }
        ),
        "temper serve: brief read refused task=7 section=pull stage=item error=InvalidAnswer\n"
    );
    assert_eq!(
        note_line(
            "temper",
            temper_service::Note::BriefReadRefused { task: 7, section: b"ci", stage: "checks", error: None }
        ),
        "temper serve: brief read refused task=7 section=ci stage=checks error=None\n"
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
