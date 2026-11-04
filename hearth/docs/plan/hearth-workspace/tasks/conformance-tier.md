+++
id = "conformance-tier"
part = "tier"
kind = "feature"
needs = ["join-worlds"]
files = [
  ".config/nextest.toml",
  ".config/gate/README.md",
  "tools/tiers/packages.toml",
]
effort = "medium"
minutes = 40
+++

# hearth's conformance tier: a `<tier>-hearth` profile, out of the gate

## Goal

`join-worlds` keeps `conformance_*` and `recorded_*` out of `default` and `fuzzy`. Give them the tier
profile `conformance-hearth` (`notes/contracts.md`, 3), in the package map, so `tests/boundary` and
the tier tools read it. `tools/tiers/budgets.toml` doesn't change: its header says a pair may have no
entry, and hearth's is the user's.

Acceptance:

- `TIERS_WRAPPER=heavy tools/tiers/run --verify hearth conformance` passes, and the report gives both
  counts.
- `python3 -I -B tools/tiers/test_run.py` passes.
- `heavy cargo nextest run -p hearth-conformance-world --profile conformance-hearth -E 'binary(/^recorded_/)'`
  passes: those run offline, and they just left `default`.
- `heavy cargo nextest run -p fabrig-boundary` passes.

## Inputs

- `notes/common.md` and `notes/contracts.md`, section 3, in this plan's directory
  (`hearth/docs/plan/hearth-workspace/`).
- `tools/tiers/design.md`, 1, 2.1 and 6.
- `hearth/docs/design/domain/testing.md`, 2.8 and 9.2.
- This cut wrote no design: those sections say what the pair is.

**Departures from the implementer's text:**

- `--verify` lists the whole workspace, so it builds every test executable: that one command is the
  whole workspace's, because `tools/tiers/design.md`, 6, asks it of a changed map.
- `conformance_*` isn't run: it needs real programs and the mirror, and it's no gate of yours.

## Owns and may change

**Owns:** nothing alone. **May change, only as the steps say:** `.config/nextest.toml` (the profile and
the header's list), `.config/gate/README.md` (its list of hyphenated profiles),
`tools/tiers/packages.toml` (the pair's two entries).

## Steps

1. **`.config/nextest.toml`,** after `browser-temper`:

   ```toml
   # The conformance tier, outside the gate (hearth/docs/design/domain/testing.md, 2.8);
   # HEARTH_RECORD=1 captures.
   [profile.conformance-hearth]
   default-filter = "(binary(/^conformance_/) | binary(/^recorded_/)) & package(/^hearth(-|$)/)"
   global-timeout = "4m"
   slow-timeout = { period = "60s", terminate-after = 2 }
   retries = 0
   fail-fast = false
   ```

   Those are the keys of hearth's old `conformance` profile, as they stood in `hearth/.config/nextest.toml`
   before `join-steps` removed it. The header's list of tier profiles gains `conformance-hearth`.
2. **`tools/tiers/packages.toml`:** generate the pair from its listing, as `tools/tiers/design.md`, 6,
   says. This cut expects `conformance-hearth = ["hearth-builder-world", "hearth-conformance-world"]`,
   sorted among the pairs, and `conformance-hearth = []` in `[binaries]`.
3. **`.config/gate/README.md`:** `conformance-hearth` in the `After landing` list of profiles.
4. The acceptance's commands; commit and push.

## Report

The implementer's report, and the two verified counts.
