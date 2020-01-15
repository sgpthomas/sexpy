/*!
Sexpy automatically derives an s-expression parser from Rust type
definitions. The goal is to be able to do this from an AST defintion with minimal annotations.
This is targeted for use in prototyping programming languages
where you want informative parse error messages and human readable syntax (i.e. not JSON)
but you don't want to write a full parser.

## Default Parsers Derived for Types
To get a sense for how to use this library, let's look at the parsers
automatically derived for different types.

The default parsers generated match "head patterns". These match a keyword followed
by a list of arguments. For example, `foo <string> <u32>` is a head pattern with a
head of `foo` that takes a string argument and a unsigned 32-bit integer as the second argument.
This matches `foo cactus 20` but not `foo big cactus`.

### Structs
The default parser generated for a `struct` type uses a lowercased version of the struct
name as the head and the types of the fields as arguments. It parses the head pattern surrounded
by parentheses, brackets, or curly braces.

For example, consider the following:
```rust
#[derive(Sexpy)]
struct Port {
  name: String,
  width: u64
}
```
This generates a parser for patterns of the form `(port <string> <u64>)`.
`(port foo 10)` is parsed into `Port { name: "foo".to_string(), width: 10 }`
and `port foo 10`, `(port 10 foo)` both fail to parse.

### Enums
For enums, a parser is generated for each case in the enum. By default, each parser
uses the enum name as the head and the variant arguments as the pattern arguments.
Each parser matches the pattern surrounded in parentheses, brackets, or curly braces.

For example, consider the following enum definition:
```rust
#[derive(Sexpy)]
enum Plant {
  PalmTree(String, u64),      // parses pattern: (plant <string> <u64>)
  SageBush { height: u64 },   // parses pattern: (plant <u64>)
  BarrelCactus                // parses pattern: (plant)
}
```
This generates the three parsers annotated in the comments.
What happens if two variants have the same arguments, like in the following example?
```rust
#[derive(Sexpy)]
enum Plant {
  Palm(String, u64),   // parses pattern: (plant <string> <u64>)
  Cactus(String, u64)  // parses pattern: (plant <string> <u64>)
}
```
By default, the `cactus` variant would never get parsed. The reason for this is that there
is no way to differentiate between the `Palm` variant sub-parser and the `Cactus` variant
sub-parser; they take the same arguments! There are several ways to deal with this, but
the simplest is to force the variant sub-parsers to use a head. You can do this
with the `#[sexpy(head = "<str>")]` option.

```
#[derive(Sexpy)]
enum Plant {
  #[sexpy(head = "palm")]
  Palm(String, u64),      // parses pattern: (plant palm <string> <u64>)
  #[sexpy(head = "cactus")]
  Cactus(String, u64)     // parses pattern: (plant cactus <string> <u64>)
}
```

### Caveats
It is possible to derive two parsers that parse the exact same pattern. At the moment,
`Sexpy` does nothing to detect and prevent this. It is up to the programmer to resolve
these conflicts. The parsing options should make it easy to resolve them.

### Options
You can modify the pattern the derived parser matches by specifying some attributes.
The following are attributes that work at the type level, i.e:
```
#[derive(Sexpy)]
#[sexpy(...)] // <-----
enum Plant { ... }

// or

#[derive(Sexpy)]
#[sexpy(...)] // <-----
struct Plant { ... }
```

All attributes are specified in a comma separated list in like so:
`#[sexpy(attr = val, attr, ...)]`
Arguments are taken in the form `<attr> = <val>`. For example when providing a head, which takes a
string argument, it looks like `head = "custom-name"`. A bool argument looks like
`surround = true`.

| Attribute    | Argument | Effect |
|--------------|----------|--------|
| `nohead`     | *none*   | Ignores head and only generates pattern from arguments |
| `head`       | string   | Use custom string as head instead of lowercase type name |
| `surround`   | bool     | When true, match pattern surrounded with parens, brackets, or braces (true by default) |
| `nosurround` | *none*   | Shortcut for `surround = false` |

The following are variant level attributes. They look like:
```
#[derive(Sexpy)]
enum Plant {
  #[sexpy(head = "palm")]  // <-----
  Palm(String, u64),
  ...
}
```

| Attribute    | Argument | Effect |
|--------------|----------|--------|
| `head`       | string   | Use custom string as head instead of lowercase type name |
| `surround`   | bool     | When true, match pattern surrounded with parens, brackets, or braces (false by default) |

!*/

mod error;
#[allow(unused)]
mod parsers;
mod std_impls;

pub use sexpy_derive::Sexpy;

use error::SexpyError;
use nom::{
    character::complete::{alpha1, char, digit1, none_of},
    combinator::opt,
    multi::many0,
    sequence::{preceded, tuple},
    Err, IResult,
};
use parsers::*;

/// The trait that is automatically derived from a type definition. You should not
/// need to implement this manually unless you are writing a parser for some primitive
/// type. Parsers for several common primitive types have already been defined
pub trait Sexpy {
    /// Takes a string and tries calling the parser for this trait on it, converting
    /// any errors into a string using `SexpyError::convert_error`
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

    /// Takes a string and tries calling the parser for this trait on it, converting
    /// any errors into a string using `SexpyError::convert_error_verbose`
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

    /// The parser for this trait. Should be automatically derivable from a type definition
    /// in most cases
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
    fn same_prefix() {
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
    fn enum_differentiation() {
        #[derive(Sexpy, Debug, PartialEq)]
        enum Plant {
            #[sexpy(head = "cactus")]
            Cactus(String, u64),
            #[sexpy(head = "joshua-tree")]
            JoshuaTree(String, u64),
        }

        assert_eq!(
            Plant::parse("(plant cactus josh 400)"),
            Ok(Plant::Cactus("josh".to_string(), 400))
        );

        assert_eq!(
            Plant::parse("(plant joshua-tree carolina 4)"),
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
