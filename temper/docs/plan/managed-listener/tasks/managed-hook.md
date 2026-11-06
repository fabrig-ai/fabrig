+++
id = "managed-hook"
part = "listener"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-service/src/setup/runtime.rs",
  "temper/crates/temper-service/src/setup/lifecycle.rs",
  "temper/crates/temper-service/src/setup.rs",
  "temper/crates/temper-service/src/tests/setup_admin.rs",
  "temper/crates/temper/tests/support/forgejo_elsewhere.rs",
  "temper/crates/temper/tests/support/forgejo.rs",
  "temper/docs/help/forgejo.md",
  "temper/crates/temper-docs/src/tests/rules.rs",
  "temper/docs/design/protocol/setup.status.md",
  "temper/docs/design/STATUS.md",
]
effort = "high"
minutes = 90
+++

## Goal

A Forgejo connected as its admin whose API is on loopback, so on this host, is served the webhooks' listener: each adoption and each start's reconcile make its hook at the listener's URL, as an owned forge's. One elsewhere is served none and polls. A secret drawn on a forge already connected reaches its connection, so that hook has a path on the listener. `temper help forgejo` says what a person sets. A hook that can't deliver leaves polling working, and its failures reach no task.

Acceptance:

- A failing step test first, in `temper-service`: the report quotes the failure it saw before the change.
- `Mode::Managed` with its API on loopback: `Given.listener` is the webhooks' listener and `Local.webhooks` is true. On another host: `None` and false. Owned and fake are unchanged. With no `application.webhooks`, none has a listener.
- A managed adoption on loopback makes one hook at `http://127.0.0.1:7101/hooks/1` in the step test.
- A managed forge connected with no repository, then adopting, ends with its connection holding the drawn secret, the same bytes as its `WebhookSecret` record.
- The process story on a real Forgejo: one hook at the listener's URL on each adopted repository and on the first, and again on the adopted ones after a restart with it removed by hand.
- `temper/docs/help/forgejo.md` says it, and a test in `rules.rs` holds that.
- The status entries this closes are gone.

## Inputs

