//! Service parties through ctl (temper/docs/design/client/ctl/testing.md, 2).
mod machine;
use machine::{Answer, Machine};
use skein_io::kernel::Exit;

#[test]
fn an_owner_makes_a_service_and_a_second_ctl_directory_acts_as_it() {
    let mut world = Machine::new();
    world.call("owner", &[b"sign-in", b"printed"]);
    let made = world.call(
        "owner",
        &[b"party", b"make", b"--project", b"1", b"--role", b"2", b"--out", b"service", b"scout", b"--key", b"scout"],
    );
    let party = made.trim().strip_prefix("made: party ").expect("made party").parse::<u64>().expect("party number");
    assert_ne!(party, 1, "the service is not the owner");
    world.private_file("service");
    let signed = world.call("agent", &[b"sign-in", b"--service", b"service"]);
    let me = world.call("agent", &[b"me"]);
    for text in [&signed, &me] {
        assert!(text.starts_with(&format!("party {party} · ")), "service party: {text}");
        assert!(text.lines().next().expect("party line").ends_with(" · scout"), "service name: {text}");
        assert!(
            text.lines().any(|line| line.starts_with("  1 · ") && line.contains(" · member")),
            "project role: {text}"
        );
    }
    world.private_file("agent/token");
    world.private_file("agent/address");
}

#[test]
fn an_owner_reissues_a_service_and_its_directory_signs_in_again() {
    let mut world = Machine::findings();
    let owner = world.run("owner", &[b"sign-in", b"printed"]);
    assert_party(&owner, "1");
    let party = make_service(&mut world, b"2", b"service", b"scout");
    let initial = world.run("agent", &[b"sign-in", b"--service", b"service"]);
    assert_party(&initial, &party);

    // A member can create a person task addressed to its own role.
    let title = b"Ready to ship?";
    let need = world.call(
        "agent",
        &[
            b"need",
            b"ask",
            b"--project",
            b"1",
            b"--role",
            b"2",
            b"--choice",
            b"yes",
            b"--choice",
            b"no",
            b"--why",
            b"the deploy waits",
            title,
            b"--key",
            b"ship",
        ],
    );
    let task: u64 = need
        .trim()
        .strip_prefix("done: task ")
        .and_then(|answer| answer.strip_suffix(", need"))
        .expect("need created")
        .parse()
        .expect("task number");
    let inbox_words: &[&[u8]] = &[b"inbox", b"--json"];
    let task_words: &[&[u8]] = &[b"task", b"list", b"--project", b"1", b"--json"];
    let inbox = world.call("agent", inbox_words);
    let tasks = world.call("agent", task_words);
    let inbox_json: serde_json::Value = serde_json::from_str(&inbox).expect("inbox JSON");
    let waiting = inbox_json["inbox"]["rows"].as_array().expect("inbox rows");
    assert!(
        waiting.iter().any(|row| row["object"] == "person_task" && row["task"]["live"]["task"].as_u64() == Some(task)),
        "the task waits in the service's inbox"
    );
    assert!(tasks.contains(std::str::from_utf8(title).expect("task title")), "the task is listed");

    world.redirect_file("link");
    let reissued = world.run("owner", &[b"party", b"reissue", party.as_bytes(), b"--out", b"link"]);
    assert_eq!(reissued.exit, Exit::Code(0), "reissue to a file");
    assert_output(&reissued.stdout, reissued_line(&party, "link").as_bytes(), "reissue answer on stdout");
    assert_output(&reissued.stderr, b"", "reissue to a file leaves stderr empty");
    world.private_file("link");
    let link = world.take_redirected();
    world.assert_service_link_private(&link, &[]);

    let signed_out = world.run("agent", &[b"me"]);
    assert_eq!(signed_out.exit, Exit::Code(1));
    assert_output(&signed_out.stdout, b"", "signed-out call leaves stdout empty");
    assert_output(&signed_out.stderr, b"temper ctl: signed out\n", "the next call is signed out");
    let signed = world.run_input("agent", &[b"sign-in", b"-"], &link);
    assert_party(&signed, &party);
    let me = world.run("agent", &[b"me"]);
    assert_party(&me, &party);
    let expiry = std::str::from_utf8(&me.stdout)
        .expect("me document")
        .lines()
        .nth(1)
        .and_then(|line| line.strip_prefix("sign-in expires: "))
        .expect("sign-in expiry below the party line");
    assert_eq!(expiry.len(), 20, "UTC timestamp");
    assert!(expiry.ends_with('Z'), "UTC timestamp");
    let inbox_after = world.run("agent", inbox_words);
    let tasks_after = world.run("agent", task_words);
    for (after, before, document) in [(&inbox_after, &inbox, "inbox"), (&tasks_after, &tasks, "tasks")] {
        assert_eq!(after.exit, Exit::Code(0), "read {document}");
        assert_output(&after.stderr, b"", "successful read leaves stderr empty");
        assert_output(&after.stdout, before.as_bytes(), &format!("reissue preserves the {document}"));
    }
    world.assert_service_link_private(
        &link,
        &[&owner.stdout, &initial.stdout, inbox.as_bytes(), tasks.as_bytes(), &signed.stdout, &me.stdout],
    );
}

