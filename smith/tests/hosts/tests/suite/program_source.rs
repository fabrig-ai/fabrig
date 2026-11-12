#[cfg(test)]
mod tests {
    use smith_fake_claude::{Status, Story};
    use smith_host_domain::{FailureReason, ProgramCheck, ProgramMethod, ProgramSignOut};
    use smith_host_protocol::{SourceAnswer, SourceAsk};
    use smith_hosts_world::program_world::{Run, run};

    fn check(login: Option<&[u8]>, story: Option<Story>) -> Run {
        run(7, story, login, &[SourceAsk::Check], false, &[], [Some(0o700); 2])
    }

    fn failed(run: &Run, expected: FailureReason) {
        let SourceAnswer::Check(ProgramCheck::Failed(failure)) = run.answers.first().expect("one answer") else {
            unreachable!("failed check")
        };
        assert_eq!(failure.reason, expected);
        assert!(failure.detail.is_empty(), "no stdout in failure detail");
    }

    #[test]
    fn an_absent_source_stays_absent_until_login() {
        for directories in [[None, None], [Some(0o700), None]] {
            let result = run(7, None, None, &[SourceAsk::Check], false, &[], directories);
            assert_eq!(result.answers, [SourceAnswer::Check(ProgramCheck::NotSignedIn)]);
            assert_eq!(result.launches, 0, "declaration must not launch before login");
            assert_eq!(result.directories, directories, "declaration leaves absence unchanged");
        }
        for mode in [0o755, 0o775] {
            for directories in [[Some(mode); 2], [Some(0o700), Some(mode)]] {
                let result = run(7, None, None, &[SourceAsk::Check], false, &[], directories);
                let SourceAnswer::Check(ProgramCheck::Failed(failure)) = result.answers.first().expect("one answer")
                else {
                    unreachable!("nonprivate directories must be refused");
                };
                assert_eq!(failure.reason, FailureReason::Unreadable);
                assert_eq!(result.launches, 0, "unsafe directories never launch");
                assert_eq!(result.directories, directories, "checks never chmod");
            }
        }
        let prepared = check(None, None);
        assert_eq!(prepared.launches, 1, "private login preparation permits the check");
        assert_eq!(prepared.answers, [SourceAnswer::Check(ProgramCheck::NotSignedIn)]);
    }

    #[test]
    fn a_source_never_signed_in_is_not_signed_in() {
        assert_eq!(check(None, None).answers, [SourceAnswer::Check(ProgramCheck::NotSignedIn)]);
    }

    #[test]
    fn a_source_signed_in_is_usable() {
        let first = check(Some(b"claude.ai"), None);
        assert_eq!(
            first.answers,
            [SourceAnswer::Check(ProgramCheck::Usable { email: Some(b"you@example.com".as_slice().into()) })]
        );
        assert_eq!(first.trace, check(Some(b"claude.ai"), None).trace, "same seed replays the source world");
    }

    #[test]
    fn a_source_signed_in_otherwise_names_how() {
        assert_eq!(
            check(Some(b"api_key"), None).answers,
            [SourceAnswer::Check(ProgramCheck::SignedInOtherwise(ProgramMethod::new(b"api_key").expect("method")))]
        );
    }

    #[test]
    fn a_silent_status_fails_at_the_deadline() {
        failed(&check(None, Some(Story { status: Status::Silent, ..Story::default() })), FailureReason::Deadline);
    }

    #[test]
    fn an_unreadable_status_fails_as_unreadable() {
        failed(&check(None, Some(Story { status: Status::Unreadable, ..Story::default() })), FailureReason::Unreadable);
    }

    #[test]
    fn a_missing_program_fails_as_missing() {
        failed(&run(7, None, None, &[SourceAsk::Check], true, &[], [Some(0o700); 2]), FailureReason::Missing);
    }

    #[test]
    fn signing_out_removes_the_login() {
        let run =
            run(7, None, Some(b"claude.ai"), &[SourceAsk::SignOut, SourceAsk::Check], false, &[], [Some(0o700); 2]);
        assert_eq!(
            run.answers,
            [SourceAnswer::SignOut(ProgramSignOut::SignedOut), SourceAnswer::Check(ProgramCheck::NotSignedIn)]
        );
        assert_eq!(run.login, None);
    }

    #[test]
    fn the_status_never_sees_another_login_s_variables() {
        let run = run(
            7,
            None,
            Some(b"claude.ai"),
            &[SourceAsk::Check],
            false,
            &[b"ANTHROPIC_API_KEY=x".as_slice().into()],
            [Some(0o700); 2],
        );
        assert_eq!(
            run.answers,
            [SourceAnswer::Check(ProgramCheck::Usable { email: Some(b"you@example.com".as_slice().into()) })]
        );
        assert_eq!(run.observations.len(), 1);
        assert_eq!(run.observations.first().expect("one observation").arguments, ["auth", "status"]);
        assert_eq!(
            run.observations.first().expect("one observation").environment,
            ["HOME", "CLAUDE_CONFIG_DIR", "ANTHROPIC_CONFIG_DIR"]
        );
    }
}
