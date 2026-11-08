# skein-limit-40: skein's focused suite limit goes from 30 to 40 CPU seconds

The user's word, 11-08 about 14:10Z (to agents/loop): "skein/default sits near its 30 (28.5 s on today's full reads), raise it to 40." Changing a figure is the user's, and this is the user's figure.

One change: in `tools/tiers/budgets.toml`, `[gate.skein.default]` `cpu_suite_s = 30` becomes `40`, and the header's skein line says the focused figure is 40 by the user's word of 11-08 (the fuzzy figure stays 120 by the word of 11-07). Nothing else changes: no other entry, no code, no test. One direct task, no survey. The queue reads the file at each batch.
