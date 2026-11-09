//! Seeded address lines in the two-project person world
//! (temper/docs/design/domain/agent.md, 8.4.1; skein/docs/foundation/testing-strategy.md, 8).

use crate::configuring::asking_person::{self, PersonWorld};
use serde_json::json;
use skein_lib::Rng;
use temper_engine_domain_world::host_tool::host_tool;

// The grammar and next-address lines are the oracle from agent.md, 8.4.1's table.
// N and A.T are filled from the world's tasks and real or made-up turn pairs.
const FORMS: &[(&str, &str)] = &[
    ("me", "inbox · project · goals · tasks"),
    ("project", "goals · tasks · changes · inbox · settings"),
    ("goals", "task N · task N tree · goals ended"),
    ("goals ended", "task N · goals"),
    ("tasks", "task N · goals · inbox"),
    ("task N", "task N tree · task N activity · change N · tasks"),
    ("task N tree", "task N · task N activity · task N tree"),
    ("task N activity", "task N turn A.T · task N · task N tree"),
    ("task N turn A.T", "task N activity · task N"),
    ("change N", "task N · task N activity · changes · settings gates OWNER/NAME"),
    ("changes", "change N · settings landing · ways"),
    ("inbox", "task N · me"),
    (
        "settings",
        "settings charter N · settings template N · settings policy N · settings model N · settings forge-charter N · settings rules · settings landing-rule N [project] · settings landing · settings gates OWNER/NAME · settings project-ways · settings project-questions · settings project-classes · revisions · help settings",
    ),
    ("settings charter N", "settings · revisions · help settings-fields"),
    ("settings template N", "settings · revisions · help settings-fields"),
    ("settings policy N", "settings · revisions · help settings-fields"),
    ("settings model N", "settings · revisions · help settings-fields"),
    ("settings forge-charter N", "settings · revisions · help settings-fields"),
    ("settings rules", "settings · revisions · help settings-fields"),
    ("settings template-goal N", "settings template N · ways · help settings-fields"),
    ("settings landing-rule N", "settings landing · settings · help landing-rule"),
    ("settings landing-rule N project", "settings landing · settings · help landing-rule"),
    ("settings project-ways", "settings · revisions · help settings-fields"),
    ("settings project-questions", "settings · revisions · help settings-fields"),
    ("settings project-classes", "settings · revisions · help settings-fields"),
    ("settings charter-words N", "settings · revisions · help settings-fields"),
    ("settings project-words", "settings · revisions · help settings-fields"),
    ("settings landing", "settings gates OWNER/NAME · settings landing-rule N [project] · help landing"),
    ("settings gates org/repo", "settings landing · file OWNER/NAME PATH"),
    (
        "ways",
        "settings rules · settings landing · settings project-ways · settings project-words · settings charter-words N · help settings",
    ),
    ("revisions", "revision N · settings · help revision"),
    ("revision N", "revisions · proposal tN/K · proposal pN/K · settings"),
    ("proposal tN/1", "revision N · revisions · help proposal"),
    ("proposal pN/1", "revision N · revisions · help proposal"),
    ("help", "help NAME · help search WORDS"),
    ("help see", "help · help search WORDS"),
    ("help search task turn", "help NAME"),
    ("file org/repo docs/direction.md", "file OWNER/NAME PATH [BRANCH] · settings gates OWNER/NAME"),
    ("file org/repo docs/direction.md main", "file OWNER/NAME PATH [BRANCH] · settings gates OWNER/NAME"),
];

fn at(random: &mut Rng, length: usize) -> usize {
    usize::try_from(random.below(u64::try_from(length).expect("grammar length"))).expect("grammar index")
}

fn word(random: &mut Rng) -> String {
    (0..=random.below(16)).map(|_| char::from(b'a' + u8::try_from(random.below(26)).expect("ASCII letter"))).collect()
}

fn fill(form: &str, number: &str, pair: &str) -> String {
    form.replace('N', number).replace("A.T", pair)
}

fn mutate(line: &str, seed: u64, random: &mut Rng) -> String {
    let mut words: Vec<_> = line.split_ascii_whitespace().map(str::to_owned).collect();
    let index = at(random, words.len());
    match seed % 5 {
        0 => {
            words.remove(index);
        }
        1 => words.insert(index, words[index].clone()),
        2 => {
            let index = words
                .iter()
                .position(|word| word.bytes().any(|byte| byte.is_ascii_lowercase()))
                .expect("address has a keyword");
            words[index].make_ascii_uppercase();
        }
        3 => words[index] = word(random),
        4 => {
            let separator = format!("{}{}", " ".repeat(1 + at(random, 8)), "\t".repeat(1 + at(random, 8)));
            return format!("{separator}{}{separator}", words.join(&separator));
        }
        _ => unreachable!("five mutations"),
    }
    words.join(" ")
}

