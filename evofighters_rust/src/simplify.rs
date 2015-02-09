// For simplifying thought trees

use std::cmp::{min, max, PartialEq, PartialOrd};

use dna;
use dna::{ConditionTree,ActionTree,ValueTree};


pub fn simplify(cond: ConditionTree) -> ConditionTree {

    // First, go through and determine what kind of conditionals can
    // be evaluated statically (i.e. they always come out to the same
    // result), and replace them with their appropriate branch
    let stage_1_cond = eval_static_conditionals(cond);
    // Next, evaluate redundant Always -> Subcondition branches
    let stage_2_cond = eval_redundant_conditions(stage_1_cond);
    // Crazy ideas: check if literal is greater than any possible attribute value
    stage_2_cond
}

fn eval_static_conditionals(cond: ConditionTree) -> ConditionTree {
    // Evaluate in line anywhere in the tree that contains only literals.
    use dna::ConditionTree::{Always, RangeCompare, BinCompare, ActionCompare};
    use dna::ValueTree::Literal;
    use dna::BinOp::{LT,GT,EQ,NE};
    match cond {
        Always(act) => Always(esc_action(act)),
        RangeCompare{
            value: Literal(check_val),
            bound_a: Literal(a),
            bound_b: Literal(b),
            affirmed,
            denied} => {
            if min(a, b) <= check_val && check_val <= max(a, b) {
                Always(esc_action(affirmed))
            } else {
                Always(esc_action(denied))
            }
        },
        RangeCompare{value, bound_a, bound_b, affirmed, denied} => {
            RangeCompare{
                value:value,
                bound_a: bound_a,
                bound_b: bound_b,
                affirmed: esc_action(affirmed),
                denied: esc_action(denied),
            }
        },
        BinCompare{
            operation,
            lhs: Literal(lhs),
            rhs: Literal(rhs),
            affirmed,
            denied} => {
            let op: fn(&usize, &usize) -> bool = match operation {
                LT => PartialOrd::lt,
                GT => PartialOrd::gt,
                EQ => PartialEq::eq,
                NE => PartialEq::ne,
            };
            if op(&(lhs as usize), &(rhs as usize)) {
                Always(esc_action(affirmed))
            } else {
                Always(esc_action(denied))
            }
        },
        BinCompare{operation,lhs,rhs,affirmed,denied} => {
            BinCompare{
                operation: operation,
                lhs: lhs,
                rhs: rhs,
                affirmed: esc_action(affirmed),
                denied: esc_action(denied),
            }
        },
        ActionCompare{actor_type, action, affirmed, denied} => {
            ActionCompare{
                actor_type: actor_type,
                action: esc_action(action),
                affirmed: esc_action(affirmed),
                denied: esc_action(denied),
            }
        }
    }
}

fn esc_action(act: ActionTree) -> ActionTree {
    use dna::ActionTree::Subcondition;
    match act {
        Subcondition(box cond) =>
            Subcondition(box eval_static_conditionals(cond)),
        otherwise => otherwise
    }
}

fn eval_redundant_conditions(cond: ConditionTree) -> ConditionTree {
    use dna::ConditionTree::{Always, RangeCompare, BinCompare, ActionCompare};
    use dna::ActionTree::Subcondition;
    match cond {
        Always(Subcondition(box cond)) =>
            eval_redundant_conditions(cond),
        Always(act) =>
            Always(erc_action(act)),
        RangeCompare{value, bound_a, bound_b, affirmed, denied} =>
            RangeCompare{
                value: value,
                bound_a: bound_a,
                bound_b: bound_b,
                affirmed: erc_action(affirmed),
                denied: erc_action(denied),
            },
        BinCompare{operation, lhs, rhs, affirmed, denied} =>
            BinCompare{
                operation: operation,
                lhs: lhs,
                rhs: rhs,
                affirmed: erc_action(affirmed),
                denied: erc_action(denied),
            },
        ActionCompare{actor_type, action, affirmed, denied} =>
            ActionCompare{
                actor_type: actor_type,
                action: erc_action(action),
                affirmed: erc_action(affirmed),
                denied: erc_action(denied),
            },
    }
}

fn erc_action(act: ActionTree) -> ActionTree {
    use dna::ActionTree::Subcondition;
    match act {
        Subcondition(box cond) =>
            Subcondition(box eval_redundant_conditions(cond)),
        otherwise => otherwise
    }
}
