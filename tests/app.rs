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