#[test]
fn an_owner_pipes_a_reissued_service_link_to_sign_in() {
    let mut world = Machine::new();
    let owner = world.run("owner", &[b"sign-in", b"printed"]);
    assert_party(&owner, "1");
    let party = make_service(&mut world, b"2", b"service", b"scout");
    let initial = world.run("agent", &[b"sign-in", b"--service", b"service"]);
    assert_party(&initial, &party);

    let reissued = world.run("owner", &[b"party", b"reissue", party.as_bytes(), b"--out", b"-"]);
    assert_eq!(reissued.exit, Exit::Code(0), "reissue to a pipe");
    assert_output(&reissued.stderr, reissued_line(&party, "-").as_bytes(), "reissue answer on stderr");
    world.assert_service_link_private(&reissued.stdout, &[&owner.stdout, &initial.stdout]);
    let signed = world.run_input("agent", &[b"sign-in", b"-"], &reissued.stdout);
    assert_party(&signed, &party);
    world.assert_service_link_private(&reissued.stdout, &[&signed.stdout]);
}

#[test]
fn reissue_refuses_a_member_as_asker_and_a_person_as_target() {
    let mut world = Machine::findings();
    world.call("owner", &[b"sign-in", b"printed"]);
    let member = make_service(&mut world, b"2", b"service", b"scout");
    let steward = make_service(&mut world, b"0", b"steward", b"steward");
    assert_party(&world.run("agent", &[b"sign-in", b"--service", b"service"]), &member);
    assert_party(&world.run("docs", &[b"sign-in", b"--service", b"steward"]), &steward);

    let role = world.run("agent", &[b"party", b"reissue", steward.as_bytes(), b"--out", b"-"]);
    assert_eq!(role.exit, Exit::Code(2));
    assert_output(&role.stdout, b"", "role refusal emits no link");
    assert_output(
        &role.stderr,
        format!("temper ctl: refused: party {steward}'s credential is reissued by an owner of the deployment, or of every project it holds a role in\n").as_bytes(),
        "a member cannot reissue another service's credential",
    );

    let person = world.run("docs", &[b"party", b"reissue", b"1", b"--out", b"-"]);
    assert_eq!(person.exit, Exit::Code(2));
    assert_output(&person.stdout, b"", "not_offered refusal emits no link");
    assert_output(
        &person.stderr,
        b"temper ctl: refused: party 1 is not a service; only a service's credential is reissued\n",
        "a person target is not offered",
    );
}

fn make_service(world: &mut Machine, role: &[u8], file: &[u8], name: &[u8]) -> String {
    let made = world
        .call("owner", &[b"party", b"make", b"--project", b"1", b"--role", role, b"--out", file, name, b"--key", name]);
    let party = made.trim().strip_prefix("made: party ").expect("made party");
    assert!(party.parse::<u64>().is_ok_and(|party| party > 1), "new service party number");
    party.to_owned()
}

fn assert_party(answer: &Answer, party: &str) {
    assert_eq!(answer.exit, Exit::Code(0), "party {party} is signed in");
    assert_output(&answer.stderr, b"", "signed-in party leaves stderr empty");
    assert!(answer.stdout.starts_with(format!("party {party} · ").as_bytes()), "same service party {party}");
}

// Output may contain a credential when the assertion fails.
fn assert_output(actual: &[u8], expected: &[u8], context: &str) {
    assert!(actual == expected, "{context}");
}

