/// Basic type alias to hold DNA information
pub type DNA = Vec<i8>;

/// Produce an empty DNA value. This is used by feeders.
pub fn empty_dna() -> DNA {
    Vec::with_capacity(0)
}

/// The lexical module is for raw enums that are used as tokens from
/// the `DNA`, and are fed to the parser.
pub mod lex {
    use std::fmt;

    enum_from_primitive! {
        #[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Copy, Clone)]
        /// Conditions are parsed from `DNA` and specify a test to do
        /// at fight time
        pub enum Condition {
            /// Always do the specified action
            Always,
            /// Do the specified action if the target value is in a
            /// particular range
            InRange,
            /// Do the specified action if the target value is less
            /// than another value
            LessThan,
            /// Do the specified action if the target value is greater
            /// than another value
            GreaterThan,
            /// Do the specified action if the target value is equal
            /// to another value
            EqualTo,
            /// Do the specified action if the target value is not
            /// equal to another value
            NotEqualTo,
            /// Do the specified action if my last action is the specified value
            MyLastAction,
            /// Do the specified action if the other fighter's last action is
            /// the specified value
            OtherLastAction,
            // pay attention to settings::MAX_GENE_VALUE if adding items
        }
    }

    enum_from_primitive! {
        #[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Copy, Clone)]
        /// Values are parsed from `DNA` and specify how to get a value at fight time
        pub enum Value {
            /// A literal value is hardcoded in the `DNA` itself
            Literal,
            /// A random value will be generated each time
            Random,
            /// An attribute from the current fighter will be used as the value
            Me,
            /// An attribute from the opponent will be used as the value
            Other,
            // pay attention to settings::MAX_GENE_VALUE if adding items
        }
    }


    enum_from_primitive! {
        #[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Copy, Clone)]
        /// Actions are parsed from `DNA` and specify an action to take
        pub enum Action {
            /// Subconditions check something and fork into two possible actions to take
            Subcondition,
            /// Attack the opponent
            Attack,
            /// Mate with the opponent
            Mate,
            /// Defend against the opponent (who may or may not be attacking)
            Defend,
            /// Attempt to eat an item from your inventory
            Eat,
            /// Signal a color to the opponent
            Signal,
            /// Attempt to take something from the opponent
            Take,
            /// Don't do anything
            Wait,
            /// Attempt to flee the encounter
            Flee,
            // If adding an action, update settings::MAX_GENE_VALUE to match
        }
    }

    enum_from_primitive! {
        #[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Copy, Clone, Serialize, Deserialize)]
        /// Attributes are parsed from `DNA`. When a `Value` requires looking
        /// at a fighter's attributes, this decides which one is selected
        pub enum Attribute {
            /// the value of the fighter's energy
            Energy,
            /// The value of the signal the fighter is signalling
            Signal,
            /// The value of the generation the fighter belongs to
            Generation,
            /// The number of kills the fighter has
            Kills,
            /// The number of fights the fighter has survived
            Survived,
            /// The number of children the fighter has sired
            NumChildren,
            /// The value of the top item in the fighter's inventory
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
        #[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Copy, Clone, Serialize, Deserialize)]
        /// Parsed from `DNA`, this represents the value of an item in the inventory
        pub enum Item {
            Food = 1,
            GoodFood,
            BetterFood,
            ExcellentFood,
            // pay attention to settings::MAX_GENE_VALUE if adding items
        }
    }

    enum_from_primitive! {
        #[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Copy, Clone, Serialize, Deserialize)]
        /// Parsed from `DNA`, this represents the color of a signal
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
        #[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Copy, Clone, Serialize, Deserialize)]
        /// Parsed from `DNA`, this represents a damage type
        pub enum DamageType {
            /// Fire damage
            Fire,
            /// Ice damage
            Ice,
            /// Electricity damage
            Electricity,
            // pay attention to settings::MAX_GENE_VALUE if adding items
        }
    }
}

/// The `ast` module is structured trees of conditions and actions
/// that need to be evaluated at fight time in order to determine
/// which action the fighter should take. Unlike the `lex` module,
/// these are not simply tokens.
pub mod ast {
    use std::fmt;
    use dna::lex;

    #[derive(PartialEq, Eq, Debug, Copy, Clone, Serialize, Deserialize)]
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

    #[derive(PartialEq, Eq, Debug, Copy, Clone, Serialize, Deserialize)]
    pub enum ActorType {
        Me, Other
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
    pub enum Condition {
        Always(Action),
        RangeCompare {
            value: Value,
            bound_a: Value,
            bound_b: Value,
            affirmed: Action,
            denied: Action,
        },
        BinCompare {
            operation: BinOp,
            lhs: Value,
            rhs: Value,
            affirmed: Action,
            denied: Action,
        },
        ActionCompare {
            actor_type: ActorType,
            action: Action,
            affirmed: Action,
            denied: Action,
        }
    }

    #[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub enum Value {
        Literal(u8),
        Random,
        Me(lex::Attribute),
        Other(lex::Attribute),
    }

    impl fmt::Display for Value {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                Value::Literal(lit) =>
                    write!(f, "{}", lit),
                Value::Random => write!(f, "a random number"),
                Value::Me(ref attr) => write!(f, "my {}", attr),
                Value::Other(ref attr) => write!(f, "my target's {}", attr),
            }
        }
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
    pub enum Action {
        Subcondition(Box<Condition>),
        Attack(lex::DamageType),
        Defend(lex::DamageType),
        Signal(lex::Signal),
        Eat,
        Take,
        Mate,
        Wait,
        Flee
    }
}
