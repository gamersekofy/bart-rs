//! Route colours.
//!
//! BART publishes route colours as six-digit hex strings. `colored`'s own
//! string-to-colour conversion only understands the sixteen named colours
//! and falls back to *white* for anything else, so passing `"FF0000"`
//! straight to it produced a white block instead of a red one. This is the
//! regression test for that.
//!
//! The tests live in one function because the `colored` crate's colour
//! support is process-global state, and the tests run in parallel.

#![cfg(feature = "cli")]

use bart_rs::commands::swatch;

#[test]
fn route_colours_are_rendered_in_colour() {
    // `colored` decides between true colour and the sixteen ANSI colours
    // by looking at COLORTERM, so it is set here to make the expected
    // output deterministic.
    std::env::set_var("COLORTERM", "truecolor");

    // Colour is off by default when the output is not a terminal, so it has
    // to be forced on to see the escape sequence.
    colored::control::set_override(true);

    // The colours BART actually uses, as published in routes.txt.
    for (hex, rgb) in [
        ("FFFF33", "255;255;51"),
        ("FF9933", "255;153;51"),
        ("339933", "51;153;51"),
        ("FF0000", "255;0;0"),
        ("0000FF", "0;0;255"),
        ("FFFFFF", "255;255;255"),
    ] {
        let swatch = swatch(Some(hex));
        let expected = format!("\u{1b}[38;2;{rgb}m");
        assert!(
            swatch.contains(&expected),
            "the colour {hex} was not rendered as truecolour: {swatch:?}"
        );
    }

    // The bug this guards against: an unparseable colour name fell back to
    // white rather than to the colour that was asked for.
    let red = swatch(Some("FF0000"));
    assert!(
        !red.contains("[37m"),
        "the colour fell back to white: {red:?}"
    );
    assert!(
        !red.contains("[0;255;0m"),
        "the colour fell back to green: {red:?}"
    );

    // A colour that is not six hex digits has no swatch at all.
    assert_eq!(swatch(None), " ");
    assert_eq!(swatch(Some("")), " ");
    assert_eq!(swatch(Some("FF00")), " ");
    assert_eq!(swatch(Some("GGGGGG")), " ");
    assert_eq!(swatch(Some("not a colour")), " ");
}
