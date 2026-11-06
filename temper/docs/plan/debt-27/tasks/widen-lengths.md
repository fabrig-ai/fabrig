+++
id = "widen-lengths"
part = "widen-lengths"
kind = "migration"
needs = []
files = ["jig/crates/**", "jig/testing/**", "jig/tests/**", "jig/examples/**", "jig/scripts/old-forms.txt", "jig/docs/development/debt/D043-length-widening.md"]
effort = "medium"
minutes = 70
+++

# jig widens a native length with `len_u64`

## Goal

Pay jig's D043 as far as this plan may reach. A mechanical sweep of every hand-written `u64::try_from(<a length>)` in jig onto `skein_lib::length::len_u64`, each refusal path that can't be taken going with it. Two sites stay: both are in `jig-client-view`'s tests, a crate this plan stays out of, so the item stays open and names them.

Acceptance:

- The measure reads 2, both under `jig/crates/jig-client-view/src/tests/tree/fragments/`: `rg -c --type rust -g '!**/generated/**' 'u64::try_from\([^)]*\.len\(\)\)' jig | awk -F: '{s+=$2} END {print s}'`.
- The measure misses nested calls, as in `u64::try_from(frames::frame(6, &expected).len())`. So outside `jig-client-view` this wider search also finds nothing: `rg --type rust -g '!**/generated/**' -g '!jig/crates/jig-client-view/**' 'u64::try_from\(.*\.len\(\)' jig`. The cut counted 240 such lines before the sweep.
- `git diff --stat` shows nothing under `jig/crates/jig-client-view/`, and no `generated/` file.
- `jig/scripts/old-forms.txt` falls, by `python3 -I -B jig/scripts/old_forms.py --write`.
- `jig/docs/development/debt/D043-length-widening.md` stays, status `open`. Its **Where** is the two files, its **What** and **Measure** say 2, and a sentence says this plan left the crate alone for fabrig/ui. **Paid when** doesn't change.

## Inputs

- `jig/docs/development/debt/D043-length-widening.md`, whole.
- `skein/docs/design/lib.md`, 6, the widening bullet.
- `temper/docs/plan/debt-27/notes/common.md`.

## Owns and may change

Owns those sites and no others: `u64::try_from(index)` and other widenings that aren't a length stay as they are. The crates are the cut's count: `jig-domain-core` 45, `jig-domain-people` 39, `jig-protocol-store` 34, `jig-client-protocol-link` 14, `jig-protocol-engine-clients` 8, `jig-domain-tasks` 8, `jig-domain-host` 6, `jig-domain-fleet` 6, the rest under 4 each; `jig/tests` about 30, `jig/testing` and `jig/examples/ops` about 20.

| Today | Becomes |
|---|---|
| `u64::try_from(x.len()).expect("...")`, `.unwrap()` (about 115) | `len_u64(x.len())` |
| `u64::try_from(x.len()).ok()?` (about 60) | `len_u64(x.len())` |
| `.or(Err(E))?`, `.map_err(..)?` | `len_u64(x.len())` |
| `.ok()` feeding an `Option` chain | `Some(len_u64(x.len()))`, or the chain without its first step |
| `.ok().ok_or(failure.clone())` | `len_u64(x.len())`, and the `clone` goes if nothing else reads it |

Options, each pre-approved:

- Take `len_u64` however the file already reaches `skein_lib`: a `use skein_lib::length::len_u64;`, or the path. A test package whose manifest lacks `skein-lib` gets the one dependency line.
- A throwaway codemod, untracked and deleted before the push, is fine. Read the diff by `git diff --stat`, then spot-check the forms in the table's last three rows.
- A function left returning `Option` or `Result` with no way left to `None` or `Err`: if it's private and has few callers, drop the wrapper and fix those callers; otherwise keep the signature, under a scoped `#[expect(clippy::unnecessary_wraps, reason = "...")]` if clippy asks.
- An error variant the sweep leaves unbuilt: leave it, and report it under `## Left`. A public or stored shape doesn't change here.

**Departure from the implementer's text:** a sweep with no change of meaning doesn't run dependents' tests. Run each changed package's own, and `heavy cargo check --tests` on a dependent only if you changed a signature it reads.

## Steps

Crate by crate, each with its tests: `heavy cargo nextest run -p PKG`. `cargo fmt` once at the end; the old forms; `heavy cargo clippy` naming each changed package, `--all-targets -- -D warnings`, in one command or a few; `python3 -I -B tools/citations/citations.py --all`; `python3 -I -B tools/debt/debt.py`. Commit crate by crate if that's easier to read. Push.

A failing test is a stop: this sweep moves no outcome. Read its log, and ask if the sweep is what moved it.

## Report

As your text says: both searches' readings, the old-forms count before and after, the net lines, and any signature you changed.
