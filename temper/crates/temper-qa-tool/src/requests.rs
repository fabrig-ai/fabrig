//! Request files and deployment commands (qa/findings.md, 4–6).
use crate::faces::{
    DirectoryMode, Environment, FilesFace, PathKind, ProcessCommand, Processes, RemoveMode, WriteOptions,
    remove_missing,
};
use crate::{Exit, Home, Ledger, Pass, PassState, Refusal, Request, refused};
use std::{
    ffi::OsStr,
    fmt::Write,
    os::unix::ffi::OsStrExt,
    path::{Component, Path, PathBuf},
};

const QA: &str = "temper/docs/development/qa";

pub(crate) fn step(
    home: &Home,
    ledger: &mut Ledger,
    processes: &mut impl Processes,
    files: &mut impl FilesFace,
    captured: &Environment,
    now: u64,
) -> Result<(), Refusal> {
    let environment = environment(home, captured);
    let mut failures = Vec::new();
    let mut unfiled: Vec<_> = ledger
        .passes
        .iter()
        .flat_map(|pass| {
            pass.requests
                .iter()
                .filter(|request| request.filed.is_none())
                .map(|request| (request.time, pass.number, request.number))
        })
        .collect();
    unfiled.sort_unstable();
    for (_, pass, request) in unfiled {
        if let Err(why) = file_request(home, ledger, processes, files, &environment, (pass, request), now) {
            failures.push(format!("pass {pass}, request {request}: {why}"));
        }
    }
    if let Err(why) = file_gaps(home, ledger, processes, files, &environment, now) {
        failures.push(why.to_string());
    }
    if let Some(arguments) = &home.configuration.request.state {
        let waiting: Vec<_> =
            ledger
                .passes
                .iter()
                .filter(|pass| pass.state == PassState::Waiting)
                .filter_map(|pass| {
                    let request = pass.requests.last()?;
                    let goal = request.goal?;
                    (request.ended.is_none() && !ledger.landings.iter().any(|landing| landing.goal == goal))
                        .then_some((pass.number, request.number, goal))
                })
                .collect();
        for (pass, request, goal) in waiting {
            let directory = directory(home, pass, request);
            let result = files
                .directory(&directory, DirectoryMode::Parents)
                .map_err(io)
                .and_then(|()| check_goal(ledger, processes, arguments, &directory, &environment, goal, now));
            if let Err(why) = result {
                failures.push(format!("goal {goal}: {why}"));
            }
        }
    }
    if failures.is_empty() { Ok(()) } else { Err(refused(failures.join("; "))) }
}

fn file_request(
    home: &Home,
    ledger: &mut Ledger,
    processes: &mut impl Processes,
    files: &mut impl FilesFace,
    environment: &Environment,
    number: (u64, u64),
    now: u64,
) -> Result<(), Refusal> {
    let (pass, request) = number;
    let record = ledger.passes.iter().find(|record| record.number == pass).expect("unfiled pass");
    let ask = record.requests.iter().find(|record| record.number == request).expect("unfiled request");
    let directory = directory(home, pass, request);
    prepare(home, ledger, record, ask, &directory, files)?;
    let answer =
        processes.run(&command(home.configuration.request.command.clone(), &directory, "output", environment), 600)?;
    if answer.exit != Exit::Code(0) {
        return Err(refused(format!("command exited {:?}", answer.exit)));
    }
    let goal = if ask.findings.is_empty() {
        None
    } else {
        Some(answer.last_stdout.as_deref().and_then(goal_number).ok_or_else(|| refused("expected goal N on stdout"))?)
    };
    ledger.file_request(pass, request, goal, now)
}

fn check_goal(
    ledger: &mut Ledger,
    processes: &mut impl Processes,
    arguments: &[String],
    directory: &Path,
    environment: &Environment,
    goal: u64,
    now: u64,
) -> Result<(), Refusal> {
    let mut arguments = arguments.to_vec();
    arguments.push(goal.to_string());
    let answer = processes.run(&command(arguments, directory, "state-output", environment), 60)?;
    if answer.exit != Exit::Code(0) {
        return Err(refused(format!("state command exited {:?}", answer.exit)));
    }
    match answer.last_stdout.as_deref() {
        Some("live") => Ok(()),
        Some("ended") => ledger.end_goal(goal, now),
        other => Err(refused(format!("expected live or ended on stdout, got {other:?}"))),
    }
}

