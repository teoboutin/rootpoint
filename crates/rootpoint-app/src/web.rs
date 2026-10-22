//! What the browser build does through the page: the pickers that read a presentation directory
//! or an image file, and full screen. A picker reads its files in a task of the page, after the
//! frame that asked for them, so it puts them in an inbox that `App::frame` empties, waking the
//! window when they are read.

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
    /// An image, by its file name.
    Image(String, Vec<u8>),
    /// A file that could not be read, and why.
    Failed(String),
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
        self.pick("", true);
    }

    /// Ask for one image file.
    pub(crate) fn pick_image(&mut self) {
        self.pick("image/png,image/jpeg", false);
    }

    /// Show the browser's file picker: an `<input type="file">`, with `webkitdirectory` for a
    /// directory. The click comes from a frame drawn after the author's key or click, which the
    /// browser still counts as the author's activation.
    fn pick(&mut self, accept: &str, directory: bool) {
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
                    // has no binding for it.
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
                    Some((name, bytes)) => Some(Picked::Image(name, bytes)),
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

    /// Whether the page left full screen after it was seen in it: the browser takes Esc to leave
    /// full screen, and the page may not see the key.
    pub(crate) fn left_fullscreen(&mut self) -> bool {
        let now = web_sys::window()
            .and_then(|w| w.document())
            .is_some_and(|d| d.fullscreen_element().is_some());
        let left = self.fullscreen_seen && !now;
        self.fullscreen_seen = now;
        left
    }
}

/// Keep F5, the key that starts presenter mode, from reloading the page, which would lose every
/// change held in memory: eframe 0.37.0 lets the browser act on F5, in
/// `should_prevent_default_for_key` of `path@elsewhere@src/web/events.rs`. Its listener on the
/// canvas also stops every key's propagation, `WebOptions::should_stop_propagation`, so a listener
/// of the window prevents F5 in the capture phase, before the canvas has it, and the canvas still
/// gets the key.
pub fn keep_f5() {
    let Some(window) = web_sys::window() else {
        return;
    };
    let listener =
        Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(|e: web_sys::KeyboardEvent| {
            if e.key() == "F5" {
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
