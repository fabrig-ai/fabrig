//! A person asks a real model to change a repository on the owned forge.
use skein_lib::Duration;
use temper_box_kit as support;
#[test]
#[ignore = "live: a person's fresh ChatGPT sign-in, through scripts/proof-live.sh"]
fn a_real_model_answers_a_chat_and_pushes_a_small_change() {
    let clock = skein_shell::Clock::new();
    let started = clock.now().now;
    let scratch = support::Scratch::live();
    let forgejo = support::forgejo::Throwaway::start(&scratch);
    support::configuration(scratch.path(), &forgejo.forge());
    let instructions = support::live::configure(scratch.path(), &forgejo);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        support::live::run(scratch.path(), &forgejo, &instructions, Duration::from_secs(2_580))
    }));
    let completions = support::live::completions(scratch.path());
    support::live::notice(&format!(
        "live completions={completions}; wall={}s; state={}",
        clock.now().now.saturating_since(started).as_nanos() / 1_000_000_000,
        scratch.path().display()
    ));
    let seen = result.unwrap_or_else(|failed| std::panic::resume_unwind(failed));
    support::live::notice(&format!("live completions={completions}; wall={}s", seen.wall_seconds));
    assert_eq!((seen.seen.signed_in, seen.seen.finished), (1, true), "{}", String::from_utf8_lossy(&seen.seen.stderr));
    assert_eq!(forgejo.branch_head(&seen.branch), Some(seen.pushed), "the branch is at the run's commit");
    assert!((1..=25).contains(&completions), "at most 25 completions and at least one: {completions}");
    forgejo.stop();
    scratch.remove();
}

/// testing/live-tier.md: init's shipped chat chooses change on real Responses.
#[test]
#[ignore = "live: a held ChatGPT account, TEMPER_TEST_LIVE_ACCOUNT; scripts/proof-live.sh forge-land"]
fn a_person_asks_chatgpt_to_land_a_change_on_the_owned_forge() {
    let account = support::forge_land::live_account();
    owned_forge_landing(support::forge_land::LiveModel::ChatGpt { account: account.as_deref() });
}

/// testing/live-tier.md: B0's passing walk, with its original phase bounds.
#[test]
#[ignore = "live: a held ChatGPT account, on the rung's sol; scripts/proof-live.sh rung-b0"]
fn rung_b0_a_person_agrees_a_plan_before_the_change_lands() {
    support::forge_land::rung::green_light();
}

/// testing/live-tier.md and protocol/credentials.md, 3: a held login used in place.
#[test]
#[ignore = "live: a held Claude Code login, TEMPER_TEST_LIVE_CLAUDE; the after-landing tier, or scripts/proof-live.sh forge-land-claude"]
fn a_person_asks_claude_code_to_land_a_change_on_the_owned_forge() {
    let login = std::path::PathBuf::from(
        temper_test_env::value(&temper_test_env::LIVE_CLAUDE).expect("held Claude Code login"),
    );
    owned_forge_landing(support::forge_land::LiveModel::ClaudeCode { login: &login });
}

/// testing/live-tier.md: the direct backend on an Anthropic key, with no held record.
#[test]
#[ignore = "live: an Anthropic API key, TEMPER_TEST_LIVE_ANTHROPIC_KEY; the after-landing tier, or scripts/proof-live.sh forge-land-claude-key"]
fn a_person_asks_claude_on_an_api_key_to_land_a_change_on_the_owned_forge() {
    use support::forge_land::{LiveModel, anthropic_key};
    let key = match anthropic_key(temper_test_env::value(&temper_test_env::LIVE_ANTHROPIC_KEY)) {
        Ok(Some(key)) => key,
        Ok(None) => {
            support::live::notice("TEMPER_TEST_LIVE_ANTHROPIC_KEY is unset; forge-land-claude-key skips");
            return;
        }
        Err(reason) => panic!("{reason}"),
    };
    owned_forge_landing(LiveModel::AnthropicKey { key: &key });
}

