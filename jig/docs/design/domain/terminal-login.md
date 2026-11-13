# A program's login in a lent terminal

A person's program source (`accounts.md`, 3.4, *program*) is signed in
by the program's own sign-in, which the application's command runs on
the host that keeps the login, in a terminal the person holds (smith's
`protocol/programs.md`, 4.1). This document is how a client of the
person's that holds such a terminal lends it on the engine's ask, so
that the person starts that sign-in from a page and types nothing: the
client offers its terminal by a watch, the person's request asks it,
the client runs the application's command and says how it ended, and
the engine then declares or checks the source as the person's own
requests would (`accounts.md`, section 5). The engine relays an ask one
way and an exit status the other, and runs nothing.

## 1. In one page

- **The rule it keeps:** the program's own, unmodified sign-in, in the
  person's own terminal, and nothing of it through the engine
  (`providers.md`, 3.1 to 3.3; smith's `protocol/programs.md`, 4.1). No
  request, answer, document or log of this document's carries a code,
  a sign-in address, a redirect or a token: the ask names a provider
  and a source's name, and the answer an exit status.
- **Offered by a watch:** a client that can lend its terminal watches
  `terminal{n}`, its own number for it (2). While a watch of the
  party's is open, the party's `accounts` document says a terminal can
  be lent.
- **Asked by a request:** *log a program in*, the person's, under a
  source's name (3). The engine asks the party's newest terminal, by
  that terminal's document.
- **Ended by the client's report** of the program's exit, by the
  person's *cancel*, or by the terminal's watch ending (4). An exit 0
  declares the name, or checks the person's source of that name, and
  the host's check says whether it is signed in.
- **The contract** is section 5; what a client owes, section 6.

## 2. A terminal offered

- **The subject** `terminal{n}`: `n` a number the client draws for its
  terminal, once, from its seed, which tells its terminal from another
  of the party's. Its document is the login asked of that terminal while
  one is under way, and none otherwise (5). The party watches it, any
  sign-in of theirs; no one else may (`../client/domain/contract/README.md`,
  5.1).
- **The watch is the offer.** From its admission the party offers that
  terminal; its end withdraws it, whether the client closed it or its
  stream went (`../protocol/client/README.md`). A party offers at most
  `TERMINALS`, 4, at once: a watch past them is refused as a watch past
  its bound is.
- **`lendable`,** in the party's `accounts` document, is true while the
  party offers at least one terminal. A change of it touches the party's
  `accounts` watches.
- **The engine trusts the offer:** it cannot see where a client runs. A
  client offers only a terminal on the machine of the host that keeps
  its person's program logins (section 6; `accounts.md`, 3.4, *The host
  rule*).
- **Nothing is kept:** offers live in the engine's memory, and a restart
  forgets them; a client's watch, opened again after it, offers again.

## 3. A login asked

- **The request,** the party's, with the party's own as holder:
  *log a program in*, `AccountChange::LoginProgram { provider, name }`.
  It is checked, in this order, refused at the first that fails:
  1. as a declaration under that holder, provider and name would be
     (`accounts.md`, 3.4 and 5): a name out of its bounds, a provider
     that offers the holder no program's login (*not offered*), a name
     another holder's live source or reservation holds (*name taken*).
     The deployment as holder is refused *not offered*: a lent terminal
     is a person's;
  2. `NoTerminal`, the party offering no terminal;
  3. `LoginUnderWay`, a login of the party's under way: one at a time.
- **Asked:** the login takes a number from the child's counter, in
  memory, from 1 at each start and never reused while the engine runs,
  and is asked of the terminal the party offered last. It is answered
  `AccountLoginAsked { number }`. Its terminal's document then names it,
  and the party's `accounts` document names it as `lending`; while it is
  under way the party's `settled` is false.
- **A name of the party's own** live source is asked too: its end checks
  that source (4), so that a person signs a source in again by the same
  press.

## 4. Its end

- **Reported:** the party's `AccountChange::LoginEnded { number, exit }`,
  from any sign-in of theirs, for the login under way of that number;
  refused `NotLending` otherwise, as for a login already ended. It is
  answered `AccountChanged` once the end below is decided:
  - asked to cancel (below), whatever its exit: `Cancelled`;
  - `Exited { code: 0 }`: a live source of the party's under the name
    is checked, as the holder's check (`accounts.md`, 5, *check a
    program's login*); otherwise the name is declared, joining no
    account, as the holder's declaration (`accounts.md`, 5, *declare a
    program's login*), so that each host checks the new source's login
    (`accounts.md`, 3.4). Either ends `SignedIn { source }`; a
    declaration refused, the name taken meanwhile or the lifetime names
    spent, ends `Refused { refusal }`, its refusal;
  - any other exit: `NotFinished { exit: Some(exit) }`.
- **Cancelled:** the party's `AccountChange::CancelLogin { number }` for
  its login under way marks it cancelled, and its terminal's document
  says so, so that the client stops the program and reports (section 6).
  It is answered `AccountChanged`; refused `NotLending` otherwise. A
  login cancelled whose terminal is no longer offered ends `Cancelled`
  at once.
