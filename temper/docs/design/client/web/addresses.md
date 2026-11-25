# The browser's addresses

The browser's addresses and navigation are split from [the browser overview](README.md).

## 1. Addresses

- **Each page's address is a path,** by temper's address codec, a
  module of `temper-client-domain`, step code, total and bounded:

  | Page | Path |
  |---|---|
  | `Inbox { filter }` | `/needs`, and its filter's query |
  | `Home { project }` | `/projects/<project>` |
  | `AllHome` | `/home` (`../pages/README.md`, *Home on All projects*) |
  | `Changes { project }` | `/projects/<project>/changes` |
  | `Work { project, filter }` | `/projects/<project>/work`, and its filter's query |
  | `Settings { project }` | `/projects/<project>/settings` |
  | `Repositories { project }` | `/projects/<project>/settings/repositories` |
  | `Models { project }` | `/projects/<project>/settings/models` |
  | `Builders { project }` | `/projects/<project>/settings/builders` |
  | `Role { project, charter }` | `/projects/<project>/settings/roles/<charter>` |
  | `Rule { project, kind, number }` | `/projects/<project>/settings/rules/<kind>/<number>` |
  | `Ask { project }` | `/projects/<project>/ask` |
  | `Goals { project, filter }` | `/projects/<project>/goals`, and its filter's query |
  | `Team { project, role }` | `/projects/<project>/team`, and `?role=N` for `Some(N)` |
  | `Chats { filter }` | `/chats`, and its filter's query |
  | `NewChat { draft }` | `/chats/new`, and `?draft=1` with a draft |
  | `Accounts` | `/accounts` |
  | `Members` | `/deployment/members` |
  | `Limits` | `/deployment/limits` |
  | `Chat { task }` | `/chats/<task>` |
  | `Task { task }` | `/tasks/<task>` |
  | `Adopt { project }` | `/projects/<project>/adopt` |
  | `Welcome { step }` | `/welcome`, and its step's query |
  | `Forges` | `/forges` |
  | `ConnectForge { forge }` | `/forges/connect`, and its choice's query |
  | `ConnectModel { way }` | `/accounts/connect`, and its way's query |
  | `Authorize { query }` | `/oauth/authorize`, and the query as the address carried it, within jig's bound on an authorization's query (`../../protocol/openid-provider.md`, 3) |
  | `Approve { code }` | `/terminal`, and `?code=` the code as shown, `WDJB-MJHT` (`../pages/README.md`, *The approval page*) |

  `<task>` and `<project>` are the task's and the project's numbers in
  decimal. A later page's path comes with it, in this shape
  (`/projects/<project>/board`). The
  public pages' paths, under `/public`, are `public.md`'s.
- **The root, `/`, opens Home,** never Needs you, whatever waits (the
  user's ask of 2026-11-18). It is no page's address: the codec reads `/`
  whose query sets no field of the inbox's filter (below) as
  `Place::Root`, and the client opens, once `me` is read, the page
  `temper_client_domain::pages::rail::root_home(me: Option<&Me>) ->
  Address` names: `Home { project }` for a person in exactly one
  project, `AllHome` for one in two or more or in none, and for no `me`.
  The address is then replaced with that page's; until then it stays `/`,
  and the main is empty and busy, as at every address while `me` is read
  (`../view.md`, 1). `/` whose query sets a field is an inbox link from
  before Needs you had its own address: it opens Needs you with that
  filter, its address replaced with the printed one,
  `/?kind=question&goal=9` with `/needs?kind=question&goal=9`. The
  terminal opens at the inbox as before (`../line.md`, 10).
- **The inbox's filter is its query:** the fields it sets, in one
  order, `?project=<project>&kind=<kind>&goal=<goal>`, and none with
  every field unset, so that `/needs` is the inbox unfiltered and
  `/needs?kind=question&goal=9` one filtered by kind and goal. `<project>`
  and `<goal>` are numbers in decimal, as a path's; `<kind>` is one of
  `proposal`, `question`, `held` and `person-task`: the kind's button
  word (`../pages/README.md`'s inbox), in lower case, its space a hyphen.
  The codec reads the fields between `&`, each `name=value`, and drops
  one it doesn't know, one whose value doesn't read as its field's
  (`kind=reply`, `project=07`, a number past its type), and one named
  again after a first that read; the rest set the filter. So a stale
  or edited link opens the inbox less filtered, never refused, and its
  address is replaced with the printed one. It reads numbers, not the
  person's projects: a project the person isn't in shows an empty
  inbox, which its unfiltered address undoes. The inbox's filter, the chats
  list's (below), the welcome's step,
  `ConnectForge`'s choice, `ConnectModel`'s way (below),
  `Approve`'s code (below) and
  `Authorize`'s query, kept as it came and never printed again, alone
  are in an address, and another page's query is dropped, `/chats/7?x=1`
  replaced with `/chats/7`.
- **Work's filter is its query,** as the chats list's: none for All, so
  that `/projects/<project>/work` is Work whole, and `?show=drafts` for
  Drafts. A `show` that doesn't read as Drafts, or a field of another name,
  is dropped, and the address replaced with the printed one. Work's other
  filters are their pages' addresses, Goals' and Changes' (`../pages/README.md`,
  *Work*).
- **The chats list's filter is its query,**
  as the inbox's (the user's pick, 10-23): none for "All", so that
  `/chats` is the list whole; `?state=live` for "Live",
  `?state=drafts` for "Drafts" and `?state=closed` for "Closed". A `state` that
  doesn't read as one of the page's, or a field of another name, is dropped,
  and the address replaced with the printed one. `/tasks/9?state=live` reads as
  `/tasks/9`. A filter chosen is a move (below).
