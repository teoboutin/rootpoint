<!-- The report of the toolkit survey run on 2026-10-20 for the design discussion of the
milestone of the first prototype, on Rust GUI toolkits and slide renderers. Kept as the subagent wrote
it; the milestone document quotes its facts. Its scratchpad paths no longer exist. -->

# GUI toolkit and renderer survey for a Rust presentation editor

Date: 2026-10-20. Versions and dates come from the crates.io API. Repository activity comes from the GitHub API. Crate sources were downloaded to `.../scratchpad/toolkits/crates/` and read there.
V = VERIFIED: I read the source, the manifest or the docs. A = ASSUMED: from prior knowledge, not checked in this pass.

## Toolkits

**egui / eframe 0.37.0 (2026-10-15).** Releases come about monthly (0.36.0 2026-08-05, 0.36.2 2026-09-08). Apps: Rerun (A).
- Wasm: eframe has a first-class web target (A, well known). egui has `wasm32` deps (V).
- Text: epaint has its own stack: harfrust shaping, skrifa outlines, and glyphs rasterised by vello_cpu into an atlas (V, `epaint/src/text/font_face.rs`). For display, `LayoutJob` takes several sections, each with its own `TextFormat` (A). `TextEdit` takes a custom layouter, so styled text can be edited, but its model is plain text (A).
- Headless: `egui_kittest::Harness::render() -> RgbaImage` under the `wgpu`/`snapshot` features (V). It needs a wgpu adapter. On CI, a software Vulkan driver such as lavapipe is the usual way to get one (A).
- Wakeup: `Context::request_repaint()` can be called from any thread (`Context` is `Send+Sync`, A). Repaint callbacks are `Send+Sync` (V, `context.rs:443`).

**iced 0.14.0 (2025-12-07).** Releases are slow: 0.13 came out 2024-09. The repository is active (pushed 2026-10-20). Apps: COSMIC desktop, which uses a fork (A).
- Wasm: a `webgl` feature, and wasm deps in iced_winit (V).
- Text: cosmic-text in iced_graphics (V). The wgpu and tiny-skia renderers therefore share one layout engine. `text::rich` displays spans (V, `widget/text/rich.rs`). `text_editor` takes one style plus a highlighter, not rich editing (A).
- Headless: `iced_test` has a "headless mode" `Simulator::snapshot` and `screenshot`. The renderer is picked by `ICED_TEST_BACKEND`, and tiny-skia does screenshots (V).
- Wakeup: a `Subscription` fed from a channel or stream (A).
- Fit: `pane_grid` is a split tree with draggable dividers (V, widget exists). That matches requirement 2 for the editor UI.

**Slint 1.18.1 (2026-09-21).** A minor release comes about every 2 to 3 months (V). It is commercially backed by SixtyFPS. Licence: GPLv3, royalty-free or commercial (V, file header).
- Wasm: wasm32 deps in core (V). Browser demos exist (A).
- Text: Parley, through a shared layout module `textlayout/sharedparley` (V). All renderers use it. The software renderer uses swash (V). `StyledText` can be built from markdown, for display (V). Rich editing in TextInput: A, probably absent.
- Headless: the software renderer has `render(&mut [TargetPixel], stride)` into a buffer, used with `MinimalSoftwareWindow` (V). It needs no GPU.
- Wakeup: `slint::invoke_from_event_loop(FnOnce + Send)` (V, `api.rs:1286`).
- Risk: custom painting is limited to Path/Image items. A free-form slide canvas would be built from declarative items, or drawn into an image that Slint shows (A).

**xilem / masonry 0.4.0 (2025-10-29).** Releases come every 5 to 6 months. The repository is active (2026-09-14).
- Wasm: masonry_winit has no web deps (V, absence in its manifest). `xilem_web` is a separate DOM backend (V, README), where the browser lays out the text. So there is no shared canvas in the browser.
- Text: Parley, drawn with vello (V). Widgets: `text_area`, `prose` and `split` (V).
- Headless: `masonry_testing` `Harness::render() -> RgbaImage`. It goes through vello with `use_cpu: true`, but still needs a wgpu device (V, `harness.rs:458`).
- Wakeup: winit `EventLoopProxy` and xilem async worker views (A).

**makepad.** makepad-widgets 1.0.0 on crates.io dates from 2025-05-13. Main is at 2.0.0, unreleased (V), and very active (pushed today).
- Text: its own stack, built on vendored rustybuzz, ttf-parser and an SDF rasteriser `sdfer` (V).
- Wasm: a flagship target (A).
- Headless rendering: none known (A).
- Risks: a custom DSL, few external apps (A), and crates.io releases that lag the code.

