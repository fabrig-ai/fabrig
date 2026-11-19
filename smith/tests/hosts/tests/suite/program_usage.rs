#[cfg(test)]
mod tests {
    use skein_lib::{Civil, Duration, Wall};
    use smith_fake_claude::{Story, UsageReply};
    use smith_host_domain::{FailureReason, ProgramUsage, UsageState, UsageWindow};
    use smith_hosts_world::program_world::{Run, usage};

    fn reply(text: &str) -> Story {
        Story {
            usage_read: UsageReply::Result(serde_json::json!({
                "type": "result", "is_error": false, "result": text,
                "usage": {"input_tokens": 0, "output_tokens": 0}, "total_cost_usd": 0,
            })),
            ..Story::default()
        }
    }

    fn read(story: Story) -> Run {
        usage(7, story, false, &[], [Some(0o700); 2])
    }

    fn failed(run: &Run, reason: FailureReason, detail: &[u8]) {
        let ProgramUsage::Failed(failure) = run.usage_answers.first().expect("one usage answer") else {
            unreachable!("failed usage read");
        };
        assert_eq!(failure.reason, reason);
        assert_eq!(failure.detail.as_ref(), detail);
    }

    #[test]
    fn local_usage_reads_windows_in_the_sources_environment() {
        let story = reply(
            "Current session: 16% used · resets Nov 19, 4:30am (UTC)\nCurrent week (all models): 60% used · resets Nov 23, 5am (UTC)\nCurrent week (Fable): 8% used · resets Nov 23, 4:59am (UTC)\n",
        );
        let environment: Vec<Box<[u8]>> = [
            "ANTHROPIC_API_KEY=x",
            "ANTHROPIC_AUTH_TOKEN=x",
            "CLAUDE_CODE_OAUTH_TOKEN=x",
            "ANTHROPIC_PROFILE=x",
            "CLAUDE_CODE_USE_VERTEX=x",
            "AWS_BEARER_TOKEN_BEDROCK=x",
            "TZ=Europe/Rome",
            "SOURCE_VARIABLE=kept",
        ]
        .iter()
        .map(|entry| entry.as_bytes().into())
        .collect();
        let run = usage(7, story.clone(), false, &environment, [Some(0o700); 2]);
        let expected = [
            (b"five_hour".as_slice(), 18_000, 1600, 19, 4_u64, 30_u64),
            (b"seven_day".as_slice(), 604_800, 6000, 23, 5, 0),
            (b"seven_day_fable".as_slice(), 604_800, 800, 23, 4, 59),
        ]
        .map(|(label, seconds, used, day, hour, minute)| UsageWindow {
            label: label.into(),
            length: Some(Duration::from_secs(seconds)),
            used: Some(used),
            resets: Some(Wall::from_nanos(
                (Civil { year: 2026, month: 11, day })
                    .wall()
                    .expect("fixture date")
                    .as_nanos()
                    .checked_add(
                        Duration::from_secs(
                            hour.checked_mul(3600)
                                .expect("fixture hour")
                                .checked_add(minute.checked_mul(60).expect("fixture minute"))
                                .expect("fixture clock"),
                        )
                        .as_nanos(),
                    )
                    .expect("fixture wall time"),
            )),
            state: UsageState::Open,
        });
        assert_eq!(run.usage_answers, [ProgramUsage::Read(expected.into())]);
        assert_eq!(run.launches, 1);
        assert_eq!(run.trace, usage(7, story, false, &environment, [Some(0o700); 2]).trace, "same seed replays usage");
        let observation = run.observations.first().expect("one local command");
        assert_eq!(run.observations.len(), 1, "no status or model command");
        assert_eq!(
            observation.arguments,
            ["-p", "/usage", "--output-format", "json", "--setting-sources", "user", "--tools", ""]
        );
        assert_eq!(
            observation.environment,
            ["HOME", "SOURCE_VARIABLE", "TZ", "CLAUDE_CONFIG_DIR", "ANTHROPIC_CONFIG_DIR"]
        );
        let limited = read(reply("Current session: 100% used\n"));
        let ProgramUsage::Read(windows) = limited.usage_answers.first().expect("limited answer") else {
            unreachable!("windows");
        };
        assert_eq!(windows.first().expect("session").state, UsageState::Limited);
    }

    #[test]
    fn usage_inspects_both_private_levels_before_starting_a_child() {
        for directories in [[None, None], [Some(0o700), None]] {
            let run = usage(7, Story::default(), false, &[], directories);
            assert_eq!(run.usage_answers, [ProgramUsage::NotSignedIn]);
            assert_eq!(run.launches, 0);
            assert_eq!(run.observations, []);
            assert_eq!(run.directories, directories);
        }
        for directories in [[Some(0o755), Some(0o700)], [Some(0o700), Some(0o775)]] {
            let run = usage(7, Story::default(), false, &[], directories);
            failed(&run, FailureReason::Unreadable, b"program source directory is not private");
            assert_eq!(run.launches, 0);
            assert_eq!(run.directories, directories);
        }
    }

    #[test]
    fn usage_failures_keep_their_reason_and_bounded_detail() {
        let error = Story {
            usage_read: UsageReply::Result(
                serde_json::json!({"type":"result", "is_error":true, "result":"usage refused"}),
            ),
            ..Story::default()
        };
        failed(&read(error), FailureReason::Exited, b"usage refused");
        let exit =
            Story { usage_read: UsageReply::Exit { code: 2, detail: "program exited".into() }, ..Story::default() };
        failed(&read(exit), FailureReason::Exited, b"program exited");
        failed(&read(reply("No allowance information")), FailureReason::Unreadable, b"");
        failed(&usage(7, Story::default(), true, &[], [Some(0o700); 2]), FailureReason::Missing, b"");
        failed(&read(reply(&"x".repeat(16_384))), FailureReason::Unreadable, b"");
    }

    #[test]
    fn usage_refuses_tokens_or_positive_cost_before_reading_windows() {
        for (tokens, cost) in [(1_u64, serde_json::json!(0_u64)), (0_u64, serde_json::json!(0.001_f64))] {
            let story = Story {
                usage_read: UsageReply::Result(serde_json::json!({
                    "type":"result", "is_error":false, "result":null,
                    "usage":{"input_tokens":tokens, "output_tokens":0_u64}, "total_cost_usd":cost,
                })),
                ..Story::default()
            };
            assert_eq!(read(story).usage_answers, [ProgramUsage::Spent]);
        }
    }

    #[test]
    fn usage_deadline_answers_after_the_silent_child_settles() {
        let run = read(Story { usage_read: UsageReply::Silent, ..Story::default() });
        failed(&run, FailureReason::Deadline, b"");
        assert_eq!(run.launches, 1);
        assert_eq!(run.observations.len(), 1);
    }
}
