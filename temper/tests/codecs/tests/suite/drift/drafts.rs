//! Landed-draft examples at the design's bounds, beside the schema generator's small examples.
//! Field order and tags: temper/docs/design/client/forge/records.md, `SetDraft` and drafts.
use skein_codegen::Golden;

fn bytes(out: &mut Vec<u8>, value: &[u8]) {
    out.extend(u32::try_from(value.len()).expect("fixture length fits").to_be_bytes());
    out.extend(value);
}
fn golden(name: &str, bytes: Vec<u8>) -> Golden {
    Golden { name: format!("draft_{name}.bin"), bytes }
}

fn landed() -> Vec<u8> {
    let mut out = Vec::new();
    out.extend(u16::MAX.to_be_bytes()); // forge
    for (byte, length) in [(b'o', 1024), (b'n', 1024), (b'p', 1024), (b'b', 1024), (b'c', 64), (b't', 200)] {
        bytes(&mut out, &vec![byte; length]); // owner, name, path, branch, commit, title
    }
    out.extend(u64::MAX.to_be_bytes()); // writer
    out.extend(u64::MAX.to_be_bytes()); // at
    out.push(1);
    out.extend(7_u64.to_be_bytes()); // chat
    out.push(1);
    out.extend(9_u64.to_be_bytes()); // goal
    out.push(1); // ready
    out
}

pub(super) fn goldens() -> Vec<Golden> {
    let mut out = Vec::new();
    for (repository, suffix) in [(false, "home"), (true, "repository")] {
        for (change, chat, name) in
            [(0, true, "ready_chat"), (0, false, "ready"), (1, false, "not_ready"), (2, false, "started")]
        {
            let mut value = 16_u16.to_be_bytes().to_vec();
            value.push(17); // Request::set_draft, after try_account
            value.extend(3_u32.to_be_bytes());
            value.push(u8::from(repository));
            if repository {
                bytes(&mut value, b"acme");
                bytes(&mut value, b"shop");
            }
            bytes(&mut value, b"docs/design/draft/catch-up.md");
            value.push(change);
            if change == 0 {
                value.push(u8::from(chat));
                if chat {
                    value.extend(7_u64.to_be_bytes());
                }
            } else if change == 2 {
                value.extend(9_u64.to_be_bytes());
            }
            out.push(golden(&format!("request_{name}_{suffix}"), value));
        }
    }
    let row = landed();
    let mut drafted = vec![0, 16, 27]; // Answer::drafted, after not_theirs
    drafted.extend(&row);
    out.push(golden("answer_drafted", drafted));
    for (tag, name) in
        ["no_repository", "path", "missing", "not_a_file", "unreachable", "unknown", "full"].iter().enumerate()
    {
        out.push(golden(&format!("answer_{name}"), vec![0, 16, 28, u8::try_from(tag).expect("refusal tag")]));
    }
    let mut watch = vec![0, 16, 12]; // Watch::drafts, after ways
    watch.extend(3_u32.to_be_bytes());
    out.push(golden("watch", watch));
    for (count, name) in [(0_u32, "smallest"), (64, "full")] {
        let mut panel = 16_u16.to_be_bytes().to_vec();
        panel.extend(3_u32.to_be_bytes());
        panel.extend(count.to_be_bytes());
        for _ in 0..count {
            panel.extend(&row);
        }
        out.push(golden(&format!("panel_{name}"), panel));
    }
    out
}