#[expect(clippy::too_many_lines, reason = "The guard spans both sittings and their landing assertions.")]
fn owned_forge_landing(model: support::forge_land::LiveModel<'_>) {
    use std::{fs, os::unix::fs::PermissionsExt};
    use support::forge_land::{self, Face, LivePlan};
    let clock = skein_shell::Clock::new();
    let started = clock.now().now;
    let deadline = started.saturating_add(Duration::from_secs(2_400));
    let (store, kept) = forge_land::landing_store(model);
    let scratch = support::Scratch::live();
    let paths = ["config", "data", "cache"].map(|name| store.join(name));
    let address = if kept {
        None
    } else {
        let address = forge_land::live_deployment(&paths, model);
        if let forge_land::LiveModel::ChatGpt { .. } = model {
            forge_land::retain_live_model(&paths[0].join("temper.json"), model.name());
        }
        let _ = forge_land::cache_forgejo_binaries(&paths[2]);
        Some(address)
    };
    let tag = forge_land::live_tag();
    let line = format!("Hello from temper, {tag}.");
    let words = format!("Please add the line \"{line}\" to the end of README.md in work/hello and land it.");
    support::live::notice(&format!("TAG {tag}"));
    let _lent = if let forge_land::LiveModel::ChatGpt { account: Some(account) } = model {
        Some(support::held_login::Lent::new(&skein_scratch::paths::binary("temper-box"), &paths, account))
    } else {
        None
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut bot = None;
        let mut previous: Option<forge_land::LiveEvidence> = None;
        let mut completions = 0;
        for (first, run) in [(true, "first"), (false, "second")] {
            let client = store.join(if kept { format!("{tag}-{run}") } else { run.to_owned() });
            fs::create_dir(&client).expect("fresh client root");
            fs::set_permissions(&client, fs::Permissions::from_mode(0o700)).expect("private client root");
            let remaining = deadline.saturating_since(clock.now().now);
            assert!(remaining > Duration::from_secs(10), "40 minute total story budget");
            let outcome = forge_land::run_live(
                LivePlan {
                    paths: &paths,
                    client: &client,
                    first,
                    kept,
                    model,
                    address: previous.as_ref().map(|before| before.address).or(address),
                    tasks: previous.as_ref().map(|before| before.tasks.clone()),
                    baseline: previous.as_ref().map(|before| &before.baseline),
                    roots: previous.as_ref().map(|before| &before.roots),
                    words: &words,
                    line: &line,
                },
                remaining,
                Face::Live,
            )
            .unwrap_or_else(|message| panic!("{message}"));
            let stderr = String::from_utf8_lossy(&outcome.stderr);
            assert_eq!(outcome.exit, Some(skein_io::kernel::Exit::Code(0)), "{run}: {stderr}");
            assert_eq!(outcome.left, Some(vec![]), "no surviving children: {stderr}");
            bot = Some(forge_land::assert_forgejo_setup(
                if first && !kept { "first" } else { "second" },
                &outcome.calls[..if first && !kept { 7 } else { 4 }],
                &stderr,
                bot,
                &paths[1],
            ));
            let evidence = outcome.checked.expect("landing and reply checked before stopping");
            completions = completions.max(evidence.completions);
            let identity = forge_land::landed_identity(&evidence.task, &evidence.page);
            assert!(identity.0.contains(&evidence.forge.merge), "temper reads the forge's merge hash");
            assert!(identity.1.starts_with(&format!("PR #{} ·", evidence.forge.number)), "temper reads the same pull");
            support::first_repository::assert_landed_ci(run, &evidence.page);
            if first {
                previous = Some(evidence);
            } else {
                let before = previous.as_ref().expect("first landing");
                let before_identity = forge_land::landed_identity(&before.task, &before.page);
                assert_eq!(identity, before_identity, "same change identity after restart");
                assert_eq!(evidence.forge, before.forge, "no second push or merge: same branches and pulls");
                assert_eq!(evidence.tasks, before.tasks, "same task numbers after restart");
                assert_eq!(evidence.delegates, before.delegates, "same change and producer tasks");
                for (position, turn) in &before.replies {
                    assert_eq!(evidence.replies.get(position), Some(turn), "committed chat reply survives restart");
                }
            }
        }
        let evidence = previous.expect("landed identity");
        let forge = evidence.forge;
        support::live::notice(&format!(
            "TAG {tag}\nPULL {} #{}\nMERGE {}\nCOMPLETIONS {}",
            forge.url, forge.number, forge.merge, completions
        ));
    }));
    support::live::notice(&format!(
        "TAG {tag}; wall={}s",
        clock.now().now.saturating_since(started).as_nanos() / 1_000_000_000
    ));
    result.unwrap_or_else(|failed| {
        let message = failed
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| failed.downcast_ref::<&str>().copied())
            .unwrap_or("live story failed");
        support::live::notice(&format!("FAILURE {message}"));
        std::panic::resume_unwind(failed)
    });
    assert!(clock.now().now <= deadline, "40 minute total story budget");
    scratch.remove();
}
