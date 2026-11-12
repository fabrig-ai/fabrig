use super::{checks::*, *};
use std::{fs, path::Path};

#[test]
#[expect(clippy::disallowed_macros, reason = "the step test locates the pages through CARGO_MANIFEST_DIR")]
fn the_table_is_the_directory() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/help");
    let mut names = Vec::new();
    for (directory, reference) in [(directory.clone(), false), (directory.join("reference"), true)] {
        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(error) if reference && error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => panic!("{}: {error}", directory.display()),
        };
        for entry in entries {
            let path = entry.expect("page file").path();
            if path.extension().is_some_and(|ext| ext == "md") {
                let name = path.file_stem().expect("page stem").to_str().expect("ASCII name");
                names.push(String::from(name));
                let entry = find(name.as_bytes()).expect("file in table");
                let text = fs::read_to_string(&path).expect("page bytes");
                assert_eq!(entry.text, text, "{name}: file bytes");
                let kind = if reference {
                    "reference"
                } else {
                    text.lines().nth(2).expect("kind line").strip_prefix("Kind: ").expect("kind prefix")
                };
                assert_eq!(entry.kind.word(), kind, "{name}: file kind");
            }
        }
    }
    assert!(PAGES.len() <= MAX_PAGES, "table: MAX_PAGES");
    let mut previous = None;
    for entry in &PAGES {
        let kind = KINDS.iter().position(|kind| *kind == entry.kind).expect("page kind");
        if entry.kind == Kind::Reference && entry.name.starts_with("ctl-") {
            // temper checks these against GROUPS, immediately after commands.
            assert_eq!(previous.map(|(kind, _)| kind), Some(kind), "{}: reference section", entry.name);
            continue;
        }
        let key = (kind, entry.name);
        if let Some(previous) = previous {
            assert!(previous < key, "{}: kind and name order", entry.name);
        }
        previous = Some(key);
    }
    assert_eq!(PAGES[0].name, "index");
    assert_eq!(PAGES.iter().filter(|page| page.kind == Kind::Index).count(), 1);
    let mut table_names = PAGES.iter().map(|entry| String::from(entry.name)).collect::<Vec<_>>();
    names.sort();
    table_names.sort();
    assert_eq!(names, table_names, "table names equal the directory");
}

#[test]
fn each_page_keeps_its_head_template_links_and_voice() {
    for entry in &PAGES {
        for check in [head, template, links, placeholders, voice] {
            check(entry).unwrap_or_else(|why| panic!("{why}"));
        }
    }
}

#[test]
fn tester_recipe_enables_the_shell_but_keeps_the_coding_charters_tools() {
    let text = find(b"tester-that-cannot-land").expect("tester recipe").text;
    let command = text
        .lines()
        .find(|line| line.starts_with("temper ctl DIR settings show charter 2 --json"))
        .expect("copy stock coding charter 2");
    assert!(command.contains(".shell = true"), "the stock shell is disabled; the tester must run tests");
    assert!(!command.contains(".tools ="), "keep the coding charter's tool-family bits");
    assert!(command.contains(".instructions ="), "give the tester its own instructions");
    assert!(command.contains("Report each test, its path, its content and its result. Fix nothing."));
}

#[test]
fn chat_help_shows_the_goal_questions_answer_form() {
    let text = find(b"chat").expect("chat page").text;
    assert!(text.contains("`G asks (question Q; answer with message target=G, form=answer, question=Q): ...`"));
}

#[test]
fn task_help_explains_refused_invalid_results() {
    let found = search(&PAGES, b"invalid results", 20).expect("search words");
    assert!(found.hits.as_slice().iter().any(|hit| hit.page == "task"), "invalid results must find task help");
    let text = find(b"task").expect("task page").text.split_whitespace().collect::<Vec<_>>().join(" ");
    for instruction in [
        "invalid results",
        "assignment was refused before any model turn",
        "serve's stderr",
        "`DATA/box/logs/serve.log`",
        "precise refusal reason",
    ] {
        assert!(text.contains(instruction), "task help must explain: {instruction}");
    }
}