fn environment(home: &Home, captured: &Environment) -> Environment {
    let mut environment = captured.clone();
    environment.retain(|(name, _)| {
        name.as_ref() != crate::HOME.name.as_bytes() && name.as_ref() != crate::AREA.name.as_bytes()
    });
    environment.push((crate::HOME.name.as_bytes().into(), home.directory.as_os_str().as_encoded_bytes().into()));
    environment.push((crate::AREA.name.as_bytes().into(), home.configuration.request.area.as_bytes().into()));
    environment
}
fn command(arguments: Vec<String>, directory: &Path, output: &str, environment: &Environment) -> ProcessCommand {
    ProcessCommand {
        arguments,
        directory: directory.into(),
        stdin: PathBuf::from("/dev/null"),
        output: directory.join(output),
        environment: environment.clone(),
    }
}
fn directory(home: &Home, pass: u64, request: u64) -> PathBuf {
    home.directory.join("passes").join(pass.to_string()).join("requests").join(request.to_string())
}
fn goal_number(line: &str) -> Option<u64> {
    let number = line.strip_prefix("goal ")?;
    if number.is_empty() || !number.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    number.parse().ok().filter(|number| *number > 0)
}

fn prepare(
    home: &Home,
    ledger: &Ledger,
    pass: &Pass,
    request: &Request,
    directory: &Path,
    files: &mut impl FilesFace,
) -> Result<(), Refusal> {
    remove_missing(files, directory, RemoveMode::Tree).map_err(io)?;
    let tree = directory.join("tree");
    files.directory(&tree, DirectoryMode::Parents).map_err(io)?;
    for recipe in &request.recipes {
        if !matches!(Path::new(recipe).components().collect::<Vec<_>>().as_slice(), [Component::Normal(_)]) {
            return Err(refused(format!("invalid recipe name {recipe}")));
        }
        copy_recipe(
            &home.directory.join("passes").join(pass.number.to_string()).join("setups").join(recipe),
            &tree.join(QA).join("setups").join(recipe),
            files,
        )?;
    }
    let subject = if request.findings.is_empty() {
        format!("QA pass {}: recipes", pass.number)
    } else {
        let subject = format!(
            "QA pass {}, request {}: {} findings at {}",
            pass.number,
            request.number,
            request.findings.len(),
            pass.build.chars().take(12).collect::<String>()
        );
        let path = format!("{QA}/requests/pass-{}-{}.md", pass.number, request.number);
        let document = tree.join(&path);
        files.directory(document.parent().expect("request path parent"), DirectoryMode::Parents).map_err(io)?;
        write(files, &document, &render(ledger, request, &subject)).map_err(io)?;
        write(files, &directory.join("goal"), &format!("Read the QA request at {path} and fix each finding it lists, each with a test in the lowest world that reproduces it, starting from its lines.\n")).map_err(io)?;
        subject
    };
    write(files, &directory.join("message"), &format!("{subject}\n\nQA-pass: {}\n", pass.number)).map_err(io)
}

fn render(ledger: &Ledger, request: &Request, subject: &str) -> String {
    let mut document = format!(
        "# {subject}\n\nFix each finding, each fix with a test in the lowest world that reproduces it, starting from the finding's lines. Deliver what the cases expected and nothing past it.\n"
    );
    for number in &request.findings {
        let finding = ledger.findings.iter().find(|finding| finding.number == *number).expect("request finding");
        writeln!(
            document,
            "\n## {}. {}\n\nSeverity: {}\n\nSurface: {}\n\nPage: {}\n\nArea: {}\n\nLandings:",
            finding.number, finding.title, finding.severity, finding.surface, finding.page, finding.area
        )
        .expect("string");
        for number in &finding.landings {
            let landing = ledger.landings.iter().find(|landing| landing.number == *number).expect("finding landing");
            writeln!(document, "- goal {}: {}", landing.goal, landing.title).expect("string");
        }
        let setup =
            if finding.setup == "fresh" { "fresh".to_owned() } else { format!("{QA}/setups/{}/", finding.setup) };
        writeln!(document, "\nSetup: {setup}\n\nReproduction:\n").expect("string");
        // A drive line may itself contain backticks; keep it inside the text fence.
        let fence = "`".repeat(
            finding
                .lines
                .iter()
                .flat_map(|line| line.split(|ch| ch != '`'))
                .map(str::len)
                .max()
                .unwrap_or(0)
                .saturating_add(1)
                .max(3),
        );
        writeln!(
            document,
            "{fence}text\n{}\n{fence}\n\nExpected: {}\n\nSeen: {}\n\nEvidence (kept beside the QA ledger):",
            finding.lines.join("\n"),
            finding.expected,
            finding.seen
        )
        .expect("string");
        for name in &finding.evidence {
            writeln!(document, "- evidence/{}/{name}", finding.number).expect("string");
        }
        document.push_str("\nEarlier goals:");
        if finding.goals.is_empty() {
            document.push_str(" none\n");
        } else {
            document.push('\n');
            for goal in &finding.goals {
                writeln!(document, "- goal {goal}").expect("string");
            }
        }
    }
    if !request.recipes.is_empty() {
        document.push_str("\n## Recipes\n\n");
        for recipe in &request.recipes {
            writeln!(document, "- {QA}/setups/{recipe}/").expect("string");
        }
    }
    document
}
fn copy_recipe(source: &Path, destination: &Path, files: &mut impl FilesFace) -> Result<(), Refusal> {
    files.directory(destination, DirectoryMode::Parents).map_err(io)?;
    for entry in files.scan(source).map_err(io)? {
        let entry = entry.map_err(io)?;
        let name = OsStr::from_bytes(&entry.name);
        let from = source.join(name);
        let to = destination.join(name);
        let kind = entry.kind.map_err(io)?;
        if kind == PathKind::Directory {
            copy_recipe(&from, &to, files)?;
        } else if kind == PathKind::File {
            files.copy(&from, &to).map_err(io)?;
        } else {
            return Err(refused(format!("recipe entry {} is not a file or directory", from.display())));
        }
    }
    Ok(())
}

