# component-kit: status

2026-11-02. Where the design lags or departs from [component-kit.md](component-kit.md).

- **Design lags:** 3.1: `../lib.md` names none of lib's new public items: `Lifecycle`, `earliest`, `Queue::clear` and `Stack::clear`, `Slab::iter`, `bytes::decimal` and `bytes::hex_value`; `../io.md`, 2, drives io by its four entry points and names neither `skein_io::stage::{up, down}` nor the kernel's submission room they check. 3.3, 3.4, 3.5: the private copies and forwarders still in the code are counted in `skein/scripts/old-forms.txt` (`io_resume`, `earliest_helper`, `lifecycle_enum`, `hex_decimal_fn`, `host_forwarder`).
- **Open:** 6.1: the forwarding `Host`; no option is taken. 6.2: a component-kit role in programming-model.md, 4.
