# one-owner: status

2026-11-15. Where the design lags or departs from [one-owner.md](one-owner.md).

- **Design lags:** none.
- **Design departs:**
  - 2: configuration's `Shape`, and its `Fault`'s bare variants, are kept beside render's `Decode`, which they map one to one, with no reason in the goal's note (O3's review, `temper/docs/development/debt/reviews/2026-11-15-one-owner-o2.md`, 4, finding 6).
  - 4: O2's designer wrote the model's text itself, delivered with the goal's design, rather than in its first change: it is Markdown that no test reads, and the changes that build on it start sooner (the goal's designer, 2026-11-14).
  - 5: the projections the client's records carried (a schema fault's typed names, a filter's name and arity, an export map's absent entries) are kept as the printer's, where they print, not as types of the records' own: the printer maps render's values to the version's form (2), and the copies around them go (the goal's designer, 2026-11-14).
- **Open:** 8, the steward's records and adoption.
