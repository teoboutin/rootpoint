//! A pack saved at every version of the format from the oldest that loads keeps opening, to the
//! same presentation and the same assets, per `design@rootpoint@saved-presentations-stay-loadable`.
//!
//! `v<N>.rpk` of `path@rootpoint-protocol@tests/data/formats/` is a pack of version N, frozen: it
//! is written once, when version N is the current one, and never rewritten: `rootpoint-headless`
//! runs `open` of a directory holding the model's frozen `v<N>.ron` of
//! `path@rootpoint-model@tests/data/formats/` as its `presentation.ron`, the bytes of
//! `path@rootpoint-protocol@sample/splits.png` as `path@elsewhere@images/figure.png` and one
//! line of text as `path@elsewhere@fonts/body.ttf`, then `pack`.

mod common;

use std::path::PathBuf;

use common::*;
use rootpoint_model::{FORMAT_VERSION, OLDEST_VERSION, load_ron};
use rootpoint_protocol::{Session, Store};

fn data(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)
}

/// AC1 of the premortem on compatibility, for packs: a frozen pack of every version from
/// `OLDEST_VERSION` opens, to the presentation the model's frozen file of that version loads to,
/// with its two assets.
#[test]
fn a_frozen_pack_of_every_version_opens() {
    for version in OLDEST_VERSION..=FORMAT_VERSION {
        let pack = data(&format!("tests/data/formats/v{version}.rpk"));
        let bytes = std::fs::read(&pack).unwrap_or_else(|e| {
            panic!(
                "{}: {e}; a new version of the format freezes a pack of itself here",
                pack.display()
            )
        });
        let ron = data(&format!(
            "../rootpoint-model/tests/data/formats/v{version}.ron"
        ));
        let expected = load_ron(&std::fs::read_to_string(&ron).unwrap()).unwrap();
        let mut files = MemoryFiles::default();
        files.files.insert("p.rpk".into(), bytes);
        let mut s = Session::new();
        ok(&mut s, &mut files, "open-pack p.rpk");
        assert_eq!(s.document(), &expected, "the pack of version {version}");
        let Store::Memory(map) = s.store() else {
            panic!("a pack opens in memory");
        };
        assert_eq!(
            map.keys().collect::<Vec<_>>(),
            ["fonts/body.ttf", "images/figure.png"],
            "the assets of the pack of version {version}"
        );
    }
}
