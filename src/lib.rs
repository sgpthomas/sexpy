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
```rust,ignore
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
```rust,ignore
#[derive(Sexpy)]
enum Plant {
  PalmTree(String, u64),      // parses pattern: (plant <string> <u64>)
  SageBush { height: u64 },   // parses pattern: (plant <u64>)
  BarrelCactus                // parses pattern: (plant)
}
```
This generates the three parsers annotated in the comments.
What happens if two variants have the same arguments, like in the following example?
```rust,ignore
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

```rust,ignore
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
```rust,ignore
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
```rust,ignore
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

pub mod error;
#[allow(unused)]
pub mod parsers;
pub mod std_impls;

pub use nom;
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
