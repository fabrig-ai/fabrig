# Names: the goal's contracts

The shapes the changes of the names goal share, and no other area reads (`../../draft/design-economy.md`, 4.3): names on templates, charters and policies (`../../draft/ways.md`, 9; jig's `domain/engine/settings.md`, 1, Names). They bind as any delivered section does. The goal's last change deletes this document and its status.

## 1. A template's shown name

- **jig's domain:** `jig_domain_core_vocabulary::settings::Template`'s display text, `name: Box<[u8]>` today, is `shown: Box<[u8]>`, its meaning unchanged.
- **The client's:** `jig_client_domain::contract::Template.shown` and `SettingLine.shown`, both the display text their `name` carries today, and the client wire's `Template.shown` and `SettingLine.shown` with them.
- **The settings form:** a template's member `shown` in `temper-settings-form`'s JSON, read and printed; a template object with no `shown` member is the earlier form, its `name` read as `shown` (jig's `protocol/configuration-templates.md`, 1). jig's document reads a template the same way.
- **The store:** no layout moves for this: the stored template keeps its display field under its stored name, mapped to `shown`.

## 2. Names

- **The vocabulary** (`jig-domain-people`'s `settings` module): `NAME_BYTES: u32 = 32`; `is_name(bytes: &[u8]) -> bool`, a name's form whole (1 to `NAME_BYTES` bytes of `a`-`z`, `0`-`9` and `-`, the first a letter); `SettingFault::Name`.
- **jig's domain:** `name: Option<Box<[u8]>>` on `jig_domain_core_vocabulary::settings::Template` and `Charter`, and on `jig_domain_core_vocabulary::ConfiguredPolicy`; `None` for an item with no name.
- **The core's lookup:** `jig_domain_core::Core::charter_named(&self, name: &[u8]) -> Option<u32>`, the number of the charter the newest settings offer under `name`, with defaults' names filled at their numbers.
- **The client's:** `jig_client_domain::contract::Template.name`, `SettingLine.name` (the item's name, a charter's, template's or policy's), and `Me.charters`, one entry per named charter the settings offer, each with `number: u32` and `name`; their types are the change's.
- **The settings form:** a member `name` on a charter's, a template's and a policy's value, absent for none.
