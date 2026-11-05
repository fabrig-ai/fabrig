# ui-settings-tabs: what crosses tasks

The run's binding calls. Design: `temper/docs/design/client/README.md`, 3.5, *Settings*; `temper/docs/design/client/browser.md`, 2.5.

## The language, in `temper-client-view`

`forges-tab` makes these, and `accounts-tab`, `flow-paths` and `builders-tab` build on them.

```rust
// look.rs, the next free token, its name `temper-settings-row`:
pub const SETTINGS_ROW: Style; // on each folded row's Role::Disclosure

// pages/settings.rs, beside `open`:

/// A tab's page with no project: opens the column and draws its level-1 heading and lede, as `open`
/// does with its kicker and tabs left out; the caller draws the body and closes the column.
pub(crate) fn open_alone(b: &mut Builder, heading: &[u8], lede: &[u8]);

/// Opens a section and its head, and draws its level-2 title. The caller draws the head's act,
/// closes the head, draws the section's body and closes the section.
pub(crate) fn section_head(b: &mut Builder, title: &[u8]);

/// Opens a quiet band and draws its tag and words. The caller draws its act and closes it.
pub(crate) fn quiet_band(b: &mut Builder, tag: &[u8], words: &[u8]);

/// A link as a `.btn`: `.primary`, or else `.small`.
pub(crate) fn button_to(b: &mut Builder, name: &[u8], to: Binding, primary: bool);
```

The page, on both tabs:

```text
Main (MAIN, SETTINGS)
  settings::open(.., Tab) | settings::open_alone(..)      // leaves the column open
  section_head("Your forges · 2"), the act, close         // the first section also carries FORGES or ACCOUNTS_PAGE
    quiet_band(..), its act, close | Slot (SETTINGS_LINES)
      Disclosure (SETTINGS_ROW): name, description, then its content; a fact is Slot (SETTINGS_SETTING) { Term, Description }
  close section
  ...further sections
  close column; close main
```

`.temper-settings-lines > details.temper-settings-row` takes `.lines > div`'s hairline and padding. Under `.temper-settings-row`, by scoped rules: `summary` as *The rows*, `button`, `.temper-flow-primary`, `.temper-flow-secondary` and `.temper-flow-quiet` as `.btn`, `.btn.primary`, `.btn` and `.btn.quiet`, `input` as `.text`. `SETTINGS_PRIMARY` alone is `.btn.primary`, full size; `SETTINGS_SMALL` sizes it small: `forges-tab` moves the `:is(...)` rule that sizes them together, and `button_link`'s primary gains `SETTINGS_SMALL`, so that How work runs draws as it did.

## `Builders { project }`, in `temper-client-domain`

`builders-tab` makes these.

```rust
// address.rs
Address::Builders { project: u32 } // /projects/<project>/settings/builders

// pages/builders.rs
//! The deployment's hosts as Settings' Builders tab reads them (client/README.md, 3.5, Settings).
#[derive(Debug, Default)]
pub struct Builders { handle: Option<Handle> }

pub enum Read<'a> {
    NoRole,                // `me` read, no role in the deployment
    Unknown,               // `me` or the document not read, ended or refused
    Hosts(&'a c::Deployment),
}

impl Builders {
    pub(crate) fn configure(&mut self, role: bool, next: &mut u64, out: &mut Queue<Request>);
    pub(crate) fn leave(&mut self, out: &mut Queue<Request>);
    #[must_use]
    pub fn read<'a>(&self, client: &'a Client) -> Read<'a>;
}

// pages.rs
impl Pages { pub fn builders(&self) -> Option<&builders::Builders>; } // while open
```

As `pages/accounts.rs` does, but on `me.deployment_role.is_some()`: jig gives the document to any role in the deployment (`jig/crates/jig-domain-core/src/documents/deployment.rs`, `deployment`). jig changes nothing.
