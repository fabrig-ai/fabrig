# ui-goal-page: the run's binding shapes

What crosses tasks. A task builds to these; a name here is never another task's to change.

## When a task was made (jig)

On `jig_domain_core`'s document `Task`, the wire's `Task` and `jig_client::contract::Task`, each directly
after `last_activity`:

```rust
/// When the task was made: its record's `created_at` (jig/docs/design/client/domain/contract.md, 5.1).
pub made: Wall,
```

The wire's field is `made: u64`, as `last_activity` is, in nanoseconds. An ended task's document has it too:
it comes through the same function. It has no other reader in jig.
