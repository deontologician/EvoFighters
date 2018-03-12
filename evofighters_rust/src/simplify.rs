// For simplifying thought trees

use std::cmp::{max, min, PartialEq, PartialOrd};

use dna::{ast, DNA};
use parsing;

/// Simplifies a condition tree by replacing it with another condition
/// tree that takes less (or at least not more) time to execute at
/// runtime.
///
/// Evolutionary algorithms are good at generating code that "works"
/// but does so by being very redundant, or by doing straightforward
/// things in roundabout ways. This is essentially a compiler that
/// reduces these "bushy" trees to "trim and fit" versions that are
/// equivalent.
///
/// As a bonus, the behavior of the simplified trees is often easier
/// to interpret by a human.
pub fn simplify(cond: ast::Condition) -> ast::Condition {
    let stage_1_cond = eval_static_conditionals(cond);
    // Next, evaluate redundant Always -> Subcondition branches
    eval_redundant_conditions(stage_1_cond)
}

/// Evaluates static conditionals at compile time.
///
/// Static conditionals always evaluate to the same thing, so they can
/// be compiled to one branch or the other before we ever run
/// anything, saving a check at runtime.
fn eval_static_conditionals(cond: ast::Condition) -> ast::Condition {
    // Evaluate in line anywhere in the tree that contains only literals.
    use dna::ast::Condition::{ActionCompare, Always, BinCompare, RangeCompare};
    use dna::ast::Value::Literal;
    use dna::ast::BinOp::{EQ, GT, LT, NE};
    match cond {
        Always(act) => Always(esc_action(act)),
        RangeCompare {
            value: Literal(check_val),
            bound_a: Literal(a),
            bound_b: Literal(b),
            affirmed,
            denied,
        } => {
            if min(a, b) <= check_val && check_val <= max(a, b) {
                Always(esc_action(affirmed))
            } else {
                Always(esc_action(denied))
            }
        }
        RangeCompare {
            value,
            bound_a,
            bound_b,
            affirmed,
            denied,
        } => RangeCompare {
            value: value,
            bound_a: bound_a,
            bound_b: bound_b,
            affirmed: esc_action(affirmed),
            denied: esc_action(denied),
        },
        BinCompare {
            operation,
            lhs: Literal(lhs),
            rhs: Literal(rhs),
            affirmed,
            denied,
        } => {
            let esc_affirmed = esc_action(affirmed);
            let esc_denied = esc_action(denied);
            if esc_affirmed == esc_denied {
                Always(esc_affirmed)
            } else {
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
        }
        BinCompare {
            operation,
            lhs,
            rhs,
            affirmed,
            denied,
        } => {
            let esc_affirmed = esc_action(affirmed);
            let esc_denied = esc_action(denied);
            if esc_affirmed == esc_denied {
                Always(esc_affirmed)
            } else {
                BinCompare {
                    operation: operation,
                    lhs: lhs,
                    rhs: rhs,
                    affirmed: esc_affirmed,
                    denied: esc_denied,
                }
            }
        }
        ActionCompare {
            actor_type,
            action,
            affirmed,
            denied,
        } => {
            let esc_affirmed = esc_action(affirmed);
            let esc_denied = esc_action(denied);
            if esc_affirmed == esc_denied {
                Always(esc_affirmed)
            } else {
                ActionCompare {
                    actor_type: actor_type,
                    action: esc_action(action),
                    affirmed: esc_affirmed,
                    denied: esc_denied,
                }
            }
        }
    }
}

fn esc_action(act: ast::Action) -> ast::Action {
    use dna::ast::Action::Subcondition;
    use dna::ast::Condition::Always;
    match act {
        Subcondition(box Always(act)) => esc_action(act),
        Subcondition(box cond) => {
            Subcondition(Box::new(eval_static_conditionals(cond)))
        }
        otherwise => otherwise,
    }
}

/// Simplifies redundant conditionals.
///
/// For example, "Always(Always(action))" is equivalent to
/// "Always(action)".
fn eval_redundant_conditions(cond: ast::Condition) -> ast::Condition {
    use dna::ast::Condition::{ActionCompare, Always, BinCompare, RangeCompare};
    use dna::ast::Action::Subcondition;
    match cond {
        Always(Subcondition(box cond)) => eval_redundant_conditions(cond),
        Always(act) => Always(erc_action(act)),
        RangeCompare {
            value,
            bound_a,
            bound_b,
            affirmed,
            denied,
        } => RangeCompare {
            value: value,
            bound_a: bound_a,
            bound_b: bound_b,
            affirmed: erc_action(affirmed),
            denied: erc_action(denied),
        },
        BinCompare {
            operation,
            lhs,
            rhs,
            affirmed,
            denied,
        } => BinCompare {
            operation: operation,
            lhs: lhs,
            rhs: rhs,
            affirmed: erc_action(affirmed),
            denied: erc_action(denied),
        },
        ActionCompare {
            actor_type,
            action,
            affirmed,
            denied,
        } => ActionCompare {
            actor_type: actor_type,
            action: erc_action(action),
            affirmed: erc_action(affirmed),
            denied: erc_action(denied),
        },
    }
}

fn erc_action(act: ast::Action) -> ast::Action {
    use dna::ast::Action::Subcondition;
    match act {
        Subcondition(box cond) => {
            Subcondition(Box::new(eval_redundant_conditions(cond)))
        }
        otherwise => otherwise,
    }
}

#[derive(Debug, Clone)]
pub struct ThoughtCycle {
    thoughts: Vec<ast::Condition>,
    cycle_offset: usize,
}

pub fn cycle_detect(dna: &DNA) -> Result<ThoughtCycle, parsing::Failure> {
    if !dna.valid() {
        return Err(parsing::Failure::DNAEmpty)
    }
    let f = |offset: usize| -> usize {
        parsing::Parser::new(dna, offset).next().unwrap().offset()
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
    let mut new_iter = parsing::Parser::new(dna, 0);
    let mut thought: parsing::Decision;
    let mut thoughts = Vec::new();
    for _ in 0..(mu + lam) {
        thought = new_iter.next().unwrap().into_result()?;
        let simplified = simplify(thought.tree);
        thoughts.push(simplified);
    }
    Ok(ThoughtCycle {
        thoughts: thoughts,
        cycle_offset: mu,
    })
}
