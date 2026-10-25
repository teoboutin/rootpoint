//! What the browser build does through the page: the pickers that read a presentation directory,
//! a pack or an asset's file, the reading of a file dropped on the window, the download of a pack,
//! full screen, and the question before the page unloads. A picker reads its files in a task of the
//! page, after the frame that asked for them, so it puts them in an inbox that `App::frame`
//! empties, waking the window when they are read.

use std::cell::RefCell;
use std::rc::Rc;

use eframe::egui::Context;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen_futures::JsFuture;

/// The files a picker read.
pub(crate) enum Picked {
    /// Each file of the directory, by its path under the directory picked, that directory's name
    /// its first component.
    Directory(Vec<(String, Vec<u8>)>),
    /// An image, by its file name, or a file dropped on the window.
    Image(String, Vec<u8>),
    /// The new bytes of an asset, by the name of the file picked.
    Replacement(rootpoint_model::AssetId, String, Vec<u8>),
    /// A pack, by its file name.
    Pack(String, Vec<u8>),
    /// A file that could not be read, and why.
    Failed(String),
}

/// What a picker asks for, which names what its file becomes.
#[derive(Clone, Copy)]
enum Purpose {
    Directory,
    Image,
    Pack,
    Replacement(rootpoint_model::AssetId),
}

/// The page's side of the app.
#[derive(Default)]
pub(crate) struct Web {
    /// The window's context, which a picker wakes once its files are read.
    ctx: Option<Context>,
    inbox: Rc<RefCell<Vec<Picked>>>,
    /// Whether the page was seen full screen since presenter mode asked for it.
    fullscreen_seen: bool,
}

impl Web {
    pub(crate) fn set_context(&mut self, ctx: &Context) {
        if self.ctx.is_none() {
            self.ctx = Some(ctx.clone());
        }
    }

    /// What the pickers read since the last call.
    pub(crate) fn take(&mut self) -> Vec<Picked> {
        std::mem::take(&mut *self.inbox.borrow_mut())
    }

    /// Ask for a directory, with the browser's directory picker.
    pub(crate) fn pick_directory(&mut self) {
        self.pick("", Purpose::Directory);
    }

    /// Ask for one image or font file.
    pub(crate) fn pick_image(&mut self) {
        self.pick("image/png,image/jpeg,.ttf,.otf", Purpose::Image);
    }

    /// Ask for a pack, a `.rpk` file.
    pub(crate) fn pick_pack(&mut self) {
        self.pick(".rpk", Purpose::Pack);
    }

    /// Hand bytes to the page as a download under `name`: a `Blob` of them, an object URL, and a
    /// click on an `<a download>` element, then the URL is revoked.
    pub(crate) fn download(&mut self, name: &str, bytes: &[u8]) {
        let fail = |inbox: &Rc<RefCell<Vec<Picked>>>| {
            inbox
                .borrow_mut()
                .push(Picked::Failed(format!("{name} could not be downloaded")));
        };
        let parts = js_sys::Array::of1(&js_sys::Uint8Array::from(bytes));
        let Some((document, url)) = web_sys::window()
            .and_then(|w| w.document())
            .zip(web_sys::Blob::new_with_u8_array_sequence(&parts).ok())
            .and_then(|(d, blob)| {
                Some((d, web_sys::Url::create_object_url_with_blob(&blob).ok()?))
            })
        else {
            fail(&self.inbox);
            return;
        };
        let Some(anchor) = document
            .create_element("a")
            .ok()
            .and_then(|e| e.dyn_into::<web_sys::HtmlAnchorElement>().ok())
        else {
            fail(&self.inbox);
            return;
        };
        anchor.set_href(&url);
        anchor.set_download(name);
        // An anchor in the document, which some browsers need for the click to download.
        let body = document.body();
        if let Some(body) = &body {
            let _ = body.append_child(&anchor);
        }
        anchor.click();
        if let Some(body) = &body {
            let _ = body.remove_child(&anchor);
        }
        let _ = web_sys::Url::revoke_object_url(&url);
    }

    /// Ask for the file whose bytes replace an asset's.
    pub(crate) fn pick_replacement(&mut self, asset: rootpoint_model::AssetId) {
        self.pick("", Purpose::Replacement(asset));
    }

    /// Read the files dropped on the window, as a picker reads the files it is given.
    pub(crate) fn read_dropped(&mut self, files: Vec<eframe::egui::DroppedFileHandle>) {
        let (inbox, ctx) = (self.inbox.clone(), self.ctx.clone());
        wasm_bindgen_futures::spawn_local(async move {
            for file in files {
                let name = file.path().display().to_string();
                inbox.borrow_mut().push(match file.bytes_async().await {
                    Ok(bytes) => Picked::Image(name, bytes),
                    Err(e) => Picked::Failed(format!("{name} could not be read: {e}")),
                });
            }
            if let Some(ctx) = ctx {
                ctx.request_repaint();
            }
        });
    }

