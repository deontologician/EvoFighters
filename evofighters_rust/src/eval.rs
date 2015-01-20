
extern crate core;

use dna::{Signal, DamageType, ConditionTree, ActionTree, ValueTree,
          BinOp, ActorType, Attribute};
use creatures::Creature;
use std::fmt;
use std::rand;
use std::cmp::{min, max, PartialOrd, PartialEq};

// PerformableAction is the result of evaluating a thought tree
#[derive(Show, Copy, PartialEq, Eq)]
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

pub fn evaluate(me: &Creature,
                other: &Creature,
                tree: &ConditionTree,
                rng: &mut rand::Rng) -> PerformableAction {
    match *tree {
        ConditionTree::Always(ref action) =>
            eval_action(me, other, action, rng),
        ConditionTree::RangeCompare{
            ref value,
            ref bound_a,
            ref bound_b,
            ref affirmed,
            ref denied
        } => {
            let a = eval_value(me, other, bound_a, rng);
            let b = eval_value(me, other, bound_b, rng);
            let check_val = eval_value(me, other, value, rng);
            if min(a, b) <= check_val && check_val <= max(a, b) {
                print3!("{} was between {} and {}", check_val, a, b);
                eval_action(me, other, affirmed, rng)
            } else {
                print3!("{} was not between {} and {}", check_val, a, b);
                eval_action(me, other, denied, rng)
            }
        },
        ConditionTree::BinCompare{
            ref operation,
            ref lhs,
            ref rhs,
            ref affirmed,
            ref denied,
        } => {
            let op: fn(&usize, &usize) -> bool = match *operation {
                BinOp::LT => PartialOrd::lt,
                BinOp::GT => PartialOrd::gt,
                BinOp::EQ => PartialEq::eq,
                BinOp::NE => PartialEq::ne,
            };
            let evaled_lhs = eval_value(me, other, lhs, rng);
            let evaled_rhs = eval_value(me, other, rhs, rng);
            if op(&evaled_lhs, &evaled_rhs) {
                print3!("{:?}({}) was {} {:?}({})",
                        lhs, evaled_lhs, operation, rhs, evaled_rhs);
                eval_action(me, other, affirmed, rng)
            } else {
                print3!("{:?}({}) was not {} {:?}({})",
                        lhs, evaled_lhs, operation, rhs, evaled_rhs);
                eval_action(me, other, denied, rng)
            }
        },
        ConditionTree::ActionCompare{
            ref actor_type,
            ref action,
            ref affirmed,
            ref denied,
        } => {
            let (actor, actor_str) = match *actor_type {
                ActorType::Me => (me, "my"),
                ActorType::Other => (other, "other's"),
            };
            let my_action = eval_action(me, other, action, rng);
            if my_action == actor.last_action {
                print3!("{}'s last action was {:?}",
                        actor_str, actor.last_action);
                eval_action(me, other, affirmed, rng)
            } else {
                print3!("{}'s last action was not {:?}",
                        actor_str, actor.last_action);
                eval_action(me, other, denied, rng)
            }
            
        }
    }
}

fn eval_action(me: &Creature,
               other: &Creature,
               action: &ActionTree,
               rng: &mut rand::Rng) -> PerformableAction {
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
            evaluate(me, other, sub, rng)
        },
    }
}

fn eval_value(me: &Creature,
              other: &Creature,
              val: &ValueTree,
              rng: &mut rand::Rng) -> usize {
    match *val {
        ValueTree::Literal(x) => x as usize,
        ValueTree::Random => rng.gen(),
        ValueTree::Me(attr) => get_attr(me, attr),
        ValueTree::Other(attr) => get_attr(other, attr),
    }
}

fn get_attr(actor: &Creature, attr: Attribute) -> usize {
    match attr {
        Attribute::Energy => actor.energy,
        Attribute::Signal => match actor.signal {
            Some(sig) => sig as usize,
            None => 0,
        },
        Attribute::Generation => actor.generation,
        Attribute::Kills => actor.kills,
        Attribute::Survived => actor.survived,
        Attribute::NumChildren => actor.num_children,
        Attribute::TopItem => match actor.top_item() {
            Some(item) => item as usize,
            None => 0,
        },
    }
}
