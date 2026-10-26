# Supplement skein-01.s4: no Close after Closed in the browser harness's HTTP client

Added by ruling 173, from skein-01.s3's gate of 2026-10-16T20:37Z
(`logs/20261016T203701-599.log`, :2724-:2746): the browser profile's
`skein-ui-web-tests::browser_harness
a_request_with_another_host_has_its_status_awaited`, skein-01.3's test
(`b23380a`), panicked at `crates/skein-http/src/client.rs:899`,
`unreachable!("a Close after Closed")`, in `close`; the test had passed
in the run's earlier gates. Read first: skein's `AGENTS.md`, its gate
`scripts/check.sh`; this plan's README, "Rules of this plan".

Scope: the cause, in skein's code (skein-http's client, or the harness
of skein-01.2 and 01.3): a defect of this plan's skein 01, not of the
design. A second `Close` either cannot reach `close` once `Closed`, or
is answered as the design of skein-http says for it; quote that text in
the report. Needs: nothing.
Files: those of the cause, named in the report; at most 2 crates.
Test shape: one test that drives the order which reached `close` in
`Closed`, without a browser and without sleeping (a unit test of
`client.rs`'s states, or of the harness's), failing before the fix.
Proof: skein's gate; and once, before landing, the browser
test 20 times: `for i in $(seq 20); do cargo nextest run
--profile browser -p skein-ui-web-tests --test browser_harness
a_request_with_another_host_has_its_status_awaited || break; done`,
each a pass. Nothing else is wanted.
Don't: replace the `unreachable!` with a silent arm without the cause
named; retry or skip the test; name a downstream repository.
Size: at most 60 lines. Past 120, or a cause in the design, a question.
Done when: `git -C ~/src/rust/skein log origin/main --all-match
--grep='^Plan: client-api$' --grep='^Step: skein-01.s4$' --format=%H -1`
prints its landing.
