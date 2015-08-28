use std::iter::Iterator;
use std::option::Option::*;
use std::num::FromPrimitive;
use std::error::FromError;

use dna::*;
use settings;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Failure {
    NoThoughtsYet,
    TookTooLong,
    ParseTreeTooDeep,
}

#[derive(Debug, Clone)]
pub struct Decision {
    tree: Box<ConditionTree>,
    icount: usize,
    skipped: usize,
    offset: usize,
}

pub struct Indecision {
    reason: Failure,
    icount: usize,
    skipped: usize,
    offset: usize,
}

impl FromError<Indecision> for Failure {
    fn from_error(indecision: Indecision) -> Failure {
        indecision.reason
    }
}

type Thought = Result<Decision, Indecision>;

fn feeder_decision() -> Thought {
    Ok(Decision {
        tree: Box::new(ConditionTree::Always(ActionTree::Wait)),
        icount: 0,
        skipped: settings::MAX_THINKING_STEPS + 1,
        offset: 0,
    })
}

pub fn icount(thought: &Thought) -> usize {
    match *thought {
        Ok(Decision{icount, ..}) => icount,
        Err(Indecision{icount, ..}) => icount,
    }
}

pub fn skipped(thought: &Thought) -> usize {
    match *thought {
        Ok(Decision{skipped, ..}) => skipped,
        Err(Indecision{skipped, ..}) => skipped,
    }
}

type ParseResult<T> = Result<T, Failure>;

#[derive(Debug, Clone)]
struct DNAIter {
    dna: DNA,
    pub progress: usize,
    len: usize,
}

impl DNAIter {
    fn new(dna: DNA, offset: usize) -> DNAIter {
        let len = dna.len(); // avoid borrow issues
        DNAIter {
            dna: dna,
            progress: offset % len,
            len: len,
        }
    }
}

impl Iterator for DNAIter {
    type Item = i8;
    fn next(&mut self) -> Option<i8> {
        let ret = Some(self.dna[self.progress]);
        self.progress = (self.progress + 1) % self.len;
        ret
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, None)
    }
}

#[derive(Debug, Clone)]
pub struct Parser {
    icount: usize,
    skipped: usize,
    depth: usize,
    for_feeder: bool,
    dna: DNAIter,
}

impl Parser {
    /// Handles parsing from dna and returning a parse tree which
    /// represents a creature's thought process in making
    /// encounter decisions
    pub fn new(dna: DNA, offset: usize) -> Parser {
        Parser {
            icount : 0,
            skipped : 0,
            dna : DNAIter::new(dna, offset),
            depth : 0,
            for_feeder: false,
        }
    }

    pub fn feeder_new() -> Parser {
        Parser {
            icount: 0,
            skipped: 0,
            dna : DNAIter::new(vec![-1], 0),
            depth: 0,
            for_feeder: true,
        }
    }

    pub fn current_offset(&self) -> usize {
        self.dna.progress
    }

    fn next_valid<T: FromPrimitive>(&mut self, minimum: i8) -> ParseResult<T> {
        let mut next_i8 = self.dna.next();
        let mut next_val : Option<T> =
            next_i8.and_then(FromPrimitive::from_i8);
        self.icount += 1;
        while next_val.is_none() || next_i8.unwrap() < minimum {
            next_i8 = self.dna.next();
            next_val = next_i8.and_then(FromPrimitive::from_i8);
            self.skipped += 1;
            if self.icount + self.skipped > settings::MAX_THINKING_STEPS {
                return Err(Failure::TookTooLong)
            }
        }
        Ok(next_val.unwrap())
    }

    fn parse_condition(&mut self) -> ParseResult<Box<ConditionTree>> {
        if self.depth > settings::MAX_TREE_DEPTH {
            return Err(Failure::ParseTreeTooDeep)
        }
        Ok(box match try!(self.next_valid(0)) {
            Condition::Always =>
                ConditionTree::Always(try!(self.parse_action())),
            Condition::InRange =>
                ConditionTree::RangeCompare {
                    value: try!(self.parse_value()),
                    bound_a: try!(self.parse_value()),
                    bound_b: try!(self.parse_value()),
                    affirmed: try!(self.parse_action()),
                    denied: try!(self.parse_action()),
                },
            cnd @ Condition::LessThan |
            cnd @ Condition::GreaterThan |
            cnd @ Condition::EqualTo |
            cnd @ Condition::NotEqualTo =>
                ConditionTree::BinCompare {
                    operation: match cnd {
                        Condition::LessThan => BinOp::LT,
                        Condition::GreaterThan => BinOp::GT,
                        Condition::EqualTo => BinOp::EQ,
                        Condition::NotEqualTo => BinOp::NE,
                        _ => panic!("Not possible")
                    },
                    lhs: try!(self.parse_value()),
                    rhs: try!(self.parse_value()),
                    affirmed: try!(self.parse_action()),
                    denied: try!(self.parse_action()),
                },
            actor @ Condition::MyLastAction |
            actor @ Condition::OtherLastAction =>
                ConditionTree::ActionCompare {
                    actor_type: match actor {
                        Condition::MyLastAction => ActorType::Me,
                        Condition::OtherLastAction => ActorType::Other,
                        _ => panic!("Not possible")
                    },
                    action: try!(self.parse_action()),
                    affirmed: try!(self.parse_action()),
                    denied: try!(self.parse_action()),
                }
        })
    }

    fn parse_action(&mut self) -> ParseResult<ActionTree> {
        Ok(match try!(self.next_valid(0)) {
            Action::Subcondition => {
                self.depth += 1;
                let subcond = ActionTree::Subcondition(
                    try!(self.parse_condition()));
                self.depth -= 1;
                subcond
            },
            Action::Attack => ActionTree::Attack(try!(self.next_valid(0))),
            Action::Defend => ActionTree::Defend(try!(self.next_valid(0))),
            Action::Signal => ActionTree::Signal(try!(self.next_valid(0))),
            Action::Eat => ActionTree::Eat,
            Action::Take => ActionTree::Take,
            Action::Mate => ActionTree::Mate,
            Action::Wait => ActionTree::Wait,
            Action::Flee => ActionTree::Flee,
        })
    }

    fn parse_value(&mut self) -> ParseResult<ValueTree> {
        Ok(match try!(self.next_valid(0)) {
            Value::Literal => ValueTree::Literal(try!(self.next_valid(0))),
            Value::Random => ValueTree::Random,
            Value::Me => ValueTree::Me(try!(self.next_valid(0))),
            Value::Other => ValueTree::Other(try!(self.next_valid(0))),
        })
    }
}

impl Iterator for Parser {
    type Item = Thought;
    fn next(&mut self) -> Option<Thought> {
        if self.for_feeder {
            return Some(feeder_decision());
        }
        let value = Some(match self.parse_condition() {
            Err(msg) => Err(Indecision {
                icount: self.icount,
                skipped: self.skipped,
                reason: msg,
                offset: self.current_offset(),
            }),
            Ok(tree) => Ok(Decision {
                icount: self.icount,
                skipped: self.skipped,
                tree: tree,
                offset: self.current_offset(),
            })
        });
        // Reset counts so the creatures get a new budget next time!
        self.icount = 0;
        self.skipped = 0;
        value
    }
}
