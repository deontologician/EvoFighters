use std::fmt;

#[derive(Ord, PartialOrd, Eq, PartialEq, Show, FromPrimitive, Copy)]
pub enum Condition {
    Always,
    InRange,
    LessThan,
    GreaterThan,
    EqualTo,
    NotEqualTo,
    MyLastAction,
    OtherLastAction,
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Show, FromPrimitive, Copy)]
pub enum Value {
    Literal,
    Random,
    Me,
    Other,
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Show, FromPrimitive, Copy)]
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

#[derive(Ord, PartialOrd, Eq, PartialEq, Show, FromPrimitive, Copy)]
pub enum Attribute {
    Energy,
    Signal,
    Generation,
    Kills,
    Survived,
    NumChildren,
    TopItem,
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Show, FromPrimitive, Copy)]
pub enum Item {
    Food,
    GoodFood,
    BetterFood,
    ExcellentFood,
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Show, FromPrimitive, Copy)]
pub enum Signal {
    Red,
    Yellow,
    Blue,
    Purple,
    Orange,
    Green,
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Show, FromPrimitive, Copy)]
pub enum DamageType {
    Fire,
    Ice,
    Electricity
}


#[derive(PartialEq, Eq, Show, Copy)]
pub enum BinOp {
    LT, GT, EQ, NE
}

impl String for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LT => write!(f, "is less than"),
            GT => write!(f, "is greater than"),
            EQ => write!(f, "is equal to"),
            NE => write!(f, "is not equal to"),
        }
    }
}

#[derive(PartialEq, Eq, Show, Copy)]
pub enum Actor {
    Me, Other
}

#[derive(Show)]
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
        actor: Actor,
        action: ActionTree,
        affirmed: ActionTree,
        denied: ActionTree,
    }
}

impl fmt::String for ConditionTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Always(act) => {
                write!(f, "always {{{}}}", act);
            },
            RangeCompare{value, bound_a, bound_b, affirmed, denied} => {
                let rng_min = min(rng.bound_a, rng.bound_b);
                let rng_max = max(rng.bound_a, rng.bound_b);
                write!(f, "if ({} is in the range {} to {}){{{}}}else{{{}}}",
                       rng.value, rng_min, rng_max, affirmed, denied);
            },
            BinCompare{operation, lhs, rhs, ..} => {
                panic!("no binop!")
            },
            ActionCompare{actor, action, ..} => {
                panic!("no action!")
            }
        }
    }
}

#[derive(Show, Copy)]
pub enum ValueTree {
    Literal(u8),
    Random,
    Me(Attribute),
    Other(Attribute),
}

#[derive(Show)]
pub enum ActionTree {
    Subcondition(Box<ConditionTree>),
    Attack(DamageType),
    Defend(DamageType),
    Signal(Signal),
    Use,
    Take,
    Mate,
    Wait,
    Flee
}
