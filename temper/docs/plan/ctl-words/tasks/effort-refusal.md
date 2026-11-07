+++
id = "effort-refusal"
part = "effort-refusal"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-tools/src/jig/runner/acts.rs",
  "temper/crates/temper-client-tools/src/jig/runner/acts/choice.rs",
  "temper/crates/temper-client-tools/src/tests/jig/settings.rs",
  "temper/tests/client/tests/suite/settings_tools.rs",
  "temper/docs/help/settings.md",
  "temper/docs/design/client/tools.status.md",
]
effort = "high"
minutes = 60
+++

# A settings item's effort its model doesn't offer is refused by name

## Goal

On a box whose efforts lack `xhigh`, `settings apply` with a template asking for it printed `the engine ... did not answer within 30 s`, and a `change-settings` stayed kept. A person should get a refusal naming the effort and the model. `ctl-waits/settings-answer` owns the engine's silence, at the clients' door in jig. This task owns what a person reads: ctl checks the item against `me` and refuses before sending.

Acceptance:

- A settings file whose template's `llm` (main or a sub-agent), or whose charter's `model` or an alternative, names an effort its model doesn't offer, on a model `me` shows in any of the party's projects, is refused with exit 1 and nothing sent: `settings file item 1: model M offers no effort xhigh; it offers high, medium`, or `it offers none` for a model with none listed. Item is 1-based, as `settings file item 3: setting: ...` is.
- `settings propose` does the same.
- An effort the model offers, and an item naming no effort, are sent as they are today.
- A model `me` doesn't show is sent, and the engine's refusal is what the person reads.
- A name from the file reaches the terminal cut as `choice::named` cuts it: no control byte, at most `NAME_ECHO_BYTES`.
- **The report says what the base did,** from step 1: the door's refusal, or silence.

## Inputs

- `temper/docs/design/client/tools.md`, 5.3, the paragraph on settings files, its sentence beginning `So is a template's or charter's effort`.
- `jig/docs/design/client/domain/contract.md`, 4.
- `temper/docs/plan/ctl-words/notes/common.md`.

## Owns and may change

Owns the files in the header. `jig/crates/jig-protocol-engine-clients/` is `ctl-waits/settings-answer`'s: don't touch it. `model-catalogue-1` changes `acts/choice.rs` and `acts.rs` on its own trunk: keep your hunks small and put the new function at the end of `choice.rs`.

The file's own refusals are in `jig/settings_file.rs`; it has no `me`, so this check doesn't go there.

Departs from the implementer's text: `temper/docs/help/settings.md` is open for the one line in step 4, and `temper/docs/design/client/tools.status.md` for the one in step 5.

Options, each pre-approved:

- **If step 1 shows silence at the base,** that's the door's and `ctl-waits/settings-answer`'s. Your check still answers the story. List it under `## Left`, with what you saw, and don't change the door.
- **If the door answers `This model cannot be chosen: M`,** say so: the request's silence was gone at this base, and this task is what's left of the item, the words.
- `me(runner, client)` returns `None` until `me` is live: use it as the `CHAT_CREATE_AT` branch beside it does.
- Default for the printing: `Failed::Detail` with the bytes built in `choice.rs`, using a cut shared with `named`. If `named` won't share without reshaping it, add `Failed::EffortNotOffered { .. }` and an arm where `Failed::Unknown` is printed.
- The world's `settings_tools.rs` helper `call` asserts exit 0: use `fixtures::answered`, as the story near line 177 does.

## Steps

1. **Reproduce at the base.** At the end of `temper/tests/client/tests/suite/settings_tools.rs`, `a_settings_file_asking_an_effort_its_model_does_not_offer_is_refused_by_name`: a template item, built as `each_kind_through_ctl_proposed_shown_accepted_and_restored` builds one, whose `llm.main` names a model the world configures with an effort it lacks. Run it before any fix, with its assertion on the final words. Keep in your report what it answered, or that it never settled.
2. **The check,** in `acts/choice.rs`:

   ```rust
   /// The first named effort of `items` that its model, as `me` shows it, doesn't offer
   /// (client/tools.md, 5.3).
   pub(crate) fn settings_effort(items: &[c::SettingItem], me: &c::Me) -> Option<Box<[u8]>>
   ```

   It reads `c::SettingEdit::Set` items alone: `c::SettingValue::Template`'s `llm` (`main`, `subagents`), and `c::SettingValue::Charter`'s `model` and `alternatives`. Every other value holds no effort: list them, with no `_`. A model is found across `me.projects`' `models`, by name, as `offered` finds one in a project; its efforts are `MeModel.efforts`. In `runner/acts.rs`'s `CHANGE_SETTINGS_AT || PROPOSE_SETTINGS_AT` branch, with `Ok(ask)`: where the check finds one, `prepared(.., Err(..))`, nothing sent.
3. **Stories.** In `tests/jig/settings.rs`, a table: each of a template's main, a template's sub-agent, a charter's model and a later alternative, through `settings apply` and `settings propose`; a model with no efforts listed; an offered effort and no effort, which send; a model `me` doesn't show, which sends; a name with a control byte, which is cut. The world's story from step 1 asserts the final words, exit 1, and that the world holds no kept `change-settings`.
4. **Help.** In `temper/docs/help/settings.md`, after the bullet `A refused change commits nothing. Its refusal names the item, its`, as a bullet of its own: ``An effort its model doesn't offer is refused before anything is sent, naming the item, the model and what it offers, where `me` shows that model; the engine refuses any other by the model's name.``
5. **Status.** In `temper/docs/design/client/tools.status.md`, delete the bullet beginning `**Not yet:** 5.3: a settings item's effort`.
6. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-tools -p temper-client-world --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-tools`; `heavy cargo nextest run -p temper-client-world`; `heavy cargo nextest run -p temper-docs`; `heavy cargo nextest run -p temper --lib -E 'test(help)'`. Commit and push.

## Report

As the implementer's text has it. First after `Done:`: what the base did at step 1, plainly.