#[test]
fn task_help_says_what_taken_by_and_waiting_on_print() {
    let text = find(b"task").expect("task page").text.split_whitespace().collect::<Vec<_>>().join(" ");
    for instruction in [
        "**A person task** prints `taken by:` naming its holder, or `none`.",
        "Once an answer has ended the task, `answered by:` takes its place.",
        "A closing person task with no holder prints neither line.",
        "For a held task that awaits nothing else, `waiting on:` prints `held`.",
        "When no account serves its next run, `next run:` stands alone above the lines that say what to do, and `waiting on:` prints `an account`.",
        "`task amend`'s line is its call alone; `--json` carries the bounds it is offered within.",
    ] {
        assert!(text.contains(instruction), "task help must explain: {instruction}");
    }
}

#[test]
fn configuration_help_gives_the_merge_commands_contract() {
    let page = find(b"configuration").expect("configuration page").text;
    let text = page.split_whitespace().collect::<Vec<_>>().join(" ");
    let (members, contract) = text.split_once("## The merge command").expect("merge command section");
    let row = members.split_once("- `merge_job`:").expect("merge_job row").1;
    let row = row.split(" - `").next().expect("merge_job line");
    assert!(row.contains("see The merge command, below."), "merge_job must point to its contract");
    for instruction in [
        "`TEMPER_JOB`",
        "`request.json`",
        "`result.json`",
        "`git add`",
        "**Exit 0:**",
        "**Exit 3:**",
        "**Any other exit,**",
    ] {
        assert!(contract.contains(instruction), "merge command contract must explain: {instruction}");
    }
}

#[test]
fn change_help_points_resolving_to_the_merge_command() {
    let text = find(b"change").expect("change page").text.split_whitespace().collect::<Vec<_>>().join(" ");
    let resolving = text.split_once("- `resolving`:").expect("resolving step").1;
    let resolving = resolving.split_once("- `repairing`:").expect("repairing step").0;
    assert!(resolving.contains("temper help configuration"), "resolving must point to the merge command");
}

#[test]
fn change_help_says_what_a_resumed_producers_checkout_holds() {
    let text = find(b"change").expect("change page").text.split_whitespace().collect::<Vec<_>>().join(" ");
    for instruction in
        ["`Merge SHA`", "`MERGE_HEAD`", "`git status`", "Never answer it to remove or restore those files"]
    {
        assert!(text.contains(instruction), "change help must explain: {instruction}");
    }
}

#[test]
fn change_and_task_help_say_what_a_cancelled_early_start_means() {
    for name in [b"change".as_slice(), b"task"] {
        let page = find(name).expect("change or task page");
        let text = page.text.split_whitespace().collect::<Vec<_>>().join(" ");
        for instruction in ["`Cancelled · its early start conflicts with SHA`", "goes on with a new producer"] {
            assert!(text.contains(instruction), "{} help must explain: {instruction}", page.name);
        }
    }
}

#[test]
fn tier_help_explains_that_clearance_waits_through_verifying_and_red() {
    let text = find(b"tier").expect("tier page").text;
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    for instruction in [
        "`change show TASK --until cleared` waits for `cleared` or ended landed and uncleared",
        "`verifying` and `red` remain pending",
        "Inspect the verdict with `change show TASK`",
        "use `--until changed` for panel updates",
    ] {
        assert!(text.contains(instruction), "tier help must explain: {instruction}\n{text}");
    }
    assert!(!text.contains("waits for a landed verdict: `verifying`, `red` or `cleared`"));
}

#[test]
fn budget_help_sizes_a_hundred_change_goal() {
    let text = find(b"budget").expect("budget page").text;
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    for instruction in [
        "For a goal of up to a hundred changes",
        "100 change procedures, 100 producers, 200 reviews and 100 repairs",
        "12 coordinator tasks including the goal",
        "512 tasks in the tree, 511 delegated tasks",
        "412K units",
        "five delegation levels below the goal",
        "executor kinds",
        "deployment's rules, the project's policy ceiling and your role authority",
        "policy 1 changes affect projects created afterward",
        "Before forge setup",
        "Raising policy authority alone does not raise deployment limits",
        "min(role tasks, deployment tree tasks) - 1",
        "role depth - 1",
        "1024/6 and a tree limit of 32 yield 31/5",
        "estimate, not guaranteed capacity",
        "hundred-change profile remains a documented gap (N21)",
    ] {
        assert!(text.contains(instruction), "budget help must explain: {instruction}\n{text}");
    }
}

