mod parsers;
mod std_impls;
pub use parsers::*;
pub use sexpy_derive::Sexpy;

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
    use std::fs;

    #[derive(Debug, Sexpy)]
    #[sexpy(name = "define/component")]
    struct Component {
        name: String,
        inputs: Vec<Portdef>,
        outputs: Vec<Portdef>,
        structure: Vec<Structure>,
    }

    #[derive(Debug, Sexpy)]
    #[sexpy(name = "port")]
    struct Portdef {
        name: String,
        width: u64,
    }

    #[derive(Debug, Sexpy)]
    enum Port {
        Comp(String, String),
        This(String),
    }

    #[derive(Debug, Sexpy)]
    enum Structure {
        #[sexpy(name = "new")]
        Decl(String, String),
        #[sexpy(name = "->")]
        Wire(Port, Port),
    }

    #[test]
    fn it_works() {
        let contents = fs::read_to_string("test.futil").unwrap();
        match Component::parse(&contents) {
            Ok(x) => println!("{:#?}", x),
            Err(s) => println!("{}", s),
        }
    }
}
