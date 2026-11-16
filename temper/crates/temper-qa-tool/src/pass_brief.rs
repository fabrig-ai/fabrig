//! The ordered pass brief (qa/passes.md, 4), resolving each landing's drafts as triage does.
use crate::{
    Case, CaseState, FindingState, Home, Kind, Ledger, Pass, PassStart, Refusal, Session, brief::landing_drafts,
    faces::GitFace, pass_files,
};
use std::fmt::Write;

pub(crate) fn render(
    git: &mut impl GitFace,
    home: &Home,
    start: &PassStart,
    ledger: &Ledger,
) -> Result<String, Refusal> {
    let pass = ledger.passes.iter().find(|pass| pass.number == start.pass).expect("recorded pass");
    let session = ledger.sessions.iter().find(|session| session.number == start.session).expect("recorded session");
    let resumed = session.before.as_ref().is_some_and(|before| before.state.is_some());
    let mut text = String::from(
        "# What a pass is\n\nRun the green path only. Run ctl cases first in a ctl session, `temper-qa pass open --ctl`, with no browser. A ctl case's steps are ctl and forge lines; compare its see with what ctl printed. Start each case from its setup's restored copy, restoring between cases. Cases on one setup may share a restore only where none changes what the next sees; say which in the outcome's notes. Compare a page's words where a case gives them (see, as a `web see` line for a `web` case and a `tui see` line for a `tui` case); judge the rest against the draft's promises. A `tui` case's steps are `tui` lines; the TUI starts at its first line and the session signs it in. Keep its finding's evidence as the screen's rows: `pass line \"tui look\"`'s answer saved to a file. Looks are a person's judgement, not this pass's. Record each outcome as reached and each finding as found. A finding never stops the pass. Blocked is for the pass's own reasons, failed for temper's. What is off the path is a note unless it is a security hole or lost data. A fault beside a case on its landing's surface is a finding. The fakes alone: the person's forge is the box's fake Forgejo. A case no line or setup can bring to its state is a gap, not blocked: `temper-qa pass gap CASE KIND WHY`.\n\n## Pass\n\n",
    );
    writeln!(text, "Pass {}; place {} ({}); build {}", pass.number, pass.place, pass.kind.name(), pass.build)
        .expect("string");
    if pass.kind == Kind::Deployment {
        text.push_str("At this deployment's place run ctl cases first in a ctl session, `temper-qa pass open --ctl`, then web and tui cases in a full session, `temper-qa pass open`, the browser and the TUI on the deployment's engine. The owner's changes are `deployment patch DOCUMENT PATH` lines; write their PATHs in the pass's files directory first. These lines end the TUI and load the page again.\n");
    }
    writeln!(text, "Tree: {}", home.pass_tree(&pass.place).display()).expect("string");
    if resumed {
        let request = pass.requests.last().expect("resume's request");
        let goal = request.goal.expect("resume's goal");
        let landed = ledger.landings.iter().any(|landing| landing.goal == goal);
        writeln!(text, "Resumed after goal {goal}: {}. Check open findings before anything else, then failed cases, then the cases taken at this resume.", if landed { "landed" } else { "ended without landing" }).expect("string");
    }
    writeln!(text, "Session {}; bound {} seconds.\n\n## Landings\n", start.session, home.configuration.bounds.pass)
        .expect("string");
    let mut cases: Vec<_> = ledger
        .cases
        .iter()
        .filter(|case| case.pass == Some(pass.number) && case.state == CaseState::Assigned)
        .collect();
    cases.sort_by_key(|case| (case.surface != "ctl", case.surface != "web"));
    write_landings(git, &mut text, ledger, &cases)?;
    if resumed {
        write_resume(&mut text, ledger, session, &cases);
    }
    text.push_str("\n## Cases\n\n");
    for case in cases {
        write_case(&mut text, case);
    }
    write_setups(git, &mut text, home, pass)?;
    writeln!(text, "\nCommit files directory: {}", pass_files::files(home, pass.number).display()).expect("string");
    text.push_str(COMMANDS);
    Ok(text)
}

