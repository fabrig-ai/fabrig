# D011: The benchmarks carry failures as strings

- **Kind:** other
- **Where:** `benchmarks/src/`
- **Seen:** 2026-10-31 at `aa95945cc`, in the smith code-quality review (`reviews/2026-10-31-quality-areas.md`: report 12, finding 7)
- **Seen:** 2026-10-31 at `679eacb3c`, in the quality run's closing report (outside the repository): left as a benchmark redesign
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low (typed faults the report can classify; less `map_err` noise)
- **Size:** M
- **Status:** open
- **Paid when:** the harness has one fault type that names its kind, and no `Result<_, String>` remains in `benchmarks/src` outside its edges (argument parsing, the report's text).
- **Measure:** `git ls-files 'smith/benchmarks/src/*.rs' | xargs grep -o 'Result<[^>]*, String>' | wc -l` is 185

## What

The harness (ordinary Rust, not step code) returns `Result<_, String>` 185
times and converts with `.map_err(|error| error.to_string())` 184 times,
beside its typed `Refusal`, `Setup` and `HarnessFault`. Immediately called
closures attach an error or a cleanup, and `run.rs` relabels failures by
mutating a step variable.

## Why it matters

A string fault cannot be told apart from another by the summary, which the
benchmarks' verdicts need ("unknown is never zero" holds for measures, not
for faults). Each new step repeats the conversion.

## What pays it back

One `Fault` with a kind and a detail, `?` through it, and the closures
replaced by functions. Mechanical once the kinds are chosen; no result
format changes unless a fault kind reaches the summary.
