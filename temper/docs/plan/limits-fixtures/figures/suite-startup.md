# suite-startup: figures

Base `76453cdc2bffd377992317f45c083aab001743ff`, machine
`debian-32gb-hel1-11` through the build wrapper, 2026-11-07 UTC.
Before measurements use the untouched base. After measurements use the final
product and test sources committed with this file. No build settings changed.

The initial focused run passed all 89 `startup::` and 31 `forge::` tests.
Sorting each module's elapsed results by time and then name selected its first
and middle test. Times select the names; they are not performance claims.
The four names stay fixed below. `SF`, `SM`, `FF`, and `FM` identify those rows.

Every measurement runs one test through nextest's default-profile tracer with
double spawning disabled, `TEMPER_BUNDLE=none`, and
`valgrind --tool=callgrind --trace-children=yes`. Both inclusive and self
annotations were read. Counts include loader, harness, setup, assertions and
teardown, not just work before the first assertion. Each total sums the
process files listed below; the `temper` column includes its loader too.

| Row | Test | Floor or median | Ir before | Ir after | What was built before the first assertion |
|---|---|---|---:|---:|---|
| startup, SF | `startup::a_rung_reads_created_chats_waited_changes_and_both_child_sets_in_ctls_forms` | Floor | 12,484,190 | 12,489,183 | A serde JSON chat-creation answer, then its extracted chat number. Subsequent assertions build other small CLI-answer values. No scratch, deployment, init or `temper` process. |
| startup, SM | `startup::forge_run_refuses_a_listener_at_the_wrong_address_before_taking_the_lock` | Median | 79,022,736 | 72,479,448 | Private scratch, a held loopback listener, agent/deployment files with the owned machine, and `temper forge run` through a shell that gives it fd 3. The command validates configuration and refuses the mismatched address before opening a forge lock. No init or backend program runs. |
| forge, FF | `forge::the_run_names_prepare_for_a_runner_removed` | Floor | 83,075,762 | 76,448,197 | Private scratch, two scripted backend executables, agent/deployment/group files, prepared state with a recorded runner, and deployment JSON edited to remove that runner. `temper forge run` validates configuration and refuses the recorded/configured runner mismatch. Neither script executes; no init runs. |
| forge, FM | `forge::a_forge_directory_open_to_others_is_refused_naming_it` | Median | 81,524,432 | 74,478,434 | Private scratch, two scripts, agent/deployment/group files and prepared state, then forge-directory permissions changed to 0755. `temper forge prepare` validates configuration and refuses the directory before executing a backend. No init runs. |

