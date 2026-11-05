# runner-labels: a re-prepare doesn't strand a runner on another host

One task in temper. Prepare can leave a declared runner's row alone, so this takes the request's first branch and `temper help server` doesn't change.

## Parts and tasks

| Part | Task | Needs | What |
|---|---|---|---|
| `prepare-row` | `prepare-row` | | register only where the row may not exist; otherwise render the configuration from the `uuid` it already names |

Critical path: `prepare-row`.

## What was decided

- **Prepare can leave the row alone (`temper/docs/design/protocol/setup.md`, 6 and 7).** The row needs writing once. The runner declares over it at each start, which conformance already shows: a runner registered as `kit-host:host` takes a `runs-on: kit-host` job (`runner-connection-alone-takes-the-jobs`). Rewriting it at a later preparation gains nothing a start doesn't do. A box that restarts Forgejo while its runner runs (6.3, item 2) strands that runner the same way, so the fault wasn't only on another host.
- **The test for "the row may not exist" is the configuration.** Prepare can't read the row: it uses Forgejo's command line only, with no Forgejo running, and that command line lists no runners. `runner/config.yaml` is written only after `register` has answered, so a `uuid` in it means the row was made.
- **The `uuid` comes from the configuration, not from the secret.** Deriving it from `runner/secret` would also work (`runner-uuid-is-the-secrets-first-sixteen-characters-in-hex`), but it would make a later preparation read the secret and keep a second copy of Forgejo's rule in temper.
- **The rule covers every course, not just the identical configuration in the report.** A capacity edit, a Forgejo upgrade and a runner upgrade all rewrite the row today, and a runner on another host holds no lock here to refuse them. Changed labels reach the row when the runner starts on its new configuration, as a first preparation's do.
- **A runner added with its secret and configuration kept doesn't register.** Its row was left in place (6.3, item 4). If that preparation was cut short before the configuration was written, there is no `uuid`, so it registers.

## What it costs

- **A row a person removes is no longer restored by the next start.** Prepare can't notice the removal, and restoring it silently is the same write that strands a declared runner. The way back is to remove `runner/config.yaml` before a start. 6.3 says so; no help page does, since none covers removing a runner.
- **Registering again still writes the raw row** on the paths that keep `register`. A runner elsewhere that is still running through one of those needs a restart, as it does today.

## The tests

- **Cheapest tier:** the shell's story runs `prepare::run` against a scripted Forgejo whose `actions register` writes a `row` file, and the machine's step tests pin each course. Nothing reaches a real host.
- **No conformance change:** the Forgejo behaviour this relies on is already checked (13.3). This cut only names it there.
- **No help change:** no page says a later preparation registers. So the request's item 3 doesn't apply, and `docs/help/` is left alone.

## Overlaps

- **`temper/docs/design/protocol/setup.status.md`, with `adopt-access/adopt-grant`, unavoidable:** both edit this file. This cut's entry is its own last bullet, away from the **Not yet** line that other draft would change, so that git can merge them.
- **Otherwise none:** no draft beside this one lists `temper-forge-setup`, `temper-forge-setup-shell` or `setup.md`.