- **The terminal gone:** the end of the watch the login was asked of
  ends it `NotFinished { exit: None }`, or `Cancelled` where it was
  asked to cancel. A report for it after that is refused `NotLending`,
  and its client shows its own end regardless.
- **A restart** forgets a login under way, as it forgets the offers, and
  records no end for it.
- **The last end** of the party's is `lent_end`, `{ number, end }`, in
  the party's `accounts` document, in memory, until the next login of
  theirs ends; as `last_end` is for a sign-in (`accounts.md`, 5).
- **Usable is the host's to say.** `SignedIn` says the program exited 0
  and the source was declared or checked; whether the login is usable
  comes as any check's answer does (`accounts.md`, 3.4 and 3.5), and the
  `accounts` document lists the check under way until it does.

## 5. The contract

The client's contract (`../client/domain/contract/README.md`, 5.1;
`../client/domain/contract/documents.md`, 1;
`../client/domain/contract/requests.md`, 2), in `jig-client-domain`'s
words, and the client wire's schema carrying each form in place:

```rust
// watches.rs
Subject::Terminal { terminal: u64 }          // the client's own number for its terminal (2)
Delivery::Terminal(Box<Terminal>)

// accounts.rs
pub struct Terminal { pub login: Option<TerminalLogin> }
pub struct TerminalLogin {
    pub number: u32,
    pub provider: u32,
    pub name: Box<[u8]>,                     // the source's name, 1 to 32 bytes (accounts.md, 3.4)
    pub cancelled: bool,
}
// Accounts gains, beside `signing` and `last_end`:
pub lendable: bool,                          // a terminal of the party's offered (2)
pub lending: Option<TerminalLogin>,          // the party's login under way (3)
pub lent_end: Option<AccountLentEnd>,        // the party's last login's end (4)
pub struct AccountLentEnd { pub number: u32, pub end: LoginEnd }
pub enum LoginEnd {
    SignedIn { source: u32 },
    NotFinished { exit: Option<LoginExit> }, // None: its terminal went
    Cancelled,
    Refused { refusal: AccountRefusal },
}
pub enum LoginExit { Exited { code: u8 }, Killed { signal: u8 }, NotStarted }
// refusals.rs, a request's refusal, Refusal gains, as NameTaken and NotWaiting are:
NoTerminal, LoginUnderWay, NotLending,

// asks.rs, AccountChange gains:
LoginProgram { provider: u32, name: Box<[u8]> },
CancelLogin { number: u32 },
LoginEnded { number: u32, exit: LoginExit },

// outcome.rs, DocumentOutcome gains, beside AccountProgramDeclared:
AccountLoginAsked { number: u32 },
```

A request's refusals are `Refusal`'s, as the account requests' others
are; an end's `Refused` carries the declaration's refusal as the
`AccountRefusal` nearest it, as a sign-in's end does (`ProgramNameReserved`
for a name taken or reserved, `Limit` for the lifetime names spent).
`LoginExit::NotStarted` is a client that could not start the command:
a name it refused, a spawn that failed. The wire carries each new form
as the client wire's other forms are carried, its schema's layout
moving its version to 21 (`../protocol/client/README.md`, 6).

## 6. What a client owes

- **It offers only where it can lend:** a terminal the person holds, on
  the machine of the host that keeps the person's program logins, with
  no other program holding it.
- **It runs the application's own command,** from the application's own
  binary and directories, for the name the login names, which it checks
  against the name's bounds again (`accounts.md`, 3.4), taking nothing
  else from the document. The program's own sign-in holds the terminal
  while it runs, and the client reads nothing of it, draws nothing and
  keeps nothing.
- **One at a time:** while it lends its terminal it takes no other
  login, whatever its document names.
- **Cancelled,** it stops the command as a shell stops a job, and waits
  for its end.
- **It reports** the exit, or `NotStarted`, by `LoginEnded`, once, and
  shows its own end whatever the answer.

## 7. Tests

The accounts world (`testing.md`), with a client's terminal watch and
the party's requests: a watch offered and ended turning `lendable`; a
fifth watch refused; each refusal of section 3, in order; a login asked
of the newest of two terminals, the other's document naming none; an
exit 0 declaring a new name, and checking a live source of the party's,
each `SignedIn`; a declaration refused at the end, `Refused`; a non-zero
exit and a kill, `NotFinished`; a cancel marking the document, then the
report, `Cancelled`; a cancel with the terminal gone, `Cancelled` at
once; the terminal's watch ending, `NotFinished { exit: None }`, and its
late report refused; a report from another party refused; `settled`
false while under way; a restart forgetting both. The client wire's
round trip of each new form, and the client link's translation of each
both ways.

## 8. Open questions

- **Several terminals of one party:** the newest takes the ask. A
  person who keeps two clients in front of one engine has the older
  never asked; a choice between them waits for a client that offers
  more than one.
- **Where a client runs:** the engine cannot check that a terminal is on
  the host's machine (2). A client that offers one elsewhere makes the
  login there, and the host's check then says not signed in, which ends
  nothing worse than a sign-in left unfinished.
