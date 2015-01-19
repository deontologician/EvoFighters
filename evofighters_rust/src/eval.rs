
extern crate core;

use dna::{Signal, DamageType, ConditionTree, ActionTree};
use creatures;
use std::fmt;
use std::ops::Deref;

// PerformableAction is the result of evaluating a thought tree
#[derive(Show, Copy)]
pub enum PerformableAction {
    Attack(DamageType),
    Defend(DamageType),
    Signal(Signal),
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

pub fn evaluate(me: &creatures::Creature,
            tree: &ConditionTree) -> PerformableAction {
    match *tree {
        ConditionTree::Always(ref action) => {
            eval_action(me, action)
        },
        _ => panic!("oh noes")
    }
}

fn eval_action(me: &creatures::Creature,
               action: &ActionTree) -> PerformableAction {
    match *action {
        ActionTree::Attack(dmg) => PerformableAction::Attack(dmg),
        ActionTree::Defend(dmg) => PerformableAction::Defend(dmg),
        ActionTree::Signal(sig) => PerformableAction::Signal(sig),
        ActionTree::Use => PerformableAction::Use,
        ActionTree::Take => PerformableAction::Take,
        ActionTree::Wait => PerformableAction::Wait,
        ActionTree::Flee => PerformableAction::Flee,
        ActionTree::Mate => PerformableAction::Mate,
        ActionTree::Subcondition(box ref sub) => {
            evaluate(me, sub)
        },
    }
}