- **The goals' filter is its query,** as the chats list's: `/projects/<project>/goals`
  is the list whole; `?state=needs` for "Needs you", `?state=live` for
  "Under way" and `?state=ended` for "Ended"; any other value reads as the
  list whole and prints without it. Choosing one is a move.
- **The welcome's step is its query:** none for its first step,
  Welcome, so that `/welcome` is the welcome and decides by the store's
  state; `?at=welcome` for that step asked by its link, which stays on it
  unless the state is whole; `?at=forge`, `?at=model`
  and `?at=ready` for the others (`../pages/README.md`, The welcome). As
  the inbox's filter, an `at` that doesn't read as a step, or a field
  of another name, is dropped, and the address replaced with the
  printed one: `/welcome?at=done` opens the welcome at `/welcome`.
  Its Forge step adds its choice, as `ConnectForge`'s (below):
  `/welcome?at=forge&forge=3`; its Model step its way, as
  `ConnectModel`'s: `/welcome?at=model&provider=1&way=key`.
- **`ConnectForge`'s choice is its query:** none at Choose, so that
  `/forges/connect` is the flow's first step; `?forge=github`,
  `?forge=owned` or `?forge=managed`, a kind to make, GitHub, Forgejo
  on this machine or Your Forgejo; or `?forge=N`, N a kept forge's
  number in decimal (`../forge/deployment.md`). A value that doesn't read as
  one, or a field of another name, is dropped, and the address
  replaced with the printed one. It reads a number, not the
  deployment's forges: a number that names no forge shows Choose.
- **`ConnectModel`'s way is its query:** none at Choose, so that
  `/accounts/connect` is the flow's first step; `?provider=N&way=W`,
  N an LLM provider's number in decimal and W `sign-in`, `program` or
  `key`, in that order, for the way of that kind its provider offers
  (`../pages/README.md`, `ConnectModel`). The two go together: one without
  the other, a value that doesn't read as its field's, or a field of
  another name, is dropped, and the address replaced with the printed
  one. It reads a number and a kind, not the configured providers: a
  way they do not offer shows Choose. A sign-in's way may add its
  sign-in's source, `&source=S`, S a source's number in decimal, once the
  sign-in is answered, its history entry replaced (`../pages/README.md`,
  `ConnectModel`, A reload while it waits):
  `/accounts/connect?provider=1&way=sign-in&source=7`. A source on
  another way, or one that doesn't read as a number, is dropped as
  above.
