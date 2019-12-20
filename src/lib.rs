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
    error::{convert_error, VerboseError},
    multi::many0,
    sequence::{preceded, tuple},
    Err, IResult,
};

pub trait Sexpy {
    fn process(input: &str) -> Result<Self, String>
    where
        Self: Sized,
    {
        match Self::parser(input) {
            Ok((_, x)) => Ok(x),
            Err(Err::Error(e)) => Err(convert_error(input, e)),
            Err(Err::Failure(e)) => Err(convert_error(input, e)),
            Err(Err::Incomplete(_)) => Err("Need more bytes to nom".to_string()),
        }
    }

    fn parser<'a>(
        input: &'a str,
    ) -> IResult<&'a str, Self, VerboseError<&'a str>>
    where
        Self: Sized;
}

#[cfg(test)]
mod tests {
    use crate::*;
    use sexpy_derive::Sexpy;

    #[derive(Debug, Sexpy)]
    enum Input {
        Port(String, u64),
        Snort(String, String),
    }

    #[derive(Debug, Sexpy)]
    enum Output {
        Hi(Input, Option<u64>),
        Bye(String),
    }

    #[derive(Debug, Sexpy)]
    struct IO {
        ins: Vec<Input>,
        outs: Vec<Output>,
    }

    #[derive(Debug, Sexpy)]
    enum Expr {
        Add(Box<Expr>, Box<Expr>),
        Sub(Box<Expr>, Box<Expr>),
        Num(u64),
    }

    #[test]
    fn it_works() {
        let input = "(add (num 4) (sub (num 10) (num 4)))";
        match Expr::process(input) {
            Ok(x) => println!("{:?}", x),
            Err(s) => println!("{}", s),
        }
    }
}
