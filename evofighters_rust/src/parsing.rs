use std::iter::Iterator;
use std::option::Option::*;
use std::convert::From;
use num::FromPrimitive;

use dna::{lex,ast,DNA,DNAIter};
use settings;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Failure {
    NoThoughtsYet,
    TookTooLong,
    ParseTreeTooDeep,
}

#[derive(Debug, Clone)]
pub struct Decision {
    pub tree: Box<ast::Condition>,
    pub offset: usize,
    pub icount: usize,
    pub skipped: usize,
}

pub struct Indecision {
    pub reason: Failure,
    pub icount: usize,
    pub skipped: usize,
    pub offset: usize,
}

impl From<Indecision> for Failure {
    fn from(indecision: Indecision) -> Failure {
        indecision.reason
    }
}

pub type Thought = Result<Decision, Indecision>;

fn feeder_decision() -> Thought {
    Ok(Decision {
        tree: Box::new(ast::Condition::Always(ast::Action::Wait)),
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
pub struct Parser {
    icount: usize,
    skipped: usize,
    depth: usize,
    for_feeder: bool,
    dna_stream: DNAIter,
}

impl Parser {
    /// Handles parsing from dna and returning a parse tree which
    /// represents a creature's thought process in making
    /// encounter decisions
    pub fn new(dna: DNA, offset: usize) -> Parser {
        Parser {
            icount: 0,
            skipped: 0,
            dna_stream: dna.base_stream(offset),
            depth: 0,
            for_feeder: false,
        }
    }

    pub fn feeder_new() -> Parser {
        Parser {
            icount: 0,
            skipped: 0,
            dna_stream : DNA::feeder().base_stream(0),
            depth: 0,
            for_feeder: true,
        }
    }

    pub fn current_offset(&self) -> usize {
        self.dna_stream.offset()
    }

    fn next_valid<T: FromPrimitive>(&mut self, minimum: i8) -> ParseResult<T> {
        let mut next_i8 = self.dna_stream.next();
        let mut next_val : Option<T> =
            next_i8.and_then(FromPrimitive::from_i8);
        self.icount += 1;
        while next_val.is_none() || next_i8.unwrap() < minimum {
            next_i8 = self.dna_stream.next();
            next_val = next_i8.and_then(FromPrimitive::from_i8);
            self.skipped += 1;
            if self.icount + self.skipped > settings::MAX_THINKING_STEPS {
                return Err(Failure::TookTooLong)
            }
        }
        Ok(next_val.unwrap())
    }

    fn parse_condition(&mut self) -> ParseResult<Box<ast::Condition>> {
        if self.depth > settings::MAX_TREE_DEPTH {
            return Err(Failure::ParseTreeTooDeep)
        }
        Ok(Box::new(match self.next_valid(0)? {
            lex::Condition::Always =>
                ast::Condition::Always(self.parse_action()?),
            lex::Condition::InRange =>
                ast::Condition::RangeCompare {
                    value: self.parse_value()?,
                    bound_a: self.parse_value()?,
                    bound_b: self.parse_value()?,
                    affirmed: self.parse_action()?,
                    denied: self.parse_action()?,
                },
            cnd @ lex::Condition::LessThan |
            cnd @ lex::Condition::GreaterThan |
            cnd @ lex::Condition::EqualTo |
            cnd @ lex::Condition::NotEqualTo =>
                ast::Condition::BinCompare {
                    operation: match cnd {
                        lex::Condition::LessThan => ast::BinOp::LT,
                        lex::Condition::GreaterThan => ast::BinOp::GT,
                        lex::Condition::EqualTo => ast::BinOp::EQ,
                        lex::Condition::NotEqualTo => ast::BinOp::NE,
                        _ => panic!("Not possible")
                    },
                    lhs: self.parse_value()?,
                    rhs: self.parse_value()?,
                    affirmed: self.parse_action()?,
                    denied: self.parse_action()?,
                },
            actor @ lex::Condition::MyLastAction |
            actor @ lex::Condition::OtherLastAction =>
                ast::Condition::ActionCompare {
                    actor_type: match actor {
                        lex::Condition::MyLastAction => ast::ActorType::Me,
                        lex::Condition::OtherLastAction => ast::ActorType::Other,
                        _ => panic!("Not possible")
                    },
                    action: self.parse_action()?,
                    affirmed: self.parse_action()?,
                    denied: self.parse_action()?,
                }
        }))
    }

    fn parse_action(&mut self) -> ParseResult<ast::Action> {
        Ok(match self.next_valid(0)? {
            lex::Action::Subcondition => {
                self.depth += 1;
                let subcond = ast::Action::Subcondition(
                    self.parse_condition()?);
                self.depth -= 1;
                subcond
            },
            lex::Action::Attack => ast::Action::Attack(self.next_valid(0)?),
            lex::Action::Defend => ast::Action::Defend(self.next_valid(0)?),
            lex::Action::Signal => ast::Action::Signal(self.next_valid(0)?),
            lex::Action::Eat    => ast::Action::Eat,
            lex::Action::Take   => ast::Action::Take,
            lex::Action::Mate   => ast::Action::Mate,
            lex::Action::Wait   => ast::Action::Wait,
            lex::Action::Flee   => ast::Action::Flee,
        })
    }

    fn parse_value(&mut self) -> ParseResult<ast::Value> {
        Ok(match self.next_valid(0)? {
            lex::Value::Literal => ast::Value::Literal(self.next_valid(0)?),
            lex::Value::Random  => ast::Value::Random,
            lex::Value::Me      => ast::Value::Me(self.next_valid(0)?),
            lex::Value::Other   => ast::Value::Other(self.next_valid(0)?),
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