fn write(files: &mut impl FilesFace, path: &Path, text: &str) -> std::io::Result<()> {
    files.write(path, text.as_bytes(), WriteOptions::REPLACE, false)
}
#[expect(clippy::needless_pass_by_value, reason = "map_err passes its owned I/O error to this adapter")]
fn io(why: std::io::Error) -> Refusal {
    refused(format!("request files: {why}"))
}

// A gap request has only documents and a message; it never has a goal (qa/gaps.md, 6).
fn file_gaps(
    home: &Home,
    ledger: &mut Ledger,
    processes: &mut impl Processes,
    files: &mut impl FilesFace,
    environment: &Environment,
    now: u64,
) -> Result<(), Refusal> {
    let gaps: Vec<_> = ledger.gaps.iter().filter(|gap| gap.state == crate::GapState::Open).collect();
    if gaps.is_empty() {
        return Ok(());
    }
    let numbers = gaps.iter().map(|gap| gap.number).collect::<Vec<_>>();
    let directory =
        home.directory.join("gaps").join("requests").join(numbers.first().expect("unfiled gaps").to_string());
    remove_missing(files, &directory, RemoveMode::Tree).map_err(io)?;
    let tree = directory.join("tree").join(QA).join("gaps");
    files.directory(&tree, DirectoryMode::Parents).map_err(io)?;
    for gap in gaps {
        let landing = ledger.landings.iter().find(|landing| landing.number == gap.landing).expect("gap landing");
        let mut document = format!(
            "# QA gap {}: {}\n\nKind: {}\n\nOwner: {}\n\nLanding {}: goal {}, {}\n\nCommit: {}\n\nSession: {}\n\nWhy: {}\n",
            gap.number,
            gap.kind,
            gap.kind,
            gap.owner,
            landing.number,
            landing.goal,
            landing.title,
            landing.commit,
            gap.session,
            gap.why
        );
        if let Some(case) = gap.case {
            let case = ledger.cases.iter().find(|record| record.number == case).expect("gap case");
            writeln!(document, "\nCase {}: {} / {}", case.number, case.surface, case.page).expect("string");
            for ask in &case.asks {
                writeln!(document, "\nLanding {}; setup {}; adds: {}", ask.landing, ask.setup, ask.adds)
                    .expect("string");
                for step in &ask.steps {
                    writeln!(document, "- do: {}\n  expect: {}", step.act, step.expect).expect("string");
                    if let Some(see) = &step.see {
                        writeln!(document, "  see: {see}").expect("string");
                    }
                }
            }
        }
        writeln!(document, "\nA landing closes this gap with the trailer `QA-gap: {}`.\n", gap.number).expect("string");
        write(files, &tree.join(format!("gap-{}.md", gap.number)), &document).map_err(io)?;
    }
    let list = crate::ledger::numbers(&numbers);
    write(files, &directory.join("message"), &format!("QA gaps {list}: {} open\n\nQA-gaps: {list}\n", numbers.len()))
        .map_err(io)?;
    let answer =
        processes.run(&command(home.configuration.request.command.clone(), &directory, "output", environment), 600)?;
    if answer.exit != Exit::Code(0) {
        return Err(refused(format!("gaps {list}: command exited {:?}", answer.exit)));
    }
    for gap in ledger.gaps.iter_mut().filter(|gap| numbers.contains(&gap.number)) {
        gap.state = crate::GapState::Filed;
        gap.filed = Some(now);
    }
    Ok(())
}
