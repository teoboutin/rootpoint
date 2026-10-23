# rootpoint-protocol

The text protocol of rootpoint: commands, one per line, that build and change a presentation and
read its state back, so that an agent or a script can drive rootpoint, headless or in the running
editor, through its control socket. Every reply is one line, starting with `ok` or `err`.

Its public API: `Session` runs commands on one document, its undo history and its view state;
`Command` and `parse_command` are the grammar, and `read_line` the rule for one line that every
host follows; `Files` is the trait through which a session reads and writes files, which its host
implements; on Unix, `control::Connection` and `control::send` are the client of a running
editor's control socket.

The binary `rootpoint-headless` reads commands from stdin and writes the replies to stdout:

```text
$ printf 'add-slide\nsplit n2 beside 0.5\ntree\n' | rootpoint-headless
ok s1 root=n2
ok new=n3 split=n4
ok s1 n4:beside(0.5)[n2 text "" | n3 text ""]
```

The binary `rootpoint-ctl`, on Unix, sends commands to a running editor's control socket and
prints each reply as it comes:

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
- The paths of `save`, `load` and `render` are relative to the editor's working directory, not to
  the client's; an asset's path is relative to the directory of the presentation last saved or
  loaded, as below.

`control` asks for the authority to edit a running editor, and `release` gives it back, refused
while the client holding it has a group open; headless, `control` replies `ok`, since a headless session has one
controller.

The paths of `save`, `load` and `render` are relative to the working directory. An asset's path is
relative to the directory of the presentation last saved or loaded, or to the working directory
before either.

`image` and the `font` option of `text` refuse an asset the document does not hold, with
`err no asset a<n>`, so no command makes an area name a missing asset. A file edited by hand that
names one, by an id below its `next_id`, still loads: `render` draws the image grey, or the text in the built-in sans, and its
reply names the asset after `missing=`, or the area after `fallback=`.

`sample` opens the built-in sample, a presentation of three slides that says how the editor's
gestures work; the crate holds its RON text and its image, in `sample/`, so it opens in every host
with no file.

A `save` leaves every asset readable beside the saved file. An asset whose file lies inside the
saved file's directory, at any depth, keeps its file, and its path is rewritten relative to that
directory. Any other asset, and an embedded one such as an image of the built-in sample, is copied
into that directory under its file name, or `asset-<id>` and an extension for embedded bytes; a
name whose file holds other bytes takes a suffix, `-2`, `-3` and on, so no file is overwritten. A
save that cannot read an asset it must copy replies `err`, naming it, and writes nothing.
