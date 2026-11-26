//! Seeded arbitrary producer bytes preserve the tooling message contract.
use skein_lib::Rng;
use temper_engine_domain_forge::{
    commit_text,
    member_message::{Kind, Member, commit_message, names_head, trailers},
};

#[test]
fn arbitrary_member_words_preserve_message_bounds_and_trailers() {
    for seed in 0..256 {
        let mut rng = Rng::new(seed);
        let mut words = Vec::new();
        let length = rng.below(8193);
        let mode = rng.below(4);
        for _ in 0..length {
            match mode {
                0 => words.push(rng.next_u64().to_le_bytes()[0]),
                1 => words.extend_from_slice(match rng.below(8) {
                    0 => b"\n\n",
                    1 => b"\n## Summary\n",
                    2 => b" \t\0\x7f",
                    3 => "é\u{85}\u{2003}".as_bytes(),
                    4 => b"\xff",
                    _ => b"word ",
                }),
                2 => words.push(b'a'),
                _ => words.extend_from_slice("é".as_bytes()),
            }
        }
        let mut title = Vec::new();
        for _ in 0..rng.below(257) {
            title.push(if seed % 2 == 0 {
                rng.next_u64().to_le_bytes()[0]
            } else {
                // Valid UTF-8 titles as well as arbitrary bytes: controls and
                // long runs must not split the subject into paragraphs.
                u8::try_from(rng.below(128)).expect("ASCII byte")
            });
        }
        let mut head = [0; 32];
        for byte in &mut head {
            *byte = rng.next_u64().to_le_bytes()[0];
        }
        if seed % 2 == 0 {
            head[20..].fill(0);
        }
        let goal = if seed % 3 == 0 { None } else { Some(rng.next_u64()) };
        let member = Member {
            title: &title,
            words: &words,
            change: rng.next_u64(),
            pull: rng.next_u64(),
            head,
            goal,
            head_trailers: &[],
        };
        let with_goal = seed % 2 == 0;
        let bytes = commit_message(&member, Kind::Merge { into_main: with_goal });
        assert!(!bytes.contains(&0), "seed {seed}: NUL");
        // limits.checkout.message_bytes in the small profile (service-core/src/limits.rs).
        assert!(bytes.len() <= 4096, "seed {seed}: checkout message bound");
        assert!(names_head(&bytes, head), "seed {seed}: head recognition");
        let message = std::str::from_utf8(&bytes).expect("message UTF-8");
        let paragraphs: Vec<_> = message.split("\n\n").collect();
        assert!((2..=3).contains(&paragraphs.len()), "seed {seed}: paragraphs");
        let goal_line =
            if with_goal { goal.map_or_else(String::new, |goal| format!("\nGoal: {goal}")) } else { String::new() };
        let expected_trailers = format!(
            "Change: {}\nPull-request: {}\nHead: {}{goal_line}",
            member.change,
            member.pull,
            std::str::from_utf8(&commit_text(&head)).expect("hex")
        );
        assert_eq!(paragraphs.last().copied(), Some(expected_trailers.as_str()), "seed {seed}: last paragraph");
        if paragraphs.len() == 3 {
            let description = paragraphs[1];
            assert!(!description.is_empty(), "seed {seed}: description present");
            assert!(description.len() <= 1024, "seed {seed}: description byte bound");
            for line in description.lines() {
                assert!(!line.is_empty(), "seed {seed}: no blank description line");
                assert!(!line.chars().any(char::is_control), "seed {seed}: no description control");
                assert!(line.chars().count() <= 72 || !line.contains(' '), "seed {seed}: wrapped at spaces");
            }
        }

        let mut head_message = b"Subject\n\n".to_vec();
        head_message.extend_from_slice(&trailer_like(&mut rng));
        let head_trailers = trailers(&head_message);
        let mut squash_words = words.clone();
        if rng.below(3) != 0 {
            squash_words.extend_from_slice(b"\n\n");
            squash_words.extend_from_slice(&trailer_like(&mut rng));
        }
        let member = Member { words: &squash_words, head_trailers: &head_trailers, ..member };
        let squash = commit_message(&member, Kind::Squash);
        assert!(squash.len() <= 4096, "seed {seed}: squash bound");
        assert!(!squash.contains(&0), "seed {seed}: squash NUL");
        assert!(names_head(&squash, head), "seed {seed}: squash head recognition");
        let squash = std::str::from_utf8(&squash).expect("squash UTF-8");
        let last = squash.rsplit("\n\n").next().expect("squash trailers");
        let mut required = [0; 4];
        for line in last.split('\n') {
            let (token, value) = line.split_once(": ").unwrap_or_else(|| panic!("seed {seed}: trailer syntax"));
            assert!(token.starts_with(|ch: char| ch.is_ascii_alphabetic()), "seed {seed}: trailer starts with letter");
            assert!(token.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-'), "seed {seed}: token");
            assert!(
                !value.is_empty() && !value.chars().any(char::is_control) && line.trim_end() == line,
                "seed {seed}: trailer value",
            );
            for (index, name) in ["Change", "Pull-request", "Head", "Goal"].iter().enumerate() {
                if token.eq_ignore_ascii_case(name) {
                    required[index] += 1;
                }
            }
        }
        assert_eq!(required, [1, 1, 1, u32::from(goal.is_some())], "seed {seed}: unique required trailers");
    }
}

fn trailer_like(rng: &mut Rng) -> Vec<u8> {
    match rng.below(5) {
        0 => b"Look-request: example\nLook-landed: example\t\r\n".to_vec(),
        1 => b"cHaNgE: 1\npUlL-rEqUeSt: 2\nhEaD: 3\nGoAl: 4\nTicket-2: example".to_vec(),
        2 => b"Key: v\nKe y: v".to_vec(),
        3 => (0..rng.below(128)).map(|_| rng.next_u64().to_le_bytes()[0]).collect(),
        _ => {
            let mut block = b"Link: https://example.invalid/".to_vec();
            block.extend((0..rng.below(6000)).map(|_| b'x'));
            block
        }
    }
}