#[test]
fn capacity_help_names_members_and_dependencies() {
    for name in [b"budget".as_slice(), b"configure"] {
        let text = find(name).expect("capacity help page").text;
        let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
        for words in [
            "engine.limits.overrides",
            "tasks.tasks",
            "tasks.project_tasks",
            "tasks.tree_tasks",
            "tasks.depth",
            "tasks.delegates",
            "tasks.batch",
            "tasks.batch_bytes",
            "tasks.project_goals",
            "documents.lines",
            "people.inbox_tasks",
            "people.operations",
            "128",
            "lifetime tree capacity",
            "profile store ceiling",
            "reduce",
            "spec, message, result and authority sizes",
        ] {
            assert!(text.contains(words), "capacity help must explain {words}: {text}");
        }
    }
    let configure = find(b"configure").expect("configure help page").text;
    assert!(configure.contains("schema ceiling"), "configure help must explain the schema ceiling");
    let budget = find(b"budget").expect("budget help page").text;
    assert!(budget.contains("tasks.inbox_messages"), "budget help must name the inbox capacity");
}

#[test]
fn configure_help_says_how_authority_widens_and_holds_no_recipe_of_the_reference() {
    let text = find(b"configure").expect("configure page").text;
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    for instruction in [
        "Installing charters alone grants no task authority",
        "rules ceiling, policy 1's ceiling and the creator-role authority",
        "Policy 1 affects future projects. Existing projects need project-policy",
        "saved whole current `project-policy` settings item",
        "Existing tasks do not gain widened authority automatically",
        "amend or allot their authority explicitly",
        "`temper/docs/examples/reference/README.md`",
    ] {
        assert!(text.contains(instruction), "configure help must explain: {instruction}");
    }
    for recipe in ["reference charters", "tools 4031", "coordinator charter 5", "reference-authority"] {
        assert!(!text.contains(recipe), "the binary's help must hold no recipe of the reference: {recipe}");
    }
}

#[test]
fn the_reference_readme_grants_its_tools_and_note_scopes() {
    let text = include_str!("../../../../docs/examples/reference/README.md");
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    for instruction in [
        "Installing charters alone grants no task authority",
        "decide_escalation",
        "propose 16",
        "subscribe_forge",
        "note 128",
        "recall 256",
        "tools 4031",
        "goal and project note scopes",
        "notes 3",
        "rules ceiling, policy 1's ceiling and the creator-role authority",
        "coordinator charter 5",
        "fifteen executor places",
        "merge job 3",
        "Policy 1 affects future projects. Existing projects need project-policy",
        "saved whole current `project-policy` settings item",
        "Existing tasks do not gain widened authority automatically",
        "amend or allot their authority explicitly",
    ] {
        assert!(text.contains(instruction), "the reference's README must explain: {instruction}");
    }
    for retired in ["charter 7", "charter 8", "\"charter\": 7", "\"charter\": 8"] {
        assert!(!text.contains(retired), "reference recipe names retired {retired}");
    }
    for command in [
        "settings show rules --json",
        "settings show policy 1 --json",
        "--slurpfile required reference-authority.json",
        "select(.number == $role)",
        ".set.ceiling |= widen",
        ".notes |= grant($required[0].notes)",
        "map(select(. != {\"charter\": 12} and . != {\"charter\": 13} and . != {\"charter\": 6}))",
    ] {
        assert!(text.contains(command), "the reference's recipe must use: {command}");
    }
}

