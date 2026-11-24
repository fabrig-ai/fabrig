//! Refresh the requests stack (temper/docs/design/client/web/journeys.md, 7;
//! temper/docs/design/client/journey-writer.md, 6).

use std::{
    borrow::Cow, collections::BTreeMap, ffi::OsStr, fmt::Write as _, os::unix::ffi::OsStrExt as _, path::PathBuf,
};

use crate::host::{Answer, Ask, Host, Program, Ran, Stream};

const MAIN: &str = "origin/main";
const TODO: &str = "LOOK_REFRESH_TODO";
const STATES: [&str; 5] = ["rebase-merge", "rebase-apply", "MERGE_HEAD", "CHERRY_PICK_HEAD", "sequencer"];

enum Stop {
    Refused(String),
    Failed,
    Moved(String),
}

pub(crate) fn run(arguments: &[Box<[u8]>], host: &mut Host, pid: u32) -> u8 {
    match refresh(arguments, host, pid) {
        Ok(()) => 0,
        Err(Stop::Failed) => 1,
        Err(Stop::Refused(why)) => {
            diagnostic(host, &why);
            2
        }
        Err(Stop::Moved(branch)) => {
            diagnostic(
                host,
                &format!(
                    "remote {branch} moved elsewhere; fetch it and settle it by hand before running again; branch left rebased"
                ),
            );
            3
        }
    }
}

fn refresh(arguments: &[Box<[u8]>], host: &mut Host, pid: u32) -> Result<(), Stop> {
    if !arguments.is_empty() {
        return Err(Stop::Refused("takes no arguments".into()));
    }
    let branch = checkout(host)?;
    // Read the lease before fetching. An absent ref requires an absent remote branch.
    let reference = format!("refs/remotes/origin/{branch}");
    let lease = git(host, &["rev-parse", "--verify", &reference], Stream::Keep, Stream::Keep)?;
    let lease = if lease.code == Some(0) { text(&lease.stdout) } else { Cow::Borrowed("") };
    git_to_err(host, &["fetch", "origin", "main"])?;

    let requests = requests(host, branch)?;
    let landings = git_text(host, &["log", "--format=commit %H%n%(trailers:key=Look-request,valueonly)", MAIN])?;
    let done = done_on_main(&landings);
    let (printed, mut report) = printed_on_main(host)?;
    let mut todo = String::new();
    for (commit, name) in &requests {
        let done_by = done.get(name.as_str());
        if let (Some(done_by), Some(printed)) = (done_by, &printed)
            && ancestor(host, done_by, printed)?
        {
            writeln!(todo, "drop {commit}").expect("write to String");
            writeln!(report, "dropped {name}, done on main by {done_by:.12}, printed at {printed:.12}")
                .expect("write to String");
            continue;
        }
        writeln!(todo, "pick {commit}").expect("write to String");
        if let Some(done_by) = done_by {
            writeln!(report, "kept {name}, done on main by {done_by:.12}, not yet printed").expect("write to String");
        } else {
            writeln!(report, "kept {name}").expect("write to String");
        }
    }
    if todo.is_empty() {
        todo.push_str("noop\n");
    }
    rebase(host, branch, pid, todo, &requests)?;
    host.out(report.as_bytes());
    push(host, branch, &lease)
}

fn checkout(host: &mut Host) -> Result<&'static str, Stop> {
    let head = git(host, &["symbolic-ref", "--quiet", "--short", "HEAD"], Stream::Keep, Stream::Err)?;
    let branch = match (head.code, text(&head.stdout).as_ref()) {
        (Some(0), "look-requests") => "look-requests",
        (Some(0), "tui-requests") => "tui-requests",
        _ => return Err(Stop::Refused("check out look-requests or tui-requests first".into())),
    };
    for state in STATES {
        let path = git_path(host, &["rev-parse", "--git-path", state])?;
        match host.ask(Ask::Exists(path)) {
            Answer::Is(false) => (),
            Answer::Is(true) => return Err(Stop::Refused(format!("{state} is in progress"))),
            answer => return Err(effect_failed(host, answer)),
        }
    }
    if !git_text(host, &["status", "--porcelain", "--untracked-files=no"])?.is_empty() {
        return Err(Stop::Refused("tracked files must be clean".into()));
    }
    Ok(branch)
}

