# The browser's layer: what an application writes

## 1. What an application writes

Its presentation, its browser shell and its server. Its extension points
in the tree are skein's (skein's `ui/README.md`): whatever tree it builds with
`skein-ui`, with the reference UI's primitives where it likes, or none
of them (`../../client/domain/view.md`). In a browser it writes its browser
shell (1.1), copied from a reference, its client host (1.2), its look
(1.3), its stylesheets, pictures and phrasebooks (1.4), and its server
(`serving.md`, section 1); what more it may change is 3.5.

### 1.1 Its browser shell

- **`<application>-client-shell-web`,** its own crate, ordinary Rust, as
  shells are (skein's `ui/web/README.md`, 3.1). Its start, through
  `wasm-bindgen`'s start attribute, which `skein-ui-web` re-exports so
  its version is the workspace's, makes skein's parts and those of jig's it
  takes, with the look (1.3), and starts its client host (`loop.md`, section 1);
  its loop is the turn (`loop.md`, 1.2). It orders the platform's work and decides
  nothing: what to show, send and keep is its client host's; carrying
  a request safely is a part's (`loop.md`, 1.3).
- **jig's reference shell,** `jig-client-test-shell-web`, jig's test
  application's, is written after skein's reference shell (the UI
  example's `skein-ui-example-shell-web`: skein's `ui/web/README.md`, 3.1;
  skein's `examples.md`, section 9), and adds `jig-client` through the
  client's adapter, the printed link's sign-in, and the reference UI
  with jig's look. It is written to be copied and commented as such:
  about 200 lines beside its client host (a guide).
- **Which to copy:** an application that takes the reference UI copies
  jig's; one with its own presentation may copy skein's and add the
  client's adapter alone (1.5, step 1).
- **The example of an application's three:** with its server in
  `jig-test-service` (`serving.md`, section 1; `../service.md`, section 7) and its
  page's policy (`serving.md`, 1.1), jig's reference shows a server, a page policy
  and a shell an application may copy, each of which it may write
  otherwise.

### 1.2 The client host

- **The application's:** its client root, `<application>-client-domain`,
  which holds `jig_client::Client` as a component beside its own
  children (`../../client/domain/root.md`); its presentation, the reference
  UI, its own view root, or both (`../../client/domain/view.md`); and its
  transport's client end, jig's `jig-client-protocol-http` in the
  reference (`../client/default-http.md`, 4). They step as one machine, what skein's
  `ui/web/README.md`, 3.1 calls the application's state machine. The client
  runs no loop of its own and never calls the application: the client
  root steps it with the events the shell's reports carry, and its
  transport requests, mapped onto HTTP, are among the host's step's
  duties.
- **Shared by every shell** the application uses: its terminal, over
  skein's text shell (`../../client/domain/view.md`, section 15), runs the
  same host; a native shell steps the same client.
- **Its functions,** which the browser shell calls, are skein's
  reference's (skein's `ui/web/README.md`, 3.1): *start,* *step,* *build,* the
  last tree, the path a link node leads to without acting, the next
  deadline, whether closing the tab would lose requests, and the
  language its phrasebook chose. jig's reference adds:
  - to *start,* the printed link's secret, when the page opened at the
    link's route (`sign-in.md`, section 1), with which the host signs the client in
    by the link, and otherwise by the transport's cookie;
  - to *step*'s duties, the client's transport requests as HTTP, in the
    order the client gave them, and a service's credential to show once
    (`sign-in.md`, section 1);
  - to the next deadline, the client's, and to the closing question, the
    client's pending requests.
- **jig's test application's client host,** its client root
  `jig-client-test-domain`, its view root `jig-client-test-view` with
  the reference UI, and `jig-client-protocol-http`, is the one jig's
  reference shell drives, and is copied with it
  (`../../client/domain/root.md`, section 12).
- **Effects** (skein's `ui/README.md`, section 11) are the application's: its
  client host gives those its presentation asks with a step's duties,
  and steps with each report; the reference UI asks none.

### 1.3 The look

skein's (skein's `ui/web/README.md`, 3.2; skein's `ui/README.md`, section 8), as plain
data the application's browser shell gives skein's page part when it
makes it. With the reference UI, jig's part of it, which
`jig-client-shell-web` gives whole:

- **jig's prefix and names:** the prefix `jig` and the names of jig's
  style tokens and region names (`README.md`, section 13), from `jig-client-view`,
  under jig's owner, whose number as skein-ui's opaque owner is jig's
  convention (`../../client/domain/root.md`, section 15);
- **jig's pictures,** which ship with `jig-client-shell-web`, under the
  bundle's `pictures/jig/` (`README.md`, 4.2);
- **jig's phrases** for the parts' own words (`loop.md`, section 1).

The application's browser shell adds its own owners' prefixes, names
and pictures, its drawing code (its client host's `draw`, routing a
drawing by its kind's owner: `loop.md`, section 4; skein's `ui/scenes.md`, 1), and its
elements and components (`loop.md`, 2.1), and gives the page part the whole. An
application without the reference UI gives its own alone. **Its name**
is in the title the shell sets (`loop.md`, section 3) and in the bundle's file
names (`README.md`, 4.2).

### 1.4 Its stylesheet, pictures and phrasebooks

- **Its stylesheet,** CSS, and **its pictures' directory** go to skein's
  bundler with its module (`README.md`, section 4); its stylesheet loads after
  `skein-ui.css`, and after `jig.css` where it takes the reference UI
  (`README.md`, section 13), or where its own page puts it (1.5, step 4).
- **Its phrasebooks are in its module,** data its presentation takes
  (skein's `ui/README.md`, section 9; `../../client/domain/view.md`, section 7).

### 1.5 What it may change

skein's steps (skein's `ui/web/README.md`, 3.3), with what is jig's in each,
and two of jig's: its page's policy (step 8) and its transport (step
10); each needing one before it only where it says.

1. **Copy a reference shell** and change its names: jig's, with the
   reference UI; or skein's, with `jig-client-shell-web`'s client
   adapter added (1.1).
2. **Restyle:** its stylesheet after `jig.css`, which may restyle jig's
   tokens and regions (`README.md`, section 13); its pictures (`README.md`, 4.2).
3. **Its paths,** wholly its codec's: with the reference UI, its codec
   for jig's places as is, under a prefix of its own, with its own
   segment names, or its own (`README.md`, section 9).
4. **Its page:** its own `index.html`, copied by skein's bundler with
   its marker (skein's `ui/web/bundle.md`, 3), loading the starter by a
   script file, with nothing inline: the policy refuses at run what
   breaks that (`serving.md`, 1.1), and the browser tier fails on any violation.
5. **Its files:** skein's (skein's `ui/web/README.md`, 3.3, step 5), served by
   its server, root files among them (`serving.md`, section 1).
6. **Its elements and components** (`loop.md`, 2.1): another element for a role
   carrying one of its style tokens; a component for such a node or a
   drawing of its kinds.
7. **Its JavaScript,** from the origin only: `wasm-bindgen`'s snippets,
   its own modules the glue imports, or files its page loads (step 4);
   never from another origin, never inline (skein's `ui/web/README.md`,
   section 14).
8. **Its page's policy,** its own above skein's strict policy (`serving.md`, 1.1):
   named origins for its data and other sites' pages, never a source of
   code or style beyond the origin.
9. **A part replaced or left out:** any of skein's or jig's. One that
   holds an invariant (the transport, the inbox, the page) moves it to
   the application (`loop.md`, 1.3), which skein's stories for a browser shell
   check; one that carries jig's client (the client's adapter, fetch,
   the sign-in part) leaves the mapping's requests, the browser header
   and the printed link to the application (`README.md`, section 10, and `sign-in.md`, section 1), which
   jig's stories for a browser shell check (`testing.md`). There is no
   shell to write whole: the application's browser shell is its own.
10. **Its transport:** jig's default mapping as it is, its guards
    wrapped (a check of its own added before them, a header added to
    its answers), or its own transport between `jig-client` and the
    clients' component at both ends, which takes over what the guards
    held (`serving.md`, 1.2) and which jig's attack stories check, if it runs them
    (`testing.md`).
