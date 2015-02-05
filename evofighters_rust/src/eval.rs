use std::fmt;
use std::cmp::{min, max, PartialOrd, PartialEq};

use util;
use dna::{Signal, DamageType, ConditionTree, ActionTree, ValueTree,
          BinOp, ActorType, Attribute};
use creatures::Creature;
use settings;

// PerformableAction is the result of evaluating a thought tree
#[derive(Debug, Copy, PartialEq, Eq, Clone, RustcEncodable, RustcDecodable)]
pub enum PerformableAction {
    Attack(DamageType),
    Defend(DamageType),
    Signal(Signal),
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
            PerformableAction::Attack(dmg) =>
                write!(f, "attack with damage type: {:?}", dmg),
            PerformableAction::Defend(dmg) =>
                write!(f, "defend against damage type: {:?}", dmg),
            PerformableAction::Signal(sig) =>
                write!(f, "signal with the color: {:?}", sig),
            PerformableAction::Eat =>
                write!(f, "use an item in his inventory"),
            PerformableAction::Take =>
                write!(f, "take something from target"),
            PerformableAction::Wait =>
                write!(f, "wait"),
            PerformableAction::Flee =>
                write!(f, "flee the encounter"),
            PerformableAction::Mate =>
                write!(f, "mate with the target"),
            PerformableAction::NoAction =>
                write!(f, "(no action)"),
        }
    }
}

pub fn evaluate(me: &Creature,
                other: &Creature,
                tree: &ConditionTree,
                app: &mut util::AppState) -> PerformableAction {
    match *tree {
        ConditionTree::Always(ref action) =>
            eval_action(me, other, action, app),
        ConditionTree::RangeCompare{
            ref value,
            ref bound_a,
            ref bound_b,
            ref affirmed,
            ref denied
        } => {
            let a = eval_value(me, other, bound_a, app);
            let b = eval_value(me, other, bound_b, app);
            let check_val = eval_value(me, other, value, app);
            if min(a, b) <= check_val && check_val <= max(a, b) {
                print3!("{} was between {} and {}", check_val, a, b);
                eval_action(me, other, affirmed, app)
            } else {
                print3!("{} was not between {} and {}", check_val, a, b);
                eval_action(me, other, denied, app)
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
            let evaled_lhs = eval_value(me, other, lhs, app);
            let evaled_rhs = eval_value(me, other, rhs, app);
            if op(&evaled_lhs, &evaled_rhs) {
                print3!("{:?}({}) was {} {:?}({})",
                        lhs, evaled_lhs, operation, rhs, evaled_rhs);
                eval_action(me, other, affirmed, app)
            } else {
                print3!("{:?}({}) was not {} {:?}({})",
                        lhs, evaled_lhs, operation, rhs, evaled_rhs);
                eval_action(me, other, denied, app)
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
            let my_action = eval_action(me, other, action, app);
            if my_action == actor.last_action {
                print3!("{}'s last action was {:?}",
                        actor_str, actor.last_action);
                eval_action(me, other, affirmed, app)
            } else {
                print3!("{}'s last action was not {:?}",
                        actor_str, actor.last_action);
                eval_action(me, other, denied, app)
            }

        }
    }
}

fn eval_action(me: &Creature,
               other: &Creature,
               action: &ActionTree,
               app: &mut util::AppState) -> PerformableAction {
    match *action {
        ActionTree::Attack(dmg) => PerformableAction::Attack(dmg),
        ActionTree::Defend(dmg) => PerformableAction::Defend(dmg),
        ActionTree::Signal(sig) => PerformableAction::Signal(sig),
        ActionTree::Eat => PerformableAction::Eat,
        ActionTree::Take => PerformableAction::Take,
        ActionTree::Wait => PerformableAction::Wait,
        ActionTree::Flee => PerformableAction::Flee,
        ActionTree::Mate => PerformableAction::Mate,
        ActionTree::Subcondition(box ref sub) => {
            evaluate(me, other, sub, app)
        },
    }
}

fn eval_value(me: &Creature,
              other: &Creature,
              val: &ValueTree,
              app: &mut util::AppState) -> usize {
    match *val {
        ValueTree::Literal(x) => x as usize,
        ValueTree::Random =>
            app.rand_range(0, settings::MAX_GENE_VALUE as usize),
        ValueTree::Me(attr) => get_attr(me, attr),
        ValueTree::Other(attr) => get_attr(other, attr),
    }
}

fn get_attr(actor: &Creature, attr: Attribute) -> usize {
    match attr {
        Attribute::Energy => actor.energy(),
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
