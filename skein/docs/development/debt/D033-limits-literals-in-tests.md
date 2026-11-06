# D033: Fake forge Config test literals still need one constructor

- **Kind:** tests
- **Where:** `skein_fake_forge_domain::Config` in skein's tests
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (theme T5, fixtures) and `reviews/2026-10-31-skein-areas.md` (A0, F11)
- **Seen:** 2026-10-31 at `337102750`, in `reviews/2026-10-31-skein-first.md` (theme 4)
- **Checked:** 2026-11-06
- **Value:** low
- **Size:** S
- **Status:** open
- **Paid when:** `skein_fake_forge_domain::Config`'s test literals start from one constructor, and change only the fields they test (`skein/docs/foundation/testing-strategy.md`, 10).
- **Measure:** `rg -U --pcre2 -o 'Config \{\s*limits:(?<fields>(?:(?!\.\.)[^{}]|\{(?&fields)\})*)\}' skein -g '*.rs' | rg -c 'settle_min:'`: 9 whole literals, 429 lines, all distinct.

## What

`io::Limits` and `ClientLimits` start from a `TEST`. Their whole-literal
counts are 6 and 0 respectively, excluding the `Self` definitions of `TEST`;
the six io sites are real or computed configuration. The forge machine's
15 memory counts have field names. What remains is the fake forge's `Config`,
with `fake_config` in `skein/testing/skein-forge-kit/src/fixtures.rs` and a
private `CALM` in its tests. One constructor would show each test's differences.

The item no longer asks for constructors for `tokenizer::Limits`: 46 of its
59 literals are one line, and a mean of 3.9 of 5 fields would differ from any
base, so a constructor saves nothing and says less. One- and two-field types
(`json::writer`, `json::document`, `skein_shell::Config`) cannot get shorter.
Local `fn limits()` helpers are “a setup written once”
(`skein/docs/foundation/testing-strategy.md`, 10): 62 of 67 return other or
composite types. The measure excludes signatures and updates.
