//! An external ctl directory signs in as a service (client/ctl/README.md, 7).
mod machine;
use machine::Machine;

#[test]
fn an_owner_makes_a_service_and_a_second_ctl_directory_acts_as_it() {
    let mut world = Machine::new();
    world.call("owner", &[b"sign-in", b"link"]);
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
fn an_outside_party_files_hands_on_and_asks_the_owners_through_ctl() {
    let mut world = Machine::findings();
    world.call("owner", &[b"sign-in", b"link"]);
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
