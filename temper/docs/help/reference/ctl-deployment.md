# temper ctl deployment

Kind: reference

Read the store's census in the deployment's document.

## deployment census

```
deployment census
Read the store's census in the deployment's document
Prints: `census at commit N, writes since commit S`, then a line per keyspace in prefix order; `no census yet` before the first report
--json: the record census.
Key: none.
Needs a deployment role; jig's refusal exits 2 without one.
Prefixes print in lower-case hex, - for the empty prefix or no name.
Heads print H:R,H:R, - for an unnumbered head, heads - for none.
JSON keeps the census whole, or null before the first report.
```
