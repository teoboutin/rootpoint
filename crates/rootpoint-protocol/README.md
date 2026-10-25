# rootpoint-protocol

The text protocol of rootpoint: commands, one per line, that build and change a presentation and
read its state back, so that an agent or a script can drive rootpoint, headless or in the running
editor, through its control socket. Every reply is one line, starting with `ok` or `err`.

Its public API: `Session` runs commands on one document, its undo history and its view state;
`Command` and `parse_command` are the grammar, and `read_line` the rule for one line that every
host follows; `Store` is where a session finds its presentation's files; `Files` is the trait through which a session reads and writes files, which its host
implements; on Unix, `control::Connection` and `control::send` are the client of a running
editor's control socket.

The binary `rootpoint-headless` reads commands from stdin and writes the replies to stdout:

```text
$ printf 'add-slide\nsplit n2 beside 0.5\ntree\n' | rootpoint-headless
ok s1 root=n2
ok new=n3 split=n4
ok s1 n4:beside(0.5)[n2 text [] style="Body" | n3 text [] style="Body"]
```

The binary `rootpoint-ctl`, on Unix, sends commands to a running editor's control socket and
prints each reply as it comes:

- `rootpoint-ctl <workspace-dir | socket>` reaches the window that holds the workspace, by its
  `.rootpoint/socket`, or the socket at the path; `rootpoint-ctl list` prints the name of each
  window's socket that answers.
- `rootpoint-ctl <socket> <words>…` sends one line: the words joined with single spaces, so a line
  holding a quoted text is given as one word, `rootpoint-ctl /tmp/rp 'text n3 "two words"'`.
- `rootpoint-ctl <socket>` sends the lines of its stdin, a batch, over one connection; a line that
  is not UTF-8 is sent as it is, and gets its `err` reply. The editor runs them one a frame. When
  the author takes control during a batch, the next edit gets the refusal, which tells the agent
  to stop and wait for the author's instructions, the editor ends the connection, and the batch's
  other lines are not sent.
- It exits 0 when every reply is `ok`, a blank line or a comment getting none; 1 when a reply is
  `err`, also when the editor then ends the connection, as after a take-over; 2 when the socket
  cannot be reached, or stdin or the connection fails before any `err`, as when the editor is
  closed. A line holding a newline is refused before it is sent.
- The host paths of `open`, `open-copy`, `open-pack`, `pack`, `unpack`, `import` and `render` are
  relative to the editor's working directory, not to the client's; an asset's path is relative to
  the store's root, as below.

`control` asks for the authority to edit a running editor, and `release` gives it back, refused
while the client holding it has a group open; headless, `control` replies `ok`, since a headless session has one
controller.

A session reads its presentation from a store. `open <dir>` opens a workspace: `presentation.ron`
at the directory's root, or an empty presentation when it has none, and each asset a file at its
path below the root; `save` writes `presentation.ron` there. `new`, `sample` and `open-copy <dir>`,
which copies a directory's presentation and its assets' files, open in memory: the assets' bytes are
the session's own, and `save` replies `err` until a pack holds the presentation. `store` replies
`ok memory unsaved=<n>` or `ok workspace <root> unsaved=<n>`, the root absolute. `open` and
`unpack`, in `rootpoint-headless` and in the window, take the lock of the workspace, which keeps it
to one process: another one's open replies `err <dir> is open in another process, pid <pid>`.
In `store`'s reply, `n` counts the undo
steps made, undone or redone since the store was opened or saved, or, in memory, written as a pack.
The host paths of `open`, `open-copy`, `open-pack`, `pack`, `unpack`, `import` and `render` are
relative to the working directory.

`pack <file>` writes the presentation and its assets' files as a pack, a zip file of extension
`.rpk`: first an entry `mimetype` holding `application/x-rootpoint`, stored, then
`presentation.ron`, deflated, then each asset's file at its path, stored. Its reply names, after
`missing=`, the assets it had no bytes for, `-` for none. `open-pack <file>` opens a pack in memory,
and refuses one whose first entry is not that `mimetype`, one with no `presentation.ron`, and one
that holds a symlink, an entry whose name is not an asset path, or a file that is another's
folder. In memory, after `open-pack` or `pack`, `save` writes that pack
again, and `store` names it, `pack=<file>`. `unpack <dir>` writes a presentation held in memory into
an empty or absent directory, as `presentation.ron` and its assets' files, and the store is then a
workspace there. Any unzip tool also extracts a pack into a workspace, its `mimetype` file aside.

`reload` reads `presentation.ron` at a workspace's root again and makes it the document, one undo
step; it replies `err` when the file does not load, and in memory.

`import <host-path> [<asset-path>]` adds a file as an asset, and replies its id. In a workspace, a
file below the root is added at its path there, with no copy. Any other file is copied into the
store, at `<asset-path>` or at the root under its file name; a name whose file holds other bytes
takes a suffix, `-2`, `-3` and on, before the extension, and one whose file holds the same bytes is
used as it is. A file an asset already names adds no asset: the reply is that asset's id.

