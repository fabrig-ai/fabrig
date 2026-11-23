//! Frozen inputs and offline suite decisions (smith/docs/design/benchmarks/README.md, 5).

use skein_fake_machine::{How, Is, Item, Machine, Opened};
use smith_bench::{
    CatalogueTask, Kind, ModelTiers, PinnedConfiguration, Refusal, Suite,
    testing::task::{
        SeedEntry, SeedFiles, SeedKind, check_task_with, configuration, parse_document, parse_task, probe_cases,
        sample_catalogue, seed_digest_with, validate_suite_with,
    },
};
use std::{
    cell::RefCell,
    io,
    path::{Path, PathBuf},
};

const DOCUMENT: &str = include_str!("../recorded/empty-seed-probe.toml");
const SUITE: &str = include_str!("../recorded/suites/sample.toml");

struct Entry {
    path: PathBuf,
    kind: Is,
}

impl SeedEntry for Entry {
    fn path(&self) -> PathBuf {
        self.path.clone()
    }

    fn kind(&self) -> io::Result<SeedKind> {
        Ok(match self.kind {
            Is::Directory => SeedKind::Directory,
            Is::File => SeedKind::File,
            Is::Link | Is::Fifo | Is::Device => SeedKind::Other,
        })
    }
}

struct SeedTree<'a> {
    path: &'a Path,
    root: Opened,
    machine: RefCell<Machine>,
}

impl<'a> SeedTree<'a> {
    fn new(path: &'a Path, items: &[Item]) -> Self {
        let mut machine = Machine::new();
        let root = machine.lay(items);
        Self { path, root, machine: RefCell::new(machine) }
    }

    fn with<T>(&self, path: &Path, how: How, read: impl FnOnce(&mut Machine, Opened) -> T) -> T {
        let mut machine = self.machine.borrow_mut();
        let relative = path.strip_prefix(self.path).expect("path in fake seed");
        let relative = if relative.as_os_str().is_empty() { Path::new(".") } else { relative };
        let handle = machine.open(self.root, relative.as_os_str().as_encoded_bytes(), how).expect("fake seed node");
        let result = read(&mut machine, handle);
        machine.close(handle);
        result
    }
}

impl SeedFiles for SeedTree<'_> {
    type Entry = Entry;
    type Children = std::vec::IntoIter<io::Result<Entry>>;

    fn ordinary_directory(&self, path: &Path) -> io::Result<bool> {
        Ok(self.with(path, How::PathNoFollow, |machine, handle| machine.stat(handle).is == Is::Directory))
    }

    fn children(&self, path: &Path) -> io::Result<Self::Children> {
        let entries = self.with(path, How::Directory, |machine, handle| {
            machine.list(handle, u32::MAX, u32::MAX).expect("fake directory entries")
        });
        Ok(entries
            .into_iter()
            .map(|(kind, bytes)| {
                let name = std::str::from_utf8(&bytes).expect("UTF-8 fixture name");
                Ok(Entry { path: path.join(name), kind })
            })
            .collect::<Vec<_>>()
            .into_iter())
    }

    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        Ok(self.with(path, How::Read, |machine, handle| {
            let size = u32::try_from(machine.stat(handle).size).expect("small fixture file");
            machine.read(handle, 0, size).expect("fake seed bytes")
        }))
    }

    fn executable(&self, path: &Path) -> io::Result<bool> {
        Ok(self.with(path, How::PathNoFollow, |machine, handle| machine.stat(handle).mode & 0o111 != 0))
    }
}

fn recorded_seed(root: &Path) -> SeedTree<'_> {
    SeedTree::new(root, &[Item::file(b"README.md", include_bytes!("../recorded/tasks/valid/seed/README.md"))])
}

pub(super) fn check_probe(file: &Path, document: &str, seed: &[Item]) -> Result<smith_bench::Task, Refusal> {
    let task = parse_task(file, document)?;
    check_task_with(file, task, read_guard, |root| seed_digest_with(root, &SeedTree::new(root, seed)))
}

