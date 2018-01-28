use std::fmt;
use std::cmp::{min, max, PartialOrd, PartialEq};

use util;
use dna::{lex,ast};
use creatures::Creature;
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
                tree: &ast::Condition,
                app: &mut util::AppState) -> PerformableAction {
    match *tree {
        ast::Condition::Always(ref action) =>
            eval_action(me, other, action, app),
        ast::Condition::RangeCompare{
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
        ast::Condition::BinCompare{
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
        ast::Condition::ActionCompare{
            ref actor_type,
            ref action,
            ref affirmed,
            ref denied,
        } => {
            let (actor, actor_str) = match *actor_type {
                ast::ActorType::Me => (me, "my"),
                ast::ActorType::Other => (other, "other's"),
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
               action: &ast::Action,
               app: &mut util::AppState) -> PerformableAction {
    match *action {
        ast::Action::Attack(dmg) => PerformableAction::Attack(dmg),
        ast::Action::Defend(dmg) => PerformableAction::Defend(dmg),
        ast::Action::Signal(sig) => PerformableAction::Signal(sig),
        ast::Action::Eat => PerformableAction::Eat,
        ast::Action::Take => PerformableAction::Take,
        ast::Action::Wait => PerformableAction::Wait,
        ast::Action::Flee => PerformableAction::Flee,
        ast::Action::Mate => PerformableAction::Mate,
        ast::Action::Subcondition(ref sub) => {
            evaluate(me, other, sub, app)
        },
    }
}

fn eval_value(me: &Creature,
              other: &Creature,
              val: &ast::Value,
              app: &mut util::AppState) -> usize {
    match *val {
        ast::Value::Literal(x) => x as usize,
        ast::Value::Random =>
            app.rand_range(0, settings::MAX_GENE_VALUE as usize),
        ast::Value::Me(attr) => get_attr(me, attr),
        ast::Value::Other(attr) => get_attr(other, attr),
    }
}

fn get_attr(actor: &Creature, attr: lex::Attribute) -> usize {
    match attr {
        lex::Attribute::Energy => actor.energy(),
        lex::Attribute::Signal => match actor.signal {
            Some(sig) => sig as usize,
            None => 0,
        },
        lex::Attribute::Generation => actor.generation,
        lex::Attribute::Kills => actor.kills,
        lex::Attribute::Survived => actor.survived,
        lex::Attribute::NumChildren => actor.num_children,
        lex::Attribute::TopItem => match actor.top_item() {
            Some(item) => item as usize,
            None => 0,
        },
    }
}
