# temper

The application, built on skein, smith and jig. Start from `temper/docs/design/README.md`, which says where each part lives.

- **`temper/docs/design/testing/README.md`** is how the testing strategy applies to temper.
- **`temper ctl` reads stdin:** in a script, give each call `</dev/null`.
- **Dogfood hacks:** a plan adding a temporary change for running our development on temper adds its entry to `temper/docs/design/draft/dogfood-hacks.md`, in the same landing.
- **Look requests:** a commit on the person's `look-requests` branch with a `Look-request: NAME` trailer asks for a change of the look (`temper/docs/design/client/web/journeys.md, 7`); whoever lands one follows `temper/docs/design/client/web/look-requests.md`, every step and its account.
