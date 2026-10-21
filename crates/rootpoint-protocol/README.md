# rootpoint-protocol

The text protocol of rootpoint: commands, one per line, that build and change a presentation and
read its state back, so that an agent or a script can drive rootpoint, headless or, later, in the
running editor. Every reply is one line, starting with `ok` or `err`.

Its public API: `Session` runs commands on one document, its undo history and its view state;
`Command` and `parse_command` are the grammar; `Files` is the trait through which a session reads
and writes files, which its host implements.

The binary `rootpoint-headless` reads commands from stdin and writes the replies to stdout:

```text
$ printf 'add-slide\nsplit n2 beside 0.5\ntree\n' | rootpoint-headless
ok s1 root=n2
ok new=n3 split=n4
ok s1 n4:beside(0.5)[n2 empty | n3 empty]
```

The paths of `save`, `load` and `render` are relative to the working directory. An asset's path is
relative to the directory of the presentation last saved or loaded, or to the working directory
before either.