fn requests(host: &mut Host, branch: &str) -> Result<Vec<(String, String)>, Stop> {
    let range = format!("{MAIN}..{branch}");
    let commits = git_text(host, &["rev-list", "--reverse", &range])?;
    let mut requests: Vec<(String, String)> = Vec::new();
    for commit in commits.split_terminator('\n') {
        if git_text(host, &["show", "-s", "--format=%P", commit])?.contains(' ') {
            return Err(Stop::Refused(format!("{commit} is a merge commit")));
        }
        // Retain keys until counting: a second, empty trailer is still a trailer.
        let trailers = git_text(host, &["show", "-s", "--format=%(trailers:key=Look-request)", commit])?;
        let name =
            trailers.split_once(':').map_or(trailers.as_str(), |(_, value)| value.trim_start_matches([' ', '\t']));
        if trailers.contains('\n') || !valid_name(name) {
            return Err(Stop::Refused(format!(
                "{commit} must carry exactly one Look-request trailer with NAME [a-z0-9][a-z0-9-]*"
            )));
        }
        if requests.iter().any(|(_, seen)| seen == name) {
            return Err(Stop::Refused(format!("{commit} repeats Look-request: {name}")));
        }
        requests.push((commit.to_owned(), name.to_owned()));
    }
    Ok(requests)
}

fn valid_name(name: &str) -> bool {
    let word_byte = |byte: u8| byte.is_ascii_lowercase() || byte.is_ascii_digit();
    let mut bytes = name.bytes();
    bytes.next().is_some_and(word_byte) && bytes.all(|byte| word_byte(byte) || byte == b'-')
}

fn done_on_main(log: &str) -> BTreeMap<&str, &str> {
    let mut commit = "";
    let mut done = BTreeMap::new();
    for line in log.split_terminator('\n') {
        if let Some(carrier) = line.strip_prefix("commit ") {
            commit = carrier;
        } else if valid_name(line) {
            done.entry(line).or_insert(commit);
        }
    }
    done
}

fn printed_on_main(host: &mut Host) -> Result<(Option<String>, String), Stop> {
    // Count keys separately: an empty Printed value is not a missing trailer.
    let log = checked_git(
        host,
        &[
            "log",
            "--topo-order",
            "--format=commit %H %(trailers:key=Printed,keyonly,separator=,)%n%(trailers:key=Printed,valueonly)",
            MAIN,
        ],
    )?;
    let mut lines = log.split(|&byte| byte == b'\n');
    while let Some(line) = lines.next() {
        let Some(carrier) = line.strip_prefix(b"commit ") else { continue };
        let carrier = String::from_utf8_lossy(carrier);
        let mut fields = carrier.split_ascii_whitespace();
        let (Some(carrier), Some(keys)) = (fields.next(), fields.next()) else { continue };
        for value in lines.by_ref().take(keys.split(',').count()) {
            let value = trim_fields(value);
            if matches!(value.len(), 40 | 64) && value.iter().all(u8::is_ascii_hexdigit) {
                let printed = String::from_utf8_lossy(value);
                let object = git(host, &["cat-file", "-t", &printed], Stream::Keep, Stream::Keep)?;
                if object.code == Some(0) && text(&object.stdout) == "commit" && ancestor(host, &printed, MAIN)? {
                    let since = landings_since(host, carrier)?;
                    let report = format!("pages printed at {printed:.12}, {since} landings since\n");
                    return Ok((Some(printed.into_owned()), report));
                }
            }
            host.err(
                &[
                    b"look-refresh: ignoring Printed: ".as_slice(),
                    value,
                    b" in ",
                    carrier.as_bytes(),
                    b"; expected a full commit hash in origin/main history\n",
                ]
                .concat(),
            );
        }
    }
    Ok((None, "pages not printed yet\n".into()))
}