/// Successful parts have a body, optionally a continuation, and their final see line.
/// The literal bound is independent of the renderer's `SEE_PART` so a raised bound fails here.
fn frame(text: &str, next: Option<&str>) -> Option<String> {
    let (part, following) = text.rsplit_once("\nsee: ").expect("answer ends with its see line");
    let following = following.strip_suffix('\n').expect("see line ends with a newline");
    assert!(!following.is_empty() && !following.contains('\n'), "one final see line: {text}");
    if let Some(next) = next {
        assert_eq!(following, next, "the address's next reads");
    }
    let (body, token) = part.rsplit_once("\nmore: ").map_or((part, None), |(body, token)| {
        assert!(!token.is_empty() && !token.bytes().any(|byte| byte.is_ascii_whitespace()), "more token: {text}");
        (body, Some(token.to_owned()))
    });
    assert!(body.len() < 6_144, "body within 6144 bytes, got {}", body.len() + 1);
    token
}

#[test]
fn seeded_addresses_stay_bounded_framed_and_in_their_project() {
    // Reads commit nothing and do not advance time: every call checks this same snapshot.
    let mut person = asking_person::world();
    let before = person.world.driver.store.rows.clone();
    let mut completion = 0;
    let mut check = |person: &mut PersonWorld, seed, line: &str, next| {
        completion += 1;
        eprintln!("seed {seed}, call {completion}: {line:?}");
        let (body, error) = host_tool(
            &mut person.world.driver,
            &person.owner,
            completion,
            b"see",
            json!({"address":line}).to_string().as_bytes(),
        );
        let text = std::str::from_utf8(&body).expect("UTF-8 answer");
        let limits = &person.world.driver.env.limits;
        let allowance = limits
            .call_answer_bytes
            .saturating_sub(3)
            .min(smith_agent_world::LIMITS.run.host_reply_bytes)
            .min(limits.host_reply_bytes.unwrap_or(u32::MAX));
        assert!(body.len() <= usize::try_from(allowance).unwrap(), "seed {seed}: {line:?}: answer allowance");
        assert!(!text.contains("ELSEWHERE"), "seed {seed}: {line:?}: other project: {text}");
        assert_eq!(person.world.driver.store.rows, before, "seed {seed}: {line:?}: reads commit nothing");
        if error { None } else { frame(text, next) }
    };

    let turn = format!("task {} turn {}.1", person.producer.task, person.producer.attempt);
    let next = "task N activity · task N";
    let token = check(&mut person, 0, &turn, Some(next)).expect("long turn has a real more token");
    let (encoded, _) = token.rsplit_once('@').expect("address and offset");
    let numbers = [
        person.goal.task,
        person.delegate.task,
        person.change,
        person.producer.task,
        person.owner.task,
        person.member.task,
        person.elsewhere,
        person.elsewhere + 1,
        0,
        999_999,
        u64::MAX,
    ]
    .map(|number| number.to_string())
    .into_iter()
    .chain(["18446744073709551616".to_owned()])
    .collect::<Vec<_>>();
    let pairs = [
        format!("{}.1", person.producer.attempt),
        format!("{}.0", person.producer.attempt),
        format!("{}.2", person.producer.attempt),
        "0.0".to_owned(),
        "999999.999999".to_owned(),
        format!("{}.{}", u64::MAX, u32::MAX),
        "18446744073709551616.1".to_owned(),
        "1.4294967296".to_owned(),
    ];
    let numeric = [5, 6, 7, 8, 9];
    for seed in 0..64_u64 {
        let mut random = Rng::new(seed + 8_410);
        let (form, following) = FORMS[usize::try_from(seed).unwrap() % FORMS.len()];
        let line = fill(form, &numbers[at(&mut random, numbers.len())], &pairs[at(&mut random, pairs.len())]);
        check(&mut person, seed, &line, Some(following));
        // The first 60 seeds cross every numeric person address with every number class.
        let (form, following) = FORMS[numeric[usize::try_from(seed % 5).unwrap()]];
        let number = &numbers[usize::try_from(seed / 5).unwrap() % numbers.len()];
        let pair = &pairs[usize::try_from(seed / 5).unwrap() % pairs.len()];
        check(&mut person, seed, &fill(form, number, pair), Some(following));
        let mutated = mutate(&line, seed, &mut random);
        check(&mut person, seed, &mutated, None);
        let mut long = format!("{line} {}", word(&mut random).repeat(513));
        long.truncate(513);
        check(&mut person, seed, &long[..512], None);
        check(&mut person, seed, &long, None);
        check(&mut person, seed, &format!("more {token}"), Some(next));
        let offset = match seed % 4 {
            0 => random.below(u64::try_from(person.turn_text.len()).unwrap()),
            1 => u64::from(u32::MAX),
            2 => u64::MAX,
            3 => u64::from(u32::MAX) + 1,
            _ => unreachable!("four offset classes"),
        };
        check(&mut person, seed, &format!("more {encoded}@{offset}"), Some(next));
        check(&mut person, seed, &format!("more {}", word(&mut random)), None);
    }
}
