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
    support::live::notice("TAG live-native; sitting=first; phase=chat-and-push; budget=2580s");
    let result = (|| -> Result<(), String> {
        let seen = support::live::run(scratch.path(), &forgejo, &instructions, Duration::from_secs(2_580))?;
        if (seen.seen.signed_in, seen.seen.finished) != (1, true) {
            return Err(String::from_utf8_lossy(&seen.seen.stderr).into_owned());
        }
        if forgejo.branch_head(&seen.branch) != Some(seen.pushed) {
            return Err("the branch is at the run's commit".into());
        }
        Ok(())
    })();
    let completions = support::live::check_completions(scratch.path());
    support::live::notice(&format!(
        "live completions={}; wall={}s; state={}",
        completions.as_ref().map_or_else(|why| format!("unavailable: {why}"), u32::to_string),
        clock.now().now.saturating_since(started).as_nanos() / 1_000_000_000,
        scratch.path().display()
    ));
    let result = result.and_then(|()| {
        let completions = completions?;
        if !(1..=25).contains(&completions) {
            return Err(format!("at most 25 completions and at least one: {completions}"));
        }
        Ok(())
    });
    result.unwrap_or_else(|why| {
        support::live::notice(&format!("FAILURE {why}"));
        panic!("{why}")
    });
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

/// testing/live-tier.md: the launcher supplies the story's own store after its process ends.
#[test]
#[ignore = "live: the held record's return on a box of its own, run by scripts/proof-live.sh once a held-login story's process has ended"]
fn the_held_record_returns_on_a_box_of_its_own() {
    let store = std::path::PathBuf::from(temper_test_env::value(&temper_test_env::LIVE_STORE).expect("story's store"));
    let account =
        std::path::PathBuf::from(temper_test_env::value(&temper_test_env::LIVE_ACCOUNT).expect("held record's path"));
    let paths = ["config", "data", "cache"].map(|name| store.join(name));
    support::forge_land::rung::return_on_a_box_of_its_own(
        &skein_scratch::paths::binary("temper-box"),
        &store,
        &paths,
        &account,
    )
    .unwrap_or_else(|message| panic!("{}", support::held_login::not_back(&message, &store)));
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

#[expect(clippy::too_many_lines, reason = "The summary follows both sittings and their returned landing findings.")]
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
    let mut completions = Ok(0);
    let result = (|| -> Result<(), String> {
        let mut bot = None;
        let mut previous: Option<forge_land::LiveEvidence> = None;
        for (first, run) in [(true, "first"), (false, "second")] {
            let client = store.join(if kept { format!("{tag}-{run}") } else { run.to_owned() });
            fs::create_dir(&client).map_err(|error| format!("fresh client root: {error}"))?;
            fs::set_permissions(&client, fs::Permissions::from_mode(0o700))
                .map_err(|error| format!("private client root: {error}"))?;
            let remaining = deadline.saturating_since(clock.now().now);
            support::live::notice(&format!(
                "TAG {tag}; sitting={run}; phase=forge-landing; budget={}s",
                remaining.as_nanos() / 1_000_000_000
            ));
            if remaining <= Duration::from_secs(10) {
                return Err("40 minute total story budget".into());
            }
            let outcome = forge_land::run_live_counted(
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
                &mut completions,
            )?;
            let stderr = String::from_utf8_lossy(&outcome.stderr);
            if outcome.exit != Some(skein_io::kernel::Exit::Code(0)) {
                return Err(format!("{run}: {stderr}"));
            }
            if outcome.left != Some(vec![]) {
                return Err(format!("no surviving children: {stderr}"));
            }
            bot = Some(forge_land::check_forgejo_setup(
                if first && !kept { "first" } else { "second" },
                &outcome.calls[..if first && !kept { 7 } else { 4 }],
                &stderr,
                bot,
                &paths[1],
            )?);
            let evidence = outcome.checked.ok_or("landing and reply checked before stopping")?;

            let identity = forge_land::check_landed_identity(&evidence.task, &evidence.page)?;
            if !identity.0.contains(&evidence.forge.merge) {
                return Err("temper reads the forge's merge hash".into());
            }
            if !identity.1.starts_with(&format!("PR #{} ·", evidence.forge.number)) {
                return Err("temper reads the same pull".into());
            }
            support::first_repository::check_landed_ci(run, &evidence.page)?;
            if first {
                previous = Some(evidence);
            } else {
                let before = previous.as_ref().ok_or("first landing")?;
                let before_identity = forge_land::check_landed_identity(&before.task, &before.page)?;
                if identity != before_identity {
                    return Err("same change identity after restart".into());
                }
                if evidence.forge != before.forge {
                    return Err("no second push or merge: same branches and pulls".into());
                }
                if evidence.tasks != before.tasks {
                    return Err("same task numbers after restart".into());
                }
                if evidence.delegates != before.delegates {
                    return Err("same change and producer tasks".into());
                }
                for (position, turn) in &before.replies {
                    if evidence.replies.get(position) != Some(turn) {
                        return Err("committed chat reply survives restart".into());
                    }
                }
            }
        }
        let evidence = previous.ok_or("landed identity")?;
        let forge = evidence.forge;
        support::live::notice(&format!(
            "TAG {tag}\nPULL {} #{}\nMERGE {}\nCOMPLETIONS {}",
            forge.url, forge.number, forge.merge, evidence.completions
        ));
        if clock.now().now > deadline {
            return Err("40 minute total story budget".into());
        }
        Ok(())
    })();
    support::live::notice(&format!(
        "TAG {tag}; completions={}; wall={}s",
        completions.as_ref().map_or_else(|why| format!("unavailable: {why}"), u32::to_string),
        clock.now().now.saturating_since(started).as_nanos() / 1_000_000_000
    ));
    result.unwrap_or_else(|message| {
        support::live::notice(&format!("FAILURE {message}"));
        panic!("{message}")
    });
    scratch.remove();
}
