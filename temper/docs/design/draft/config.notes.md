# config: the reasons behind it

This holds the reasons for `config.md`, moved from [`notes.md`](notes.md), section 21.

The reasons behind `config.md`, from the user, 2026-11-16.

- **The ask:** do for configs what the CLI pass did: "configs that are simple and mostly human readable (as the CLI) and move any complexity to temper, where it can be managed by simply asking temper itself". What's easy to change live moves to the store; "for the rest we should have reasonable defaults that don't need to show up in the configs".
- **Profiles:** "the "small profile" stuff [...] feels a bit wrong [...] it shouldn't show up in the stock config". The survey found no allocation ahead proportional to a profile's ceilings (`config.md`, section 2), so there's one stock and `small` is a test's fixture.
- **The stock's scale:** "whatever settings we ship by default should work for a deployment of reasonable scale, at least as large as dev.fabrig.ai, if not more", within "a few tens or a hundred" megabytes.
- **Agreed:** the approach ("what you say is sound") and the format ("toml +1").
- **One list:** "so Attention would be a separate UI than Needs you? I'd rather not", so a limit reached stays in Needs you (6.6). A settings change's size: "something we can deal with later".
- **This draft alone, with the workers' lifecycle:** "you don't need to do any compromise", names "a human would understand immediately", and "I suggest we do workers lifecycles as part of this draft". The draft holds the full story, runs "incrementally but eagerly", and each landing is "compatible with dev.fabrig.ai" or "offers the cheapest possible upgrade path (that can involve manual steps from ops [...])", with more discipline "after v0.1.0 has users", so `per-wake.notes.md` departs from `upgrades.md`, section 10. The workers' lifecycle "gets implemented reasonably cheaply [...] As long as it fits dev.fabrig.ai current needs".
- **The parser:** "pick a third-party crate for it, the most minimal and lightweight one, enough to cover our needs".
- **The draft's readings,** not the user's words:
  - counts at twice dev's, times at dev's, the work's pace rather than its scale; counts as formulas of `live_goals`, carriers of `tool_payload`, places summed from the workers' settings (5.2);
  - a raise at once and a lowering at the next start, since a lowered ceiling may be below what's held (6.1);
  - the sign-in's lifetime a day, dev's seven days kept as its override for its automations (5.2);
  - the memory budget 1 GiB, named `transcript_memory` for what it bounds, not the process (5.3);
  - workers as settings, each a name, places and a secret's digest, the secret made on the worker's host as today; the channel as today, on its own listener over a private network; a join by token and a channel over the public address left for later, changing two keys at most (4.3, 6.7);
  - the workers' wire declared breaking rather than bridged, dev's parts redeploying together (6.3);
  - the `toml` crate's parser alone, already in the workspace for smith's `config.toml`, so no crate is added; `toml_parser` alone is lighter but leaves tables, dotted keys and duplicates to temper, and `basic-toml` or `toml-span` would add a second parser (4.1).
- **2026-11-17, the caps,** after N1 met three counts a format caps (`forge.changes`, `fleet.slots`, `forge.landing_members`). The user: "okay let's act only on forge.changes", then "yes, add N8 with the 512 and hand it over for execution". The draft's readings: a failing test's id held within 1 KiB in the store, a narrowed bound's written patch at one record, rather than blame by test index, a retype across the finding that a patch must convert by hand; 512, twice the longest id today; the other two at their caps, since dev's workers and landings sit below them (5.2, N8).

- **2026-11-18, the store for 100 live goals.** N2 found that the stock store's value bound caps `live_goals` at 40, and the user answered "ok for 100 goals": milestone N9 sizes the store for 100, its memory at rest measured. On 2026-11-19 its row was corrected: a raise to 100 is taken at the next start, as `live_goals` is since N2, not at once; making jig's children take a raise in place is other work. N9's reading, not theirs: the store's ceilings derived from its layout at 100 rather than written, a refusal past 100 naming the store's figure before the store's sizing, and jig's conversion buffer grown with what it converts, since at the value bound it would hold about 249 MB at rest in any store whose layouts convert.
- **2026-11-19, charter versions held.** 0.0.2's first start over a copy of dev's store was refused, holding 19 charter versions besides the newest's, against jig's stock of 16, which the stock's sizing missed. A version goes once no live task's revision reads it, so the user chose N10: the figure follows `live_goals`.
