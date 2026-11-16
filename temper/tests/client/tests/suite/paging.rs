//! Paging belongs to the tools' root against the engine root (client/ctl/testing.md, 2).
use jig_client_kit::harness::{ClientId, World};
use temper_client_tools::jig::Printed;
use temper_client_world::{app::Temper, fixtures, front::Joint};

fn call(world: &mut World<Temper, Joint>, agent: ClientId, words: &[&[u8]]) -> Printed {
    let number = fixtures::give(world, agent, words);
    let printed = fixtures::answered(world, agent, number);
    assert_eq!(printed.status, 0, "{printed:?}");
    assert!(printed.stderr.is_empty(), "{printed:?}");
    printed
}

fn read_on(world: &mut World<Temper, Joint>, agent: ClientId, next: &str, json: bool) -> Printed {
    assert!(world.root(agent).agent().calls() < 20, "bounded continuation calls");
    let mut words: Vec<_> = next.split_ascii_whitespace().map(str::as_bytes).collect();
    if json {
        words.push(b"--json");
    }
    call(world, agent, &words)
}

#[test]
fn small_template_pages_keep_every_activity_and_live_delegate() {
    let mut world = fixtures::paging::world(fixtures::SEED);
    let agent = fixtures::agent(&mut world);
    let task = call(&mut world, agent, &[b"task", b"show", b"3", b"--json"]);
    let task: serde_json::Value = serde_json::from_slice(&task.stdout).expect("task JSON");
    let head = task["activity"].as_u64().expect("task activity");
    assert!(head >= 3);
    let bound = usize::try_from(fixtures::paging::ROWS).expect("page bound");
    let mut activity = call(&mut world, agent, &[b"task", b"activity", b"3", b"--json"]);
    let mut pages = 0;
    let mut positions = Vec::new();
    loop {
        let page: serde_json::Value = serde_json::from_slice(&activity.stdout).expect("activity JSON");
        let rows = page["rows"].as_array().expect("activity rows");
        assert!(!rows.is_empty() && rows.len() <= bound, "template row bound: {page}");
        pages += 1;
        for row in rows {
            let position = row["position"].as_u64().expect("activity position");
            if let Some(previous) = positions.last() {
                assert_eq!(position + 1, *previous, "no missing activity");
            } else {
                assert_eq!(position, head, "newest activity first");
            }
            positions.push(position);
        }
        if let Some(next) = page["read_on"].as_str() {
            activity = read_on(&mut world, agent, next, true);
        } else {
            break;
        }
    }
    assert!(pages >= 2);
    assert!(positions.len() >= 3);
    assert_eq!(positions.last(), Some(&1));

    let start = world.now();
    let printed = call(&mut world, agent, &[b"task", b"show", b"1", b"--tree"]);
    assert!(world.now() < start.saturating_add(skein_lib::Duration::from_secs(1)), "tree answers within a second");
    let text = std::str::from_utf8(&printed.stdout).expect("tree UTF-8");
    let tree = text.split_once("tree:\n").expect("tree").1.split_once("digest:").expect("digest").0;
    assert_eq!(tree.lines().filter(|line| line.contains(" · ")).count(), 3);
    assert!(tree.starts_with("1 · "));
    assert!(tree.contains("cut: 1 more not shown (1 Waiting)\nread on with: task list --project 1 --parent 1\n"));
    let next = tree.lines().find_map(|line| line.strip_prefix("read on with: ")).expect("tree continuation");
    let mut listed = read_on(&mut world, agent, next, false);
    let mut delegates = Vec::new();
    loop {
        let text = std::str::from_utf8(&listed.stdout).expect("delegates UTF-8");
        let tasks: Vec<u64> = text
            .lines()
            .filter_map(|line| line.split_once(" · ").and_then(|(number, _)| number.parse().ok()))
            .collect();
        assert!(!tasks.is_empty() && tasks.len() <= bound, "template task page bound: {text}");
        delegates.extend(tasks);
        if let Some(next) = text.lines().find_map(|line| line.strip_prefix("read on with: ")) {
            listed = read_on(&mut world, agent, next, false);
        } else {
            break;
        }
    }
    assert_eq!(delegates, [4, 5, 6]);
    call(&mut world, agent, &[b"task", b"stop", b"1", b"--key", b"paging-stop"]);
}
