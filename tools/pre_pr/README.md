# Delivery checks

`.temper/pre-pr` checks what a delivery adds over the checkout's `HEAD`: `rustfmt --check` on changed Rust files, generated files left out, and `scripts/old_forms.py` in each tree whose Rust files or old-forms files change; a shared counter change checks all five trees, a documents-only delivery runs neither, and a passed tree is cached (`temper/docs/design/protocol/configuration.md`, 13.9). It needs only Python's standard library, git and rustfmt.

Run `python3 -I -B tools/pre_pr/test_pre_pr.py`. The table test runs fakes of rustfmt, cargo and python3 on `PATH`, never a real cargo or rustfmt.