- `temper/docs/design/protocol/setup.md`, 5.3 (the listener's row), 5.4 (a secret drawn on a connected forge), 5.5 (its listener), 5.6 (what it cannot set), 10, 13.4. These were written by this cut and bind.
- `skein/docs/design/forge-setup.md`, 4.5; `skein/docs/design/forge-connection.md`, 3.5 (a replacement) and 6.1 (a forge with no secret has no path).
- `temper/docs/plan/managed-listener/inputs/request.md`, the request.
- No test reaches a real host. The story's Forgejo is the throwaway on loopback; the elsewhere step test does no io to its documentation address (`192.0.2.9`).

## Owns and may change

Owned, the `files` above. **Departures from the implementer's text,** because the request has them in this one landing: you edit `temper/docs/help/forgejo.md`, and in `temper/docs/design/protocol/setup.status.md` and `temper/docs/design/STATUS.md` you remove what you close. All other `docs/design/` stays closed: what the code teaches goes in the report.

Readers and neighbours, and how they may change:

- `temper/crates/temper-service/src/tests/forge_runtime.rs`: only if you place the elsewhere test there (below).
- Any other `temper-service` test that pinned a managed forge's having no hook with a listener configured: turned round to the design; list each in the report. `setup_create.rs`, `setup_adoption.rs` and `setup_bound_webhook.rs` are owned forges' and should not change.
- `setup_admin.rs:215`, `assert!(forge.service.setup().repository_hooks(1).next().is_none())`, stays: `config()` there has `webhooks: None`, so no listener and no hook is still right. The request's check that turns round is `a_managed_adoption_grants_write_on_a_private_repository_without_a_hook`, whose `access_host(false)` configures one.
- `temper/crates/temper/tests/process.rs` stays: the story keeps its name. It is another draft's file.

Options, each pre-approved:

- If `setup_admin.rs` can't build a service with a TLS forge, write the elsewhere test in `forge_runtime.rs`, with `service.configure_forge_runtime` and `route::request(.., root::Request::ForgeKept(record))` as `a_kept_github_uses_the_machine_endpoint_and_the_records_tls_name` does, reading `engine.setup.served`.
- If the step harness shows no fresh read within a run past the keep's `poll` and `slow` in the polling test (below), drop that one assertion, keep the rest, and name `a_watched_pulls_ci_at_the_same_head_wakes_its_change` (`temper/tests/engine/forge/tests/suite/simulation.rs`, whose `World::new` hears no hook) as polling's check at the lowest tier: run it, and say so in the report.
- If the story's listener socket reads port 0, take the hook's URL from the story's own listening note rather than from `temper.json`, and say so.
- If the real Forgejo's hook JSON holds the URL elsewhere than `config.url`, read it there; `skein/tests/forge/conformance` records the shape.

## Steps

### 1. The turned tests, failing

In `temper/crates/temper-service/src/tests/setup_admin.rs`:

- `string_managed_serves_no_listener_even_when_owned_forges_have_webhooks` becomes `a_managed_forge_on_this_host_is_served_the_listener`: `given.listener == Some(127.0.0.1:7101)`, `lifecycle.local(1).unwrap().webhooks`, orgs and org owner still none, not waiting for webhooks.
- `a_managed_adoption_grants_write_on_a_private_repository_without_a_hook` becomes `a_managed_adoption_on_this_host_grants_write_and_makes_one_hook_at_the_listener`, with `an_owned_adoption_makes_one_hook_at_the_listener`'s assertions (`setup_adoption.rs`).
- New `a_managed_forge_elsewhere_is_served_no_listener`: a `crate::ForgeRuntime` at `([192, 0, 2, 9], 443)` with `Transport::Tls`, built as `forge_runtime.rs` builds it, the same `config.webhooks`: `given.listener` none, `local.webhooks` false.
- New `a_secret_drawn_at_a_first_adoption_reaches_the_connection`: `access_host(false)`, no `WebhookSecret` row and the last of `engine.forge.starts` (`#[cfg(test)]`) holding none; adopt and finish; the last of `starts` holds a secret equal to the `WebhookSecret` record, and the forge is `Connected`.
- New `a_managed_forge_whose_hook_delivers_nothing_stays_connected_and_keeps_reading`: `access_host(false)`, whose fake has `hooks: None`, so it never delivers; adopt and finish; then run past the keep's paces with larger ticks. The hook is there; `forge_connection(1)` stays `Connected`; the row's `ended` and `ended_stop` are none; the fake's request lines show the forge's token reading `other/closed` again after the hook was made, and none names `/tasks` under `/hooks/`.

Run them; see each fail for the rule's reason.

### 2. `runtime.rs`

```rust
/// The listener a forge's plans are given: a managed forge's only with its API on loopback
/// (temper/docs/design/protocol/setup.md, 5.5).
fn listener(&self, mode: domain::Mode, address: skein_io::Addr) -> Option<core::net::SocketAddr> {
    match mode {
        domain::Mode::Managed => match address {
            skein_io::Addr::Ip(api) if api.ip().is_loopback() => self.webhooks,
            skein_io::Addr::Ip(_) | skein_io::Addr::InProcess(_) => None,
        },
        domain::Mode::Owned | domain::Mode::Fake | domain::Mode::Connected => self.webhooks,
    }
}
```

`forge` gives `temper_forge_setup::given::values` `self.listener(record.mode, address)`. `local` takes the forge's address and sets `webhooks: self.listener(record.mode, address).is_some()`; `keep` passes `forge.address`.

### 3. The secret reaches the connection

`temper/crates/temper-service/src/setup/lifecycle.rs` gains `Purpose::HostSecret(Token)`; the `host::Need::Write` arm in `setup.rs` keeps an `Item::WebhookSecret` with it, every other item with `Purpose::Host`. In `kept`:

```rust
Purpose::HostSecret(run) => {
    let forge = engine.setup.runs.get(&run).expect("a plan waits on its secret").forge;
    if connected_in_root(&engine.domain, forge) {
        hand_secret(engine, forge);
    }
    engine.setup.down(host::Request::Answer { run, answer: host::Answer::Written });
}
```

`hand_secret` reads the forge's rows, calls `records::access` with `local.webhooks` as `connect_local` does, and hands the result to `crate::forge::connect`. It leaves the identity alone, since the git token has not changed; that is why it is not `connect_local` itself. A forge not yet connected takes the secret at the connect that ends its run, as it does now. `HostSecret` joins `Host` in `kept`'s first match.

### 4. The process story

`temper/crates/temper/tests/support/forgejo_elsewhere.rs`:

- The no-hook assertion and its comment (`setup.md, 5.5: managed forges poll`) become: exactly one hook, at `http://<socket>/hooks/1`, `<socket>` read from `config/temper.json`'s `/application/webhooks/listener/socket`, on `other/open`, `other/closed` and `elsewhere/first`.
- Beside `forge.remove_adoption_collaborator(name)`, a new `forge.remove_adoption_hooks(name)` (`temper/crates/temper/tests/support/forgejo.rs`, beside it, each answer `204`).
- After the restart, beside its collaborator check, the same one hook on `other/open` and `other/closed`.
- `configure`'s message becomes `listener configured for the hooks' URL`.

Delivery is not this story's to prove: ops proves it at its next deployment.

### 5. The help

`temper/docs/help/forgejo.md`:

- In **The mode**: `... and at each adoption the bot's access and, on a Forgejo on this host, the repository's hook.`
- After the `ADDRESS` bullet:

  ```
  - **Its hooks,** on a Forgejo on this host, whose `ADDRESS` is on
    loopback: in its `app.ini`, `[webhook] ALLOWED_HOST_LIST` names the
    IP address of temper's `application.webhooks.listener`; restart that
    Forgejo after. Its default refuses loopback, and temper can't change
    it. Without it no hook delivers: temper finds each change by polling,
    later, and nothing fails. A Forgejo elsewhere has no hook, and polls.
  ```

In `temper/crates/temper-docs/src/tests/rules.rs`, new `forgejo_help_says_what_a_hook_on_this_host_needs`, in the manner of `box_help_locates_forgejo_lifecycle_and_hands`: the page, whitespace joined, holds `ALLOWED_HOST_LIST`, `application.webhooks.listener` and `by polling`.

### 6. The status

In `temper/docs/design/protocol/setup.status.md` remove this plan's first **Not yet** entries (5.3, 5.5, 10; 5.4; 5.6), and the 13.4 one that opens `on a real Forgejo no story shows`. In `temper/docs/design/STATUS.md` remove the clause `a Forgejo connected as its admin on this host polls ... making a hook (`protocol/setup.md`);`. Run `python3 -I -B tools/citations/citations.py --base origin/main`.

### 7. Checks

This changes what a managed forge is given, so its readers' tests run:

- `heavy cargo nextest run -p temper-service --lib`
- `heavy cargo nextest run -p temper-docs`
- `heavy cargo nextest run -p temper --test suite`
- `heavy cargo nextest run -p temper-one-process-world`
- `heavy cargo nextest run --profile process-temper -p temper --test process -E 'test(=a_forgejo_elsewhere_connected_as_its_admin_makes_the_first_repository_lands_a_chats_change_and_is_found_after_a_restart)'`
- `heavy cargo nextest run --profile process-temper -p temper --test process -E 'test(/fake_backends::walk::(goal|copy|status|away)/)'`: these walks connect a Forgejo as its admin on the fake, whose hooks deliver nothing (`temper-fake/src/forge.rs`, `hooks: None`), so a managed goal here lands on polling with a hook made. Name in the report what it showed.

Then `cargo fmt`, `python3 -I -B temper/scripts/old_forms.py`, and `heavy cargo clippy -p temper-service -p temper-docs -p temper --all-targets -- -D warnings`. Commit and push.

A new test must bite: for the listener, the secret's hand and the help test, break the rule once and name the failure.

## Report

The failures seen first; each test turned round or added; what the fake-backed walk showed; whether the story's hook delivered on the throwaway Forgejo, if its log shows it; the test budget if any neared it; anything left outside under `## Left`.
