//! Route colours and the tables they appear in.
//!
//! BART publishes route colours as six-digit hex strings. `colored`'s own
//! string-to-colour conversion only understands the sixteen named colours
//! and falls back to *white* for anything else, so passing `"FF0000"`
//! straight to it produced a white block instead of a red one. This is the
//! regression test for that.
//!
//! Both checks live in one test function on purpose. `colored`'s colour
//! support is process-global state, and the test harness runs the tests in a
//! file in parallel threads: one of them writes `COLORTERM` and the other
//! reads it through `swatch`, which would be a data race. One test means
//! one thread, and separate `tests/*.rs` files are separate processes, so
//! nothing else in the suite touches the environment either.

#![cfg(feature = "cli")]

use bart_rs::commands::{swatch, visible_width};

#[test]
fn route_colours_are_rendered_in_colour() {
    // `colored` decides between true colour and the sixteen ANSI colours
    // by looking at COLORTERM, so it is set here to make the expected
    // output deterministic.
    //
    // SAFETY: this is the only test in this binary, so the harness has
    // spawned exactly one worker thread and its main thread is blocked
    // waiting for it. Nothing else reads or writes the environment
    // concurrently. Test binaries are separate processes, so the other
    // test files cannot interfere either.
    unsafe { std::env::set_var("COLORTERM", "truecolor") };

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

    // The second check: escape sequences must not count towards the width of
    // a column, or every table with a swatch in it comes out ragged.
    assert_eq!(visible_width("plain"), 5);
    assert_eq!(visible_width(""), 0);
    assert_eq!(visible_width("█"), 1);
    assert_eq!(visible_width("\u{1b}[38;2;255;0;0m█\u{1b}[0m"), 1);
    assert_eq!(visible_width("\u{1b}[1mbold\u{1b}[0m"), 4);
    assert_eq!(visible_width("\u{1b}[38;2;255;0;0mred\u{1b}[0m"), 3);

    // A swatch is one column wide on screen, however many bytes it takes.
    let block = swatch(Some("FF0000"));
    assert_eq!(visible_width(&block), 1);
}
