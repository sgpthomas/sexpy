mod parsers;
mod std_impls;
pub use parsers::*;
pub use sexpy_derive::Sexpy;

// List of all the parsers used by the derive function so that automatically
// deriving things works.
pub use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{
        alpha1, alphanumeric0, digit1, multispace0, multispace1,
    },
    combinator::opt,
    error::{context, convert_error, VerboseError},
    multi::many0,
    sequence::{preceded, tuple},
    Err, IResult,
};

pub trait Sexpy {
    /// Takes a string and tries calling the parser for this trait on it.
    fn parse(input: &str) -> Result<Self, String>
    where
        Self: Sized,
    {
        match Self::sexp_parse(input) {
            Ok((_, x)) => Ok(x),
            Err(Err::Error(e)) => Err(convert_error(input, e)),
            Err(Err::Failure(e)) => Err(convert_error(input, e)),
            Err(Err::Incomplete(_)) => Err("Need more bytes to nom".to_string()),
        }
    }

    /// The core parsing function that should be defined for each trait.
    fn sexp_parse<'a>(
        input: &'a str,
    ) -> IResult<&'a str, Self, VerboseError<&'a str>>
    where
        Self: Sized;
}

#[cfg(test)]
mod tests {
    use crate::*;
    use sexpy_derive::Sexpy;

    #[test]
    fn simple_struct() {
        #[derive(Sexpy, Debug, PartialEq)]
        struct Portdef {
            name: String,
            width: u64,
        }

        let input = "(portdef foo 20)";
        let gold = Portdef {
            name: "foo".to_string(),
            width: 20,
        };
        assert_eq!(Portdef::parse(input), Ok(gold))
    }

    #[test]
    fn simple_struct_one_field() {
        #[derive(Sexpy, Debug, PartialEq)]
        struct Portdef {
            name: String,
        }

        let input = "(portdef foo)";
        let gold = Portdef {
            name: "foo".to_string(),
        };
        assert_eq!(Portdef::parse(input), Ok(gold))
    }

    #[test]
    fn simple_struct_no_fields() {
        #[derive(Sexpy, Debug, PartialEq)]
        struct Portdef {}

        assert_eq!(Portdef::parse("(portdef)"), Ok(Portdef {}));
        assert_eq!(Portdef::parse("(portdef   )"), Ok(Portdef {}));
        assert!(Portdef::parse("(portdef hi)").is_err());
    }

    #[test]
    fn struct_rename_head() {
        #[derive(Sexpy, Debug, PartialEq)]
        #[sexpy(head = "port")]
        struct Portdef {
            name: String,
            width: u64,
        }

        let input = "(port foo 32)";
        let gold = Portdef {
            name: "foo".to_string(),
            width: 32,
        };
        assert_eq!(Portdef::parse(input), Ok(gold))
    }

    #[test]
    fn enum_rename_head() {
        #[derive(Sexpy, Debug, PartialEq)]
        #[sexpy(head = "plt")]
        enum Plant {
            PalmTree(String, u64),
            Cactus,
        }

        assert_eq!(
            Plant::parse("(plt test 4)"),
            Ok(Plant::PalmTree("test".to_string(), 4))
        );
        assert_eq!(Plant::parse("(plt)"), Ok(Plant::Cactus));
    }

    #[test]
    fn unit_enum() {
        #[derive(Sexpy, Debug, PartialEq)]
        enum Plant {
            PalmTree,
            Cactus,
        }

        let input = "(plant)";
        assert_eq!(Plant::parse(input), Ok(Plant::PalmTree))
    }

    #[test]
    fn named_enum_fields() {
        #[derive(Sexpy, Debug, PartialEq)]
        enum Plant {
            PalmTree { width: u64, name: String },
            Cactus { height: u64 },
        }

        assert_eq!(
            Plant::parse("(plant 200 cm)"),
            Ok(Plant::PalmTree {
                width: 200,
                name: "cm".to_string()
            })
        )
    }
}
