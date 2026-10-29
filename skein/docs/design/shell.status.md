# shell: status

2026-10-29. Where skein's code lags or departs from [shell.md](shell.md).

- **Built:** 12: `Host` has no heap bound. 7.4: uncancelled interrupted transfers reissued unchanged without a retry bound. 1, 2, 4, 7 to 10: epoll backend, startup fallback and platform split; Darwin's shell parts are written. 14: the portable face, `Host` and a command's settings on wasm32, the rest fenced by `cfg(unix)`. 6: the fail-stop report carries the panic's backtrace once, whatever RUST_BACKTRACE says.
- **Not yet:** 6: startup names and TLS server keys; 6.4: containment helper and service cgroup. 8: deferred optimisations. 7.4: an interrupted `Socket`, `Accept`, `Connect` or `Open` completes `Other(EINTR)`; its rule is undesigned (`Connect` needs a replay rule).
