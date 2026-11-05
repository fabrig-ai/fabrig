# project-landing

The intended walk is on a fresh `temper box --fake-backends`: the box's
printed sign-in link opened, the welcome walked through, Forgejo run on
this machine with an empty `work/hello`, and the ChatGPT plan signed in
through the fake sign-in. Ready offers "Hello"; "Create your project"
creates it and adopts `work/hello`. Once Home says "work/hello is in
Hello", reloading `/projects/1` starts the recording there.

The fresh walk stops at `/welcome?at=ready`: the fake ChatGPT sign-in
returns there, but Ready says "Model none yet" and "Create your project"
is disabled. It also says "Try later" and offers "Continue with ChatGPT"
again. Forgejo is connected as temper, `work/hello` is made, and the
project's offered name is "Hello". No project can be created yet.

The offered retry reaches the Model step with "Signing in to ChatGPT"
and "Pending". Reopening the browser at Ready shows "OpenAI refused
temper’s sign-in. Check the configuration’s registration." A further
sign-in returns to Ready with "Model none yet" and "Try later" again.
The last page reached is `/welcome?at=ready`, with "Hello" offered and
"Create your project" disabled; `/projects/1` has not been reached.

The kept recording starts at `/welcome` and holds no act. It has not
been replaced with a Home recording.

- `01.html`: the welcome, as the kept recording ends; it does not show
  the intended project's Home or the fresh walk's blocked Ready page.

The `project-home` look request draws Home in Hello, with `work/hello`
adopted. It cannot equal this Welcome print while the walk is blocked.
`temper-client-gallery` draws the Home implementation through its
`home-three-left`, `home-two-left`, `home-one-left`, `home-ready`,
`home-adopting`, `home-refused`, `home-no-repository`, `home-github`,
`home-after-skip`, `home-skip-refused`, `home-needs-under-begin`,
`home-begin-settings-left` and `home-begin-one-chats` states. They are
hand-made deliveries to the client, not a recording of the box. The recording gap is `../../journeys.status.md`, 9.
