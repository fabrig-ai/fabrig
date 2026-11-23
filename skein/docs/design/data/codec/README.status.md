# codec: status

2026-11-23. Where skein's code lags or departs from [README.md](README.md).

- **Not yet:** 4: paths held by name with `Path::name`, no names' table; one bound shape per declaration in `DECLARATIONS`, naming the declarations and limits it reaches; `Limits` under `#[repr(C)]` with `at_most`; no generated line spelling a position [in flight: change 14018]
- **Design departs:** 4: `Record`'s and `Field`'s methods keep their `limits` in every implementation, though a declaration that reaches no limit, and a scalar's field, read none. The request asked that no unused parameter be passed; a trait's method has one signature for every implementation, which a nested field's call and a frame's writer rely on, so the parameter stays there alone, and the module's own functions and the translation (`maps.md`, 4.1) take only what they read. Decided by the codec debt goal's designer, 2026-11-23.
