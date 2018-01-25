use std::fmt;
use std::rc;

pub type DNA = Vec<i8>;

pub fn empty_dna() -> DNA {
    Vec::with_capacity(0)
}

enum_from_primitive! {
#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Copy, Clone)]
pub enum Condition {
    Always,
    InRange,
    LessThan,
    GreaterThan,
    EqualTo,
    NotEqualTo,
    MyLastAction,
    OtherLastAction,
    // pay attention to settings::MAX_GENE_VALUE if adding items
}
}

enum_from_primitive! {
#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Copy, Clone)]
pub enum Value {
    Literal,
    Random,
    Me,
    Other,
    // pay attention to settings::MAX_GENE_VALUE if adding items
}
}


enum_from_primitive! {
#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Copy, Clone)]
pub enum Action {
    Subcondition,
    Attack,
    Mate,
    Defend,
    Eat,
    Signal,
    Take,
    Wait,
    Flee,
    // If adding an action, update settings::MAX_GENE_VALUE to match
}
}

enum_from_primitive! {
#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Copy, Clone, RustcEncodable, RustcDecodable)]
pub enum Attribute {
    Energy,
    Signal,
    Generation,
    Kills,
    Survived,
    NumChildren,
    TopItem,
    // pay attention to settings::MAX_GENE_VALUE if adding items
}
}

impl fmt::Display for Attribute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Attribute::Energy => write!(f, "energy"),
            Attribute::Signal => write!(f, "signal"),
            Attribute::Generation => write!(f, "generation"),
            Attribute::Kills => write!(f, "kills"),
            Attribute::Survived => write!(f, "encounters survived"),
            Attribute::NumChildren => write!(f, "number of children"),
            Attribute::TopItem => write!(f, "top inventory item"),
        }
    }
}

enum_from_primitive! {
#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Copy, Clone, RustcEncodable, RustcDecodable)]
pub enum Item {
    Food = 1,
    GoodFood,
    BetterFood,
    ExcellentFood,
    // pay attention to settings::MAX_GENE_VALUE if adding items
}
}

enum_from_primitive! {
#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Copy, Clone, RustcEncodable, RustcDecodable)]
pub enum Signal {
    Red = 1,
    Yellow,
    Blue,
    Purple,
    Orange,
    Green,
    // pay attention to settings::MAX_GENE_VALUE if adding items
}
}

enum_from_primitive! {
#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Copy, Clone, RustcEncodable, RustcDecodable)]
pub enum DamageType {
    Fire,
    Ice,
    Electricity,
    // pay attention to settings::MAX_GENE_VALUE if adding items
}
}


#[derive(PartialEq, Eq, Debug, Copy, Clone, RustcEncodable, RustcDecodable)]
pub enum BinOp {
    LT, GT, EQ, NE
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BinOp::LT => write!(f, "less than"),
            BinOp::GT => write!(f, "greater than"),
            BinOp::EQ => write!(f, "equal to"),
            BinOp::NE => write!(f, "not equal to"),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone, RustcEncodable, RustcDecodable)]
pub enum ActorType {
    Me, Other
}

#[derive(Debug, RustcEncodable, RustcDecodable, PartialEq, Eq, Clone)]
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
        operation: BinOp,
        lhs: ValueTree,
        rhs: ValueTree,
        affirmed: ActionTree,
        denied: ActionTree,
    },
    ActionCompare {
        actor_type: ActorType,
        action: ActionTree,
        affirmed: ActionTree,
        denied: ActionTree,
    }
}

#[derive(Debug, Copy, Clone, RustcEncodable, RustcDecodable, PartialEq, Eq)]
pub enum ValueTree {
    Literal(u8),
    Random,
    Me(Attribute),
    Other(Attribute),
}

