# Tripwires of rootpoint-protocol

Evidence that would flip a recorded decision of rootpoint-protocol: one level-two heading per
tripwire, each naming its decision. What is outstanding is in `register@rootpoint-protocol@issue`.

## Guarding `design@rootpoint-protocol@final-preview`: a click meant only to look ends the preview `##click-ends-the-preview`

T2 of the premortem of the milestone on the conveniences of the editor, recorded on the owner's
word.

**Fires when:** the owner reports, more than once, leaving the preview by a click the owner did not
mean as an edit.
**Response:** reopen `design@rootpoint-protocol@final-preview`, toward a click in the preview that
does nothing, `rejected@rootpoint-protocol@click-does-nothing-in-preview`.
**Re-entry:** the standing-state review before every merge.

## Guarding `design@rootpoint-protocol@workspace-lock`: a workspace shared by a sync tool is open on two machines `##workspace-on-two-machines`

T1 of the premortem of the milestone on asset storage, recorded on the owner's word.

**Fires when:** a workspace opened on two machines through a sync tool leaves a conflict copy of
`presentation.ron`, or loses an edit made on one of them, the lock holding on one machine only.
**Response:** reopen `design@rootpoint-protocol@workspace-lock`, toward the host's name in the lock
file, and a refusal when it names another host.
**Re-entry:** at its firing, as the session proposed and the owner approved; and the
standing-state review before every merge, the home's standing re-entry.

## Guarding `design@rootpoint-protocol@table-of-added-assets`: an author loses an untracked file through a pack `##untracked-file-lost-in-a-pack`

T3 of the premortem of the milestone on asset storage, recorded on the owner's word.

**Fires when:** an author keeps a file the presentation does not name, as a source `.svg` beside
its `.png`, packs, unpacks elsewhere, finds the file missing, and expected it kept.
**Response:** an option of `pack` that packs the untracked files too.
**Re-entry:** at its firing, as the session proposed and the owner approved; and the
standing-state review before every merge, the home's standing re-entry.

## Guarding `design@rootpoint-protocol@text-edit-commands` and `design@rootpoint-protocol@protocol-transport`: agents write the one-line tree badly `##one-line-tree-written-badly`

T2 of the premortem of the milestone on rich text, recorded on the owner's word.

**Fires when:** in one session of the owner's, an agent has `text` with a tree refused for its
grammar three times or more.
**Response:** reopen the grammar of `text`'s tree in
`design@rootpoint-protocol@protocol-transport`, toward a flatter form.
**Re-entry:** the design audit of the next work on the protocol; and the standing-state review
before every merge.