- **The approval page's code is its query:** `/terminal` is the page
  asking for a code, and `?code=WDJB-MJHT` carries one, the code as
  shown, four letters, a hyphen and four (`../pages/README.md`, *The
  approval page*). The codec reads the first `code` field whose value
  reads by jig's `ApprovalCode::parse`: eight letters of the code's
  alphabet in either case, hyphens and spaces ignored, within 32 bytes,
  with no percent-decoding; it drops one that doesn't read, one named
  again after it and a field of another name, and the address is
  replaced with the printed one: `/terminal?code=wdjb-mjht` with
  `/terminal?code=WDJB-MJHT`, `/terminal?code=AEIO` and
  `/terminal?x=1` with `/terminal`; so is a code whose read answers a
  `typed` ask, which only a typed code approves. Its printing,
  `address::print_approve(code: Option<ApprovalCode>)`, `/terminal`
  for none, is the shown form, upper case, and the only one: the TUI makes its tab's address from the engine's
  address and this path alone (`../sign-in.md`, 1.1). The place is no
  page's address, as `Authorize` is none: the pages' links never lead
  to it, and the person reaches it by the address alone. The code is
  no secret (jig's `protocol/client/sign-in.md`, *Its risk*), and
  the server's `Referrer-Policy: no-referrer` (`serving.md`, 1) keeps it from any
  other site the page might link to. temper's server answers `GET
  /terminal` with the page, as it answers every path that is no route
  of its own (`serving.md`, 1).
- **A filter chosen is a move** (below), with its history entry: a
  link's, at the page's top with the focus on its heading, and back
  returns to the filter before. Like any move it folds the open
  disclosures (`look.md`). Needs you has no filter menus; its filters remain
  address fields (`../pages/README.md`). A move
  that stays on the chats list, between `Chats` and `NewChat`, or on one
  goal's page, changing its filter alone, keeps the page: its composers' words, the
  closed chats and ended delegates it read, which it reads not again;
  the inbox's filter, a query the engine answers, reads its page anew.
- **A path that is no page** opens Home as the root does, and the
  address is replaced with that Home's (draft-2's answer of 2026-11-18:
  a broken path lands where the root does). The codec never prints a path under `/jig`,
  `/bundle/`, `/test/` or `/forges/N/github/`, temper's routes for
  GitHub's steps (`../forge/served.md`, 1), nor `/robots.txt`.
- **Moving** is skein's (skein's `ui/web/README.md`, section 7): a link pressed
  is a move within the client, with a new history entry; back and
  forward are moves too, each entry keeping its scroll; a reload, or an
  address typed or opened, is a new client at that address (jig's
  `protocol/browser/README.md`, section 10). Back or forward to an entry
  of a client left by such a load is a new client at its address, or,
  where the browser kept that client, its return (skein's
  `ui/web/README.md`, 7): the client leaves
  the page it showed and comes back to it, so what goes when the person
  leaves a page has gone, as Accounts' completion line
  (`../pages/accounts.md`, 1, and `../pages/accounts-words.md`, 1). That a client opens at the inbox
  (`../root.md`, 1.4) is the terminal's: in a browser the address is the
  browser's, and the client keeps no record of its own.
- **A heading drawn after its page** (the answer of 2026-11-06 to the
  `ui-composer-focus` task `composer-focus`): a new entry whose level-1 heading
  isn't drawn yet, as a chat's isn't until its task is read, has the focus on the
  main (skein's `ui/web/patches.md`, 1). When the heading is drawn, the shell moves the focus from the
  main to it, unless the focus has moved meanwhile: a focus placed since keeps its
  place, even back on the main, and a focus request ends the wait. While
  `me` is read the main is empty and marked busy (`../view.md`, 1), so a
  new entry then has the focus on it, and the page's heading drawn once
  `me` is read takes it alike, the page's main being that node (skein's
  `ui/README.md`, 7). A new entry with no main yet waits for its
  main alike: when the main is drawn, its heading takes the focus, or the
  main until the heading comes, unless the focus has moved or a focus
  request came meanwhile.

- **Internal links** resolve through the completed view's bindings. The
  signed-in shell supplies the view's revision to the page part: no patches
  and the same revision leave its links alone. A new revision can change
  their paths even when their visible nodes did not change.

## 2. Open questions

- **The terminal approval page after landing** (`addresses.md`, 1, and `sign-in.md`, 1 and `look.md` and `gallery.md`, 2):
  check `/terminal` in a signed-in browser on a fresh box, its typed-code
  entry and the twelve `approve-*` gallery states at narrow and wide
  widths. Domain, view and shell tables and gallery prints establish no
  rendered browser result. The TUI's own flow and walk follow
  `../../draft/tui.execution.md`, M2.
- **A heading drawn after its page, in skein's reference** (`addresses.md`, 1): the kit's rule
  stops at the main (skein's `ui/web/patches.md`, 1), and any application whose page reads
  before it draws its heading meets what this shell repairs. Default: this shell's
  alone, until a second application writes the same.
