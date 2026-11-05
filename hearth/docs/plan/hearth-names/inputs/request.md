# Request: hearth's documents and fixtures name no real machine layout, provider or key

From fabrig/hearth, 2026-11-05, on the user's decision (`hearth-docs`' open question `names-left`: neutralize). fabrig may go public. hearth's current files still say which real
machine runs where and what it serves, and with which hosting providers. No git history is rewritten. **Nothing of hearth's behaviour or design changes:** only example names do.

## The mapping

Use it everywhere under `hearth/`, consistently, so a fixture or an example still reads as one fleet:

| Now | Becomes |
|---|---|
| `serverhino`, ServeRhino | `provider-a`, "provider A" |
| `ionos`, IONOS | `provider-b`, "provider B" |
| `vultr`, Vultr | `provider-c`, "provider C" |
| `rhi-prod`, the host `rhi` | `east-prod`, `east` |
| `ion-prod`, the host `ion` | `west-prod`, `west` |
| the host `vul` | `south` |
| the key fragment `m3NY8…=` | `AAAA…=` |

Find them with `git grep -n -i -E 'serverhino|serve rhino|ionos|vultr|m3NY8|\b(rhi|ion|vul)\b|\b(rhi|ion)-[a-z]+' -- hearth`, and read each hit: `ion` and `rhi` may be parts of unrelated words, and a host's name may appear in a path, a document's name or a digest's input.

**The layout.** `hearth/docs/design/domain/machine.md`, about line 109, says what each production machine serves (primary MX, Forgejo, Matrix, backup MX, VPN hub). Keep the design fact, that two of three production VPSes are BIOS, and drop what they serve and any other sentence that describes the user's own deployment rather than a requirement. A firmware's behaviour stays as a requirement, with the neutral name: "provider C's firmware discards its variables". Look through the rest of `machine.md`, `protocol/format.md` and `README.md` for the same kind of sentence, and say in the plan's notes what you changed.

## Leave

These were judged harmless, or aren't in the user's decision: `x1-prod`, `mail-prod`, `carbon-prod`, `chat-prod`, `7th-gen`, `stoa`, the name `os`, the `ai/` path, and the digest `5a403c6e…`. Addresses and domains already neutral stay. `hearth/docs/design/draft/archive/factory.md` is archived: apply the mapping only where one of these names appears. `hearth/docs/development/debt/` only if a name appears there.

## Fixtures and goldens

They change with the names, by whatever regenerates them; digests in goldens change for that reason alone. Directory or file names that hold a name are renamed with `git mv`. A test failing for any other reason after a rename is a finding, not something to update. Don't move a memory ceiling unless a measurement shows the peak moved, and say why.

## Rules

- Afterwards, the search above finds nothing under `hearth/` that's one of these names.
- Mechanical: rewrite by script, then read the diff. One task or very few; this isn't design work.
- The focused suite stays under what `tools/tiers/budgets.toml` holds.
- `tools/citations/citations.py --all` stays at 0, and `tests/boundary/tests/outward.rs` passes.
- Nothing changes outside `hearth/`.
- All parts must be in the plan when the cut ends.
