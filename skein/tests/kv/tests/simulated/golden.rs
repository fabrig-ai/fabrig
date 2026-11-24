use skein_fake_machine::Item;
use skein_kv::{Event, Range, Request};
use skein_kv_tests::golden::{KEPT, LAST, LIMITS, commits, expect_event, model, open};
use skein_lib::Token;
use std::collections::BTreeMap;

fn format_name(name: &[u8]) -> &'static str {
    if name == b"snapshot" {
        "skein-kv's snapshot format (data/kv.md 3 and 5)"
    } else {
        "skein-kv's log frame format (data/kv.md 3)"
    }
}

fn assert_kept(made: &BTreeMap<Box<[u8]>, Box<[u8]>>) {
    let kept: BTreeMap<&[u8], &[u8]> = KEPT.into_iter().collect();
    let mut differences = Vec::new();
    for (&name, &bytes) in &kept {
        let format = format_name(name);
        let label = String::from_utf8_lossy(name);
        match made.get(name) {
            None => differences.push(format!(
                "missing file {label}: {format}: first differing offset 0, kept length {}, made length 0",
                bytes.len()
            )),
            Some(written) if written.as_ref() != bytes => {
                let offset = bytes
                    .iter()
                    .zip(written.iter())
                    .position(|(a, b)| a != b)
                    .unwrap_or(bytes.len().min(written.len()));
                differences.push(format!(
                    "{label}: {format}: first differing offset {offset}, kept length {}, made length {}",
                    bytes.len(),
                    written.len()
                ));
            }
            Some(_) => {}
        }
    }
    for (name, bytes) in made {
        if !kept.contains_key(name.as_ref()) {
            differences.push(format!(
                "extra file {}: {}: first differing offset 0, kept length 0, made length {}",
                String::from_utf8_lossy(name),
                format_name(name),
                bytes.len()
            ));
        }
    }
    assert!(
        differences.is_empty(),
        "{}\nfrom the first release a deployment's store holds these bytes: changing them needs an upgrade of the store's own, run before any records are patched (data/kv.md 9), and moves the format's version",
        differences.join("\n")
    );
}

#[test]
fn the_fixed_commits_write_the_kept_files_byte_for_byte() {
    let mut world = open(&[], 0);
    for (index, ops) in commits().into_iter().enumerate() {
        let number = u64::try_from(index).expect("commit index fits u64") + 1;
        world.request_no_fire(Request::Commit { owner: Token::new(2), ops });
        expect_event(&mut world, &Event::Committed { owner: Token::new(2), number });
        if number == 1 {
            // Finish the snapshot before later commits. They retain its start
            // and roll the log without starting another snapshot before close.
            world.drive(true);
        }
    }
    world.request(Request::Close { owner: Token::new(3) });
    expect_event(&mut world, &Event::Closed { owner: Token::new(3) });
    assert_kept(&world.files());
}

#[test]
fn the_kept_files_recover_to_the_models_map() {
    let items: Vec<Item> = KEPT.iter().map(|(name, bytes)| Item::file(name, bytes)).collect();
    let mut world = open(&items, LAST);
    let mut map = BTreeMap::new();
    let mut range = Range::prefix(b"");
    loop {
        world.request(Request::Load { owner: Token::new(4), range: range.clone(), max: LIMITS.page });
        let Event::Loaded { rows, next, .. } = world.pop() else {
            panic!("setup expected a loaded page");
        };
        for row in rows {
            map.insert(row.key, row.value);
        }
        match next {
            Some(start) => range.start = start,
            None => break,
        }
    }
    assert_eq!(map, model());
}
