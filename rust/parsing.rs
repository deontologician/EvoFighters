use parsing::{Parser, Thought};

pub mod dna {
    #[derive(Ord, PartialOrd, Eq, PartialEq, Show, FromPrimitive)]
    pub enum Condition {
        Always,
        InRange,
        LessThan,
        GreaterThan,
        EqualTo,
        NotEqualTo,
        MyLastAction,
        TargetLastAction,
    }

    #[derive(Ord, PartialOrd, Eq, PartialEq, Show, FromPrimitive)]
    pub enum Value {
        Literal,
        Random,
        Me,
        Target,
    }

    #[derive(Ord, PartialOrd, Eq, PartialEq, Show, FromPrimitive)]
    pub enum Action {
        Subcondition,
        Attack,
        Mate,
        Defend,
        Use,
        Signal,
        Take,
        Wait,
        Flee,
    }

    #[derive(Ord, PartialOrd, Eq, PartialEq, Show, FromPrimitive)]
    pub enum Attribute {
        Energy,
        Signal,
        Generation,
        Kills,
        Survived,
        NumChildren,
        TopItem,
    }

    #[derive(Ord, PartialOrd, Eq, PartialEq, Show, FromPrimitive)]
    pub enum Item {
        Food,
        GoodFood,
        BetterFood,
        ExcellentFood,
    }

    #[derive(Ord, PartialOrd, Eq, PartialEq, Show, FromPrimitive)]
    pub enum Signal {
        Red,
        Yellow,
        Blue,
        Purple,
        Orange,
        Green,
    }

    #[derive(Ord, PartialOrd, Eq, PartialEq, Show, FromPrimitive)]
    pub enum Damage {
        Fire,
        Ice,
        Electricity
    }

    #[derive(Ord, PartialOrd, Eq, PartialEq, Show)]
    pub enum ConditionTree {
        Always(ActionTree),
        RangeCompare {
            value: ValueTree,
            bound_a: ValueTree,
            bound_b: ValueTree,
            affirmed: ActionTree,
            denied: ActionTree,
        },
        BinCompare {
            operation: Condition,
            lhs: ValueTree,
            rhs: ValueTree,
            affirmed: ActionTree,
            denied: ActionTree,
        },
        ActionCompare {
            actor: Condition,
            action: ActionTree,
            affirmed: ActionTree,
            denied: ActionTree,
        }
    }

    #[derive(Ord, PartialOrd, Eq, PartialEq, Show)]
    pub enum ValueTree {
        Literal(u8),
        Random,
        Attribute {
            target: Value,
            attribute: Attribute,
        }
    }

    #[derive(Ord, PartialOrd, Eq, PartialEq, Show)]
    pub enum ActionTree {
        Subcondition(Box<ConditionTree>),
        AttackDefend {
            action: Action,
            damage: Damage,
        },
        Signal(Signal),
        Use,
        Take,
        Mate,
        Wait,
        Flee
    }
}

pub mod parsing {
    use dna::*;
    use std::iter::Iterator;
    use std::option::Option::*;
    use std::num::FromPrimitive;

    #[derive(Show)]
    pub struct Thought {
        tree: ConditionTree,
        icount: usize,
        skipped: usize,
    }

    #[derive(Show)]
    pub struct Parser<'a> {
        pub icount: usize,
        progress: usize,
        skipped: usize,
        depth: usize,
        dna: &'a[u8],  // TODO: have creatures share DNA if possible
    }

    impl<'a> Parser<'a> {
        /// Handles parsing from dna and returning a parse tree which
        /// represents a creature's thought process in making
        /// encounter decisions
        pub fn new(dna: &[u8]) -> Parser {
            Parser {
                icount : 0,
                progress : 0,
                skipped : 0,
                dna : dna,
                depth : 0,
            }
        }

        fn next_valid<T: FromPrimitive>(&mut self, minimum: u8) -> T {
            let mut next_val = FromPrimitive::from_u8(
                self.dna[self.progress % self.dna.len()]);
            self.icount += 1;
            self.progress += 1;
            next_val.unwrap()
        }

        fn parse_condition(&mut self) -> ConditionTree {
            match self.next_valid(0) {
                Condition::Always => ConditionTree::Always(ActionTree::Mate),
                _ => ConditionTree::Always(ActionTree::Mate)
                    // Working here
            }
        }
    }
    impl<'a> Iterator for Parser<'a> {
        type Item = Thought;
        fn next(&mut self) -> Option<Thought> {
            Some(Thought {
                tree: self.parse_condition(),
                icount: 0,
                skipped: 0,
            })
        }
    }
}

fn main() {
    let my_dna = vec![1,2,3,4,5,6,7];
    let mut parser = Parser::new(my_dna.as_slice());
    println!("{:?}", parser.next().unwrap());
}

