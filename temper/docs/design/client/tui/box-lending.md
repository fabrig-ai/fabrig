# Lending the box front's terminal

Lending the box front's terminal is split from [the TUI in front of a box](box.md).

## 1. Lending its terminal

**Lending its terminal**, illustrative (`../../draft/claude-sign-in.md`,
3: t2, the terminal while lent, and t3, back; `claude-sign-in/`):

```
(a blank row, the screen's first)
temper: Claude Code signs in here. Approve it on Anthropic's page, in the tab it opens; if that page shows a code, paste it here. temper comes back when it's done, or when you press Cancel in the browser.
(a blank row)
(Claude Code's own sign-in, its screens its own)
```

```
 Setting temper up in your browser
   ✓ Signed in as the owner
   ✓ 1 Welcome
   ✓ 2 Forge
   ✓ 3 Model · Claude subscription, through Claude Code
   ▸ 4 Ready

 The steps are the browser's; this screen follows them.
```

The TUI follows the welcome's steps and takes none (*The welcome*,
above), but for this one act: on the engine's ask, after the browser's
"Sign in to Claude Code" (`../pages/welcome.md`, 1, *Claude Code's
sign-in*), it lends the terminal the person started `temper box` in to
`temper-box program-login`, as git hands a terminal to an editor, and
takes it back when Claude Code exits. It reads, draws and keeps nothing
of the sign-in (jig's `domain/terminal-login.md`, 6).

- **The offer,** in front alone: right after `Start`, after
  `Event::Launchpad` (`host.md`, 1), the host gives the root
  `Event::Lends { terminal }` once, `terminal` a number drawn from the
  host's seed, and the root keeps the watch `terminal{n}` of that number
  open for as long as the TUI runs (jig's `domain/terminal-login.md`,
  2). Out of front the TUI offers none: bare `temper` lends nothing
  (`../../draft/claude-sign-in.md`, 7, *Which TUI can lend*).
- **Asked:** after each report, while the TUI lends nothing, a login
  the root's terminal document names, not cancelled, of a number the TUI
  has not taken before, is the duty `lend`: its number, its NAME, and
  the line to print. The TUI's state holds the login it lends from then
  until its end; a login named meanwhile is not taken, and one the
  document stops naming is still lent until its end.
- **Stepping aside,** the run carrying `lend`, in this order:
  1. the terminal lent (skein's `ui/screen.md`, section 8, *Lent to a
     child*), and once `lent`, the kept modes set back (`Kept::set_kept`,
     skein's `runtime/shell-startup.md`, 1.3);
  2. the screen cleared and the cursor homed, `\x1b[H\x1b[2J`
     (`lending::CLEAR`), so that nothing of the shell's screen, which
     `LEAVE`'s `\x1b[?1049l` gives back, shows around the line or
     Claude Code's own (the user's request `tui-12`, 2026-11-15); then a
     line feed, a blank row above the line; then the line written to
     standard output, "temper: Claude Code signs in here. Approve it on
     Anthropic's page, in the tab it opens; if that page shows a code,
     paste it here. temper comes back when it's done, or when you press
     Cancel in the browser."; then two line feeds, a blank row between
     the line and Claude Code's first, "Opening browser to sign in…"
     (the user's request `tui-15`, 2026-11-15). The clear, the line feed,
     the line and its two line feeds are one write, `\x1b[H\x1b[2J\nLINE\n\n`,
     completed before the spawn. The clear erases the screen, never the
     scrollback (no `\x1b[3J`): the scrollback is the person's shell
     history, which a terminal such as emacs' vterm shows above a screen
     shorter than its window or pulls back as the window grows, and the
     blank row above the line keeps temper's line apart from it. Taking
     the terminal back is unchanged (*Back*);
  3. NAME checked against a source's name's bounds, 1 to 32 of `a` to
     `z`, `0` to `9` and `-`, then the child spawned: the box's own
     `temper-box`, the path its fetch found (`found PATH`, *The box's
     start*), with `program-login`, the box's directories as `--config
     CONFIG --data DATA --cache CACHE`, resolved from `temper box`'s
     arguments and environment as the box resolves them
     (`../box-binary.md`), and NAME; its descriptors 0, 1 and 2 the
     TUI's own (`Way::Inherit`, skein's `runtime/io/processes.md`,
     section 1); the captured environment and working directory, as the
     box's; in a process group of its own, which takes the terminal's
     foreground itself (`../../protocol/credentials-llm-accounts.md`, 1, *Its
     terminal's foreground*). Nothing from the engine but NAME reaches
     its arguments or environment. A NAME refused, a box with no path
     yet, or a spawn that fails is the end `NotStarted`, at once.
- **While lent** the TUI reads nothing and draws nothing: the keys are
  Claude Code's, and Ctrl+C there is Claude Code's, its group being the
  foreground. The engine's events, the box's lines and its stages still
  reach the TUI's state as they come, and the frame drawn once the
  terminal is back holds them. The box ending meanwhile leaves the child
  to its end; a termination signal of `temper`'s own sends the child's
  group `SIGTERM`, waits for its end and takes the terminal back before
  the run closes (`README.md`, 4).
- **Cancelled:** a login lent that the root's document marks cancelled is
  the duty `unlend`, once: the run sends the child's group `SIGTERM`
  (io's `Signal` with `Target::Group`) and waits for its end.
- **Back,** at the child's end, an exit or a signal: the run takes the
  foreground back (`take_foreground`), sets the modes raw again
  (`Kept::set_raw`), resumes the terminal (`Terminal::resume`), which
  draws the next frame whole, and reports `Lent { number, exit }`. The
  host gives the root the act that sends `LoginEnded { number, exit }`
  (jig's `domain/terminal-login.md`, 4), clears the login lent, and
  gives the TUI's notice (`pages/README.md`, 1, *The notice*): "Claude
  Code's sign-in cancelled" where it gave `unlend`, else "Claude Code
  signed in" at exit 0, else "Claude Code's sign-in didn't finish". The
  engine's answer changes none of them. Once the host's check says the
  source is usable, "3 Model" reads its detail (*Model's detail*, above).
- **`program-login` refusing** (no program configured, a directory not
  the person's): its one line shows in the terminal under the TUI's, and
  its exit 1 ends the login as any non-zero exit
  (`../../draft/claude-sign-in.md`, 7).
- **The seam** (`host.md`, 1) gains, for this alone:

  ```rust
  // temper_client_shell_tui::host
  pub struct Lend {
      pub number: u32,
      pub name: Box<[u8]>,                        // the source's NAME, as the document names it
      pub line: Box<[u8]>,                        // the TUI's line, its words, no line feed
  }
  // Duties gains
  pub lend: Option<Lend>,                         // the terminal to lend for this login
  pub unlend: bool,                               // the child to stop, the login cancelled
  // Report gains
  Report::Lent { number: u32, exit: jig_client_domain::contract::LoginExit },
  ```

  and the root, `temper-client-domain`'s, gains `Event::Lends { terminal:
  u64 }`, a read of its terminal document's login, and the act that
  reports a login's end with `LoginEnded`, its holder the person's own.
- **The TUI's state** holds, beside the rest: the login lent, its number
  and NAME, and whether `unlend` was given. **Its words**, the line and
  the three notices, are its own (*Its words*, below).
- **The recording:** `Report::Lent` is the record `lent`, `NUMBER EXIT`,
  EXIT `exited N`, `killed N` or `not-started`, and the duties are
  recorded as any (`journeys.md`, 3.2): `lend` with its number and NAME,
  never its line, and `unlend`. A replay gives `lent` to the host as the
  run reported it.

## 2. Open questions

None.
