use dna::*;
use std::iter::Iterator;
use std::option::Option::*;
use std::num::FromPrimitive;
use settings;

#[derive(Show, PartialEq, Eq, Copy)]
pub enum Failure {
    NoThoughtsYet,
    TookTooLong,
    ParseTreeTooDeep,
}

#[derive(Show)]
pub enum Thought {
    Decision {
        tree: ConditionTree,
        icount: usize,
        skipped: usize,
    },
    Indecision {
        reason: Failure,
        icount: usize,
        skipped: usize,
    }
}

pub type ParseResult<T> = Result<T, Failure>;

#[derive(Show)]
pub struct DNAIter<'a> {
    dna: &'a[u8],
    progress: usize,
    len: usize,
}

impl<'a> DNAIter<'a> {
    fn new(dna: &'a[u8]) -> DNAIter<'a> {
        DNAIter {
            dna: dna,
            progress: 0us,
            len: dna.len(),
        }
    }
}

impl<'a> Iterator for DNAIter<'a> {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        self.progress = (self.progress + 1) % self.len;
        Some(self.dna[self.progress])
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, None)
    }
}

#[derive(Show)]
pub struct Parser<'a> {
    pub icount: usize,
    pub skipped: usize,
    pub depth: usize,
    dna: DNAIter<'a>,
}

impl<'a> Parser<'a> {
    /// Handles parsing from dna and returning a parse tree which
    /// represents a creature's thought process in making
    /// encounter decisions
    pub fn new(dna: &'a[u8]) -> Parser {
        Parser {
            icount : 0,
            skipped : 0,
            dna : DNAIter::new(dna),
            depth : 0,
        }
    }

    fn next_valid<T: FromPrimitive>(&mut self, minimum: u8) -> ParseResult<T> {
        let mut next_u8 = self.dna.next();
        let mut next_val : Option<T> =
            next_u8.and_then(FromPrimitive::from_u8);
        self.icount += 1;
        while next_val.is_none() || next_u8.unwrap() < minimum {
            next_u8 = self.dna.next();
            next_val = next_u8.and_then(FromPrimitive::from_u8);
            self.skipped += 1;
            if self.icount + self.skipped > settings::MAX_THINKING_STEPS {
                return Err(Failure::TookTooLong)
            }
        }
        Ok(next_val.unwrap())
    }

    fn parse_condition(&mut self) -> ParseResult<ConditionTree> {
        if self.depth > settings::MAX_TREE_DEPTH {
            return Err(Failure::ParseTreeTooDeep)
        }
        Ok(match try!(self.next_valid(0)) {
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
                    Box::new(try!(self.parse_condition())));
                self.depth -= 1;
                subcond
            },
            Action::Attack => ActionTree::Attack(try!(self.next_valid(0))),
            Action::Defend => ActionTree::Defend(try!(self.next_valid(0))),
            Action::Signal => ActionTree::Signal(try!(self.next_valid(0))),
            Action::Use => ActionTree::Use,
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

impl<'a> Iterator for Parser<'a> {
    type Item = Thought;
    fn next(&mut self) -> Option<Thought> {
        Some(match self.parse_condition() {
            Err(msg) => Thought::Indecision {
                icount: self.icount,
                skipped: self.skipped,
                reason: msg,
            },
            Ok(tree) => Thought::Decision {
                icount: self.icount,
                skipped: self.skipped,
                tree: tree,
            }
        })
    }
}
