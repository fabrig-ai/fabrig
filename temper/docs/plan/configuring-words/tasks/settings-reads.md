+++
id = "settings-reads"
part = "reads"
kind = "feature"
needs = []
files = ["temper/crates/temper-engine-domain/src/route/host_route.rs", "temper/crates/temper-engine-domain/src/route/tools.rs", "temper/crates/temper-engine-domain/src/route/kinds.rs", "temper/crates/temper-engine-domain-forge/src/settings.rs", "temper/crates/temper-engine-smith/src/settings.rs", "temper/crates/temper-engine-smith/src/tests/settings.rs", "temper/tests/engine/domain/tests/suite/configuring.rs", "temper/tests/engine/domain/tests/suite/forge_routes/approval_identity.rs"]
effort = "high"
minutes = 70
+++

# A project's ways and questions read as settings; a read that finds nothing says so; an approval's refusal names no command

## Goal

A chat's `settings` read of `project-ways` or `project-questions` printed `unknown`: the stock has no row for them (`temper/docs/design/protocol/configuration.md`, 13.4), jig's core answers `Answer::None(Why::Unknown)` for an application setting with none stored (`jig-domain-core/src/reads/settings.rs`, `setting_value`), and the run's route, unlike the client, never took the stock. The agent asked with the right key, so this is a fault of the read and not of the words. Make the read answer the stock, make what's left of `unknown` say what it means, and make acceptance's refusal name no command.

Acceptance:

- For a project the core holds, `{"setting":{"kind":"project-ways","project":P}}` with none stored answers the stock, not as an error, with `drafts`, `goals`, `changes` and `away` false and `way` 0; `project-questions` answers `none`.
- With a value stored by an accepted change, the same read answers the stored value.
- At an older named revision, and for a project the core does not hold, it stays not read. A run without the settings family is refused as it is now: the stock is taken only where the core's read has answered.
- `SettingsAnswer::Unknown` renders the words of `notes/contracts.md`, 3, still an error. No other settings answer changes.
- `approval_words` in `temper-engine-domain/src/route/kinds.rs` reads, with its numbers, byte for byte:

  ```
  project 1 role 2 requires 1 distinct known forge people; currently 0 eligible. Before accepting, add the person as a collaborator on the forge, give that role the permission in this project's policy if needed and adopt the repository again; then propose afresh against the newest revision, or remove the approval requirement.
  ```
- `approval_identity.rs` holds those words and that the refusal holds none of `temper help`, `temper ctl`, `project show`, `repository adopt`.

## Inputs

- `temper/docs/plan/configuring-words/notes/common.md`
- `temper/docs/plan/configuring-words/notes/contracts.md`, 3 and 4
- `temper/docs/design/domain/agent.md`, 8.1, *Its input*
- `temper/docs/design/protocol/configuration.md`, 13.4, *A project's kinds 6 and 7 have no row in the stock*
- `temper/docs/design/domain/forge.md`, 8.9, *The root checks approval eligibility*

## Owns and may change

Owns the files in the header. Reader and neighbour files:

- `temper-engine-domain/src/route/forge_route/tests.rs`, `temper-engine-domain/src/tests/*`: a `FlightRender::Settings` match, or a refusal's words where a test pins them: call sites and those words only.
- `temper/tests/engine/domain/src/*`: helpers a new story needs.

Options, pre-approved:

- If `project_stock` fits better in another module of `temper-engine-domain-forge`, put it there; the name and signature stand.
- `SizedValue.size`: if the forge's own measure can't be reached from where it lives, compute it the way `temper-engine-smith/src/kinds.rs`'s `size` does for these two fixed records, or move the two figures to a constant, and say so.
- If rendering a `SettingDocument` with `application: None` fails in `settings_form` or jig's translate, give it the `application` state the wire needs from what `temper-engine-domain` can build; if only jig could build it, stop with a `Question:`.

## Steps

1. `temper-engine-domain-forge`: `project_stock`, as `notes/contracts.md`, 4. Have `temper-engine-domain-forge/src/kinds.rs`'s kind 6 fallback (near line 297) use it if that's one line; otherwise leave it.
2. `host_route.rs`: `FlightRender::Settings` gains what was asked, from `Tool::Settings`' read:

   ```rust
   Settings { from: u32, requested: Option<u64>, asked: Option<(jig_domain_core::settings::Setting, u32)> },
   ```

   In `settings(domain, to, answer)`, before the delivery: when the answer is `SettingsAnswer::Unknown`, what was asked is `Setting::Application { kind: 6 | 7, scope, number: 1 }`, its revision is 0 or the newest, and `domain.core.people.project_members(scope)` is `Some`, the answer becomes `SettingsAnswer::Setting(SettingDocument { setting, revision: newest, value: None, application: None, proposed: None })` and the delivery's `application` is `project_stock(kind)`. `tools.rs`'s `settings_value` stays as it is for a stored value.
3. `temper-engine-smith/src/settings.rs`: the words for `SettingsAnswer::Unknown`. `tests/settings.rs`'s last assertion moves to them, with "context belongs only to an inventory" still held.
4. `kinds.rs`: `approval_words`. `approval_identity.rs`: near its line 536, the new words and the four absences.
5. Stories in `temper/tests/engine/domain/tests/suite/configuring.rs`, on the shape of `an_owners_configuring_chat_proposes_and_hears_it_accepted`:
   - `a_project_with_no_ways_reads_at_their_stock`: both kinds, not an error, naming their values;
   - `a_projects_stored_ways_read_as_stored`: propose and accept `project-ways`, read it, see the new value;
   - `a_read_that_finds_nothing_says_it_was_not_read`: a project the core doesn't hold, and a charter number past the stock: error, the words of the contract.
6. For each new test, break its rule once by hand, see it fail for that reason, restore. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-engine-domain -p temper-engine-domain-forge -p temper-engine-smith -p temper-engine-domain-world --all-targets -- -D warnings`; `heavy cargo nextest run` on those packages and `temper-service`. Commit and push.

## Report

Your text's report, with the text the two stock reads now print.
