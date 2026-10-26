# rootpoint

A minimal presentation editor with an interactive GUI, written in Rust: a document model, a slide
renderer, a text protocol, and the editor, in a window and in a web browser.

You make a slide by cutting it into areas, and each area holds one piece of content: a text, an
image or an equation. There is no markup to learn and no build step: you edit on the slide itself,
and present from the same program.

## Try it

The editor runs in a web browser, with nothing to install: <https://teoboutin.github.io/rootpoint/>.
It opens the built-in sample, a short presentation whose slides explain the editor. Every change
stays in the page until you download it with "Save as pack…". The page keeps no copy when it is
closed or reloaded, and it asks before it closes while a change is not downloaded.

## Build and run

With a Rust toolchain installed:

```text
cargo run --release -p rootpoint-app                  # an empty presentation, in memory
cargo run --release -p rootpoint-app -- <dir>         # a directory, as a workspace
cargo run --release -p rootpoint-app -- <file>.rpk    # a pack, in memory
```

The button "Sample" of the toolbar opens the built-in sample. `cargo x web` builds the browser
version into `target/web/`, which any static file server serves, for example
`python3 -m http.server -d target/web`.

## How a slide is built

- **A slide is a tree of splits.** A new slide is one area. Splitting an area cuts it in two,
  side by side or one above the other. Each part can be split again, and a divider between two
  parts can be dragged. Layouts such as a title over two columns, or a figure beside a stack of
  notes, are a few splits.
- **An area holds one piece of content**: a text, an image or an equation. The side panel's
  Selection tab changes what the selected area holds and its options.
- **A text has a style.** A named style, such as "Title" or "Subtitle", is shared by every area that
  names it, so changing it in the Styles tab changes them all. A text can also be "custom", with
  values of its own. Inside a text, a span style, such as bold in a colour, marks a few words.
- **An equation** is a formula, numbered or not, either in an area of its own or inside a text. A
  text can refer to a numbered equation, and the reference shows its number.
- **Images and fonts are assets** of the presentation. The side panel's Assets tab lists them and
  shows where each is used.

## A first presentation

1. Start the editor with no argument, or open the page, and add a slide with the button "Add" of
   the slide list.
2. Click the slide to select its area. Press Shift+↓ to split it: a new area appears below.
3. Select the upper area, and choose the style "Title" in the Selection tab. Double-click it and
   type a title. Esc leaves the text.
4. Select the lower area, and split it with Shift+→ into two columns. Type a text in one. Select
   the other, add a picture with "Add an image…" of the Selection tab, and choose the content
   "image".
5. Drag a divider to give the columns other widths.
6. Press F5 to present. → and ← change slide, and Esc returns to the editor.
7. Save: Ctrl+S, or "Save as pack…" in the browser.

## Keys and gestures

| to | do |
| --- | --- |
| select an area | click it, or its row in the outline of the side panel |
| select its parent, a child or a sibling | Alt+←, Alt+→, Alt+↑, Alt+↓ |
| split the selected area at its middle | Shift and an arrow; the new area is on that side |
| split it where you click | hold Ctrl (a vertical cut) or Shift (a horizontal cut), and click; add Alt to snap |
| remove the selected area | Delete |
| resize two areas | drag the divider between them; a double-click puts it halfway |
| swap or move an area | drag the grip in its top-left corner onto another area |
| open the menu of an area | right-click it |
| edit a text | double-click it, or click the selected text area; Esc leaves |
| bold, italic | Ctrl+B, Ctrl+I, on the selection or on what you type next |
| insert a formula in a text | Ctrl+M |
| make the selected area an equation | Ctrl+Shift+M, or Ctrl+Alt+N for a numbered one |
| undo, redo | Ctrl+Z, Ctrl+Shift+Z or Ctrl+Y |
| see the slide as it will be shown | the button "Preview" |
| present | F5; → or Space for the next slide, ← for the previous, Esc to stop |

In a formula, letters, digits and operators insert themselves, `^` and `_` make a superscript and
a subscript, and `\` starts a command such as `\frac` or `\alpha`, Tab completing its name. While a
formula is edited, the Selection tab shows a palette of its structures and of every command, by
category.

## Where a presentation is saved

- **A workspace** is a directory with `presentation.ron` at its root and the presentation's images
  and fonts below it. Open a directory with Ctrl+O. A workspace saves itself after every change.
  `presentation.ron` is a text file, which you can read, edit by hand or keep in git.
- **A pack**, a `.rpk` file, holds a presentation and its assets in one file, to carry it to
  another computer. "Open pack…" opens one, and "Save as pack…" writes one.
- In the browser, there is no workspace: Save downloads a pack. The label at the right of the
  toolbar always says where the presentation is held, and whether it is saved.

## Driving it from a script or an agent

Every action of the editor is a command of a text protocol, one per line, so a script or an AI
agent can build and change a presentation. `rootpoint-headless` runs the commands of its standard
input with no window. On Unix, `rootpoint-ctl` sends them to a running editor, which shows each
change as it is made; the editor's banner "Take control" gives the authority to edit back to you.
The protocol is described in [the protocol's README](crates/rootpoint-protocol/README.md).

The complete description of the editor, every action and every key, is in
[the editor's README](crates/rootpoint-app/README.md).

## Layout of the repository

- `path@rootpoint@docs/`: the goals, the design and the open issues of the project, and
  `path@rootpoint@docs/plans/` for work that is designed and not built.
- `path@rootpoint@crates/rootpoint-app/`: the editor, the binary `rootpoint`, natively and in a
  web browser.
- `path@rootpoint@crates/rootpoint-math/`: formulas, their LaTeX form and their layout from a math
  font.
- `path@rootpoint@crates/rootpoint-model/`: the document model, which runs headless and in the
  browser.
- `path@rootpoint@crates/rootpoint-protocol/`: the text protocol, and the headless binary that
  runs it.
- `path@rootpoint@crates/rootpoint-render/`: the slide renderer, which draws a slide at any size.
- `path@rootpoint@tools/xtask/`: the maintenance tool. `cargo x gates` runs every check a branch
  owes before it merges, and `cargo klarch` runs the document checker at the version the project
  pins.

## Licence

rootpoint is licensed under either of `path@rootpoint@LICENSE-APACHE` (Apache License 2.0) or
`path@rootpoint@LICENSE-MIT` (MIT licence), at your option. The fonts and the data built into the
binary keep their own licence or notice, whose text sits beside each file.
