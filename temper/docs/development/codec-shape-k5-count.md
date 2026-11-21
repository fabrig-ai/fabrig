# codec-shape K5: the stores' translation, counted

2026-11-21, on the goal branch at `845423850d`, before K5's changes. The
count `temper/docs/design/draft/codec-shape.md`, 4.1 asks of goal K5,
owner by owner: each record and enumeration of the stores' schemas,
read against its hand-written translation and its domain type. Read-only
surveys, declaration by declaration; the lines are approximate, a mixed
function charged to its most demanding part.

## 1. The kinds

A declaration's kind is its own mapping, not its children's: a record
that copies its fields is a copy even when a field's declaration is the
owner's. *None* is a declaration with no domain type of its own: a
variant's payload whose fields sit inline in the domain's variant, a
record of one field the domain holds as that field, or a root envelope.

| Owner and family | Declarations | Copy | Scalar | Owner's shape | None | Lines: copy / scalar / shape |
|---|---:|---:|---:|---:|---:|---|
| jig core | 125 | 32 | 2 | 17 | 74 | 1,782 in all |
| jig accounts | 26 | 5 | 2 | 4 | 15 | 280 in all |
| jig component | 15 | 9 | 0 | 4 | 2 | 185 in all |
| jig notes | 15 | 2 | 0 | 5 | 8 | 165 in all |
| jig openings | 3 | 1 | 2 | 0 | 0 | 47 in all |
| jig registry | 3 | 0 | 0 | 3 | 0 | 199 in all |
| jig secrets | 11 | 0 | 0 | 5 | 6 | 102 in all |
| jig tasks | 334 | 100 | 32 | 32 | 170 | 2,600 / 850 / 2,600 |
| jig people | 354 | 80 | 7 | 20 | 247 | 1,450 / 800 / 2,250 |
| jig settings | 122 | 62 | 7 | 15 | 38 | 1,250 / 500 / 650 |
| **jig** | **1,008** | **291** | **52** | **105** | **560** | about 6,500 / 2,300 / 6,900 |
| temper store | 570 | 237 | 38 | 80 | 221 | about 4,100 / 1,200 / 3,400 |

temper's store also holds about 3,200 lines that mirror no declaration:
its keys, limits, lowering, jobs and the people family's refusal it
embeds.

## 2. What the owner's shape is

- **Framing and budgets:** lists packed as pieces under an owner's byte
  sum (20 fields in jig's people family alone), a tasks record split
  into hot and cold rows, a family's bytes embedded in another's.
- **Context:** limits, model names, a core view of the other families,
  temper's bounds, a decoded run or failing list a check reads.
- **Checks and remaps:** sorted test identities, model-name remaps,
  a share's ceiling, implications closed under transitivity.
- **Mismatched sides:** a record flattened into a nested domain type, a
  recursive hold made a flat list, a positional tuple, a domain variant
  or field with no counterpart, a retired variant that refuses.

## 3. What is mechanical

- **Scalar conversions,** by fields: a `Wall` and its nanoseconds 126
  (jig 78, temper 48); a `Time` and its nanoseconds 16 (temper); a
  `Duration` and its nanoseconds 15; a `Token` 3; text as bytes 15; an
  owner's bit set and its integer 12 (jig); an owner's newtype 3 (jig's
  `Newest`); a signed number's bits 2 (temper); a digest's bytes and
  array 4 (jig).
- **Variants,** by occurrence: unit to unit about 950; a payload's
  fields inline in the domain's variant about 640; a payload as one
  value of the domain's tuple variant about 130; a record of one field
  as the tuple variant's value about 200.
- **Containers:** the domain holds almost every list as a boxed slice,
  a few as `skein_lib::List`; a box sits on either side at a field or a
  payload.

These shapes made the maps file's forms
(`skein/docs/design/data/codec/maps.md`, 2 and 3).
