//! Ask temper's forge reads (temper/docs/design/domain/agent/ask-temper.md, 1).
use super::*;
use serde_json::json;

#[test]
#[expect(clippy::too_many_lines, reason = "one stock-world table checks the forge addresses, parts and refusals")]
fn an_asking_chat_reads_the_forge() {
    let mut config = config();
    config
        .landings
        .insert(
            PROJECT,
            engine::ProjectLanding {
                gate_retries: 2,
                gate_timeout: skein_lib::Duration::from_millis(3_600_000),
                check_timeout: None,
                by: engine::Way::Queue,
                close_job: None,
                merge_job: Some(b"scripts/merge".as_slice().into()),
                after: engine::AfterLandingCheck { tier_retries: 2, verification_bound_ms: 7_200_000 },
                repositories: Box::new([engine::RepositoryChecks {
                    repository: skein_forge::api::Repository { forge: 1, repository: 2 },
                    gates: Box::new([engine::GateCheck {
                        name: b"tests".as_slice().into(),
                        workflow: b"tests.yml".as_slice().into(),
                        ignored: Box::new([b"docs".as_slice().into()]),
                        blocking: true,
                        base: false,
                    }]),
                    tiers: Box::new([]),
                }]),
            },
        )
        .unwrap();
    let mut bounds = limits();
    bounds.authority.segments = 8; // Repository, branch and file resource names.
    bounds.forge.client.answer_bytes = 65_536; // The long file is read whole before see cuts it.
    bounds.journal.transcript_bytes = 65_536;
    let mut world = World::with_limits(config, &bounds);
    configuring_repository(&mut world);
    let chat = world.chat(CHAT, world.owner, 20);
    let (branch, error) =
        host_tool(&mut world.driver, &chat, 1, b"read_branch", br#"{"repository":"org/repo","branch":"main"}"#);
    assert!(!error, "{}", String::from_utf8_lossy(&branch));
    let head = std::str::from_utf8(&branch).unwrap().trim();
    let expected_file = format!("docs/direction.md at {head}:\n{}", "rules\n".repeat(5334));
    let run = world.chat(ASK_TEMPLATE, world.owner, 21);
    assert_eq!(run.charter, ASK_CHARTER);
    let before = world.driver.store.rows.clone();
    let mut completion = 1;
    for (line, heading, content, expected_error, following) in [
        (
            "settings landing",
            "landing: by queue",
            "org/repo: 1 gate",
            false,
            "settings gates OWNER/NAME · settings landing-rule N [project] · help landing",
        ),
        (
            "settings gates org/repo",
            "gates of org/repo:",
            "tests: workflow tests.yml, blocking, ignoring docs",
            false,
            "settings landing · file OWNER/NAME PATH",
        ),
        (
            "file org/repo docs/direction.md",
            "docs/direction.md at ",
            "rules\n",
            false,
            "file OWNER/NAME PATH [BRANCH] · settings gates OWNER/NAME",
        ),
        (
            "file org/repo docs",
            "docs at ",
            "direction.md",
            false,
            "file OWNER/NAME PATH [BRANCH] · settings gates OWNER/NAME",
        ),
        (
            "settings gates org/none",
            "see settings gates org/none: no repository org/none here;",
            "this project has: org/repo",
            true,
            "",
        ),
        (
            "file org/none README.md",
            "see file org/none README.md: no repository org/none here;",
            "this project has: org/repo",
            true,
            "",
        ),
        (
            "file org/repo docs/direction.md nope",
            "see file org/repo docs/direction.md nope: Forge read failed: missing",
            "The forge has no branch of that name",
            true,
            "",
        ),
        (
            "file org/repo docs/none.md",
            "see file org/repo docs/none.md: Forge read failed: missing",
            "Forge read failed: missing",
            true,
            "",
        ),
    ] {
        let mut address = line.to_string();
        let mut whole = String::new();
        let mut from = 0;
        let mut parts = 0;
        loop {
            let (body, error) = host_tool(
                &mut world.driver,
                &run,
                completion,
                b"see",
                json!({"address":address}).to_string().as_bytes(),
            );
            let text = std::str::from_utf8(&body).unwrap();
            assert_eq!(error, expected_error, "{address}: {text}");
            assert!(body.len() <= usize::try_from(world.driver.env.limits.call_answer_bytes - 3).unwrap());
            if let Some(bound) = world.driver.env.limits.host_reply_bytes {
                assert!(body.len() <= usize::try_from(bound).unwrap());
            }
            assert_eq!(world.driver.store.rows, before, "{address} commits nothing");
            completion += 1;
            parts += 1;
            if error {
                assert!(!text.contains("\nsee: "));
                whole.push_str(text);
                break;
            }
            assert!(text.ends_with(&format!("see: {following}\n")), "{address}: {text}");
            let end = text.find("more: ").or_else(|| text.find("see: ")).unwrap();
            assert!(end <= usize::try_from(temper_engine_smith::SEE_PART).unwrap());
            let next = text.lines().find_map(|line| line.strip_prefix("more: "));
            let bytes = next.map_or(end, |token| token.rsplit_once('@').unwrap().1.parse::<usize>().unwrap() - from);
            whole.push_str(&text[..bytes]);
            from += bytes;
            if let Some(token) = next {
                assert!(token.starts_with(&format!("{}@", line.replace(' ', "+"))));
                address = format!("more {token}");
            } else {
                break;
            }
        }
        assert!(whole.starts_with(heading), "{line}: {whole}");
        assert!(whole.contains(content), "{line}: {whole}");
        if line == "settings landing" || line == "settings gates org/repo" {
            assert!(whole.contains("not a setting yet: whoever runs temper here sets it in temper's configuration (its landing member); no proposal changes it."));
        }
        if line == "settings landing" {
            assert!(whole.contains("merge job: scripts/merge\n"));
            assert!(whole.contains("after landing: 2 tier retries, a verification's bound of 2h"));
            eprintln!("see settings landing:\n{whole}see: {following}");
        }
        if line == "file org/repo docs/direction.md" {
            assert!(parts > 1);
            assert_eq!(whole, expected_file);
        }
    }
}
