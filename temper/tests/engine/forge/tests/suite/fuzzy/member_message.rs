//! Seeded arbitrary producer bytes preserve the tooling message contract.
use skein_lib::Rng;
use temper_engine_domain_forge::{commit_text, member_message::{Member, commit_message, names_head}};

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
            title.push(rng.next_u64().to_le_bytes()[0]);
        }
        let mut head = [0; 32];
        for byte in &mut head {
            *byte = rng.next_u64().to_le_bytes()[0];
        }
        if seed % 2 == 0 {
            head[20..].fill(0);
        }
        let goal = if seed % 3 == 0 { None } else { Some(rng.next_u64()) };
        let member = Member { title: &title, words: &words, change: rng.next_u64(), pull: rng.next_u64(), head, goal };
        let with_goal = seed % 2 == 0;
        let bytes = commit_message(&member, with_goal);
        assert!(!bytes.contains(&0), "seed {seed}: NUL");
        // limits.checkout.message_bytes in the small profile (service-core/src/limits.rs).
        assert!(bytes.len() <= 4096, "seed {seed}: checkout message bound");
        assert!(names_head(&bytes, head), "seed {seed}: head recognition");
        let message = std::str::from_utf8(&bytes).expect("message UTF-8");
        let paragraphs: Vec<_> = message.split("\n\n").collect();
        assert!((2..=3).contains(&paragraphs.len()), "seed {seed}: paragraphs");
        let goal_line = if with_goal { goal.map_or_else(String::new, |goal| format!("\nGoal: {goal}")) } else { String::new() };
        let trailers = format!("Change: {}\nPull-request: {}\nHead: {}{goal_line}", member.change, member.pull,
            std::str::from_utf8(&commit_text(&head)).expect("hex"));
        assert_eq!(paragraphs.last().copied(), Some(trailers.as_str()), "seed {seed}: last paragraph");
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
    }
}
