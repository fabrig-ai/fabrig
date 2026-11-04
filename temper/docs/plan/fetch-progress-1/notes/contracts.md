# fetch-progress-1: the contracts

The shapes that cross parts, binding for every task that cites them.

## 1. skein-ui's fraction (`skein/docs/design/ui.md`, 3.2 and 5)

```rust
// skein_ui, exported beside `Spans`
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Fraction { pub done: u64, pub total: u64 }

impl Builder {
    pub fn fraction(&mut self, fraction: Fraction); // on a progress; done <= total; total > 0
    pub fn run(&mut self, run: Run, bytes: &[u8]);  // on a text node or a progress
}
// `Node.fraction` and `Attribute.fraction`: Option<Fraction>
```

- A progress's words are **its own runs:** `b.open(Role::Progress); b.name(..); b.run(Run::Plain, b"42%"); b.fraction(..); b.close();`, never `b.text(..)` inside it. `close` asserts a fraction has runs.
- The web page writes `aria-valuetext` from those runs, `aria-valuemin`, `aria-valuemax`, `aria-valuenow` and `data-progress` from the fraction (`skein/docs/design/ui-web.md`, section 6).
- `skein-ui.css` sets `--skein-ui-progress` from `data-progress` (`skein/docs/design/ui-web.md`, section 12); an owner's rule reads `var(--skein-ui-progress, 0%)`.
