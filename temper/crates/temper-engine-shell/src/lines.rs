//! The engine's lines and handover io (temper/docs/design/protocol/services.md, 7).
use temper_engine_service::{End, Stop};
use temper_shell::directories::Directories;

pub fn read_handover(
    read: &temper_engine_service::HandoverRead,
) -> Result<Option<skein_lib::Secret>, temper_engine_service::HandoverUnread> {
    use temper_engine_service::HandoverUnread;
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
        Err(skein_shell::Unread::Kernel(error)) => Err(HandoverUnread::Kernel(temper_shell::directories::errno(error))),
    }
}
pub fn remove_handover(remove: &temper_engine_service::HandoverRemoval) -> Result<(), i32> {
    match std::fs::remove_file(super::path(&remove.path)) {
        Ok(()) => Ok(()),
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(why) => Err(why.raw_os_error().expect("unlink returns an errno")),
    }
}

/// The composition's listening note, with the client-listen boundary kept beside its words.
pub fn listening_line(service: &mut temper_engine_service::Service, binary: &str) -> Option<(String, bool)> {
    let note = service.listening().pop()?;
    let clients = matches!(
        &note,
        temper_engine_service::Note::Listening { listener: temper_engine_service::Listening::Clients, .. }
    );
    Some((note_line(binary, note), clients))
}
pub fn next_note_line(service: &mut temper_engine_service::Service, binary: &str) -> Option<String> {
    service.notes().pop().map(|note| note_line(binary, note))
}
#[must_use]
pub fn service_ending(
    service: &temper_engine_service::Service,
    binary: &str,
    failed: bool,
    directories: &Directories,
    store: &[u8],
) -> (String, u8) {
    ending_with_binary(binary, service.end(), failed, directories, store, service.unsettled())
}

