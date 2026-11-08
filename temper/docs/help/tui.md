# The TUI

Kind: concept

The TUI is temper's full-screen terminal client: closing keeps its sign-in, and the line client can end that sign-in.

## What it is

The TUI has no sign-out action. Closing it keeps its sign-in for the
next start. To end it, use the line client with the same directory
and engine.

- `DIR`: your existing TUI directory, `$XDG_STATE_HOME/temper/tui`
  when `XDG_STATE_HOME` is absolute, else `$HOME/.local/state/temper/tui`.
- `HOST`, `PORT`: the engine's loopback HTTP address and port that
  the TUI signed in to, for example `127.0.0.1` and `8367`.

## What temper guarantees

`/exit` or Ctrl+C closes the TUI and keeps the sign-in. On the sign-in
screen, use Ctrl+C. `q` types a character on every screen.

## Acting on it

Close the TUI, then run `temper open DIR http://HOST:PORT` with the
existing TUI directory and the engine's loopback HTTP address. This opens the line client
with the kept sign-in; choose **Sign out**. This reuses and ends the
existing sign-in rather than creating another.

`Signed out.` appears once the engine has ended the sign-in.
Close the line client with `q` after signing out.

## Limits

If the client says the sign-out did not reach the engine,
restore the connection and choose **Sign out** again. A client that
says it is not signed in has no kept sign-in to reuse in that directory.

Start the TUI again when you want to sign in. For the command forms,
read `temper help commands`.