fn write_landings(git: &mut impl GitFace, text: &mut String, ledger: &Ledger, cases: &[&Case]) -> Result<(), Refusal> {
    let mut landings = Vec::new();
    for case in cases {
        for landing in case.landings() {
            if !landings.contains(&landing) {
                landings.push(landing);
            }
        }
    }
    for number in landings {
        let landing = ledger.landings.iter().find(|landing| landing.number == number).expect("case's landing");
        writeln!(
            text,
            "Landing {}; goal {}; merge {}\n{}",
            landing.number, landing.goal, landing.commit, landing.title
        )
        .expect("string");
        let drafts = landing_drafts(git, landing)?;
        if drafts.is_empty() {
            text.push_str("No draft is named; the request was the title's words.\n");
        }
        for path in drafts {
            writeln!(text, "- {path} (in this session's tree)").expect("string");
        }
    }
    Ok(())
}

fn write_resume(text: &mut String, ledger: &Ledger, session: &Session, cases: &[&Case]) {
    text.push_str("\n## Open findings\n\nCheck these first, from their setup and lines.\n");
    for finding in ledger
        .findings
        .iter()
        .filter(|finding| Some(finding.pass) == session.pass && finding.state == FindingState::Open)
    {
        writeln!(text, "Finding {}: {}\nSetup: {}\nLines:", finding.number, finding.title, finding.setup)
            .expect("string");
        for line in &finding.lines {
            writeln!(text, "- {line}").expect("string");
        }
        writeln!(text, "Expected: {}\nSeen: {}", finding.expected, finding.seen).expect("string");
    }
    text.push_str("\n## Failed cases\n\nRun these after the open findings.\n");
    let before = &session.before.as_ref().expect("resume snapshot").cases;
    for case in cases {
        if before.iter().any(|before| before.number == case.number && before.state == CaseState::Failed) {
            write_case(text, case);
        }
    }
}

fn write_setups(git: &mut impl GitFace, text: &mut String, home: &Home, pass: &Pass) -> Result<(), Refusal> {
    writeln!(text, "\n## Setups\n\nDirectory: {}", pass_files::setups(home, pass.number).display()).expect("string");
    if pass.kind == Kind::Box {
        text.push_str("- fresh: an empty box\n");
    }
    for (name, kind) in crate::brief::setup_names(git, &pass.build, &home.configuration.surfaces)? {
        if name != "fresh" && kind == pass.kind {
            let path = format!("{}/{name}/README.md", pass_files::SETUPS);
            let bytes = git.file(&pass.build, &path)?.unwrap_or_default();
            writeln!(text, "- {name}: {}", first_sentence(&String::from_utf8_lossy(&bytes))).expect("string");
        }
    }
    text.push_str("\nA case whose state no setup reaches is brought there from the nearest. Once it has run, turn its kept lines into a recipe. If a recipe no longer walks, compare the landings' drafts: edit it for an intended flow change and retry; otherwise record a finding for the broken step, fail the affected cases with it, and continue the others.\n");
    Ok(())
}

fn write_case(text: &mut String, case: &Case) {
    writeln!(
        text,
        "Case {}: {} / {}; landings {}",
        case.number,
        case.surface,
        case.page,
        crate::ledger::numbers(&case.landings())
    )
    .expect("string");
    for ask in &case.asks {
        writeln!(text, "- landing {}; setup {}; adds: {}", ask.landing, ask.setup, ask.adds).expect("string");
        for step in &ask.steps {
            writeln!(text, "  - do: {}\n    expect: {}", step.act, step.expect).expect("string");
            if let Some(see) = &step.see {
                writeln!(text, "    see: {see}").expect("string");
            }
        }
    }
}

fn first_sentence(text: &str) -> String {
    let paragraph =
        text.split("\n\n").find(|part| !part.trim().is_empty() && !part.trim_start().starts_with('#')).unwrap_or("");
    let paragraph = paragraph.split_whitespace().collect::<Vec<_>>().join(" ");
    paragraph.find(". ").map_or_else(|| paragraph.clone(), |end| paragraph[..=end].to_owned())
}

const COMMANDS: &str = r#"
## Commands

