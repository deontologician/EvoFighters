use std::fmt;
use std::cmp::{max, min, PartialEq, PartialOrd};

use dna::{ast, lex};
use creatures::Creature;
use rng::RngState;
use settings;

// PerformableAction is the result of evaluating a thought tree
#[derive(Debug, Copy, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum PerformableAction {
    Attack(lex::DamageType),
    Defend(lex::DamageType),
    Signal(lex::Signal),
    Eat,
    Take,
    Wait,
    Flee,
    Mate,
    NoAction,
}

impl fmt::Display for PerformableAction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PerformableAction::Attack(dmg) => {
                write!(f, "attack with damage type: {:?}", dmg)
            }
            PerformableAction::Defend(dmg) => {
                write!(f, "defend against damage type: {:?}", dmg)
            }
            PerformableAction::Signal(sig) => {
                write!(f, "signal with the color: {:?}", sig)
            }
            PerformableAction::Eat => write!(f, "use an item in his inventory"),
            PerformableAction::Take => write!(f, "take something from target"),
            PerformableAction::Wait => write!(f, "wait"),
            PerformableAction::Flee => write!(f, "flee the encounter"),
            PerformableAction::Mate => write!(f, "mate with the target"),
            PerformableAction::NoAction => write!(f, "(no action)"),
        }
    }
}

pub fn evaluate(
    me: &Creature,
    other: &Creature,
    tree: &ast::Condition,
) -> PerformableAction {
    match *tree {
        ast::Condition::Always(ref action) => eval_action(me, other, action),
        ast::Condition::RangeCompare {
            ref value,
            ref bound_a,
            ref bound_b,
            ref affirmed,
            ref denied,
        } => {
            let a = eval_value(me, other, bound_a);
            let b = eval_value(me, other, bound_b);
            let check_val = eval_value(me, other, value);
            if min(a, b) <= check_val && check_val <= max(a, b) {
                trace!("{} was between {} and {}", check_val, a, b);
                eval_action(me, other, affirmed)
            } else {
                trace!("{} was not between {} and {}", check_val, a, b);
                eval_action(me, other, denied)
            }
        }
        ast::Condition::BinCompare {
            ref operation,
            ref lhs,
            ref rhs,
            ref affirmed,
            ref denied,
        } => {
            let op: fn(&usize, &usize) -> bool = match *operation {
                ast::BinOp::LT => PartialOrd::lt,
                ast::BinOp::GT => PartialOrd::gt,
                ast::BinOp::EQ => PartialEq::eq,
                ast::BinOp::NE => PartialEq::ne,
            };
            let evaled_lhs = eval_value(me, other, lhs);
            let evaled_rhs = eval_value(me, other, rhs);
            if op(&evaled_lhs, &evaled_rhs) {
                trace!(
                    "{:?}({}) was {} {:?}({})",
                    lhs,
                    evaled_lhs,
                    operation,
                    rhs,
                    evaled_rhs
                );
                eval_action(me, other, affirmed)
            } else {
                trace!(
                    "{:?}({}) was not {} {:?}({})",
                    lhs,
                    evaled_lhs,
                    operation,
                    rhs,
                    evaled_rhs
                );
                eval_action(me, other, denied)
            }
        }
        ast::Condition::ActionCompare {
            ref actor_type,
            ref action,
            ref affirmed,
            ref denied,
        } => {
            let (actor, actor_str) = match *actor_type {
                ast::ActorType::Me => (me, "my"),
                ast::ActorType::Other => (other, "other's"),
            };
            let my_action = eval_action(me, other, action);
            if my_action == actor.last_action {
                trace!(
                    "{}'s last action was {:?}",
                    actor_str,
                    actor.last_action
                );
                eval_action(me, other, affirmed)
            } else {
                trace!(
                    "{}'s last action was not {:?}",
                    actor_str,
                    actor.last_action
                );
                eval_action(me, other, denied)
            }
        }
    }
}

fn eval_action(
    me: &Creature,
    other: &Creature,
    action: &ast::Action,
) -> PerformableAction {
    match *action {
        ast::Action::Attack(dmg) => PerformableAction::Attack(dmg),
        ast::Action::Defend(dmg) => PerformableAction::Defend(dmg),
        ast::Action::Signal(sig) => PerformableAction::Signal(sig),
        ast::Action::Eat => PerformableAction::Eat,
        ast::Action::Take => PerformableAction::Take,
        ast::Action::Wait => PerformableAction::Wait,
        ast::Action::Flee => PerformableAction::Flee,
        ast::Action::Mate => PerformableAction::Mate,
        ast::Action::Subcondition(ref sub) => evaluate(me, other, sub),
    }
}

fn eval_value(me: &Creature, other: &Creature, val: &ast::Value) -> usize {
    match *val {
        ast::Value::Literal(x) => x as usize,
        ast::Value::Random => RngState::from_creatures(me, other)
            .rand_range(0, settings::MAX_GENE_VALUE as usize),
        ast::Value::Me(attr) => me.attr(attr),
        ast::Value::Other(attr) => other.attr(attr),
    }
}