#[test]
fn the_index_names_every_concept_walk_and_recipe_once() {
    let text = find(b"index").expect("index").text;
    for entry in &PAGES {
        if written(entry) && entry.kind != Kind::Index {
            let count = text
                .split("temper help ")
                .skip(1)
                .filter(|after| {
                    after.split(|c: char| !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')).next()
                        == Some(entry.name)
                })
                .count();
            assert_eq!(count, 1, "{}: index names page exactly once", entry.name);
        }
    }
}

const CONCEPT: &str = "## What it is\n## How it changes\n## What temper guarantees\n## Acting on it\n## Limits\n";
const WALK: &str = "## Before you start\n## 1. Read\nExpect: read\n## 2. Act\nExpect: done\n## Clean up\n## Next\n";
const RECIPE: &str = "## Goal\n## Settings\n## Enforced and followed\n## Checking it took\n";

fn leaked(text: String) -> &'static str {
    Box::leak(text.into_boxed_str())
}

fn fails(check: fn(&Page) -> Result<(), String>, entry: &Page, rule: &str) {
    let error = check(entry).expect_err("fixture refused");
    assert!(error.contains(entry.name) && error.contains(rule) && error.contains("line "), "{error}");
}

#[test]
fn a_page_at_its_kinds_bound_passes_and_one_byte_more_fails() {
    for kind in KINDS {
        let text = match kind {
            Kind::Concept => CONCEPT,
            Kind::Walk => WALK,
            Kind::Recipe => RECIPE,
            Kind::Way => "## 1. First\n",
            Kind::Index | Kind::Reference => "",
        };
        let mut padded = String::from(text);
        padded.push_str(&"x".repeat(kind.bound() - text.len()));
        template(&fixture(kind, leaked(padded.clone()))).expect("exact bound passes");
        padded.push('x');
        fails(template, &fixture(kind, leaked(padded)), &format!("bound {}", kind.bound()));
    }
}

#[test]
fn a_concept_keeps_its_headings_in_order() {
    for text in [CONCEPT.into(), CONCEPT.replace("## How it changes\n", "")] {
        template(&fixture(Kind::Concept, leaked(text))).expect("concept headings");
    }
    for text in [
        CONCEPT.replace("## Limits\n", ""),
        CONCEPT.replace("What it is", "Limits").replace("## Limits\n", "## What it is\n"),
        format!("{CONCEPT}## Another\n"),
    ] {
        fails(template, &fixture(Kind::Concept, leaked(text)), "headings");
    }
}

#[test]
fn a_walks_steps_number_from_one_and_say_what_to_expect() {
    template(&fixture(Kind::Walk, WALK)).expect("steps 1 and 2");
    for text in [
        WALK.replace("2. Act", "3. Act"),
        WALK.replace("## 1. Read\nExpect: read\n", ""),
        WALK.replace("## Clean up\n", ""),
    ] {
        fails(template, &fixture(Kind::Walk, leaked(text)), "headings");
    }
    fails(template, &fixture(Kind::Walk, leaked(WALK.replace("Expect: done\n", ""))), "Expect:");
    fails(
        template,
        &fixture(Kind::Walk, leaked(WALK.replace("Expect: done\n", "```\nExpect: done\n```\n"))),
        "Expect:",
    );
}

#[test]
fn a_recipe_keeps_its_four_headings() {
    template(&fixture(Kind::Recipe, RECIPE)).expect("recipe headings");
    fails(template, &fixture(Kind::Recipe, leaked(RECIPE.replace("## Settings\n", ""))), "headings");
}

#[test]
fn a_capital_word_after_temper_help_is_no_link() {
    for binary in ["temper", "temper-box"] {
        links(&fixture(
            Kind::Index,
            leaked(format!("{binary} help NAME\n{binary} help --search\n{binary} help pages\n{binary} help\n")),
        ))
        .expect("no page names");
        fails(links, &fixture(Kind::Index, leaked(format!("{binary} help nope"))), "link nope");
    }
}

