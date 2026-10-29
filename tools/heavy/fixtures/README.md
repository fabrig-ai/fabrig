# Test support

`agents-heavy` is the unchanged agents entrypoint used to verify relocation
and switching the reconciler between installations without replacing a queue.

The adjacent `land`, `land-capture.py`, `land-feed.py`, `land-resolve.py`,
`tidy-hooks.py`, `after-landing`, `check.sh` and `gate-shares.py` in the parent
directory support the imported gate-policy and recovery fixtures. They are
not installed fleet tools. Remote feed destinations in these test copies are
inert fixture names; tests use disposable local feeds. The check script is
an agents gate fixture, not fabrig's gate.
