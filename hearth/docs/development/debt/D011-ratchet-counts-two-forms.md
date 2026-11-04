# D011: hearth's ratchet counts two forms, and counts what the others exempt

- **Kind:** other
- **Where:** `hearth/scripts/old_forms.py`, `hearth/scripts/old-forms.txt`, `hearth-protocol-config/src/document.rs`
- **Seen:** 2026-11-04 at `66968a30ef`, in `hearth/scripts/old-forms.txt`
- **Checked:** 2026-11-04 at `66968a30ef`
- **Value:** medium (new code stops adding the forms this list pays off)
- **Size:** S
- **Status:** open
- **Paid when:** `hearth/scripts/old_forms.py` exempts `io::MAX_OUT_*` as jig's and temper's do, and counts `io_resume`, `earliest_helper`, `hex_decimal_fn`, `host_forwarder`, `path_attribute` and step code's `while`, each with its baseline.

## What

`hearth/scripts/old_forms.py` has `max_out`, `worst_case` and two comment
words. skein's also counts the component kit's forms
(`skein/docs/design/draft/component-kit.md`, 4), and smith's
`path_attribute` and `domain_while`. So nothing stops new copies here:

| Form | In hearth at base |
|---|---|
| `io::resume` | 1 |
| `fn earlier` or `earliest` | 1, a fake's name, `hearth-fake-machine` |
| `fn hex`, `fn decimal` | 15 |
| `impl Host for Process` | 1, `tests/builder/src/fixtures/binary.rs` |
| `#[path]` | 25 |
| `while` in step code | 1, `document.rs`, `refuse_list_null` |

And `max_out` counts 8 `io::MAX_OUT_*` uses, the kernel's edge, which
jig's and temper's scripts exempt.

## Why it matters

The other items here fall only if nothing refills them. Until the exemption
is in, D001's count mixes the kernel's edge with the old form.

## What pays it back

Copy the forms, set each baseline with `--write`, and add the exemption.
Worth doing first: it is a small change and holds the ground for the rest.
The `while` itself is `for` over a bound the decoder's limits give.