#[test]
fn placeholders_are_the_set_or_declared_by_the_walk() {
    placeholders(&fixture(Kind::Concept, "```\ntemper ctl DIR task show TASK\n```\n")).expect("set");
    fails(placeholders, &fixture(Kind::Concept, "```\ntemper ctl DIR task show N\n```\n"), "placeholder N");
    let text = "## Before you start\n- `OWNER` is the owner.\n## 1. Act\n```\ntemper ctl DIR repository adopt --owner OWNER\n```\n";
    placeholders(&fixture(Kind::Walk, text)).expect("declared");
    placeholders(&fixture(Kind::Walk, leaked(text.replace("` is", "`:")))).expect("colon declaration");
    placeholders(&fixture(Kind::Walk, leaked(text.replace("OWNER", "N")))).expect("one-byte declaration");
    fails(
        placeholders,
        &fixture(Kind::Walk, leaked(text.replace("- `OWNER` is the owner.\n", ""))),
        "placeholder OWNER",
    );
    placeholders(&fixture(Kind::Concept, "OWNER in prose\n```\ntemper ctl DIR task show taskTASK\n```\n"))
        .expect("prose or mixed case");
    placeholders(&fixture(Kind::Reference, "```\ntemper ctl DIRECTORY task show N\n```\n")).expect("references exempt");
}

#[test]
fn the_designs_words_stay_in_code() {
    voice(&fixture(Kind::Concept, "`jig`\n```\njig\n```\njigsaw\n")).expect("code and larger words");
    for text in ["jig in prose", "smith in prose", "skein in prose", "see box.md", "temper-docs"] {
        fails(voice, &fixture(Kind::Concept, text), "voice");
    }
    voice(&fixture(Kind::Reference, "jig in prose")).expect("references exempt");
}

#[test]
fn a_head_refuses_wrong_fields_and_tests_each_byte_bound_on_both_sides() {
    let text = "# Fixture\n\nKind: index\n\nFixture.\n";
    head(&fixture(Kind::Index, text)).expect("valid head");
    head(&fixture(Kind::Index, leaked(format!("{text}\n·")))).expect("UTF-8 accepted");
    for (text, rule) in [
        (text.replace("# Fixture", "Fixture"), "title"),
        (text.replace("Kind: index", "Kind: walk"), "kind"),
        (text.replace("Fixture.\n", "Other.\n"), "summary"),
        (format!("{text}\t"), "control character"),
        (format!("{text}\r"), "control character"),
    ] {
        fails(head, &fixture(Kind::Index, leaked(text)), rule);
    }
    for (bound, field) in
        [(NAME_BYTES, "name"), (TITLE_BYTES, "title"), (SUMMARY_BYTES, "summary"), (HEADING_BYTES, "heading")]
    {
        for extra in [0, 1] {
            let value = leaked("x".repeat(bound + extra));
            let mut entry = fixture(Kind::Index, text);
            match field {
                "name" => entry.name = value,
                "title" => {
                    entry.title = value;
                    entry.text = leaked(format!("# {value}\n\nKind: index\n\nFixture.\n"));
                }
                "summary" => {
                    let summary = leaked(format!("{}.", &value[1..]));
                    entry.summary = summary;
                    entry.text = leaked(format!("# Fixture\n\nKind: index\n\n{summary}\n"));
                }
                _ => entry.text = leaked(format!("{text}\n## {value}\n")),
            }
            if extra == 0 {
                head(&entry).expect("bound passes");
            } else {
                fails(head, &entry, field);
            }
        }
    }
    for name in ["", "Upper", "under_score"] {
        fails(head, &Page { name, ..fixture(Kind::Index, text) }, "name");
    }
    head(&fixture(Kind::Index, "# Fixture\n\nKind: index\n\nFix\nture.\n")).expect_err("joined summary differs");
    head(&Page { summary: "Fix ture.", ..fixture(Kind::Index, "# Fixture\n\nKind: index\n\nFix\nture.\n") })
        .expect("joined summary");
}

#[test]
fn deployment_explains_tool_payload_and_its_dependencies() {
    let text = find(b"deployment").expect("deployment help").text;
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    for requirement in [
        "profile.declared.tool_payload",
        "per piece",
        "4,487",
        "10,662",
        "input-answer",
        "99876",
        "65536",
        "8194",
        "8192",
        "checkout.message_bytes",
        "hub.event_bytes",
        "hub.turn_bytes",
        "hub.turn_queue_bytes",
        "hub.transcript_bytes",
        "core.resume_bytes",
        "root.journal.transcript_bytes",
        "root.loads",
        "stop",
        "restart",
    ] {
        assert!(text.contains(requirement), "deployment help must name {requirement}");
    }
}

