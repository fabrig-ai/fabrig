# ui-account-again: the run's binding calls

The shapes and words that cross this plan's tasks. Each task builds its
half to this note and needs no other task's code: every client-side name
below (`c::Refusal::NameTaken`, `c::Refusal::NotOffered`,
`c::AccountChange::DeclareProgram`, `c::AccountChange::Resume`) is already
on the trunk.

Design: jig's `jig/docs/design/domain/accounts.md`, 3.4 (*A program
source's name*) and section 5 (the table's *declare a program's login*
and *order, uses, resume* rows); `jig/docs/design/client/domain/contract.md`,
section 4 (*name taken*, *not offered*); temper's
`temper/docs/design/client/tools.md`, 5.5 (*A refused account act says
why and what next*); `temper/docs/design/client/README.md`, 5.5 (*A
refused account act*).

## 1. Why the act was withheld

The accounts child keeps a removed program source's name as a durable
reservation (`programs::Issued`, `signed_out: true`) and refused any
declaration whose name matched a reservation:
`programs::declare` returned `DeclarationRefusal::Issued`, `changes.rs`
turned that into `Refusal::Uses`, and the core's
`routing/party/account_requests.rs` maps `Uses` to
`people::Refusal::NotOffered`: "This act is not offered", exit 2.

## 2. The accounts child (task `redeclare`)

- `jig_domain_accounts::programs::DeclarationRefusal`: `Issued` is renamed
  `Taken`, meaning a live source holds the name or another holder's
  reservation does. `Account(Refusal)`, `Name` and `Names` stay.
- `jig_domain_accounts::Refusal` gains one unit variant, `NameTaken`, after
  `Uses`, documented "A live source holds the program source's name a
  declaration gives, or another holder's reservation does (accounts.md,
  3.4)." `changes.rs` answers `DeclarationRefusal::Taken` with it;
  `DeclarationRefusal::Name` stays `Refusal::Uses`.
- A declaration whose name matches a reservation:

  | Reservation | Declaring holder | Answer |
  |---|---|---|
  | its source live | any | `Err(Taken)` |
  | retired, another holder's | any other | `Err(Taken)` |
  | retired, the declaring holder's, its number still in `domain.programs` (a sign-out under way) | the same | `Err(Account(Refusal::Limit { bound: Bound::Sources }))`, as `programs::check` refuses a concurrent operation |
  | retired, the declaring holder's, no operation under way | the same | admitted: a **revival** |

- **A revival** is a first declaration in every output, with three
  differences: the lifetime-name bound (`Names`) is not checked, since no
  reservation is added; the old reservation leaves `domain.issued` and its
  row is erased, `Request::Erase { key: Key::Issued { number: OLD } }`,
  pushed just before the new reservation's
  `Request::Save { record: Stored::Issued { number: NEW, holder, name, signed_out: false } }`;
  and `Request::CheckProgram { name }` is pushed last, so that each host
  checks the login now (the host's half checks a name when it is new to
  its list, and a revived name is not). `Request::Programs` then names
  the name once, `cleanup: false`. NEW is the new source's number, from
  `domain.take()` as for any declaration.
- Restart: the rows a revival leaves (one reservation per name, at the
  live source's number) restore as a first declaration's do.

## 3. The core and the wire (task `redeclare`)

- `jig/crates/jig-domain-core/src/routing/party/account_requests.rs`,
  `outcome`: `accounts::Refusal::NameTaken => people::Refusal::NameTaken`.
  `Provider`, `Kind`, `NotLimited` and `Uses` stay `NotOffered`.
- `jig/crates/jig-protocol-engine-clients/src/translate/accounts.rs`,
  `refusal` (a sign-in's end, never a declaration's):
  `a::Refusal::NameTaken => unreachable!("a taken name is only a declaration's answer")`,
  beside the `SignInRedirect` arm.
- `people::Refusal::NameTaken` and the client contract's
  `c::Refusal::NameTaken` keep their place and encoding; their doc
  comments add "or a program source's name a declaration gives is taken
  (domain/accounts.md, 3.4)". No codec, schema or generated file changes.

## 4. What a client may read from a refusal

From jig's `domain/accounts.md`, section 5, as these tasks rely on it:

| Ask | Refusal | Means |
|---|---|---|
| `AccountChange::DeclareProgram { provider, name, .. }` | `NameTaken` | a live source holds `name`, maybe the person's own, or another holder reserved it |
| `AccountChange::DeclareProgram { provider, .. }` | `NotOffered` | `provider` offers this holder no program's login |
| `AccountChange::Resume { source }` | `NotOffered` | `source` is not limited |

## 5. ctl's words (task `ctl-words`)

`temper/crates/temper-client-tools/src/jig/render/outcome.rs`. Exit
status 2, as any engine refusal. Text: `TOOL: WHY`, a newline, then NEXT,
where `outcome.rs` prints its refusal lines today. JSON: the refusal's
object as today, plus `"next":"NEXT"`. NAME and P are the declaration's
own name and provider number, from the ask (so a restored request prints
the same).

| Ask, refusal | TOOL: WHY | NEXT |
|---|---|---|
| declare, `NameTaken` | `account declare: program name NAME is taken` | `Declare under another name. If account list shows a source of yours named NAME, check its login instead: account verify ACCOUNT --source program` |
| declare, `NotOffered` | `account declare: provider P offers you no program login` | `me prints each provider's ways; declare with a provider whose line names programs` |
| retry, `NotOffered` | `account retry: the source is not limited, so there is nothing to resume` | `account list shows its state and what it needs` |

Every other refusal of these acts, and every other act's, prints as today.

## 6. The web's words (task `web-words`)

In the account act's outcome line, `temper-client-view`'s
`pages/accounts.rs` `outcome`, which `ConnectModel`'s panel draws too;
the words in `temper/crates/temper-client-view/src/accounts/words.rs`.
NAME is the declaration's name.

| Ask, refusal | Words |
|---|---|
| declare, `NameTaken` | `The name NAME is taken. Choose another name for this login.` |
| declare, `NotOffered` | `This provider offers you no program login. Choose another way to connect.` |
| resume, `NotOffered` | `This source isn't limited, so there's nothing to resume. Its state above is current.` |

Every other refusal draws `crate::cards::refusal` as today.

`ConnectModel`'s recovery (`temper-client-domain`'s
`pages/connect_model.rs`, `told`): a declaration refused `NameTaken`
whose name is a program source of the person's own with that provider
(`program_source`) sends that source's check, as it does on `NotOffered`
today; `NotOffered` no longer does. Recovered, the page draws no refusal.
