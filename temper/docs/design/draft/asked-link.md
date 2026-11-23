# A signed-in person asks for a browser's sign-in link

Draft, 2026-11-17, on the user's word. A deployment run as a service prints its sign-in links on its output, which its operator keeps. Once the links it printed have signed in command-line clients, a browser that should read the deployment as its person has no supported way in: the operator restarts the engine for a fresh link.

## 1. What happens today

- **The printed link** is drawn at start and spent by a sign-in. A sign-in tried with a secret that isn't live, spent, expired or unknown, is refused, and draws a fresh printed link, which the startup prints again (jig's `jig/docs/design/protocol/client/sign-in.md`, the printed link; `jig/crates/jig-protocol-engine-clients/src/component.rs`, `fresh_link`; `temper/crates/temper-engine-service-component/src/tests/clients.rs`, `a_spent_given_secret_prints_a_fresh_link`).
- **So a fresh link exists only on the engine's output,** as a side effect of a refused sign-in. ctl's refusals name `temper-box serve` alone as the way to a fresh one (`temper/docs/design/client/ctl/command-line.md`, 1.1). No client can ask for one, an agent driving `temper ctl` among them.
- **The local link** goes to the person's native clients on the machine, where the startup keeps it; a deployment's engine keeps none (jig's `protocol/client/sign-in.md`, *The local link*).
- **Approval runs the other way:** a browser signed in approves a terminal client's sign-in, and a program holding a person's credential never approves (jig's `protocol/client/sign-in.md`, a sign-in approved by a browser).

## 2. Wanted

- **`temper ctl DIRECTORY link --out FILE`:** a signed-in person's client asks for a fresh one-time sign-in link, and ctl writes it to FILE, `0o600`, refused if the file exists, never printed and never an argument, as `party make` writes a service's credential (`ctl/README.md`, 7). The link has the printed link's form, as serve prints it, so a browser on the machine opens it as it opens the printed link, and `sign-in` reads it too.
- **jig's clients' component keeps an asked link** beside the printed and local links:
  - asked with a person's credential, checked as any exchange's; a service's credential, or none, is refused;
  - enabled only where the printed link is, on a loopback listener (jig's `jig/docs/design/protocol/service.md`, 3);
  - it signs in the asking sign-in's own party, never the first owner's or another;
  - the sign-in it makes expires no later than the one that asked, so a credential can't outlive itself through links;
  - one-time, live for `link_span`, kept as its digest in memory alone, and a restart forgets it; each sign-in has at most one live asked link, and asking again replaces it;
  - its answers are the printed link's: `link_used`, `link_expired` and `link_unknown`, remembered as the printed link's are; not ready, busy and full spend nothing.
- **The client's wire moves its version,** as `temper/docs/design/draft/redeploys.md` says of a client's wire.
- **ctl's refusals of a spent or expired link** name both ways to a fresh one: `temper-box serve` prints one, and a signed-in client asks one with `temper ctl DIRECTORY link --out FILE`.
- Revise jig's `protocol/client/sign-in.md` and its client contract's built-ins (`jig/docs/design/client/domain/contract/README.md`, 2), temper's `../client/ctl/command-line.md`, 1.1, and `../client/ctl/README.md`, 7, and `temper/docs/design/protocol/sign-in.md`, 2.3, with their status files.

## 3. Out

- Approval, which stays a browser's to give.
- Sign-ins between hosts, over TLS.
- Google's sign-in.

## 4. Tests

Cheap: focused and fuzzy suites only; walk = false.
- jig's engine clients suite (`jig/tests/engine-clients/tests/suite/sign_in.rs`):
  - a person's asked link signs a browser in once, as the asker's party, its expiry no later than the asker's;
  - a service's ask, and one with no credential, are refused;
  - asking again replaces the live one;
  - spent, expired and unknown are answered so;
  - none off loopback;
  - a restart forgets it.
- jig's client wire fuzzy suite (`jig/tests/client-wire/tests/suite/fuzzy/client_wire.rs`): the new records round-trip.
- ctl's tests (`temper/crates/temper-client-shell-cli/src/tests.rs`): `link --out` writes the file `0o600` and prints no link; an existing file and a signed-out DIRECTORY are refused; the refusals name both ways.

Run only the new and affected tests, by name.