fn landings_since(host: &mut Host, carrier: &str) -> Result<String, Stop> {
    // Find the oldest entry on main's first-parent line containing the carrier.
    let landings = git_text(host, &["rev-list", "--first-parent", MAIN])?;
    let mut brought_by = "";
    for landing in landings.split_terminator('\n') {
        if !ancestor(host, carrier, landing)? {
            break;
        }
        brought_by = landing;
    }
    git_text(host, &["rev-list", "--first-parent", "--count", &format!("{brought_by}..{MAIN}")])
}

fn rebase(host: &mut Host, branch: &str, pid: u32, todo: String, requests: &[(String, String)]) -> Result<(), Stop> {
    let base = git_text(host, &["merge-base", MAIN, branch])?;
    // Git runs the editor from the worktree root, which may not be our cwd.
    // Per-process names also keep another invocation from replacing this plan.
    let file = format!("temper-look-refresh.{pid}.todo");
    let path = git_path(host, &["rev-parse", "--path-format=absolute", "--git-path", &file])?;
    let written = host.ask(Ask::Write(path.clone(), todo.into_bytes().into()));
    file_changed(host, written)?;
    // Only a validated trailer can complete a request, including an empty or patch-equivalent one.
    let mut rebase = git_program(
        &[
            "-c",
            "rebase.autoStash=false",
            "-c",
            "rebase.updateRefs=false",
            "-c",
            "rebase.abbreviateCommands=false",
            "rebase",
            "--interactive",
            "--no-autosquash",
            "--no-autostash",
            "--no-update-refs",
            "--reapply-cherry-picks",
            "--keep-empty",
            "--empty=keep",
            "--onto",
            MAIN,
            &base,
        ],
        Stream::Err,
        Stream::Err,
    );
    rebase.environment = vec![
        (TODO.as_bytes().into(), path.as_os_str().as_bytes().into()),
        (b"GIT_SEQUENCE_EDITOR".as_slice().into(), b"cp \"$LOOK_REFRESH_TODO\"".as_slice().into()),
    ];
    let rebased = child(host, rebase);
    let removed = host.ask(Ask::Remove(path));
    if rebased.is_ok_and(|ran| ran.code == Some(0)) {
        return file_changed(host, removed);
    }
    let stopped = git(host, &["rev-parse", "--verify", "REBASE_HEAD"], Stream::Keep, Stream::Keep)?;
    let stopped_at = text(&stopped.stdout);
    let name = requests
        .iter()
        .find(|(commit, _)| stopped.code == Some(0) && *commit == stopped_at)
        .map_or("unknown", |(_, name)| name);
    diagnostic(
        host,
        &format!(
            "stopped at request {name}; settle it, git rebase --continue, then run this script again; nothing pushed"
        ),
    );
    file_changed(host, removed)?;
    Err(Stop::Failed)
}

fn push(host: &mut Host, branch: &str, lease: &str) -> Result<(), Stop> {
    let lease = format!("--force-with-lease={branch}:{lease}");
    // The host captures separate streams. Merge them before capturing this one
    // child, so git's porcelain and progress reach stderr in their write order.
    let mut push = Program::new(
        b"sh",
        &[
            b"-c",
            b"exec git \"$@\" 2>&1",
            b"sh",
            b"push",
            b"--porcelain",
            lease.as_bytes(),
            b"origin",
            branch.as_bytes(),
        ],
    );
    push.stderr = Stream::Keep;
    let pushed = child(host, push)?;
    host.err(&pushed.stdout);
    host.err(&pushed.stderr);
    if pushed.code != Some(0) {
        return Err(if stale_lease(&pushed.stdout) { Stop::Moved(branch.into()) } else { Stop::Failed });
    }
    let short = git_text(host, &["rev-parse", "--short", "HEAD"])?;
    host.out(format!("pushed {branch} at {short}\n").as_bytes());
    Ok(())
}

