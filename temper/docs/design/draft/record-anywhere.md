# Recording on any deployment

Draft, 2026-11-12, from a session with the user that day. The
browser's recorder runs only on a loopback origin, and only when the
page's address asks for it (`../client/web/README.md`, 2.1;
`../client/web/journeys.md`, 3.1). The user: "I don't see any reason
to not enable that on any deployment (I need it for dev.fabrig.ai for
example)", and "we should add a command line flag that enables that
automatically without the url trick". A recording made on a deployment
that lives on holds a working sign-in until `add` redacts it
(`../client/web/journeys.md`, 3.6). The user chose that the recorder
leave it out when it saves: "Let's go for the version that strips the
sign-in secret, that's enough." What agents wrote may stay in a
recording: "it's okay if the recording contains agents text." The user
asked for it to be built (2026-11-12).

It changes `../client/web/README.md`, 2.1 and 2.2;
`../client/web/journeys.md`, 3.1, 3.5 and 3.6; `../protocol/sign-in.md`,
7; `../protocol/binaries.md`'s commands; and `cli.md`, 6.

## 1. In one page

- **On any origin** (section 2): the page honours `internal-record` on
  every origin. `internal-small-window` and
  `internal-loopback-navigation` stay loopback's, and the recorder no
  longer needs the latter.
- **By a flag** (section 3): `--record`, on each command that serves
  the browser's page, makes every page the deployment serves record
  from its load, with nothing added to its address.
- **No sign-in in the file** (section 4): each save leaves out the
  printed link's secret and the credential a sign-in stores, as `add`
  replaces them, so that a saved recording is never a sign-in.
- **Bounded** (section 5): a recording keeps at most the client's
  bound, as every tab of such a deployment records for as long as it
  is open.

## 2. On any origin

- **`internal-record=1`** turns the recorder on at any origin, `http`
  or `https`. The recorder adds no node, no style and no request, and a
  recording leaves the browser only by the person's own Alt+Shift+S,
  into their own downloads (`../client/web/journeys.md`, 3.1), so a
  link that sets it on someone else's page takes nothing from them.
- **The other two stay loopback's:** `internal-loopback-navigation`
  elsewhere would let a page send a person's browser to plaintext
  services on their own machine, and `internal-small-window` is the
  browser tier's.
- **The recorder runs on `internal-record` alone.** The journeys tool
  keeps giving both, as its fake box needs loopback navigation.
- **As today,** each setting is read once at the page's start and taken
  out of its address (`../client/web/README.md`, 2.1).

## 3. By a flag

- **`--record`** on `temper-box`, `temper-box serve` and `temper-engine
  serve` (`../protocol/binaries.md`), off by default and named by their
  help.
- **The page's meta:** with it, temper-service makes the bundle's page
  at start with `<meta name="temper-record" content="on">` before its
  `</head>`, as it adds `temper-sign-in` (`../protocol/sign-in.md`, 7).
  The web shell reads the meta at start and runs the recorder when the
  page has it, as if its address had `internal-record=1`.
- **Every load records:** a reload, or a return from a forge's or a
  sign-in's departure, starts a new recording
  (`../client/web/journeys.md`, 3.1), where the address's setting lasts
  one load.
- **Every person's page records,** signed in or not, in their own
  browser alone. Nothing of it reaches the deployment.

## 4. No sign-in in the file

- **At each save,** Alt+Shift+S's and the one before a departure
  (`../client/web/journeys.md`, 3.1), the file is written with two of
  3.6's originals replaced as `add` replaces them, one `x` a byte: the
  *link*, the `secret` record's value; and the *stored*, each `stored`
  record's value and each `save` of the stream's record, a `Kept`'s
  head standing. The web shell already links `temper-client-redaction`.
- **The memory keeps them,** so that the recording goes on as it is;
  only the file leaves them out.
- **Nothing else is left out:** a typed value, such as a key pasted
  into the page, and what agents wrote stay for `add`, whose redaction
  and last scan don't change (3.6 and 3.7). On a real deployment, as on
  a real box, the person types no key (3.5).

## 5. Bounded

- **A recording in memory** keeps at most a bound of bytes, one of the
  client's limits in a browser (`../client/web/README.md`, 2.1,
  `limits`), derived as the others are.
- **Past it,** the recorder ends as at a departure: it marks the end as
  the page is drawn and keeps nothing more. A save then gives what it
  kept.

## 6. Where it lands

- `../client/web/README.md`, 2.1: `internal-record` on any origin, the
  other two loopback's; 2.2: the `temper-record` meta.
- `../client/web/journeys.md`, 3.1: on by `internal-record` or the
  meta, the save's two replacements, the bound; 3.5: a recording by
  hand on any deployment with real accounts, the box serving its own
  bundle, as no recording bundle is built since the internal settings
  replaced the bundle's features (`cli.md`, 6); 3.6: the file already
  holds the link and the stored replaced.
- `../protocol/sign-in.md`, 7: `temper-record` among the page's metas.
- `../protocol/binaries.md` and each command's help: `--record`.
- `cli.md`, 6: the internal settings' line.
