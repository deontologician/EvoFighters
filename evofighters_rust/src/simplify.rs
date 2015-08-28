// For simplifying thought trees

use std::cmp::{min, max, PartialEq, PartialOrd};

use dna;
use dna::{ConditionTree,ActionTree,ValueTree};
use parsing::{Thought};
use parsing;


pub fn simplify(cond: ConditionTree) -> ConditionTree {

    // First, go through and determine what kind of conditionals can
    // be evaluated statically (i.e. they always come out to the same
    // result), and replace them with their appropriate branch Also,
    // any conditionals that have the same result, whatever the
    // condition ends up being, can be replaced by one of the branches.
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
            let esc_affirmed = esc_action(affirmed);
            let esc_denied = esc_action(denied);
            if esc_affirmed == esc_denied {
                Always(esc_affirmed)
            }else{
                let op: fn(&usize, &usize) -> bool = match operation {
                    LT => PartialOrd::lt,
                    GT => PartialOrd::gt,
                    EQ => PartialEq::eq,
                    NE => PartialEq::ne,
                };
                if op(&(lhs as usize), &(rhs as usize)) {
                    Always(esc_affirmed)
                } else {
                    Always(esc_denied)
                }
            }
        },
        BinCompare{operation,lhs,rhs,affirmed,denied} => {
            let esc_affirmed = esc_action(affirmed);
            let esc_denied = esc_action(denied);
            if esc_affirmed == esc_denied {
                Always(esc_affirmed)
            }else {
                BinCompare{
                    operation: operation,
                    lhs: lhs,
                    rhs: rhs,
                    affirmed: esc_affirmed,
                    denied: esc_denied,
                }
            }
        },
        ActionCompare{actor_type, action, affirmed, denied} => {
            let esc_affirmed = esc_action(affirmed);
            let esc_denied = esc_action(denied);
            if esc_affirmed == esc_denied {
                Always(esc_affirmed)
            }else {
                ActionCompare{
                    actor_type: actor_type,
                    action: esc_action(action),
                    affirmed: esc_affirmed,
                    denied: esc_denied,
                }
            }
        }
    }
}

fn esc_action(act: ActionTree) -> ActionTree {
    use dna::ActionTree::Subcondition;
    use dna::ConditionTree::{Always};
    match act {
        Subcondition(box Always(act)) =>
            esc_action(act),
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

pub struct ThoughtCycle {
    thoughts: Vec<parsing::Decision>,
    cycle_offset: usize,
}


pub fn cycle_detect(dna: &Vec<i8>) -> Result<ThoughtCycle, parsing::Failure> {

    let f = |offset: usize| -> usize {
        let mut parser = parsing::Parser::new(dna.clone(), offset);
        parser.next().unwrap().ok().offset
    };

    let mut tortoise = f(0);
    let mut hare = f(tortoise);
    while tortoise != hare {
        tortoise = f(tortoise);
        hare = f(f(hare));
    }
    let mut mu = 0;
    // reset tortoise
    tortoise = 0;
    while tortoise != hare {
        tortoise = f(tortoise);
        hare = f(hare);
        mu += 1;
    }
    let mut lam = 1;
    hare = f(tortoise);
    while tortoise != hare {
        hare = f(hare);
        lam += 1;
    }
    let mut new_iter = parsing::Parser::new(dna.clone(), 0);
    let mut thought: parsing::Decision;
    let thought_tree: ConditionTree;
    let mut thoughts = Vec::new();
    for i in 0..(mu + lam) {
        let mut offset = new_iter.current_offset();
        box thought = try!(new_iter.next().unwrap());
        box thought_tree = thought.tree;
        println!("{}: {:?}", i, thought.tree);
        let mut simplified = simplify(thought_tree.clone());
        if thought_tree != simplified {
            println!("  -> {:?}", simplified);
            thought.tree = simplified;
        }
        thoughts.push(simplified);
    }
    Ok(ThoughtCycle {
        thoughts: thoughts,
        cycle_offset: mu,
    })
}
