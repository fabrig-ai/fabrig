//! Request files and deployment commands (qa/findings.md, 4–6).
use crate::faces::{Environment, ProcessCommand, Processes};
use crate::{Exit, Home, Ledger, Pass, PassState, Refusal, Request, refused};
use std::{
    fmt::Write,
    fs,
    path::{Component, Path, PathBuf},
};

const QA: &str = "temper/docs/development/qa";

pub(crate) fn step<P: Processes>(
    home: &Home,
    ledger: &mut Ledger,
    processes: &mut P,
    captured: &Environment,
    now: u64,
) -> Result<(), Refusal> {
    let environment = environment(home, captured);
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
        let record = ledger.passes.iter().find(|record| record.number == pass).expect("unfiled pass");
        let ask = record.requests.iter().find(|record| record.number == request).expect("unfiled request");
        let directory = directory(home, pass, request);
        prepare(home, ledger, record, ask, &directory)?;
        let answer = processes
            .run(&command(home.configuration.request.command.clone(), &directory, "output", &environment), 600)?;
        if answer.exit != Exit::Code(0) {
            return Err(refused(format!("pass {pass}, request {request}: command exited {:?}", answer.exit)));
        }
        let goal = if ask.findings.is_empty() {
            None
        } else {
            Some(
                answer
                    .last_stdout
                    .as_deref()
                    .and_then(goal_number)
                    .ok_or_else(|| refused(format!("pass {pass}, request {request}: expected goal N on stdout")))?,
            )
        };
        ledger.file_request(pass, request, goal, now)?;
    }
    let Some(arguments) = &home.configuration.request.state else { return Ok(()) };
    let waiting: Vec<_> = ledger
        .passes
        .iter()
        .filter(|pass| pass.state == PassState::Waiting)
        .filter_map(|pass| {
            let request = pass.requests.last()?;
            let goal = request.goal?;
            (request.ended.is_none() && !ledger.landings.iter().any(|landing| landing.goal == goal)).then_some((
                pass.number,
                request.number,
                goal,
            ))
        })
        .collect();
    for (pass, request, goal) in waiting {
        let directory = directory(home, pass, request);
        fs::create_dir_all(&directory).map_err(io)?;
        let mut arguments = arguments.clone();
        arguments.push(goal.to_string());
        let answer = processes.run(&command(arguments, &directory, "state-output", &environment), 60)?;
        if answer.exit != Exit::Code(0) {
            return Err(refused(format!("goal {goal}: state command exited {:?}", answer.exit)));
        }
        match answer.last_stdout.as_deref() {
            Some("live") => (),
            Some("ended") => ledger.end_goal(goal, now)?,
            other => return Err(refused(format!("goal {goal}: expected live or ended on stdout, got {other:?}"))),
        }
    }
    Ok(())
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

fn prepare(home: &Home, ledger: &Ledger, pass: &Pass, request: &Request, directory: &Path) -> Result<(), Refusal> {
    match fs::remove_dir_all(directory) {
        Ok(()) => (),
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => (),
        Err(why) => return Err(io(why)),
    }
    let tree = directory.join("tree");
    fs::create_dir_all(&tree).map_err(io)?;
    for recipe in &request.recipes {
        if !matches!(Path::new(recipe).components().collect::<Vec<_>>().as_slice(), [Component::Normal(_)]) {
            return Err(refused(format!("invalid recipe name {recipe}")));
        }
        copy_recipe(
            &home.directory.join("passes").join(pass.number.to_string()).join("setups").join(recipe),
            &tree.join(QA).join("setups").join(recipe),
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
        fs::create_dir_all(document.parent().expect("request path parent")).map_err(io)?;
        fs::write(document, render(ledger, request, &subject)).map_err(io)?;
        fs::write(directory.join("goal"), format!("Read the QA request at {path} and fix each finding it lists, each with a test in the lowest world that reproduces it, starting from its lines.\n")).map_err(io)?;
        subject
    };
    fs::write(directory.join("message"), format!("{subject}\n\nQA-pass: {}\n", pass.number)).map_err(io)
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
fn copy_recipe(source: &Path, destination: &Path) -> Result<(), Refusal> {
    fs::create_dir_all(destination).map_err(io)?;
    for entry in fs::read_dir(source).map_err(io)? {
        let entry = entry.map_err(io)?;
        let kind = entry.file_type().map_err(io)?;
        if kind.is_dir() {
            copy_recipe(&entry.path(), &destination.join(entry.file_name()))?;
        } else if kind.is_file() {
            fs::copy(entry.path(), destination.join(entry.file_name())).map_err(io)?;
        } else {
            return Err(refused(format!("recipe entry {} is not a file or directory", entry.path().display())));
        }
    }
    Ok(())
}
#[expect(clippy::needless_pass_by_value, reason = "map_err passes its owned I/O error to this adapter")]
fn io(why: std::io::Error) -> Refusal {
    refused(format!("request files: {why}"))
}