fn read_guard(path: &Path) -> io::Result<String> {
    let text = match path.to_str() {
        Some("benchmarks/README.md") => include_str!("../../../docs/design/benchmarks/README.md"),
        Some("benchmarks/agents.md") => include_str!("../../../docs/design/benchmarks/agents.md"),
        Some("domain/budget.md") => include_str!("../../../docs/design/domain/budget.md"),
        Some("domain/run.md") => include_str!("../../../docs/design/domain/run.md"),
        Some("domain/run-messages.md") => include_str!("../../../docs/design/domain/run-messages.md"),
        Some("domain/run-start.md") => include_str!("../../../docs/design/domain/run-start.md"),
        Some("domain/tools.md") => include_str!("../../../docs/design/domain/tools.md"),
        Some("protocol/events.md") => include_str!("../../../docs/design/protocol/events.md"),
        Some("protocol/limits.md") => include_str!("../../../docs/design/protocol/limits.md"),
        Some("shell/commands.md") => include_str!("../../../docs/design/shell/commands.md"),
        _ => return Err(io::ErrorKind::NotFound.into()),
    };
    Ok(text.into())
}

fn read_pin(path: &Path) -> Result<PinnedConfiguration, Refusal> {
    if path != Path::new("agents/codex/standard.pin.toml") {
        return Err(Refusal::new(path, "document", io::Error::from(io::ErrorKind::NotFound).to_string()));
    }
    configuration(path, include_str!("../../agents/codex/standard.pin.toml"), |path| {
        assert_eq!(path, Path::new("agents/codex/overrides.toml"), "recorded configuration");
        Ok(include_bytes!("../../agents/codex/overrides.toml").to_vec())
    })
}

struct Fixtures {
    suite: Suite,
    tasks: Vec<CatalogueTask>,
    models: ModelTiers,
}

impl Fixtures {
    fn new() -> Self {
        let file = Path::new("tests/recorded/tasks/valid/task.toml");
        let task = parse_task(file, include_str!("../recorded/tasks/valid/task.toml")).expect("task document");
        let task = check_task_with(file, task, read_guard, |root| seed_digest_with(root, &recorded_seed(root)))
            .expect("recorded task");
        Self {
            suite: parse_document(Path::new("suite.toml"), SUITE).expect("recorded suite"),
            tasks: sample_catalogue(file.into(), task),
            models: parse_document(Path::new("agents/models.toml"), include_str!("../../agents/models.toml"))
                .expect("committed model tiers"),
        }
    }

    fn validate(&self, suite: &Suite) -> Result<Vec<String>, Refusal> {
        validate_suite_with(Path::new("suite.toml"), suite, &self.tasks, Path::new("agents"), &self.models, read_pin)
    }
}

fn a_task_retains_all_of_its_typed_parts() {
    let document = format!(
        r#"{DOCUMENT}
title = "Recorded probe"
guards = ["domain/tools.md, section 3"]
behaviours = ["read"]
effort = "low"
setup = [["cargo", "test", "--offline"]]
protected = ["Cargo.toml"]
repetitions = 3
waives = ["no-limit-failure"]
[[outcome]]
check = "file-content"
path = "answer.txt"
content = "ok\n"
[[outcome]]
check = "file-digest"
path = "README.md"
sha256 = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
[[outcome]]
check = "absent"
path = "error.txt"
[[outcome]]
check = "pattern"
path = "answer.txt"
pattern = "ok"
[[outcome]]
check = "protected"
path = "Cargo.toml"
[[grade]]
command = ["cargo", "test", "--offline"]
deadline_seconds = 30
max_output_bytes = 4096
[[variant]]
name = "short"
prompt = "Read one file and finish.\n"
[variant.budget]
turns = 2
[budget]
turns = 3
seconds = 10
spend = 0.02
[estimate]
seconds = 5
tokens = 1000
[smith]
"profile.concurrent_conversations" = 2
[environment]
BENCH_MARKER = "nonce"
[calibration]
binary = "digest"
binary_sha256 = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
date = "2026-10-14"
summary = "summaries/test.json"
attempts = 10
passes = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "10"]
deviations = []
control = "before"
failed_before = ["before/01"]
prompt_sha256 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
seed_sha256 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
checks_sha256 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
"#
    );
    let task = parse_task(Path::new("task.toml"), &document).expect("complete task parses");
    assert_eq!(task.kind, Kind::Probe);
    assert_eq!(task.outcome.len(), 5);
    assert_eq!(task.variant[0].budget.as_ref().expect("variant budget").turns, Some(2));
    assert_eq!(task.environment["BENCH_MARKER"], "nonce");
}