Place commands:
- `temper-qa pass setup NAME`: restore fresh or a recipe of this pass's setups. Close the drive session first. A broken recipe exits 5, a refused store 3, a failed box 4.
- `temper-qa pass open [--ctl]`: start the restored setup's drive session; it answers ready. Restore a setup first.
- `temper-qa pass line LINE`: LINE one shell argument, a drive line; prints its answer, exit 0 for ok, 5 for a failed line, 4 for the failed box.
- `temper-qa pass close`: stop the drive session and print its kept lines. Looks stay until the next session; record evidence before then.
- `temper-qa pass recipe NAME FILE`: FILE a JSON object (or - for stdin): {"after":"fresh or nearest setup","lines":["web see Ready"],"readme":"The state reached.","scripts":{}}; scripts optional. A ctl-only recipe's lines end with `check` and its ctl reads and waits. Close the drive session first. This writes and proves a new recipe by walking it from its parent.

`pass setup`, `pass recipe` and `pass open` may take minutes. Each walk or start prints a line as it goes: run them where your shell's own timeout does not cut them.

Drive lines: one client, record and fields per line, fields separated by one space, words bare and bytes in double quotes. Quoted bytes take printable ASCII alone; write other bytes as `\xHH`, with `\"` and `\\` for quote and backslash. Use `web press ROLE NAME [ORDER]`, `web submit ROLE NAME [ORDER]`, `web toggle ROLE NAME [ORDER]`, `web input ROLE NAME VALUE [ORDER]`, `web see TEXT`, `web reload PATH`, `web look [WIDTH]`; `tui key KEY`, `tui see TEXT`, `tui stop`, `tui look`; `ctl ARGS...`; `forge commit forgejo REPOSITORY MESSAGE PATH...`, `forge push forgejo REPOSITORY BRANCH MESSAGE PATH...`, `forge checks forgejo REPOSITORY` or `end`. The forge lines make one commit as the person on the box's fake Forgejo and wait on its checks. Commit writes the default branch; push writes BRANCH, making it at the default branch's head when absent. REPOSITORY is OWNER/NAME; give 1–16 distinct relative PATHs, 1–256 bytes without empty, . or .. segments. Write each named file under the pass's files directory at its repository path before the line; each a regular file at most 64 KiB. ROLE is a lowercase accessible role, ORDER starts at 0, WIDTH is 320–3840. A look prints accessible text or the TUI's frame; a web look with WIDTH also writes a screenshot. The TUI starts signed in at its first line. Each ctl field is one argument; ctl stdin is closed. An act and waiting on it are separate lines, e.g. `ctl task show 3 --until ended`. Answers print shown lines prefixed `| `, then `ok` or `failed: WHY`. Looks are not kept in reproduction lines.

Ctl's box values: a whole field, bare or quoted, is replaced when the line runs: `{forgejo.api}` is HOST:PORT of the box's fake Forgejo, `{forgejo.git}` its http://HOST:PORT git base, `{forgejo.admin}` the path of its site admin's token file, and `{git}` the box's fake git program. The kept line keeps the name, never a place's path, port or credential. For example: `ctl forge connect forgejo-admin {forgejo.admin} --first work/hello --project Hello --api {forgejo.api} --git {forgejo.git} --git-program {git} --key qa-hello`. A missing box value fails the line, naming it.

Recording commands (each record is kept at once; the session is bounded):
- `temper-qa pass gap CASE KIND WHY`: KIND ctl, fake or qa; WHY one nonempty argument. Record what is missing to judge the case.
- `temper-qa pass outcome CASE RESULT NOTES [--finding N]...`: RESULT passed, failed or blocked; NOTES one argument, nonempty for blocked. Failed names at least one finding of that case. A later outcome in this session replaces the earlier one for its end.
- `temper-qa pass finding FILE`: FILE a JSON object or - for stdin:
  {"case":7,"title":"The fault","severity":"medium","setup":"fresh","lines":["web press button Start"],"expected":"a goal starts","seen":"no goal","area":"AREA","evidence":["/path/to/look.png"]}
  area and evidence optional; severity low, medium or high. Give the kept lines from the setup to the fault without looks. Evidence: at most 16 regular files, at most 16 MiB each. It is copied at once; use distinct file names. A finding may be linked to an existing open finding; use the number returned.
- `temper-qa pass check FINDING RESULT NOTES`: on a resume, check each open finding, RESULT holds or gone, NOTES one argument.
- `temper-qa pass note WORDS`: WORDS one nonempty argument for what is off the path.
- `temper-qa report PASS`, `temper-qa show`, `temper-qa lines`: read the kept records.
"#;
