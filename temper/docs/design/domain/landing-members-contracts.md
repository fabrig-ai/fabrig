# Contracts: a landing's members and its asks

The shapes the goal building `landing-members.md` shares between its
changes, while it runs (`../draft/design-economy.md`, 4.3). The code
holds them once the goal's changes have landed, and the goal's last
change deletes this document.

## 1. The change's policy (`temper-engine-domain-forge-change`)

- **`Handed`,** an ask a landing handed this change
  (`landing-members.md`, 3.1): `number: u32`, `landing: u64`,
  `round: u16`, `ask: Ask`. Derives what `Ask` derives.
- **`Change`** loses `landing` and `landing_ask`.
- **`Facts`:** `ask: Option<Handed>`, the queue's head
  (`landing-members.md`, 3.2), was `Option<Ask>`; and a new
  `landing: Option<u64>`, the change's landing by the index, which
  every read of the old `Change::landing` reads instead.
- **`Stepped`** gains `took: Option<Took>`, with
  `pub enum Took { Taken(u32), Dropped(u32) }`, each the number of the
  ask given as `Facts::ask`: `Taken` as `landing-members.md`, 3.3 takes
  it, `Dropped` as its 3.4 drops it.

## 2. The forge's rows (`temper-engine-domain-forge-vocabulary`)

- **`ChangeRow` and `ChangeHot`** gain `asks: Box<[change::Handed]>`
  (the queue, in number order), `handed: u32` (the count of asks
  handed, the last number given) and `previous: Option<PreviousLink>`.
- **`PreviousLink`:** `landing: Option<u64>`, `ask: Option<change::Ask>`,
  the previous build's stored link and handed-over ask, restored from the
  change's `landing` and `landing_ask` and written back to them; `None`
  where both are empty.
- **`LandingRow::asks`** is renamed `previous_asks`, same type: the
  previous build's slots, restored from the row's `asks` and written
  back to it. Nothing but the conversion reads it, and nothing writes a
  slot.
- **`Request::AskDropped { change: u64, words: Box<[u8]> }`:** the
  words of `landing-members.md`, 3.4, which the root tells the change's goal as it tells a
  member's landing news (`temper-engine-domain`'s `landing_route::tell`).
- **`Request::LinkDisagreed { change: u64, named: u64, landing: Option<u64> }`:**
  one of `landing-members.md`, 4.2's log lines, `landing` the change's landing by the index.

## 3. The forge's top (`temper-engine-domain-forge`)

- **`Domain::landing_of(&self, change: u64) -> Option<u64>`,** public:
  the change's landing by the index (`landing-members.md`, 2.1), which
  the root and the views read.
- **The index** is the landing state's, unstored, at most
  `limits.changes` entries.

## 4. The landing's policy (`temper-engine-domain-forge-landing`)

- **`Hold::Claimed { change: u64 }`,** the hold of `landing-members.md`,
  2.2, its words built
  where the landing's other hold words are.

## 5. The store (`temper-protocol-store`)

- **`record LandingHanded { number: u32 landing: u64 round: u16 ask: LandingAsk }`.**
- **`ChangeHot`** gains `asks: list 16 LandingHanded = []` and
  `handed: u32 = 0`, after its last field.
- **`Change`'s `landing` and `landing_ask`** and **`LandingRow`'s
  `asks`** stay as they are.
- **`LandingOwnHold`** gains `claimed: LandingClaimed`, after its last
  variant, `record LandingClaimed { change: u64 }`.