fn unknown_top_level_keys_name_the_document_and_key() {
    let error = parse_task(Path::new("sample/task.toml"), &format!("{DOCUMENT}\nmisspelled = true"))
        .expect_err("unknown key is refused");
    assert!(error.key.contains("misspelled"), "{error}");
    assert_eq!(error.file, Path::new("sample/task.toml"));
}

fn unknown_nested_keys_name_their_full_path() {
    for (suffix, path) in [
        ("[budget]\nturnz = 3", "budget.turnz"),
        ("[[variant]]\nname = 'short'\n[variant.budget]\nturnz = 3", "variant[0].budget.turnz"),
        ("[[outcome]]\ncheck = 'absent'\npath = 'x'\nextra = true", "outcome[0].extra"),
    ] {
        let error = parse_task(Path::new("task.toml"), &format!("{DOCUMENT}\n{suffix}"))
            .expect_err("unknown nested key is refused");
        assert_eq!(error.key, path, "{error}");
    }
}

fn kinds_and_checks_are_closed() {
    for document in [
        DOCUMENT.replace("kind = \"probe\"", "kind = \"chat\""),
        format!("{DOCUMENT}\n[[outcome]]\ncheck = 'unknown'"),
        format!("{DOCUMENT}\n[[event]]\ncheck = 'unknown'"),
    ] {
        assert!(parse_task(Path::new("task.toml"), &document).is_err());
    }
}

fn seed_digest_matches_the_committed_known_bytes() {
    let directory = Path::new("tests/recorded/tasks/valid/seed");
    assert_eq!(
        seed_digest_with(directory, &recorded_seed(directory)).expect("seed hashes"),
        "452c822eb37147a50a5ddf1271245fad2f5884ded6513b8f813e81629627fb85"
    );
}

fn a_calibration_requires_ten_distinct_passes() {
    let file = Path::new("tests/recorded/tasks/bad-calibration/task.toml");
    let task = parse_task(file, include_str!("../recorded/tasks/bad-calibration/task.toml")).expect("task document");
    let error = check_task_with(file, task, read_guard, |root| {
        let tree = SeedTree::new(
            root,
            &[Item::file(b"README.md", include_bytes!("../recorded/tasks/bad-calibration/seed/README.md"))],
        );
        seed_digest_with(root, &tree)
    })
    .expect_err("nine passes do not calibrate");
    assert_eq!(error.key, "calibration");
}

fn a_seed_digest_freezes_names_content_and_executable_permissions() {
    let root = Path::new("seed");
    let observed =
        |name, bytes, mode| seed_digest_with(root, &SeedTree::new(root, &[Item::file(name, bytes).mode(mode)]));
    let baseline = observed(b"a.txt", b"one", 0o600).expect("initial digest");
    assert_ne!(observed(b"a.txt", b"two", 0o600).expect("changed digest"), baseline);
    assert_eq!(observed(b"a.txt", b"one", 0o600).expect("restored digest"), baseline);
    assert_ne!(observed(b"a.txt", b"one", 0o700).expect("executable digest"), baseline);
    assert_ne!(observed(b"b.txt", b"one", 0o600).expect("renamed digest"), baseline);
}

fn a_seed_symlink_cannot_reach_hidden_inputs() {
    let root = Path::new("seed");
    let tree = SeedTree::new(root, &[Item::link(b"hidden", b"../grader")]);
    let error = seed_digest_with(root, &tree).expect_err("seed must not follow symlinks");
    assert!(error.reason.contains("symlinks"), "{error}");
}

