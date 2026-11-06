# A box's repository link: proof

2026-11-06. Implementation and browser evidence for
`temper/docs/design/protocol/sign-in.md`, 9.9, and
`temper/docs/design/client/forge.md`, 5.4. This is not the separate security review.

## The real boundary

The walk uses a fresh `temper init --box`, temper's own issuer and generated
key, real Forgejo 17.0.1, and the shipped browser client in a debug bundle.
Chromium 156 is driven through CDP, with device metrics set to 1440 or 420
pixels. The runner is disabled; no LLM or external sign-in provider is involved.
The client setup act is driven by `temper ctl forge setup`, including the first
repository. Browser clicks use CDP mouse input and verify the tab Chromium opens.
For complete redirect and timing capture, the driver then follows the same client
URL in an explicitly initialized target in the same profile: automatic popup
attachment can miss the first request, and pausing new targets can freeze them.
The timing measurements are these URL replays, not measurements of the native popup.

At 1440 the repository link is in the switcher. At 420 the switcher has project
names alone: the `repo-link` task explicitly preserves this below 760 pixels.
The phone walk instead clicks the same helper's link in Settings' Repositories.
It does not demonstrate a repository click in the phone switcher.

Each successful visit follows Forgejo's 307 to `/oauth/authorize`, the client's
address replacement to the callback, and Forgejo's 303 to the repository: two
HTTP redirects and one client navigation. The private profile page names the
box's person. The trace contains no Link account page. The second click repeats
the authorization in the same cookie jar. The walk repeats after stopping and
starting the box, waiting for its restored source fact before fetching the page.

The project name switches between two real projects without opening a tab.
Forges offers "Open Forgejo ↗" at both widths. The account observer compares
only public identity columns, including the organization row, before and after;
the external login's subject is `1:ADDRESS`. The source list contains one
`temper` source after restart. A fresh browser profile verifies Forgejo's own
password login; its password stays in memory and is never printed.

## Continuation timing

The following milliseconds are from the instrumented URL replays. Each width
pair uses a fresh browser profile; the pair itself keeps one cookie jar. A
MutationObserver marks the heading in the DOM, a requestAnimationFrame marks
its first render opportunity, and the new document's lifecycle `init` marks
the end. The render interval is an estimate, not a screenshot at every frame.
The authorize column is the HTTP exchange, ending when its body finishes.

| Box | Width | Click | DOM interval | Render interval | Authorize HTTP | Reply to callback request |
|---|---:|---:|---:|---:|---:|---:|
| fresh | 1440 | 1 | 157.8 | 155.6 | 8.3 | 8.3 |
| fresh | 1440 | 2 | 158.5 | 156.1 | 3.0 | 21.6 |
| fresh | 420 | 1 | 148.5 | 145.7 | 5.6 | 18.1 |
| fresh | 420 | 2 | 188.9 | 185.9 | 8.0 | 18.9 |
| restart | 1440 | 1 | 192.7 | 190.3 | 5.2 | 26.1 |
| restart | 1440 | 2 | 181.1 | 178.7 | 4.4 | 22.7 |
| restart | 420 | 1 | 181.4 | 178.1 | 5.6 | 12.0 |
| restart | 420 | 2 | 169.8 | 165.6 | 10.7 | 12.7 |

The visible interval includes client startup, signing in by transport, and
the next document loading; it exceeds the bare authorize HTTP time. This
walk therefore does not prove a literal visible-time-at-most-RPC-time bound.
The implementation has no minimum display timer: the domain emits `Leave`
in the step handling `Go`, and the web host replaces the address in that turn.
The focused tests `every_answer_has_its_page_and_only_go_replaces_the_address`
and `authorization_go_replaces_the_address_once` hold that behavior.

During longer exploratory runs, an authorization page sometimes drew the
temporary busy message without sending an authorize HTTP request. Fresh
profile pairs completed the walk. The local-client admission/loss path
remains an investigation for the client; this proof does not claim it is fixed.

## What the laptop's table establishes here

| Laptop row | This proof |
|---|---|
| `login` asks to link the account | Not repeated: the box writes `auto`; no baseline run changes it to `login`. |
| `auto`, matching name, different address | Not repeated with different addresses: this box's issuer and Forgejo person have both the same login and address. The real-box run proves that ordinary case without another account. |
| `auto`, no `email` claim | Not reachable through the box's document. `component::tests::provider_claims::empty_addresses_emit_no_email_claim` in `jig-protocol-engine-clients` holds the empty-addresses case at the step boundary, including the configured-provider positive case. It does not repeat Forgejo's no-email landing. |
| A second visit passes the provider again | Repeated in the same browser jar at both widths, before and after restart. |

## Source failure

A separate fresh box uses a Forgejo wrapper that refuses `auth`. Its setup and
first repository still settle. The page has no sign-in meta, has the forge's off
meta, and Forges explains the password fallback. The box remains running and
its log contains exactly one "no sign-in through temper" line. Both widths
have screenshots of this state.

## Restart rules found by the walk

The issuer serves before restoring the owned group, because Forgejo loads its
discovery document at startup. Inserting first steps preserves that offset.
The restored group waits for its final source outcome before reporting started:
a successful Forgejo readiness probe cannot report a source that has not spawned.
A restored source fact restores the kept box-owned Forgejo's page meta without
requiring another setup ask. Unknown and managed forges cannot replace that fact.
These rules have focused regressions, including mutations that fail on the rule.

The process story
`a_box_signs_its_person_in_to_its_owned_forgejo_with_one_click` is written and
compiled. It covers the real Forgejo, cookie jar, API account comparison,
external subject, repeated visits, password login, restart, source count,
page facts and source failure. It is not run during the plan: the process tier
and its CPU budget remain for after landing.

## What remains for the security review

Claude's separate review reads part 2 of the request and
`temper/docs/design/protocol/sign-in.md`, 9.9. It must assess account linking
and excluded identities; public-client redemption, exact redirects and PKCE;
the printed link, strict cookie and browser-origin boundaries; another local
process or browser tab; key and password file handling; source failure and
restored-source fact admission. This walk supplies observations, not a security
approval. Managed deployments, multiple people, Google, remote hosts, production
TLS and the two unrepeated laptop configurations are outside the real-box proof.
