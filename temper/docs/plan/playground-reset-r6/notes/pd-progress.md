# Part `pd` (jig's deletable project): progress

Designer of draft section 5, its testing in section 8, the jig half of 6.2, and contracts J1-J8.

## Specs

(none yet)

## Contract requests

1. **People store version (J4, "people's store version moves by one").** jig's store design
   says the opposite: `jig/docs/design/protocol/store.md`, 5 (lines 382-386): "Until the first
   release, each family's version keeps its current number, and its schema, generated code and
   one set of golden bytes change in place. Versions move at a release." Recent people changes
   followed it (`jig-protocol-store/src/tests/settings_layout.rs:1`, "changes people 13 in
   place"); the layouts fingerprint is what refuses an older store. A bump to people v14 renames
   `people_v13` in about 40 files, among them temper's `temper-protocol-store`,
   `temper-forge-setup`, `temper-fake` and `tools/twin`, which section T's tasks also write.
   **Recommendation:** people v13 changes in place (schema, generated code, goldens, layouts
   fingerprint), as store.md 5 says. **Until answered** the store spec follows the contract as
   it stands (v14); it is written last of the surface so that an answer can still change it.
2. **jig's client wire version (J7, "one more client wire version").** Same rule in
   `jig/docs/design/protocol/client.md:432` and `:769` (client-v19 changes in place until the
   first release). A bump to v20 renames `wire::v19` in about 144 files of jig and temper.
   **Recommendation:** client-v19 changes in place. Until answered, the wire spec follows
   the contract (v20).
3. **J8's `AddKey { uses }` type.** `jig_domain_people::Act` cannot name
   `jig_domain_accounts::Use`: jig-domain-people does not depend on jig-domain-accounts. People
   already has the same type, `jig_domain_people::Use::{Party(u64), Project(u32)}`
   (`boundary.rs`, used by `AccountChange::SetUses`), which the core translates to accounts'
   `Use`. **Default taken:** `uses: Option<Box<[jig_domain_people::Use]>>`; temper names
   `people::Use::Project(project)`.
4. **J8 order of variants.** J1 appends `RemoveProject` after `Adopt` and last in
   `ActKind::ALL`; J8 adds `Cancel`. **Default taken:** `Act::{…, Adopt, RemoveProject,
   Cancel}`, `ActKind::{…, MakeProject, RemoveProject, Cancel}`, `ALL` in that order.
5. **Retained answers (J4's keep list).** Both a connector's `Stored::Acted` and a party's
   `Stored::Answer` naming the project are retained answers: both stay until their retention
   and answer as saved, then `Unknown`. **Default taken:** the referee allows both.