fn a_when_asked_task_runs_only_when_a_run_includes_it(fixtures: &Fixtures) {
    let mut suite = fixtures.suite.clone();
    suite.when_asked.push("probes/recorded/short".into());
    let selected = fixtures.validate(&suite).expect("asked task resolves");
    assert_eq!(selected, ["probes/recorded"]);
    assert_eq!(smith_bench::include(Path::new("suite.toml"), &suite, &selected, &[]).expect("default"), selected);
    assert_eq!(
        smith_bench::include(Path::new("suite.toml"), &suite, &selected, &suite.when_asked).expect("included task"),
        ["probes/recorded", "probes/recorded/short"]
    );
    suite.tasks = suite.when_asked.clone();
    assert!(fixtures.validate(&suite).is_err());
}

fn an_include_outside_when_asked_is_refused(fixtures: &Fixtures) {
    let suite = fixtures.suite.clone();
    let error = smith_bench::include(Path::new("suite.toml"), &suite, &suite.tasks, &["probes/recorded/short".into()])
        .expect_err("only asked tasks can be included");
    assert_eq!(error.key, "include");
}

fn a_when_asked_name_must_name_an_existing_task(fixtures: &Fixtures) {
    for name in ["probes/missing", "probes/recorded/missing"] {
        let mut suite = fixtures.suite.clone();
        suite.when_asked.push(name.into());
        let error = fixtures.validate(&suite).expect_err("asked names resolve before a run");
        assert_eq!(error.key, "when_asked");
    }
}

fn a_suite_resolves_task_variants_selectors_and_configuration_pins(fixtures: &Fixtures) {
    let mut suite = fixtures.suite.clone();
    suite.tasks.push("probes/recorded/short".into());
    suite.select.push(smith_bench::Selection { kind: Kind::Probe, behaviours: vec!["read".into()] });
    let selected = fixtures.validate(&suite).expect("suite references resolve");
    assert_eq!(selected, ["probes/recorded", "probes/recorded/short"]);
}

fn a_suite_refuses_unknown_tasks_agents_tiers_and_empty_selectors(fixtures: &Fixtures) {
    let mut suite = fixtures.suite.clone();
    suite.tasks[0] = "probes/recorded/unknown".into();
    assert!(fixtures.validate(&suite).expect_err("unknown task").reason.contains("unknown task"));
    suite = fixtures.suite.clone();
    suite.agents[0].config = "missing".into();
    assert!(fixtures.validate(&suite).is_err());
    suite = fixtures.suite.clone();
    suite.tier = "unregistered".into();
    assert!(fixtures.validate(&suite).is_err());
    suite = fixtures.suite.clone();
    suite.select.push(smith_bench::Selection { kind: Kind::Probe, behaviours: vec!["absent".into()] });
    assert!(fixtures.validate(&suite).expect_err("unmatched selector").reason.contains("no existing task"));
}

fn suite_keys_and_arms_are_strict() {
    let document = SUITE;
    let error = parse_document::<smith_bench::Suite>(Path::new("suite.toml"), &format!("{document}\nextra = true"))
        .expect_err("unknown nested agent key");
    assert_eq!(error.key, "agents[0].extra");
    let suite = parse_document::<smith_bench::Suite>(Path::new("suite.toml"), &format!("{document}\n[[arm]]\nsource='binary'\nname='before'\ncommit='30259d8'\n[[arm]]\nsource='override'\nname='control'\n[arm.smith]\naffinity=false\n[[arm]]\nsource='agent'\nname='codex'\nagent='codex'\nprovider='codex'\nconfig='standard'\n")).expect("three arm kinds");
    assert_eq!(suite.arms.len(), 3);
    assert!(
        parse_document::<smith_bench::Suite>(
            Path::new("suite.toml"),
            &format!("{document}\n[[arm]]\nsource='unknown'\nname='new'\n")
        )
        .is_err()
    );
}

