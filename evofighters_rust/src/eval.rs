use dna;
use std::fmt;

// PerformableAction is the result of evaluating a thought tree
#[derive(Show, Copy)]
pub enum PerformableAction {
    Attack(dna::DamageType),
    Defend(dna::DamageType),
    Signal(dna::Signal),
    Use,
    Take,
    Wait,
    Flee,
    Mate,
}

impl fmt::String for PerformableAction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PerformableAction::Attack(dmg) =>
                write!(f, "attack with damage type: {:?}", dmg),
            PerformableAction::Defend(dmg) =>
                write!(f, "defend against damage type: {:?}", dmg),
            PerformableAction::Signal(sig) =>
                write!(f, "signal with the color: {:?}", sig),
            PerformableAction::Use =>
                write!(f, "use an item in his inventory"),
            PerformableAction::Take =>
                write!(f, "take something from target"),
            PerformableAction::Wait =>
                write!(f, "wait"),
            PerformableAction::Flee =>
                write!(f, "flee the encounter"),
            PerformableAction::Mate =>
                write!(f, "mate with the target"),
        }
    }
}
