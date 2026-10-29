# Merge queue status

Selection implements explicit parent/candidate tree and lockfile comparisons
and keeps unmerged plan previews separate (`design.md`, sections 2 and 2.1).
Checked package selection loads ownership and the resolved graph from the
candidate checkout. The one-gate queue uses the `origin/main` / `HEAD`
defaults; merge, promotion, cleanup, full-check accounting, and rejection
policy retain their one-gate behavior.

The merge-train controller, workers, transport, result validation, durable
train state, and rollout support in `design.md`, section 4 remain unimplemented.
