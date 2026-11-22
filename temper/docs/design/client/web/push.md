# Being pushed in the browser

The browser's push notifications are split from [the browser overview](README.md).

## 1. Being pushed

What the browser does for Telling you (`../pages/told.md`;
`../../domain/forge/attention.md`, section 7). Push is the browser's,
through its push service (Web Push); the page and the module stay as
they are, and two small root files carry the rest.

- **The service worker,** `/sw.js` (`serving.md`, 1, step 6): a fixed script of
  temper's, hand-written, its scope the origin's root. It does two
  things and nothing else, with no fetch handler and no cache, so the
  page loads as it does without it:
  - *a push* shows a notification: its payload's JSON (`../../domain/forge/attention.md`,
    7.2), its `title` the notification's title and its `body` its body,
    its tag `project-N` so that a newer count replaces an older one, its
    data the project; a payload it can't read shows "temper" and
    "Something needs you.", since a browser shows one for every push;
  - *a press on a notification* closes it and opens Needs you filtered
    by its project, `Inbox { project }`'s path (`../view.md`, 3),
    `/needs?project=N`, or `/needs` for data that names none: an open
    window of the origin, focused and sent there, or a new one.
- **The manifest,** `/manifest.webmanifest`: `name` and `short_name`
  "temper", `start_url` `/`, `display` `standalone`, and the paper's
  colour of the day look as `background_color` and `theme_color`
  (`look.md`), so that a phone may add temper to its Home Screen, which
  iOS asks before it pushes to a web app. The page links it, `<link
  rel="manifest">`, in its head: from skein's bundler's page if it takes
  one, otherwise added by the shell at its start.
- **The shell's part,** in `temper-client-shell-web`, through web-sys's
  service worker, push and notification bindings: it reports this
  browser's state to the domain at start and after each act (whether a
  service worker and a push manager exist, the permission, the current
  subscription's endpoint); and on "Turn on" it asks the permission,
  registers `/sw.js`, subscribes with `userVisibleOnly` and the
  deployment's key, and gives the domain the subscription's endpoint and
  keys, which the domain sends as `AddDevice`; on "Turn off", once
  `RemoveDevice` is answered, it drops the subscription. A browser asks
  permission only within a person's gesture: the shell asks it inside
  the press's own event, before anything waits.
- **Tested** below the browser where it can be: the page and its acts
  in the domain's and the view's tests, this browser's states given as
  the shell would report them; the two root files' routes and headers
  in `temper-service`'s; the shell's push calls and the service worker
  in a browser-tier story, written and compiled, and run after landing
  (`testing.md`).

## 2. Open questions

None.