fn reissued_line(party: &str, file: &str) -> String {
    format!(
        "reissued: party {party}; its sign-ins have ended; sign its directory in with temper ctl DIRECTORY sign-in {file}\n"
    )
}

#[test]
fn an_outside_party_files_hands_on_and_asks_the_owners_through_ctl() {
    let mut world = Machine::findings();
    world.call("owner", &[b"sign-in", b"printed"]);
    // Stock policy 1 admits person Create to one's own role only: observer cannot
    // create, member addresses role 2, maintainer role 1. Owners' role 0 is needed.
    world.call(
        "owner",
        &[b"party", b"make", b"--project", b"1", b"--role", b"0", b"--out", b"filer", b"scout", b"--key", b"scout"],
    );
    world.private_file("filer");
    world.call("agent", &[b"sign-in", b"--service", b"filer"]);
    let made = world.call(
        "owner",
        &[b"party", b"make", b"--project", b"1", b"--role", b"2", b"--out", b"writer", b"writer", b"--key", b"writer"],
    );
    let writer = made.trim().strip_prefix("made: party ").expect("party").parse::<u64>().expect("party number");
    world.call("docs", &[b"sign-in", b"--service", b"writer"]);
    let areas = format!(
        r#"{{"items":[{{"setting":{{"kind":"project-areas","project":1}},"set":{{"areas":[{{"name":"tests","words":"Tests and fixtures","owner":{{"party":1}}}},{{"name":"docs","words":"Written words","owner":{{"party":{writer}}}}}]}}}}]}}"#
    );
    world.call_input("owner", &[b"settings", b"apply", b"-", b"--against", b"1", b"--key", b"areas"], areas.as_bytes());
    let filed = world.call_input(
        "agent",
        &[
            b"finding",
            b"file",
            b"--project",
            b"1",
            b"--kind",
            b"red",
            b"--area",
            b"tests",
            b"Flaky boot",
            b"-",
            b"--key",
            b"boot",
        ],
        b"Boot twice\n",
    );
    assert_eq!(filed, "filed: 1/1, held by party 1\n");
    let listed = world.call("owner", &[b"finding", b"list", b"--project", b"1", b"--mine"]);
    assert!(listed.contains("1/1 · new · red · medium · tests · party 1 · Flaky boot"), "owner's findings: {listed}");
    let handed =
        world.call("owner", &[b"finding", b"hand", b"1/1", b"--area", b"docs", b"words, not code", b"--key", b"hand"]);
    assert_eq!(handed, format!("handed: 1/1, held by party {writer}\n"));
    let listed = world.call("docs", &[b"finding", b"list", b"--project", b"1", b"--mine"]);
    assert!(
        listed.contains(&format!("1/1 · new · red · medium · docs · party {writer} · Flaky boot")),
        "new owner's findings: {listed}"
    );
    let old = world.call("owner", &[b"finding", b"list", b"--project", b"1", b"--mine"]);
    assert!(!old.contains("Flaky boot"), "old owner no longer holds it: {old}");
    let need = world.call(
        "agent",
        &[
            b"need",
            b"ask",
            b"--project",
            b"1",
            b"--choice",
            b"yes",
            b"--choice",
            b"no",
            b"--why",
            b"the deploy waits",
            b"Ship it?",
            b"--key",
            b"ship",
        ],
    );
    let task = need.trim().strip_prefix("done: task ").expect("task").strip_suffix(", need").expect("need");
    let inbox = world.call("owner", &[b"inbox", b"--kind", b"person-tasks"]);
    assert!(inbox.contains(&format!("task show {task}")), "owners' inbox: {inbox}");
    world.call("owner", &[b"task", b"take", task.as_bytes()]);
    let shown = world.call("owner", &[b"task", b"show", task.as_bytes()]);
    assert!(shown.contains("Ship it?\n\nWhy: the deploy waits"), "question: {shown}");
    let marker = format!("task answer {task} --rev ");
    let revision = shown.split(&marker).nth(1).expect("answer guidance").split_whitespace().next().expect("revision");
    world.call(
        "owner",
        &[b"task", b"answer", task.as_bytes(), b"--rev", revision.as_bytes(), b"1", b"", b"--key", b"answer"],
    );
    let answer = world.call("agent", &[b"task", b"show", task.as_bytes()]);
    assert!(answer.contains("verdict 1"), "service reads the answer: {answer}");
}