fn the_small_tier_resolves_and_the_working_tier_never_guesses(fixtures: &Fixtures) {
    let file = Path::new("agents/models.toml");
    let models = &fixtures.models;
    let codex = models.lookup(file, "small", smith_bench::Provider::Codex).expect("codex small tier");
    assert_eq!(codex.model.as_deref(), Some("gpt-6-luna"));
    assert_eq!(codex.effort.as_deref(), Some("low"));
    let anthropic = models.lookup(file, "small", smith_bench::Provider::Anthropic).expect("anthropic small tier");
    assert_eq!(anthropic.model.as_deref(), Some("claude-haiku-5-5"));
    let mut unresolved = models.clone();
    unresolved
        .0
        .get_mut("working")
        .expect("working tier")
        .insert(smith_bench::Provider::Codex, smith_bench::ModelChoice { model: None, effort: None });
    assert!(
        unresolved
            .lookup(file, "working", smith_bench::Provider::Codex)
            .expect_err("unresolved choice refuses")
            .reason
            .contains("await")
    );
}

fn guards_are_cheapest_first_and_report_budget_and_unknown_costs(fixtures: &Fixtures) {
    let mut tasks = fixtures.tasks.clone();
    let mut fast = tasks[0].clone();
    fast.name = "probes/fast".into();
    fast.task.estimate = Some(smith_bench::Estimate { seconds: 2, tokens: 20 });
    let mut expensive = fast.clone();
    expensive.name = "probes/expensive".into();
    expensive.task.estimate = Some(smith_bench::Estimate { seconds: 20, tokens: 100 });
    let mut overflow = fast.clone();
    overflow.name = "probes/overflow".into();
    overflow.task.estimate = Some(smith_bench::Estimate { seconds: u64::MAX, tokens: 1 });
    tasks.extend([expensive, fast, overflow]);
    let mut suite = fixtures.suite.clone();
    suite.max_wall_seconds = 10;
    let chosen = smith_bench::choose_guards(&suite, &tasks, &["benchmarks/README.md, section 5.2".into()]);
    assert_eq!(chosen.selected, ["probes/fast", "probes/fast/short"]);
    assert_eq!((chosen.estimated_seconds, chosen.estimated_tokens), (8, 80));
    assert!(chosen.omitted.contains(&("probes/recorded".into(), "cost unavailable".into())));
    assert!(chosen.omitted.contains(&("probes/overflow".into(), "cost overflow".into())));
    assert!(chosen.omitted.contains(&("probes/expensive".into(), "suite wall or token budget".into())));
}

#[test]
fn task_documents_and_frozen_seed_contracts() {
    let result = smith_bench::parse_result(
        Path::new("tests/recorded/results/attempt.json"),
        include_str!("../recorded/results/attempt.json"),
    )
    .expect("fixture");
    probe_cases(result);
    a_task_retains_all_of_its_typed_parts();
    unknown_top_level_keys_name_the_document_and_key();
    unknown_nested_keys_name_their_full_path();
    kinds_and_checks_are_closed();
    seed_digest_matches_the_committed_known_bytes();
    a_calibration_requires_ten_distinct_passes();
    a_seed_digest_freezes_names_content_and_executable_permissions();
    a_seed_symlink_cannot_reach_hidden_inputs();
    let fixtures = Fixtures::new();
    a_when_asked_task_runs_only_when_a_run_includes_it(&fixtures);
    an_include_outside_when_asked_is_refused(&fixtures);
    a_when_asked_name_must_name_an_existing_task(&fixtures);
    a_suite_resolves_task_variants_selectors_and_configuration_pins(&fixtures);
    a_suite_refuses_unknown_tasks_agents_tiers_and_empty_selectors(&fixtures);
    suite_keys_and_arms_are_strict();
    the_small_tier_resolves_and_the_working_tier_never_guesses(&fixtures);
    guards_are_cheapest_first_and_report_budget_and_unknown_costs(&fixtures);
}
