# Flows: acceptance cases

The cases of [`../diagrams.md`](../diagrams.md), section 8: for each,
the Markdown a model writes and what skein-ui must make of it, exactly.
The tests read them once the draft is built; until then they are its
contract for the look, so that what is built is what was drawn.

- A case's `.md` file: the fenced block, as an agent writes it.
- Its `.W.svg` files: the scene skein-ui's web mapping prints for it
  in a box `W` pixels wide (`../../ui/web/README.md`, section 6), byte for
  byte, but the final newline.
- Its `.fallback.txt` file: the drawing's name on its first line, empty
  for none, and its fallback's words on the second.
- `preview.html`: the scenes styled with temper's look, to see them.

The cases:

- `how-hello-works`: the configuring chat's flow, left to right. At
  640 pixels it is drawn left to right; at 600 left to right does not
  fit, and it is drawn top to bottom; at 240 neither fits, and the
  drawing code returns false, so the fallback shows.
- `review`: written top to bottom, drawn so at 400 pixels, with no
  captions and no name.
- `stays-code.md`: a flow that branches, past the first subset
  (`../diagrams.md`, section 2), stays fenced code, as today.
