+++
id = "people-start-story"
part = "start-story"
kind = "feature"
needs = []
files = [
  "temper/tests/engine/domain/tests/suite/root_routes.rs",
  "temper/docs/design/domain/root.status.md",
]
effort = "medium"
minutes = 30
+++

# The ignored people start story runs, on what a start refuses and what it panics at

## Goal

`invalid_nonfinal_people_restore_page_stops_before_issuing_its_continuation` is ignored: it asks a
start to refuse a duplicate person's identity. jig's design has a start judge no record, refuse
only what cannot be held, and panic at a link it cannot make, naming the record's key
(`jig/docs/design/domain/engine.md`, 6, the user's rule of 10-28). The panic at `Person(2)` is that
assertion. So this task changes no product code: the story runs on a people row a start does
refuse, and a second test holds the panic.

**Departure from the request's rule:** with no fix there is no old code to fail, so neither test
fails before this change. The plan's question `duplicate-identity` puts this reading to the user;
nothing waits on it.

Acceptance:

- `rg -n 'duplicate people identity panics' temper` reads nothing.
- Both tests below pass, unignored, and nothing outside `root_routes.rs` and the status file changes.
- The page holding the oversized row is not the last: the seeded rows pass the store's page bound
  (`temper/tests/engine/domain/src/store.rs`, `profile.store.page.rows`, 8).
- `root.status.md` holds nothing of this gap.

## Inputs

- `temper/docs/plan/bugs-temper/notes/common.md`.
- `jig/docs/design/domain/engine.md`, 6: **Load, then link**, the bullets from **It refuses** to
  **A link that cannot be made**.
- `temper/docs/design/domain/root.md`, 12: "a start refused ... for a malformed page".
- `jig-domain-people/src/domain/start.rs`, `fits`, read only: the assertion and `fit::identity`.
- The sibling story in `root_routes.rs`,
  `invalid_nonfinal_task_restore_page_stops_before_issuing_its_continuation`.

## Owns and may change

- Owns: those two tests in `root_routes.rs`; no other test there, and the `needs core-inbox`
  ignores stay.
- Not this task's: any file under `jig/`.
- Status: `temper/docs/design/domain/root.status.md`, first **Not yet**: remove the sentence from
  `12: a start story is ignored` through `root_routes.rs`), and its closing `.`, leaving the rest
  of that bullet. Date line today's.
- Pre-approved: if the fixture cannot hold enough people to pass one page (`bounds.people.people`
  past a limit the root's check refuses), keep the story on the one page, drop `nonfinal` from its
  name, and say so in the report. If the fixture needs other limits raised to hold the rows, raise
  them in the test's own `bounds`.

## Steps

1. Replace the ignored test with these two, sharing a small helper:

   ```rust
   fn person(number: u64, subject: u64, name: &[u8]) -> Record {
       Record::Core(jig_domain_core::Record::People(people::Stored::Person {
           number,
           identity: people::Identity {
               key: people::IdentityKey { provider: 0, subject: subject.to_be_bytes().into() },
               login: b"same".as_slice().into(),
               name: name.into(),
           },
       }))
   }

   #[test]
   fn invalid_nonfinal_people_restore_page_stops_before_issuing_its_continuation() {
       let mut bounds = limits();
       bounds.people.people = 12;
       let oversized = vec![b'x'; skein_lib::usize_of(bounds.people.identity_bytes) + 1];
       let mut store = Store::new();
       for number in 1..=12 {
           let name: &[u8] = if number == 2 { &oversized } else { b"Same" };
           store.seed(Box::new([person(number, number, name)]), Box::new([]));
       }
       let mut driver = Driver::configured(store, [config(91), config(91)], &bounds);
       for _ in 0..30 {
           driver.advance(true);
       }
       driver.broken("oversized identity deliberately refuses startup");
       assert!(driver.stopped, "oversized identity in a middle page is a terminal startup failure");
       assert!(!driver.root.ready());
       assert!(!driver.root.quiescent());
       assert!(driver.events.is_empty(), "failed page did not issue another continuation");
   }

   #[test]
   #[should_panic(expected = "identity index collision at Person(2)")]
   fn a_duplicate_person_identity_panics_at_its_link_naming_its_row() {
       let mut store = Store::new();
       for number in 1..=3 {
           store.seed(Box::new([person(number, 7, b"Same")]), Box::new([]));
       }
       let mut driver = Driver::new(store);
       for _ in 0..30 {
           driver.advance(true);
       }
   }
   ```

   `limits`, `config` and `Store` are this file's imports. Check the assertion's words against
   `fits` before you trust `expected`.

2. Run each test alone, by its full name as nextest lists it
   (`-p temper-engine-domain-world --test suite -E 'test(=NAME)'`), then edit the status file.

3. Run `cargo fmt`, the old-form check, clippy on the package and its focused tests, as the
   implementer's text says. Commit and push.

## Report

As the implementer's text says. State that no product code changed and the panic stands.
