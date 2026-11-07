//! The host's list answer right, through the shipped child and fake's list
//! (smith/docs/design/protocol/hosts.md, 2).
#[cfg(test)]
mod tests {
    use skein_lib::Duration;
    use smith_host_domain::{ListFailure, ListOutcome, ListedModel};
    use smith_hosts_world::list_world::{Agent, run};

    const BODY: &[u8] = br#"{"data":[{"id":"model-a","display_name":"Model A","max_input_tokens":200000,"max_tokens":8192,"capabilities":{"effort":{"low":{"supported":true},"high":{"supported":true}}}},{"id":"model-b"}]}"#;

    struct Row {
        name: &'static str,
        agent: Agent,
        cancel: Option<Duration>,
        status: Option<u16>,
        body: &'static [u8],
        unknown: bool,
        short: bool,
        peer: bool,
        expected: Vec<ListOutcome>,
        requests: usize,
    }

    fn listed() -> ListOutcome {
        ListOutcome::Listed(Box::new([
            ListedModel {
                name: b"model-a".as_slice().into(),
                shown: Some(b"Model A".as_slice().into()),
                window: Some(200_000),
                output: Some(8192),
                efforts: Box::new([b"low".as_slice().into(), b"high".as_slice().into()]),
            },
            ListedModel {
                name: b"model-b".as_slice().into(),
                shown: None,
                window: None,
                output: None,
                efforts: Box::new([]),
            },
        ]))
    }

    impl Row {
        fn ordinary(name: &'static str) -> Self {
            Self {
                name,
                agent: Agent::List,
                cancel: None,
                status: None,
                body: BODY,
                unknown: false,
                short: false,
                peer: true,
                expected: vec![listed()],
                requests: 1,
            }
        }
    }

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "one table keeps the host list's outcome and cancellation cases together"
    )]
    fn the_host_answers_one_list_after_its_child_settles() {
        let rows = [
            Row::ordinary("every field, in order"),
            Row { name: "partial pipe writes and reads", short: true, ..Row::ordinary("") },
            Row {
                name: "empty provider list",
                body: br#"{"data":[]}"#,
                expected: vec![ListOutcome::Listed(Box::new([]))],
                ..Row::ordinary("")
            },
            Row {
                name: "provider refusal",
                status: Some(403),
                expected: vec![ListOutcome::Failed(ListFailure::Refused { status: 403 })],
                ..Row::ordinary("")
            },
            Row {
                name: "unreachable provider",
                peer: false,
                requests: 0,
                expected: vec![ListOutcome::Failed(ListFailure::Unreachable)],
                ..Row::ordinary("")
            },
            Row {
                name: "unreadable provider list",
                body: b"garbage",
                expected: vec![ListOutcome::Failed(ListFailure::Unreadable)],
                ..Row::ordinary("")
            },
            Row {
                name: "unknown endpoint",
                unknown: true,
                requests: 0,
                expected: vec![ListOutcome::Failed(ListFailure::Unreadable)],
                ..Row::ordinary("")
            },
            Row {
                name: "startup failure",
                agent: Agent::Startup,
                requests: 0,
                expected: vec![ListOutcome::Failed(ListFailure::Agent)],
                ..Row::ordinary("")
            },
            Row {
                name: "empty stdout",
                agent: Agent::Empty,
                requests: 0,
                expected: vec![ListOutcome::Failed(ListFailure::Agent)],
                ..Row::ordinary("")
            },
            Row {
                name: "garbage stdout",
                agent: Agent::Garbage,
                requests: 0,
                expected: vec![ListOutcome::Failed(ListFailure::Agent)],
                ..Row::ordinary("")
            },
            Row {
                name: "oversized stdout",
                agent: Agent::Oversized,
                requests: 0,
                expected: vec![ListOutcome::Failed(ListFailure::Agent)],
                ..Row::ordinary("")
            },
            Row {
                name: "trailing stdout",
                agent: Agent::Trailing,
                requests: 0,
                expected: vec![ListOutcome::Failed(ListFailure::Agent)],
                ..Row::ordinary("")
            },
            Row {
                name: "failure exit with readable stdout",
                agent: Agent::Failed,
                requests: 0,
                expected: vec![ListOutcome::Failed(ListFailure::Agent)],
                ..Row::ordinary("")
            },
            Row {
                name: "signal exit with readable stdout",
                agent: Agent::Signalled,
                requests: 0,
                expected: vec![ListOutcome::Failed(ListFailure::Agent)],
                ..Row::ordinary("")
            },
            Row {
                name: "deadline kills a hanging agent",
                agent: Agent::Hang,
                requests: 0,
                expected: vec![ListOutcome::Failed(ListFailure::Deadline)],
                ..Row::ordinary("")
            },
            Row {
                name: "cancel settles without an answer",
                agent: Agent::Hang,
                cancel: Some(Duration::from_millis(1)),
                requests: 0,
                expected: vec![],
                ..Row::ordinary("")
            },
            Row {
                name: "cancel before spawn settles without an answer",
                agent: Agent::Hang,
                cancel: Some(Duration::ZERO),
                requests: 0,
                expected: vec![],
                ..Row::ordinary("")
            },
        ];
        for row in rows {
            let result = run(row.agent, row.cancel, row.status, row.body, row.unknown, row.short, row.peer);
            assert_eq!(result.answers, row.expected, "{} answers", row.name);
            assert_eq!(result.requests, row.requests, "{} provider requests", row.name);
            assert_eq!(result.cancelled, row.cancel.is_some(), "{} host cancellation", row.name);
            let replay = run(row.agent, row.cancel, row.status, row.body, row.unknown, row.short, row.peer);
            assert_eq!(result.trace, replay.trace, "{} seeded replay", row.name);
        }
    }
}
