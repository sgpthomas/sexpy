use crate::*;
use std::rc::Rc;

impl Sexpy for String {
    fn sexp_parse(input: &str) -> IResult<&str, Self, SexpyError<&str>>
    where
        Self: Sized,
    {
        let chars = " ()[]{}\"\'\\;";
        let (next, (s, s1)) = tuple((alpha1, many0(none_of(chars))))(input)?;
        Ok((next, format!("{}{}", s, s1.into_iter().collect::<String>())))
    }
}

impl Sexpy for u64 {
    fn sexp_parse(input: &str) -> IResult<&str, Self, SexpyError<&str>>
    where
        Self: Sized,
    {
        let (next, digits) = digit1(input)?;
        let num = digits.parse::<u64>().unwrap(); // XXX(sam) fix
        Ok((next, num))
    }
}

impl Sexpy for u32 {
    fn sexp_parse(input: &str) -> IResult<&str, Self, SexpyError<&str>>
    where
        Self: Sized,
    {
        let (next, digits) = digit1(input)?;
        let num = digits.parse::<u32>().unwrap(); // XXX(sam) fix
        Ok((next, num))
    }
}

impl Sexpy for i64 {
    fn sexp_parse(input: &str) -> IResult<&str, Self, SexpyError<&str>>
    where
        Self: Sized,
    {
        let (next, (neg, digits)) = tuple((opt(char('-')), digit1))(input)?;
        let num = digits.parse::<i64>().unwrap(); // XXX(sam) fix
        if neg.is_some() {
            Ok((next, -num))
        } else {
            Ok((next, num))
        }
    }
}

impl Sexpy for i32 {
    fn sexp_parse(input: &str) -> IResult<&str, Self, SexpyError<&str>>
    where
        Self: Sized,
    {
        let (next, (neg, digits)) = tuple((opt(char('-')), digit1))(input)?;
        let num = digits.parse::<i32>().unwrap(); // XXX(sam) fix
        if neg.is_some() {
            Ok((next, -num))
        } else {
            Ok((next, num))
        }
    }
}

impl<T: Sexpy> Sexpy for Option<T> {
    fn sexp_parse(input: &str) -> IResult<&str, Self, SexpyError<&str>>
    where
        Self: Sized,
    {
        let (next, res) = opt(T::sexp_parse)(input)?;
        Ok((next, res))
    }
}

impl<T: Sexpy> Sexpy for Vec<T> {
    fn sexp_parse(input: &str) -> IResult<&str, Self, SexpyError<&str>>
    where
        Self: Sized,
    {
        let (next, res) = many0(preceded(wordbreak0, T::sexp_parse))(input)?;
        Ok((next, res))
    }
}

impl<T: Sexpy> Sexpy for Box<T> {
    fn sexp_parse(input: &str) -> IResult<&str, Self, SexpyError<&str>>
    where
        Self: Sized,
    {
        let (next, res) = T::sexp_parse(input)?;
        Ok((next, Box::new(res)))
    }
}

impl<T: Sexpy> Sexpy for Rc<T> {
    fn sexp_parse(input: &str) -> IResult<&str, Self, SexpyError<&str>>
    where
        Self: Sized,
    {
        let (next, res) = T::sexp_parse(input)?;
        Ok((next, Rc::new(res)))
    }
}
