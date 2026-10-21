# Goals of rootpoint-protocol

What rootpoint-protocol is for, and what would show it achieved: one level-two heading per goal. A
goal is met or unmet. It stays here while it is met, and it leaves only on the owner's word. How
rootpoint-protocol is built is in `register@rootpoint-protocol@design`.

## An agent or a script builds and changes a presentation by text commands alone `##commands-drive-everything`

Every change the editor offers, and every view it shows, is a command of the protocol, one per
line, with a one-line reply an agent can read: the ids a command created, the reason a command was
refused, and what a render could not draw. It refines `goal@rootpoint@text-command-protocol`. It is
met while an agent can build, change, save, render and present a presentation with these commands
and no other means, headless or in the running editor.

## The protocol's session builds for the browser `##browser-buildable`

The session is part of the browser build, so it uses nothing that exists only on a desktop and
reads and writes files only through its host. It refines `goal@rootpoint@runs-in-browser`. It is
met while the gate `wasm` builds this crate's library for the browser's target.
