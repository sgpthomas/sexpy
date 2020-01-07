mod error;
mod parsers;
mod std_impls;
pub use error::{context, SexpyError};
pub use parsers::*;
pub use sexpy_derive::Sexpy;

// List of all the parsers used by the derive function so that automatically
// deriving things works.
pub use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{
        alpha1, alphanumeric0, char, digit1, multispace0, multispace1, none_of,
    },
    combinator::{cut, opt},
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
        match preceded(wordbreak0, Self::sexp_parse)(input) {
            Ok((_, x)) => Ok(x),
            Err(Err::Error(e)) => Err(e.convert_error(input)),
            Err(Err::Failure(e)) => Err(e.convert_error(input)),
            Err(Err::Incomplete(_)) => Err("Need more bytes to nom".to_string()),
        }
    }
    /// Takes a string and tries calling the parser for this trait on it.
    fn parse_verbose(input: &str) -> Result<Self, String>
    where
        Self: Sized,
    {
        match preceded(wordbreak0, Self::sexp_parse)(input) {
            Ok((_, x)) => Ok(x),
            Err(Err::Error(e)) => Err(e.convert_error_verbose(input)),
            Err(Err::Failure(e)) => Err(e.convert_error_verbose(input)),
            Err(Err::Incomplete(_)) => {
                Err("Incomplete input, need more bytes to nom".to_string())
            }
        }
    }

    /// The core parsing function that should be defined for each trait.
    fn sexp_parse<'a>(
        input: &'a str,
    ) -> IResult<&'a str, Self, SexpyError<&'a str>>
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
            width: i64,
        }

        assert_eq!(
            Portdef::parse("(port foo -32)"),
            Ok(Portdef {
                name: "foo".to_string(),
                width: -32,
            })
        )
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

    #[test]
    fn same_name() {
        #[derive(Sexpy, Debug, PartialEq)]
        #[sexpy(nosurround, head = "foo")]
        struct Left {
            item: String,
        }

        #[derive(Sexpy, Debug, PartialEq)]
        #[sexpy(nosurround, head = "foo-bar")]
        struct Right {
            item: u64,
        }

        #[derive(Sexpy, Debug, PartialEq)]
        enum Either {
            Left { data: Left },
            Right { data: Right },
        }

        assert_eq!(
            Either::parse("(either foo hi)"),
            Ok(Either::Left {
                data: Left {
                    item: "hi".to_string()
                }
            })
        );

        assert_eq!(
            Either::parse("(either foo-bar 32)"),
            Ok(Either::Right {
                data: Right { item: 32 }
            })
        );
    }

    #[test]
    fn no_head() {
        #[derive(Sexpy, Debug, PartialEq)]
        #[sexpy(nohead)]
        enum Plant {
            #[sexpy(head = "cactus")]
            Cactus(String, u64),
            #[sexpy(head = "joshua-tree")]
            JoshuaTree(String, u64),
        }

        assert_eq!(
            Plant::parse("(cactus josh 400)"),
            Ok(Plant::Cactus("josh".to_string(), 400))
        );

        assert_eq!(
            Plant::parse("(joshua-tree carolina 4)"),
            Ok(Plant::JoshuaTree("carolina".to_string(), 4))
        );
    }

    #[test]
    fn vector() {
        #[derive(Sexpy, Debug, PartialEq)]
        struct Song {
            name: String,
            #[sexpy(surround)]
            instrs: Vec<String>,
            notes: Vec<u64>,
        }

        assert_eq!(
            Song::parse("(song purr (piano cat) 11 12 13 12 13)"),
            Ok(Song {
                name: "purr".to_string(),
                instrs: vec!["piano".to_string(), "cat".to_string()],
                notes: vec![11, 12, 13, 12, 13]
            })
        )
    }

    #[test]
    fn comments() {
        #[derive(Sexpy, Debug, PartialEq)]
        struct Song {
            name: String,
            #[sexpy(surround)]
            instrs: Vec<String>,
            notes: Vec<u64>,
        }

        assert_eq!(
            Song::parse(
                "; my cool song\n(song purr (piano cat) ; the good part!\n11 12 13 12 13)"
            ),
            Ok(Song {
                name: "purr".to_string(),
                instrs: vec!["piano".to_string(), "cat".to_string()],
                notes: vec![11, 12, 13, 12, 13]
            })
        )
    }
}
