# Contracts: plans between any two layouts

The shapes the changes of `temper/docs/design/draft/layouts-both-ways.execution.md`'s goal L1 share and no other area reads (`temper/docs/design/draft/design-economy.md`, 4.3). They bind as any delivered design section does. The goal's last change deletes this document and its status.

## 1. The schema's tombstones and marks

Made by the grammar's change, read by the plans' tombstones change (`run-time.md`, 1.3; `README.md`, section 3):

```rust
impl Schema {
    pub fn tombstones(&self, record: DeclIx) -> (FieldIx, u32); // its tombstones' first index and count in the field table,
                                                                // after every record's fields; a count of 0 for an
                                                                // enumeration or a record with none
    pub fn droppable(&self, at: FieldIx) -> bool;               // the field's `droppable` mark; false for a tombstone
}
```

- `field`, `default`, `default_span` and `was(Named::Field(_))` answer for a tombstone's index as for a field's: its name, line and type; its default's encoding; its former names.
- A record's tombstones are in the text's order.

## 2. The layout's derivation

Made by the modes' change, built on by the tombstones change and the round trips' change: the public items of `layouts.md`, section 4, as that section gives them. Nothing else crosses: the derivation's internal policy and tables are the modes' change's to shape, and the tombstones change works inside them.