/// The terminal stderr and status shared by the shell and its real-loop host.
#[must_use]
pub fn ending_with_binary(
    binary: &str,
    end: Option<&End>,
    output_failed: bool,
    _directories: &Directories,
    store: &[u8],
    unsettled: &[temper_engine_service::Unsettled],
) -> (String, u8) {
    let mut lines = match end {
        Some(End::Stopped(why)) => match why {
            Stop::Start(found) => problems(
                binary,
                &temper_engine_service::startup::Problems::jig(found.clone(), &jig_protocol_config::Path::root()),
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
                    temper_engine_service::refused_start(why, deployment.display().to_string().as_bytes()),
                );
            }
            Stop::StoreRow { row } => {
                let deployment = super::path(store);
                return store_refusal(
                    binary,
                    temper_engine_service::refused_row_start(row, deployment.display().to_string().as_bytes()),
                );
            }
            Stop::FrameTooLarge { number, task, bytes, limit } => format!(
                "{binary} serve: commit {number} refused: task {task}'s transcript frame, {bytes} bytes, past the store's page of {limit} bytes\n"
            ),
            Stop::Journal { why } => {
                let why = match why {
                    temper_engine_service::JournalStop::Refused { commit, sent, durable } => {
                        format!("the store answered commit {commit}, with {sent} sent and {durable} durable")
                    }
                    temper_engine_service::JournalStop::PastEdge { writes } => {
                        format!("a decision past the store's {writes} writes")
                    }
                    temper_engine_service::JournalStop::Counter => "the commit counter at its end".into(),
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

fn restore_kind(kind: temper_engine_service::RestoreRangeKind) -> &'static str {
    use temper_engine_service::RestoreRangeKind as Kind;
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

#[expect(clippy::too_many_lines, reason = "one exhaustive note match keeps the engine vocabulary beside its words")]
#[must_use]
pub fn note_line(binary: &str, note: temper_engine_service::Note) -> String {
    match note {
        temper_engine_service::Note::CoreWaiting { why } => temper_service_shell::core_wait_line(binary, &why.words()),
        temper_engine_service::Note::CoreAnswers => temper_service_shell::core_answers_line(binary),
        temper_engine_service::Note::StartupWarning { line } => {
            format!("{binary} serve: warning: {}\n", String::from_utf8_lossy(&line))
        }
        temper_engine_service::Note::ForgeLink { forge, note } => {
            let words = match note {
                temper_engine_domain_forge::LinkNote::Gone => "their account is gone, the link kept",
                temper_engine_domain_forge::LinkNote::Writer => "refused, temper's writer",
                temper_engine_domain_forge::LinkNote::Replaced => "replaced",
                temper_engine_domain_forge::LinkNote::Cleared => "cleared",
            };
            format!("forge {forge}'s person: {words}\n")
        }
        temper_engine_service::Note::RunFailed(diagnostic) => run_failure_line(
            binary,
            diagnostic.task,
            diagnostic.run,
            diagnostic.attempt,
            diagnostic.failure,
            &diagnostic.detail,
        ),
        temper_engine_service::Note::RunRefused(diagnostic) => run_refusal_line(binary, diagnostic),
        temper_engine_service::Note::JobAnswered(diagnostic) => job_line(binary, diagnostic),

        temper_engine_service::Note::HelloRefused { channel, text } => {
            format!("{binary} serve: hello refused channel={channel}: {}\n", String::from_utf8_lossy(&text))
        }
        temper_engine_service::Note::ReadRefused { task, attempt, tool, layer, reason, findings, error, job } => {
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
        temper_engine_service::Note::BriefReadRefused { task, section, stage, reason, heads, error } => {
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
        temper_engine_service::Note::BriefFailed { task, counts, reason } => {
            format!(
                "{binary} serve: brief failed task={task} counts={counts} reason={}\n",
                String::from_utf8_lossy(&reason)
            )
        }
        temper_engine_service::Note::CallAdmissionRefused { task, attempt, tool, layer, reason } => {
            format!(
                "{binary} serve: call admission refused task={task} attempt={attempt} tool={} layer={layer} reason={reason}\n",
                String::from_utf8_lossy(&tool)
            )
        }
        temper_engine_service::Note::Patched { of, patch, counts } => {
            let binary = if binary == "temper" { "temper-box" } else { binary };
            format!(
                "{binary} serve: patched {} {patch}: {} records in {} commits\n",
                crate::upgrade::owner(of),
                counts.records,
                counts.commits
            )
        }
        temper_engine_service::Note::Listening { listener, addr } => {
            let name = match listener {
                temper_engine_service::Listening::Clients => "clients",
                temper_engine_service::Listening::Workers => "workers",
                temper_engine_service::Listening::Webhooks => "webhooks",
            };
            format!("listening {name} {addr}\n")
        }

        temper_engine_service::Note::WorkerLost { worker, channel, why } => {
            format!(
                "{binary} serve: worker lost worker={} channel={}: {}\n",
                worker_name(worker.as_deref()),
                channel.raw(),
                engine_worker_loss(why)
            )
        }
        temper_engine_service::Note::WorkerUnsent { worker, channel, unsent } => {
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
        temper_engine_service::Note::WorkerOpeningEnded { worker, why } => {
            format!(
                "{binary} serve: worker opening ended worker={}: {}\n",
                worker_name(worker.as_deref()),
                engine_worker_loss(why)
            )
        }
        temper_engine_service::Note::WorkerListenerFailed { address, error } => {
            listener_failed_line(binary, temper_engine_service::Listening::Workers, address, error)
        }

        temper_engine_service::Note::ForgeLacking { number, names } => {
            format!("{binary} serve: forge {number}: App lacks {}\n", String::from_utf8_lossy(&names))
        }
        temper_engine_service::Note::SetUpRefused { number, why } => set_up_refused_line(binary, number, why),
        temper_engine_service::Note::Playground { line } | temper_engine_service::Note::BoxLink { line } => {
            format!("{}\n", String::from_utf8_lossy(&line))
        }
        temper_engine_service::Note::ForgeStopped { number, why } => stopped_line(binary, number, why),
        temper_engine_service::Note::ForgeSetupStopped { number, why } => setup_stopped_line(binary, number, why),
        temper_engine_service::Note::ForgeNotConnected { number, why } => {
            format!("{binary} serve: forge {number}: not connected: {why}\n")
        }

        temper_engine_service::Note::ForgeSignIn { number, client, why } => {
            use temper_engine_service::clients::forge_sign_in::Unserved;
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
        temper_engine_service::Note::ForgeOrigin { number, origin } => origin_line(binary, number, &origin),
        temper_engine_service::Note::ForgeUnkept { number, installation } => {
            format!("{binary} serve: forge {number}: installation {installation}: not kept\n")
        }
    }
}
// temper/docs/design/domain/worker/README.md, 4.3: one bounded, escaped operator line.
fn run_refusal_line(binary: &str, diagnostic: temper_engine_service::RunRefusalDiagnostic) -> String {
    let temper_engine_service::RunRefusalDiagnostic { task, run, attempt, reason } = diagnostic;
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
fn job_line(binary: &str, diagnostic: temper_engine_service::JobDiagnostic) -> String {
    let temper_engine_service::JobDiagnostic { task, run, attempt, kind, ending } = diagnostic;
    let prefix = format!("{binary} serve: task {task} run {run} attempt {attempt} ");
    let (class, detail) = job_ending(&ending);
    run_detail_line(format!("{prefix}{class}: job {kind}: "), detail)
}

fn job_ending(ending: &temper_engine_service::JobDiagnosticEnding) -> (String, &[u8]) {
    use jig_domain_host::{JobInvalid, JobRefusal};
    use temper_engine_service::JobDiagnosticEnding;
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

fn setup_stopped_line(binary: &str, number: u16, why: temper_engine_service::SetupStopped) -> String {
    use temper_engine_service::{SetupPlan, SetupStopped, SetupWhy};
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
    listener: temper_engine_service::Listening,
    address: Option<skein_io::Addr>,
    error: Option<skein_io::Error>,
) -> String {
    let name = match listener {
        temper_engine_service::Listening::Clients => "client",
        temper_engine_service::Listening::Workers => "worker",
        temper_engine_service::Listening::Webhooks => "webhooks",
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
        2 => temper_engine_service::Refusal::Terms,
        3 => temper_engine_service::Refusal::Malformed,
        code => temper_engine_service::Refusal::Peer(code),
    })
}

fn worker_name(worker: Option<&[u8]>) -> String {
    match worker {
        Some(name) => String::from_utf8_lossy(name).into_owned(),
        None => "-".into(),
    }
}

fn engine_worker_loss(why: temper_engine_service::EngineWorkerLoss) -> String {
    use temper_engine_service::EngineWorkerLoss as Loss;
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

fn refusal(reason: temper_engine_service::Refusal) -> String {
    use temper_engine_service::WorkerRefusalReason as Reason;
    match reason {
        temper_engine_service::Refusal::Terms => "terms".into(),
        temper_engine_service::Refusal::Malformed => "malformed".into(),
        temper_engine_service::Refusal::Peer(code) => match Reason::from_code(code) {
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

fn problems(binary: &str, found: &temper_engine_service::startup::Problems) -> String {
    let lines = temper_engine_service::startup::lines(found);
    super::prefixed(&format!("{binary} serve"), &String::from_utf8_lossy(&lines))
}

#[cfg(test)]
mod tests;
