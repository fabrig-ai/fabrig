use jig_channel_world::list_read;
use jig_domain_link::Scheme;
use jig_protocol_worker_wire as wire;

#[test]
fn a_worker_reads_a_list_with_its_grant_and_releases_the_value() {
    for (label, status, scheme, endpoint, retained) in [
        ("listed with key", None, Scheme::Key, 1, 0),
        ("second endpoint", None, Scheme::Key, 2, 0),
        ("refused with key", Some(403), Scheme::Key, 1, 0),
        ("bearer until expiry", None, Scheme::Bearer, 1, 1),
    ] {
        let run = list_read::run(status, scheme, endpoint);
        assert_eq!(run.answers.len(), 1, "{label}: one terminal");
        match run.answers[0].outcome() {
            wire::ListOutcome::Listed(rows) => {
                assert_eq!(status, None, "{label}");
                assert_eq!(
                    rows.rows().iter().map(wire::ListRow::name).collect::<Vec<_>>(),
                    [b"first-model".as_slice(), b"second-model".as_slice()],
                    "{label}: fake rows cross both halves"
                );
                let first = rows.rows().get(0).expect("first listed row");
                assert_eq!(
                    first.shown().as_deref(),
                    (endpoint == 2).then_some(b"First model".as_slice()),
                    "{label}: shown"
                );
                assert_eq!(*first.window(), (endpoint == 2).then_some(200_000), "{label}: window");
                assert_eq!(*first.output(), (endpoint == 2).then_some(8192), "{label}: output");
                assert_eq!(
                    first.efforts().iter().map(AsRef::as_ref).collect::<Vec<_>>(),
                    if endpoint == 2 { vec![b"low".as_slice(), b"high".as_slice()] } else { vec![] },
                    "{label}: efforts"
                );
            }
            wire::ListOutcome::Failed(failure) => assert!(
                matches!(failure.value(), wire::ListFailed::Refused(value) if Some(value.status()) == status),
                "{label}: typed refusal: {failure:?}"
            ),
        }
        assert_eq!(
            run.requests,
            [(
                if endpoint == 1 { skein_llm::Provider::OpenAi } else { skein_llm::Provider::Anthropic },
                Some(match scheme {
                    Scheme::Key => skein_llm::Scheme::Key,
                    Scheme::Bearer => skein_llm::Scheme::Bearer,
                }),
                Some(0)
            )],
            "{label}: exactly one list request on the endpoint route with its own credential"
        );
        assert_eq!(run.values_after, Some(retained), "{label}: credential lifetime after read");
        assert_eq!(run.values_expired, Some(0), "{label}: no expired values remain");
    }
}
