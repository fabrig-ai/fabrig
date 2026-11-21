use skein_kv::{Event, Request};
use skein_kv_tests::golden::{KEPT, commits, expect_event, open};
use skein_lib::Token;
use std::collections::BTreeMap;
use std::path::Path;

fn format_name(name: &[u8]) -> &'static str {
    if name == b"snapshot" {
        "skein-kv's snapshot format (data/kv.md 3 and 5)"
    } else {
        "skein-kv's log frame format (data/kv.md 3)"
    }
}

/// Compares the files made with KEPT; on a mismatch writes the files made
/// under `CARGO_TARGET_TMPDIR` and fails naming the format.
fn assert_kept(made: &BTreeMap<Box<[u8]>, Box<[u8]>>) {
    let kept: BTreeMap<&[u8], &[u8]> = KEPT.into_iter().collect();
    let mut differences = Vec::new();
    for (&name, &bytes) in &kept {
        let format = format_name(name);
        let label = String::from_utf8_lossy(name);
        match made.get(name) {
            None => differences.push(format!("missing file {label}: {format}")),
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
    for name in made.keys() {
        if !kept.contains_key(name.as_ref()) {
            differences.push(format!("extra file {}: {}", String::from_utf8_lossy(name), format_name(name)));
        }
    }
    if differences.is_empty() {
        return;
    }
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join("kv-golden-made");
    if path.exists() {
        std::fs::remove_dir_all(&path).expect("remove the previous failure's output");
    }
    std::fs::create_dir_all(&path).expect("create the failure's output directory");
    for (name, bytes) in made {
        let name = std::str::from_utf8(name).expect("store filenames are ASCII");
        std::fs::write(path.join(name), bytes).expect("write the files made for inspection");
    }
    panic!(
        "{}\nfrom the first release a deployment's store holds these bytes: changing them needs an upgrade of the store's own, run before any records are patched (data/kv.md 9), and moves the format's version\nfiles made written to {}",
        differences.join("\n"),
        path.display()
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