fn stale_lease(output: &[u8]) -> bool {
    output.split(|&byte| byte == b'\n').any(|line| {
        line.windows(b"[rejected]".len()).position(|part| part == b"[rejected]").is_some_and(|index| {
            line[index + b"[rejected]".len()..].windows(b"(stale info)".len()).any(|part| part == b"(stale info)")
        })
    })
}

fn file_changed(host: &mut Host, answer: Answer) -> Result<(), Stop> {
    if answer == Answer::Done { Ok(()) } else { Err(effect_failed(host, answer)) }
}

fn effect_failed(host: &mut Host, answer: Answer) -> Stop {
    let Answer::Failed(why) = answer else { unreachable!("unexpected host answer: {answer:?}") };
    diagnostic(host, &why);
    Stop::Failed
}

fn diagnostic(host: &mut Host, why: &str) {
    host.err(format!("look-refresh: {why}\n").as_bytes());
}

fn text(bytes: &[u8]) -> Cow<'_, str> {
    String::from_utf8_lossy(without_newlines(bytes))
}

fn trim_fields(mut value: &[u8]) -> &[u8] {
    while let [b' ' | b'\t', rest @ ..] = value {
        value = rest;
    }
    while let [rest @ .., b' ' | b'\t'] = value {
        value = rest;
    }
    value
}

fn without_newlines(mut bytes: &[u8]) -> &[u8] {
    while let Some(trimmed) = bytes.strip_suffix(b"\n") {
        bytes = trimmed;
    }
    bytes
}

fn git_path(host: &mut Host, args: &[&str]) -> Result<PathBuf, Stop> {
    let bytes = checked_git(host, args)?;
    Ok(OsStr::from_bytes(without_newlines(&bytes)).into())
}

fn git_text(host: &mut Host, args: &[&str]) -> Result<String, Stop> {
    Ok(text(&checked_git(host, args)?).into_owned())
}

fn checked_git(host: &mut Host, args: &[&str]) -> Result<Box<[u8]>, Stop> {
    success(git(host, args, Stream::Keep, Stream::Err)?)
}

fn git_to_err(host: &mut Host, args: &[&str]) -> Result<(), Stop> {
    success(git(host, args, Stream::Err, Stream::Err)?).map(|_| ())
}

fn success(ran: Ran) -> Result<Box<[u8]>, Stop> {
    if ran.code == Some(0) { Ok(ran.stdout) } else { Err(Stop::Failed) }
}

fn ancestor(host: &mut Host, commit: &str, of: &str) -> Result<bool, Stop> {
    Ok(git(host, &["merge-base", "--is-ancestor", commit, of], Stream::Keep, Stream::Err)?.code == Some(0))
}

fn git_program(args: &[&str], stdout: Stream, stderr: Stream) -> Program {
    let mut program = Program::new(b"git", &args.iter().map(|arg| arg.as_bytes()).collect::<Vec<_>>());
    program.stdout = stdout;
    program.stderr = stderr;
    program
}

fn git(host: &mut Host, args: &[&str], stdout: Stream, stderr: Stream) -> Result<Ran, Stop> {
    child(host, git_program(args, stdout, stderr))
}

fn child(host: &mut Host, program: Program) -> Result<Ran, Stop> {
    let show_start_failure = program.stderr == Stream::Err;
    match host.ask(Ask::Run(program)) {
        Answer::Ran(ran) => {
            if show_start_failure && ran.code.is_none() {
                host.err(&ran.stderr);
            }
            Ok(ran)
        }
        answer => Err(effect_failed(host, answer)),
    }
}

#[cfg(test)]
mod tests;