impl fmt::Display for ValueTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ValueTree::Literal(lit) =>
                write!(f, "{}", lit),
            ValueTree::Random => write!(f, "a random number"),
            ValueTree::Me(ref attr) => write!(f, "my {}", attr),
            ValueTree::Other(ref attr) => write!(f, "my target's {}", attr),
        }
    }
}

#[derive(Debug, RustcEncodable, RustcDecodable, PartialEq, Eq, Clone)]
pub enum ActionTree {
    Subcondition(Box<ConditionTree>),
    Attack(DamageType),
    Defend(DamageType),
    Signal(Signal),
    Eat,
    Take,
    Mate,
    Wait,
    Flee
}

// All of this is the pretty printer I couldn't get working

// impl ActionTree {
//     fn format<'a>(&self, f: &'a mut fmt::Formatter<'a>) -> fmt::Result {
//         let mut pp: PrettyPrinter<'a> = PrettyPrinter::new(f);
//         pp.emit_action(self)
//     }
// }

// impl fmt::String for ActionTree {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         Ok(())
//     }
// }


// pub struct PrettyPrinter<'a> {
//     f: &'a mut fmt::Formatter<'a>,
//     current_indent: usize,
// }

// impl <'a> PrettyPrinter<'a> {
//     pub fn new(f: &'a mut fmt::Formatter<'a>) -> PrettyPrinter<'a> {
//         PrettyPrinter{f: f, current_indent: 0}
//     }

//     pub fn indent(&mut self) {
//         self.current_indent += 1;
//     }

//     pub fn dedent(&mut self) {
//         if self.current_indent > 0 {
//             self.current_indent -= 1;
//         }
//     }

//     pub fn emit_indentation(&mut self) -> fmt::Result {
//         const INDENT: &'static str = "  ";
//         let mut remaining: usize = self.current_indent;
//         while remaining > 1 {
//             try!(self.f.write_str(INDENT));
//             remaining -= 1;
//         }
//         if remaining == 1 {
//             self.f.write_str(INDENT)
//         } else {
//             Ok(())
//         }
//     }

//     pub fn emit_cond(&mut self, cond: &ConditionTree) -> fmt::Result {
//         match *cond {
//             ConditionTree::Always(ref act) => {
//                 try!(self.emit_indentation());
//                 write!(self.f, "Always:\n");
//                 self.indent();
//                 try!(self.emit_action(act));
//                 self.dedent()
//             },
//             ConditionTree::RangeCompare{
//                 ref value,
//                 ref bound_a,
//                 ref bound_b,
//                 ref affirmed,
//                 ref denied,
//             } => {
//                 try!(self.emit_indentation());
//                 write!(self.f, "if {} is in the range {} to {}:\n",
//                        value, bound_a, bound_b);
//                 self.indent();
//                 try!(self.emit_action(affirmed));
//                 self.dedent();
//                 try!(self.emit_indentation());
//                 write!(self.f, "else:\n");
//                 self.indent();
//                 try!(self.emit_action(denied));
//                 self.dedent()
//             },
//             ConditionTree::BinCompare{
//                 ref operation,
//                 ref lhs,
//                 ref rhs,
//                 ref affirmed,
//                 ref denied,
//             } => {
//                 try!(self.emit_indentation());
//                 write!(self.f, "if {} {} {}:\n", lhs, operation, rhs);
//                 self.indent();
//                 try!(self.emit_action(affirmed));
//                 self.dedent();
//                 try!(self.emit_indentation());
//                 write!(self.f, "else:");
//                 self.indent();
//                 try!(self.emit_action(denied));
//                 self.dedent();
//             },
//             ConditionTree::ActionCompare{
//                 ref actor,
//                 ref action,
//                 ref affirmed,
//                 ref denied,
//             } => {
//                 try!(self.emit_indentation());
//                 panic!("no action!") //working here
//             }
//         }
//         Ok(())
//     }

//     pub fn emit_action(&mut self, action: &ActionTree) -> fmt::Result {
//         Ok(())
//     }
// }