    /// Show the browser's file picker: an `<input type="file">`, with `webkitdirectory` for a
    /// directory. The click comes from a frame drawn after the author's key or click, which the
    /// browser still counts as the author's activation.
    fn pick(&mut self, accept: &str, purpose: Purpose) {
        let directory = matches!(purpose, Purpose::Directory);
        let Some(input) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.create_element("input").ok())
            .and_then(|e| e.dyn_into::<web_sys::HtmlInputElement>().ok())
        else {
            self.inbox
                .borrow_mut()
                .push(Picked::Failed("the page has no file picker".to_owned()));
            return;
        };
        input.set_type("file");
        if !accept.is_empty() {
            input.set_accept(accept);
        }
        if directory {
            let _ = input.set_attribute("webkitdirectory", "");
        }
        let (inbox, ctx) = (self.inbox.clone(), self.ctx.clone());
        let picked = input.clone();
        let on_change = Closure::once_into_js(move || {
            let Some(list) = picked.files() else {
                return;
            };
            wasm_bindgen_futures::spawn_local(async move {
                let mut files = Vec::new();
                for i in 0..list.length() {
                    let Some(file) = list.item(i) else {
                        continue;
                    };
                    // `webkitRelativePath` is the path under the picked directory's parent; web-sys
                    // 0.3.106 has no binding for it.
                    let path = if directory {
                        js_sys::Reflect::get(&file, &"webkitRelativePath".into())
                            .ok()
                            .and_then(|p| p.as_string())
                            .unwrap_or_else(|| file.name())
                    } else {
                        file.name()
                    };
                    match JsFuture::from(file.array_buffer()).await {
                        Ok(buffer) => {
                            files.push((path, js_sys::Uint8Array::new(&buffer).to_vec()));
                        }
                        Err(_) => {
                            // A directory read in part opens nothing, and the status says why.
                            files.clear();
                            inbox
                                .borrow_mut()
                                .push(Picked::Failed(format!("{path} could not be read")));
                            break;
                        }
                    }
                }
                let picked = match files.pop() {
                    None => None,
                    Some(file) if directory => {
                        files.push(file);
                        Some(Picked::Directory(files))
                    }
                    Some((name, bytes)) => Some(match purpose {
                        Purpose::Replacement(asset) => Picked::Replacement(asset, name, bytes),
                        Purpose::Pack => Picked::Pack(name, bytes),
                        Purpose::Image | Purpose::Directory => Picked::Image(name, bytes),
                    }),
                };
                inbox.borrow_mut().extend(picked);
                if let Some(ctx) = ctx {
                    ctx.request_repaint();
                }
            });
        });
        input.set_onchange(Some(on_change.unchecked_ref()));
        input.click();
    }

    /// Ask the page for full screen, or to leave it. A request the browser refuses, as without the
    /// author's activation, leaves presenter mode in the page.
    pub(crate) fn set_fullscreen(&mut self, on: bool) {
        self.fullscreen_seen = false;
        let Some(document) = web_sys::window().and_then(|w| w.document()) else {
            return;
        };
        if on {
            if let Some(root) = document.document_element() {
                let _ = root.request_fullscreen();
            }
        } else if document.fullscreen_element().is_some() {
            document.exit_fullscreen();
        }
    }

    /// Whether the page left full screen after it was seen in it, per
    /// `design@rootpoint-app@page-full-screen`.
    pub(crate) fn left_fullscreen(&mut self) -> bool {
        let now = web_sys::window()
            .and_then(|w| w.document())
            .is_some_and(|d| d.fullscreen_element().is_some());
        let left = self.fullscreen_seen && !now;
        self.fullscreen_seen = now;
        left
    }
}

/// Ask before the page unloads while `unsaved` holds: a listener of the window's `beforeunload`
/// prevents the event's default, which a browser reads as the question, and sets a return value
/// that is not empty, which older browsers read instead.
/// It holds the flag weakly, so an app dropped asks nothing,
/// `design@rootpoint-app@no-discard-confirmation`.
pub(crate) fn warn_before_unload(unsaved: std::rc::Weak<std::cell::Cell<bool>>) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let listener = Closure::<dyn FnMut(web_sys::Event)>::new(move |e: web_sys::Event| {
        if unsaved.upgrade().is_some_and(|flag| flag.get()) {
            e.prevent_default();
            // Browsers before the default's prevention asked on a return value that is not empty.
            if let Some(unload) = e.dyn_ref::<web_sys::BeforeUnloadEvent>() {
                unload.set_return_value("unsaved");
            }
        }
    });
    let _ =
        window.add_event_listener_with_callback("beforeunload", listener.as_ref().unchecked_ref());
    listener.forget();
}

/// Keep the editor's keys from the browser's own actions, per
/// `design@rootpoint-app@editor-keys-kept-from-browser`: a listener of the window, in the capture
/// phase, prevents the default of F5, which starts presenter mode, and, while `editing` holds, of
/// Ctrl+B, Ctrl+I and Ctrl+Space, which the editor consumes then. eframe 0.37.0's text agent,
/// which has the focus while a text is edited, hands its keydown to the canvas's handler, and the
/// window's capture-phase listener sees both first. It holds the flag weakly, so an app dropped
/// keeps F5 alone.
pub fn keep_editor_keys(editing: std::rc::Weak<std::cell::Cell<bool>>) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let listener =
        Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |e: web_sys::KeyboardEvent| {
            let styling = e.ctrl_key()
                && matches!(e.key().to_lowercase().as_str(), "b" | "i" | " ")
                && editing.upgrade().is_some_and(|flag| flag.get());
            if e.key() == "F5" || styling {
                e.prevent_default();
            }
        });
    let _ = window.add_event_listener_with_callback_and_bool(
        "keydown",
        listener.as_ref().unchecked_ref(),
        true,
    );
    listener.forget();
}