#[test]
fn public_reading_says_what_may_show_and_what_the_pages_hold() {
    let words = "Anything said in this project's chats may show on its public pages, in the titles of its goals and tasks. On a public repository, a change's pull request shows what its agent wrote too.";
    let walk = find(b"open-public-reading").expect("public reading walk").text;
    let walk = walk.split_whitespace().collect::<Vec<_>>().join(" ");
    let project = find(b"project").expect("project help").text;
    let project = project.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(project.contains(words), "project help must say what may show");
    let warning = walk.find(words).expect("walk must say what may show");
    let command = walk.find("--visibility public --key KEY").expect("make public command");
    assert!(warning < command, "say what may show before making the project public");
    for requirement in [
        "126 spent",
        "2 hours ago",
        "No landings to show yet.",
        "Work that ended before this project went public, or before temper last restarted, isn't shown.",
        "no budget",
        "first seven digits",
        "never links",
    ] {
        assert!(walk.contains(requirement), "public reading walk must name {requirement}");
    }
    assert!(!walk.contains("without exposing"));
}

#[test]
fn box_help_locates_forgejo_lifecycle_and_hands() {
    let text = find(b"box").expect("box help").text.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(text.contains("`DATA/box/logs/box.log`:"), "locate the lifecycle notices in the box's own log");
    assert!(text.contains("The forge's hands are in `DATA/box/logs/forgejo-FORGE.log`"), "locate the forge's hands");
}

#[test]
fn approval_help_names_the_forge_and_project_policy_route() {
    for (name, instructions) in [
        (
            b"landing-rule".as_slice(),
            [
                "An enforcing approval rule on a box needs nothing added for your approval:",
                "Name your role in the rule, owner (0) on a project you made.",
                "On this machine temper's agents and CI jobs run as you, so this stops their mistakes, not one trying to get past it.",
            ]
            .as_slice(),
        ),
        (b"party", ["on a box you need none of these steps:"].as_slice()),
        (b"configure", ["On a box you need none of these steps:"].as_slice()),
    ] {
        let text = find(name).expect("approval setup help").text;
        let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
        for words in [
            "collaborator",
            "project-policy",
            "repository adopt",
            "project show",
            "newest revision",
            "maintainer",
            "member",
            "observer",
            "temper knows your account on the box's own Forgejo, and on GitHub when temper made the App from your own account.",
            "With several members (`temper-box serve`), a managed Forgejo or a pasted App:",
        ]
        .into_iter()
        .chain(instructions.iter().copied())
        {
            assert!(text.contains(words), "{name:?} must explain {words:?}: {text}");
        }
        assert!(text.contains("writer") && text.contains("counts") || text.contains("writer is excluded"));
    }
}

#[test]
fn forgejo_help_says_what_a_hook_on_this_host_needs() {
    let text = find(b"forgejo").expect("Forgejo help").text.split_whitespace().collect::<Vec<_>>().join(" ");
    for words in ["ALLOWED_HOST_LIST", "application.webhooks.listener", "by polling"] {
        assert!(text.contains(words), "Forgejo help must explain {words}");
    }
}

#[test]
fn a_way_numbers_its_sections_from_one() {
    template(&fixture(Kind::Way, "## 1. First\n## 2. Next\n")).expect("numbered sections");
    for text in ["", "## 2. First\n", "## 1. First\n## 3. Next\n", "## First\n", "## 1. \n"] {
        fails(template, &fixture(Kind::Way, text), "headings");
    }
}

#[test]
fn the_way_names_no_address() {
    let text = HOW_TEMPER_BUILDS_TEMPER.text;
    assert!(!text.contains("://"));
}

#[test]
fn the_const_is_the_tables_row() {
    let row = find(b"how-temper-builds-temper").expect("way in table");
    let page = HOW_TEMPER_BUILDS_TEMPER;
    assert_eq!(
        (row.name, row.kind, row.title, row.summary, row.text),
        (page.name, page.kind, page.title, page.summary, page.text)
    );
}