| Row | `cmd:` (paths normalized; SCRATCH is that test's own directory) | Ir before | Ir after |
|---|---|---:|---:|
| SF | `suite --exact startup::a_rung_reads_created_chats_waited_changes_and_both_child_sets_in_ctls_forms --nocapture` | 12,484,190 | 12,489,183 |
| SM | `suite --exact startup::forge_run_refuses_a_listener_at_the_wrong_address_before_taking_the_lock --nocapture` | 33,156,367 | 33,150,547 |
| SM | `temper forge run --config SCRATCH --data SCRATCH --cache SCRATCH 1 forgejo` | 45,866,369 | 39,328,901 |
| FF | `suite --exact forge::the_run_names_prepare_for_a_runner_removed --nocapture` | 36,231,218 | 36,236,558 |
| FF | `temper forge run --config SCRATCH --data SCRATCH --cache SCRATCH 1 forgejo` | 46,844,544 | 40,211,639 |
| FM | `suite --exact forge::a_forge_directory_open_to_others_is_refused_naming_it --nocapture` | 34,265,574 | 34,306,372 |
| FM | `temper forge prepare --config SCRATCH --data SCRATCH --cache SCRATCH 1` | 47,258,858 | 40,172,062 |

SF has zero `temper` instructions. The `temper` shares before → after are
58.04% → 54.26% (SM), 56.39% → 52.60% (FF), and 57.97% → 53.94% (FM).
The shell in SM execs `temper` in the same process; its final `%p` file names
`temper`. Callgrind replaces the pre-exec image's profile, so the shell's
pre-exec instructions are unobserved, not included as an additional process.
All reported sums follow the per-process `%p` method; they are not a complete
accounting of intermediate executable images.

Kernel share is **unknown for every row**: Callgrind counts no kernel work,
and `strace` is absent on this builder. In particular, these figures do not
measure exec, page faults, directory operations or file I/O in the kernel.
No strace samples were possible. `temper_init::files` is not reached in any
selected row; init contributes zero instructions to these measurements.

The first SM trace failed only because Valgrind's diagnostics reached the
child's captured stderr and broke its exact-output assertion. After reading
that failure, the tracer gained a separate `--log-file=...%p.log`; the test
passed, and only that successful trace supplies the before count. No test
assertion or product output was changed for the profiler.

## Causes

Inclusive counts overlap and must not be added. The rule of thumb uses the
request's 5.3 + 2.1 CPU seconds and these instruction shares; it is an estimate,
not measured CPU savings, especially with kernel time unknown.

| Cause | Where | Ir per test, on which test | Fixed in | Or what fixing it would take |
|---|---|---:|---|---|
| Reparse the deployment solely to recover an already checked machine | `forge::entry`, `box_command::configuration_steps`, serve preparation | Separate machine read: 6,291,849 SM, 6,387,123 FF, 6,722,214 FM before; zero in these product paths after | `src/forge.rs`, `src/box_command.rs`, `src/lib.rs` | Use `Named.machine` from the existing complete startup check. Forge's entry function takes that checked value; serve borrows its GitHub settings; box consumes it. No validation, resolution or refusal is skipped. Applying the measured median reductions to both request rows estimates about 0.62 CPU s, above the approximately 0.3 s stop rule; this extrapolation is not an aggregate timing measurement. |
| Decode the machine again within setup validation | `setup::check_with` after `startup::named_with` | Setup falls 28,375,466 → 28,131,291 SM; 28,565,540 → 28,319,913 FF; 29,249,795 → 28,887,953 FM | `src/setup.rs` | Borrow `Named.machine` for the cross-listener and redirect checks. The old machine reader delegates to the same `read_config` as startup and ignores its limits argument. |
| Loader relocation before the test or command's first line | `_dl_relocate_object` | Suite: 11,659,809 → 11,664,909 each row; `temper`: 10,588,171 → 10,588,953 each child | Retained | About 93.4% of SF. Together, relocations are 28.15% of SM before and 30.70% after. Removing them requires loader/linker/build changes, explicitly outside this plan. The extra focused test modestly grows the suite executable's relocation floor. |
| Two remaining document parses with different consumers | `startup::named_with` and `setup::check_with` | `config::document`: 17,487,400 → 11,669,980 SM; 18,052,621 → 12,140,500 FF; 18,406,942 → 12,282,393 FM | Third parse removed; two retained | Startup parses to validate/name the complete deployment; setup parses to read and validate the demo. Sharing the parsed document would require a startup API returning or accepting that document while retaining its byte/token/depth bounds and diagnostic ordering. This is a remaining cause above the stop rule on a whole-row extrapolation, rather than an unclaimed saving or permission to skip the demo check. |
| Cold typed configuration and admission | `startup::engine_with`, application/worker parsing and cross-part checks | Engine alone: 9,581,352 → 9,581,142 SM; 9,581,211 → 9,581,048 FF; 9,580,702 → 9,580,713 FM | Retained | These commands must still diagnose invalid setup before reading files. A cheaper validated representation or parser/validator optimization is needed; deferring the whole check would change refusals. SF has none of this work. |
| Build the fixture's agent/deployment and measured turn limits | `support::configuration::documents`, `turn_bounds::turn_body`, owned configuration rewrites | `documents`: 12,603,611 SM before. `turn_body`: 8,639,827 → 8,638,001 SM; 8,637,077 → 8,637,108 FF; 8,637,944 → 8,637,952 FM. Owned setup including files/scripts: about 21 million FF/FM | Retained | The files and computed admission bounds are used by these tests. Removing this cost requires a cheaper equivalent turn-bound encoder or byte-identical template construction with runtime paths, preserving all limits. A process-wide cache does not remove its first construction. Shared fixture work belongs with `model-catalogue-2/process-fixtures`; no support lines changed here. |
| Script creation and process integration | `support::owned::scripted`, scratch and command helpers | Script-only wrapper work is about 19,400 FF/FM, outside its configuration call; kernel cost unknown | Retained; below stop rule for observed instructions | These are the owned-command integration fixtures. The selected refusals never execute the scripts; other forge tests exercise them. Avoiding the remaining file/exec kernel work would require measuring it first and preserving the process proof. |
| Small JSON-answer construction and assertions | SF's test body | 330,332 of 12,484,190 before | Retained | This is the behavior checked, with no shared deployment fixture. Charging its entire share to all 5.3 startup CPU seconds still gives about 0.14 CPU s, below the stop rule. |

Stop after the local checked-machine reuse. Whole-test savings are 8.28% SM,
7.98% FF and 8.64% FM; the `temper` process alone falls by 14.25%, 14.16% and
15.00%. The remaining large causes above are retained with the work needed
to address them, rather than hidden by a cache, reduced fixture, changed
build setting or test-tier move. No startup/forge setup, assertion, expected bytes,
timeout, generated fixture, public interface or stored format changed.

## Verification

- The new focused test
  `tests::box_kinds::a_checked_machine_keeps_resolved_programs_for_both_group_kinds`
  compares startup's retained machine with an independent machine read and
  checks resolved forgejo/runner/fake-git paths for recorded owned/fake groups.
  It passed before and after the mutation check. Forcing a recorded fake
  group to use the owned entry made it fail with `left: "fake", right: "owned"`;
  the mutation was removed.
- All 120 startup/forge tests passed before and after. Each of the four
  successful before and after traces passed its one selected test.
- The final focused run of `temper` and its sole direct dependent,
  `temper-real-loop-world`, passed 733 tests, with 27 skipped by the focused
  profile. Slow tiers did not run; all-target Clippy compiled their targets.
- `cargo fmt`, the temper old-form check from its tree root,
  `cargo clippy -p temper --all-targets -- -D warnings`, and the citation
  check against `origin/main` passed. The citation check found zero new
  broken citations. No old-form baseline or generated file changed.
