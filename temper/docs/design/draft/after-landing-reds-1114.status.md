# after-landing-reds-1114: status

2026-11-14. Where the design lags or departs from [after-landing-reds-1114.md](after-landing-reds-1114.md).

The design carries this draft. Section 1's shared lookup is `skein_scratch::paths::built` (`../../../../skein/docs/design/testing/README.md`, 6; `../testing/tools.md`, 5): another package's binary is the file of its name in the nearest directory above the test's executable, so the depth of that executable is no rule. Section 3's stories keep Panics P3's finding scope, which allows a real run nested in another on the same thread, each run keeping its own findings (`../../../../skein/docs/design/testing/README.md`, 5). Sections 2 and 4 are repairs of stories and of the product that design nothing new.

- **Design lags:** none.
- **Design departs:** none.
- **Open:** none.