An asset keeps its id through the asset commands. `asset-move <a> <asset-path>` moves its file to
a path no file and no other asset holds; `asset-relink <a> <asset-path>` points it at a file the
store holds, which repairs a missing asset; `asset-replace <a> <host-path>` gives its file the bytes
of a host file; `asset-remove <a>` removes an asset nothing names, and with `into=a<n>` makes every
image, text, span and style that names it name `a<n>` first; `asset-remove-unused` removes every asset
nothing names, in one undo step, and replies their ids. A removed or replaced file goes to the
trash, `.rootpoint/trash/` in a workspace, from which `undo` takes it back; the next `open` of the
workspace empties it. `assets [all|unused|missing|untracked]` lists the assets, each
`a<id> <path> <bytes|missing> uses=<areas and styles|->`, separated by `; `, a span style written
`span:"<name>"`, `missing` for a file
it cannot read or a path outside the root, or the paths of the store's files that no asset names,
which `pack` leaves out, but for those whose path has a component that starts with `.`, as
`.git/`. The undo of an `import` takes the file it wrote out again. A move or a replace of a file
two assets name, which only a file edited by hand holds, is refused.

An asset path is always inside the root: it has no empty, `.` or `..` component, does not start
with `/`, holds no `\`, and has a first component other than `.rootpoint`, `presentation.ron` and
`mimetype`. No command
reads or writes another path, nor a file a symlink places outside the root: a file edited by hand
that names one still loads, and `render` draws the asset as a missing one; `import` to such a path
replies `err`.

`image` and the `font` option of `text` refuse an asset the document does not hold, with
`err no asset a<n>`, so no command names a missing asset. `duplicate-slide` and `swap` copy or move
an area as it is, so an area that names a missing asset a file brought keeps naming it. A file
edited by hand that names one, by an id below its `next_id`, still loads: `render` draws the image
grey, or the text in the built-in sans, and its reply names the asset after `missing=`, or the
area after `fallback=`.

A text names a style of the document's table, `text n3 "Hi" style="Title"`, or holds its own
values, `custom`; `tree` prints which. Any other option of `text` changes the values the text
draws with, so a named text becomes custom and the style itself is unchanged. `style add`, `style
set`, `style rename` and `style remove` change the table, a name always quoted; `styles` lists it,
each style as the options `style add` takes, but for a built-in family the binary does not carry,
which only a file edited by hand holds, written as its name. `style remove` is refused while a text names the style,
unless `into="<other>"`, another style, or `into=custom`, a copy of the removed values, says what
those texts take instead. A command naming a
style the table lacks is refused; a file naming one loads, its text drawn in the default values,
and `render` names the area and the style after `styles=`.

A text is a tree of characters and spans, each span with its id, `t<n>`, and its style. `text`
takes the whole tree on one line, `text n3 ["Let x be a ", {bold "prime"}, " number"]`: `[` its
items separated by commas `]`, each a quoted text or a span, `{` its options, then its items `}`;
a quoted text alone is a tree of one run, and spaces around the brackets, the braces and the commas
are free. `tree` prints each span with its id first, `{t41 bold "prime"}`, and `text` gives every
span of its tree a new id, ignoring any given, so `tree`'s line given back to `text` copies a text,
but for what only a file edited by hand holds: a custom span that sets no value, which `text`
refuses, and a built-in family the binary does not carry, which `tree` leaves out.
`text` replies the new spans' ids. With options and no tree, `text n3 bold` changes the area's style
alone and keeps its spans and their ids; an image area refuses it. Outside a quoted text, `[`, `]`,
`{`, `}` and `,` are words
of their own, so a path holding one is quoted.

The commands that change a tree name a text node, the root of an area's text by the area's id or a
span by its own, and an offset or a range in it, read at that moment, a span counting as one
offset: `insert <n|t> <offset> "<text>"`, `delete <n|t> <a>..<b>`, a span in the range removed
whole, and `wrap <n|t> <a>..<b> <options>`, which puts the range in a new span and replies its id.
`unwrap t<n>` puts a span's content in its parent, `span t<n> <options>` changes its style, and
`split-span t<n> <offset>` cuts it in two spans of its style, replying the second's id, refused at
the span's edges. A span's options are `text`'s but alignment, which is the area's alone: values,
`style="<name>"` for a span style of the table, `custom`, and `<key>=inherit` for `font`, `size`,
`bold`, `italic` and `colour`, which clears a value so the span draws with its parent's. They apply
in order over the span's current style, so a value set on a named span makes it custom. A custom
span that would set nothing is refused: `unwrap` removes the span instead. `span-style add`,
`span-style set`, `span-style rename` and `span-style remove` change the table of span styles, as
the `style` commands change the text styles, `add` and `set` taking a span's values and
`<key>=inherit`, and `span-styles` lists it, each entry with the values
it sets, separated by `; `. A command naming a span style or a font asset the document lacks is refused; a file naming
a missing span style loads, its span drawn with its parent's values, and `render` names the span and
the style after `span-styles=`.

An equation area holds a formula, written in LaTeX, and its number. `equation <area> $<formula>$`
makes an area an equation, or changes the formula of one, `equation n3 $\frac{a}{b}$`: the formula
is read between `$` signs with no escape, up to the first `$` after an even number of `\`, so `\$`
writes a dollar and `\\` just before the closing `$` is a row break. Its options apply over the
equation's own: `numbered=yes|no`; `size=fit`, the formula at the largest size its area holds, or
`size=<n>`, at `n` slide units, shrunk only where that does not fit; and `colour=#rrggbb[aa]`. A new equation is
numbered, fitted, and in the Body style's colour, or black with no Body style. `tree` prints an equation as `equation` takes it,
with all its options, then its number, `(2)`: the numbered equations count from 1 in the order of
the slides, then of each slide's tree, so moving a slide renumbers and dragging a divider does not.
A command the math subset does not know is drawn red, and `render` names its area after
`commands=`.

`sample` opens the built-in sample in memory, a presentation of six slides that says how the
editor's gestures work, how a text is styled, where a presentation is saved and what an equation
area draws; the crate holds its RON text and its image, in `sample/`, so it opens in
every host with no file.
