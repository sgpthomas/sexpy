use crate::*;
use std::rc::Rc;

impl Sexpy for String {
    fn parser(input: &str) -> IResult<&str, Self, VerboseError<&str>>
    where
        Self: Sized,
    {
        let (next, (s, s1)) = tuple((alpha1, alphanumeric0))(input)?;
        Ok((next, format!("{}{}", s, s1)))
    }
}

impl Sexpy for u64 {
    fn parser(input: &str) -> IResult<&str, Self, VerboseError<&str>>
    where
        Self: Sized,
    {
        let (next, digits) = digit1(input)?;
        let num = digits.parse::<u64>().unwrap(); // XXX(sam) fix
        Ok((next, num))
    }
}

impl<T: Sexpy> Sexpy for Option<T> {
    fn parser(input: &str) -> IResult<&str, Self, VerboseError<&str>>
    where
        Self: Sized,
    {
        let (next, res) = opt(T::parser)(input)?;
        Ok((next, res))
    }
}

impl<T: Sexpy> Sexpy for Vec<T> {
    fn parser(input: &str) -> IResult<&str, Self, VerboseError<&str>>
    where
        Self: Sized,
    {
        let (next, res) = many0(T::parser)(input)?;
        Ok((next, res))
    }
}

impl<T: Sexpy> Sexpy for Box<T> {
    fn parser(input: &str) -> IResult<&str, Self, VerboseError<&str>>
    where
        Self: Sized,
    {
        let (next, res) = T::parser(input)?;
        Ok((next, Box::new(res)))
    }
}

impl<T: Sexpy> Sexpy for Rc<T> {
    fn parser(input: &str) -> IResult<&str, Self, VerboseError<&str>>
    where
        Self: Sized,
    {
        let (next, res) = T::parser(input)?;
        Ok((next, Rc::new(res)))
    }
}
