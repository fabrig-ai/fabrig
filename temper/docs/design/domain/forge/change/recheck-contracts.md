# The recheck's contracts

The shapes the goal *A repair that changes nothing gets one more check* shares between its two changes, the landing's and CI's, and that no other area reads (`../../../draft/design-economy.md`, 4.3). The rule they build is `repairs.md`, 1, *A repair that changes nothing is checked once more*, and the stored forms are there and in `../../landing-regate.md`, 5. The goal's last change deletes this document and its status.

## 1. In the change's policy

In `temper-engine-domain-forge-change`'s boundary, made by the landing's change and read by CI's:

- **`Hold::RedStands`,** after `Runner`: a repair that changed nothing whose one check failed again, or can't be made. The policy holds it with `was` the state `Repairing { requested: false, why }`, `why` the red's reason: the landing's handed failure as `Landing` or `LandingAt` asks it, or `Ci` at CI. Its release restores that state and resets the repairs, resolutions and updates as any hold but `Runner` does, so the step after it asks the fresh repair. `held_on_ci` reads it as CI's where that `why` is a CI reason; the landing's change returns false for it, and CI's change makes it true there.
- **`Facts::recheck: Recheck`,** the top's projection, each step, of the change's stored record (the regate, or `ci_recheck`) for the change's last head:

  ```rust
  pub enum Recheck {
      /// No check is owed at the change's last head.
      None,
      /// Asked and not taken: a red read now is not its measurement.
      Owed,
      /// Taken: the red this step reads is its measurement.
      Taken,
  }
  ```

  At CI, `Owed` with CI failed at the head is waited for as pending CI is, and `Taken` with CI failed there holds `RedStands`. At a landing, `Taken` holds `RedStands` where the failure the landing's ask hands is its taken regate's receipt, the top projecting `Taken` only then, and `Owed` leaves a handed failure an ordinary red. The landing's change computes it from the regate and `None` at CI; CI's computes it from `ci_recheck`.
- **`Facts::regate: bool`:** the change's landing repair's reason resolves to a failed receipt (`../../landing-regate.md`, 3). Where it is false, a landing repair that changed nothing holds `RedStands` at once. At CI, `Facts::rerun` already says the same of a job.

## 2. In the forge's top

In `temper-engine-domain-forge`, a module `recheck` of the landing's change holds the hold's account, read by CI's:

- `pub(crate) fn stands_again(head: Commit, repair: u64, keys: &[Box<[u8]>]) -> Box<[u8]>` and `pub(crate) fn stands_unchecked(head: Commit, repair: u64, why: &[u8]) -> Box<[u8]>`, `Commit` skein's `api::Commit`: the two accounts of `repairs.md`, 1, *Red again, it holds* and *Where the same content can't be checked again*, within jig's `HOLD_ACCOUNT_BYTES`, cut as *The words* there says. The top writes the one it chose into the row's hold words (`producer_failure`) in the step that holds.