**gpui 0.2.2 (2025-10-22).** The crates.io release is stale. Zed itself uses gpui from git.
- Web: `gpui_web` has been on main since 2026-02-26 ("GPUI on the web", #50228), with 60 commits since (V). It is unpublished. It enables `parking_lot` `nightly` and `wasm_thread` by default (V), so it implies nightly Rust and threaded wasm (A).
- Text: cosmic-text on Linux (V), and platform text elsewhere (A). Rich display: `StyledText` (A).
- Headless rendering to an image: A, not found.
- Wakeup: `AsyncApp`/executors (A).
- App: Zed.

**dioxus-native / blitz.** dioxus 0.7.10; 0.8.0-alpha.3 came out 2026-10-16. blitz 0.3.0-beta.4 came out 2026-10-16, and its README says "beta" (V).
- Stack: Stylo, Taffy, Parley and anyrender (V). The anyrender backends are vello, vello_cpu, vello_hybrid, skia and svg (V).
- Wasm: runs on `wasm32-unknown-unknown` with `anyrender_vello_hybrid` and the `webgl` feature, with bundled fonts (V, HOWTO_WASM.md).
- Headless: `examples/screenshot.rs` uses `render_to_buffer` with `VelloCpuImageRenderer` (V).
- Risk: slides become HTML/CSS. In-place rich editing would need contenteditable, which is not established (A).

**floem 0.2.0 (2024-11-14).** There has been no release in about 2 years, but the repository is active (2026-09-24).
- Main uses Parley 0.7 and has vello, tiny_skia, vger and skia renderers (V). The renderer has wasm deps (V).
- App: Lapce.
- Risk: you would depend on git.

## Renderers and text stacks

| crate | version | kind | wasm | text |
|---|---|---|---|---|
| vello | 0.11.0 (2026-10-02) | GPU compute via wgpu | WebGPU only (A) | `Scene::draw_glyphs` (V); fed by parley/skrifa |
| vello_cpu | 0.3.0 (2026-10-02) | CPU, `png` and `text` features by default (V) | pure Rust (A) | `glyph_run` (V) |
| vello_hybrid | 0.2.0 (2026-08-07) | CPU and GPU | WebGL2 (V, used by blitz) | same API family |
| tiny-skia | 0.12.0 (2026-02-02) | CPU | yes (A) | none: "The main missing feature is text rendering" (V, README) |
| femtovg | 0.27.0 | GL/wgpu | WebGL (V, deps) | own text: rustybuzz and swash (V) |
| parley | 0.12.0 (2026-10-09) | layout | yes | rich spans, `PlainEditor` with editable styles (V), fontique |
| cosmic-text | 0.19.0 (2026-04-22) | layout | yes (A) | harfrust and swash (V), `Editor` with attribute spans (A) |
| swash / skrifa | 0.2.10 / 0.48.0 | scaling and raster / outlines | yes | the low layer under the others |
| ab_glyph | 0.2.32 (2025-09) | raster | yes | no shaping (A); unsuited to good typography |

## Best combinations

1. **eframe as host, plus a separate slide renderer: Parley with vello_cpu or vello_hybrid.**
   - Mature host, with wasm and wakeup from any thread.
   - Slides are laid out once in slide units and drawn with a uniform transform.
   - The same code renders a PNG headless with vello_cpu, with no GPU.
   - The editor chrome uses egui's own text. That is acceptable, since chrome never needs to match the slides.
2. **iced alone (option a).**
   - cosmic-text is the single engine for both the GPU renderer and the headless tiny-skia renderer.
   - `pane_grid` and `iced_test` screenshots are built in.
   - Weak points: rich editing is limited, releases are slow, and uniform scaling of text needs re-layout at each scaled size.
3. **Slint (option a).**
   - Parley is shared by every renderer.
   - Headless software rendering needs no GPU.
   - `invoke_from_event_loop` covers wakeup, and wasm works.
   - Weak points: the declarative item model and the licence.

Blitz has the cleanest "same stack everywhere" story (Parley and vello everywhere), but it is in beta and imposes an HTML model. Masonry/xilem and gpui fail the browser requirement today, or need nightly.

## Key trade-off: (a) toolkit painter vs (b) separate renderer

- **(a)** has less code. Editing, caret, IME and selection come for free from the toolkit's text widget.
  - Cost: the slide's look is bound to the toolkit's text engine. That engine lays out at pixel sizes, so "uniform scaling" means re-laying out at size × scale. With hinting and pixel rounding, wrap points can move between window sizes (A).
  - Headless output is only as good as the toolkit's offscreen path: wgpu for egui and masonry, CPU for iced and Slint.
- **(b)** lays out once in slide units and applies only an affine transform. The editor, the presenter and the PNG/PDF export then share one layout by construction.
  - Cost: in-place editing must be built on the slide engine: Parley `PlainEditor` driving caret, selection and IME. The toolkit then only forwards key and IME events and composites an image or texture.

## Concrete risks

- **Two text engines on one slide.** A toolkit `TextEdit` overlaid on a slide drawn with Parley gives different line breaks and advances. The caret and the text then misalign when editing starts. Any in-slide editing must use the same layout object that draws the slide.
- **Font fallback differs by platform.** fontique/fontdb system discovery exists natively, but there are no system fonts in the browser (V, blitz HOWTO). Load every font explicitly and disable system fallback, or native and web will differ.
- **Hinting and pixel snapping** break scale invariance. Disable hinting for slide text.
- **The GPU and CPU rasterisers are not pixel-identical.** For example, vello on GPU and vello_cpu differ in anti-aliasing (A). Golden-image tests should use one backend, the CPU one.
- **vello on GPU in the browser needs WebGPU.** vello_hybrid with WebGL2 is young (0.2). vello_cpu uploaded as a texture is the safe fallback, but costs CPU at 4K presenter size (A, not measured).
- **API churn.** Parley went 0.10 to 0.12 in 4 months and vello 0.8 to 0.11 in 7 months (V, release dates). Pin versions. egui embeds vello_cpu 0.2 while vello_cpu 0.3 is current (V), so the dependency tree will hold duplicate versions.

## Decided
None.

## Met outside the task
None.
