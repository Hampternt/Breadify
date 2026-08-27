//! The window's state, which is testable without painting anything.

use breadify::app::Step;

#[test]
fn the_steps_walk_forwards_and_back() {
    assert_eq!(Step::Open.previous(), None);
    assert_eq!(Step::Open.next(), Some(Step::Check));
    assert_eq!(Step::Check.previous(), Some(Step::Open));
    assert_eq!(Step::Print.next(), None);

    let mut step = Step::Open;
    let mut walked = vec![step];
    while let Some(next) = step.next() {
        step = next;
        walked.push(step);
    }
    assert_eq!(walked, Step::ALL);
}

#[test]
fn every_step_is_numbered_and_labelled() {
    let numbers: Vec<&str> = Step::ALL.iter().map(|step| step.number()).collect();
    let labels: Vec<&str> = Step::ALL.iter().map(|step| step.label()).collect();

    assert_eq!(numbers, ["01", "02", "03", "04"]);
    assert_eq!(labels, ["Open", "Check", "Configure", "Print"]);
}

/// The Check step paints a bread roll behind itself at almost no opacity. It
/// is a joke, but a joke that has to decode.
#[test]
fn the_mascot_is_a_picture() {
    let (pixels, [width, height]) = breadify::app::mascot::decoded().expect("breadguy decodes");

    assert!(
        width > 64 && height > 64,
        "{width}x{height} is not a picture"
    );
    assert_eq!(pixels.len(), width * height * 4, "four bytes to a pixel");
    assert!(
        pixels.chunks_exact(4).all(|pixel| pixel[3] == 0xFF),
        "a JPEG has no transparency to lose"
    );
}
