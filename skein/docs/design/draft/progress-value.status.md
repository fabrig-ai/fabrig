# progress-value: status

2026-11-05. Where the design lags or departs from [progress-value.md](progress-value.md).

- **Design lags:**
  - 8: `../ui/README.md`, 14.1 says nothing of a progress, and no test has the fake person see one by its words, on the tree face or the browser's. The tree face reads a node's runs whatever its role (`skein-fake-person/src/tree.rs`), so it would.
- **Design departs:**
  - 3: a progress's text is runs of its own, not a text child (`../ui/README.md`, 3.4 and 5): the mapping's `aria-valuetext` reads a node's own runs, the builder gave a progress none, and a progress bar presents no children. The cut's call, 2026-11-04.
- **Open:** whether a progress's text is runs of its own, the departure under 3: the plan's question `progress-runs`, unanswered when the plan retired. The alternative keeps a text child and has the page part read its words up to its parent. The design and the code hold runs of its own, and another answer is a follow-up task.
