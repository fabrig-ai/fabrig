# component-kit: status

2026-11-04. Where the design lags or departs from [component-kit.md](component-kit.md).

- **Design lags:** 3.1: `../runtime/lib.md` names none of lib's new public items: `Lifecycle`, `earliest`, `Queue::clear` and `Stack::clear`, `Slab::iter`, `bytes::decimal` and `bytes::hex_value`; `../runtime/io/README.md`, 2, drives io by its four entry points and names neither `skein_io::stage::{up, down}` nor the kernel's submission room they check.
- **Open:** 6.1: the forwarding `Host`; no option is taken. 6.2: a component-kit role in programming-model.md, 4; the shared transport, `skein-link`, takes the protocol layer's row until the user says (`../net/link.md`, 2), and that role would change only its wording.
