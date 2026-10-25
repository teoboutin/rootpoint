//! What the tests share: the built-in math font, read from the render crate, which embeds it.

use rootpoint_math::MathFont;

pub const LIBERTINUS_MATH: &[u8] =
    include_bytes!("../../../rootpoint-render/fonts/libertinus/LibertinusMath-Regular.otf");

pub fn font() -> MathFont {
    MathFont::new(LIBERTINUS_MATH).expect("Libertinus Math has a MATH table")
}
