use crate::*;
use std::rc::Rc;

/// Parses a 'word', which is anything that starts with an upper or lowercase ASCII
/// character (a-z, A-Z) and ends in a space or one of the following characters: `()[]{}\;`
impl Sexpy for String {
    fn sexp_parse(input: &str) -> IResult<&str, Self, SexpyError<&str>>
    where
        Self: Sized,
    {
        let chars = " ()[]{}\\;";
        let (next, (s, s1)) = tuple((alpha1, many0(none_of(chars))))(input)?;
        Ok((next, format!("{}{}", s, s1.into_iter().collect::<String>())))
    }
}

/// Parses unsigned 64 bit integers
impl Sexpy for u64 {
    fn sexp_parse(input: &str) -> IResult<&str, Self, SexpyError<&str>>
    where
        Self: Sized,
    {
        let (next, digits) = digit1(input)?;
        match digits.parse::<u64>() {
            Ok(num) => Ok((next, num)),
            Err(_) => Err(Err::Error(SexpyError::number(input))),
        }
    }
}

/// Parses unsigned 32 bit integers
impl Sexpy for u32 {
    fn sexp_parse(input: &str) -> IResult<&str, Self, SexpyError<&str>>
    where
        Self: Sized,
    {
        let (next, digits) = digit1(input)?;
        match digits.parse::<u32>() {
            Ok(num) => Ok((next, num)),
            Err(_) => Err(Err::Error(SexpyError::number(input))),
        }
    }
}

/// Parses signed 64 bit integers
impl Sexpy for i64 {
    fn sexp_parse(input: &str) -> IResult<&str, Self, SexpyError<&str>>
    where
        Self: Sized,
    {
        let (next, (neg, digits)) = tuple((opt(char('-')), digit1))(input)?;
        match digits.parse::<i64>() {
            Ok(num) => {
                if neg.is_some() {
                    Ok((next, -num))
                } else {
                    Ok((next, num))
                }
            }
            Err(_) => Err(Err::Error(SexpyError::number(input))),
        }
    }
}

/// Parses signed 32 bit integers
impl Sexpy for i32 {
    fn sexp_parse(input: &str) -> IResult<&str, Self, SexpyError<&str>>
    where
        Self: Sized,
    {
        let (next, (neg, digits)) = tuple((opt(char('-')), digit1))(input)?;
        match digits.parse::<i32>() {
            Ok(num) => {
                if neg.is_some() {
                    Ok((next, -num))
                } else {
                    Ok((next, num))
                }
            }
            Err(_) => Err(Err::Error(SexpyError::number(input))),
        }
    }
}

/// Optionally parses `T`
impl<T: Sexpy> Sexpy for Option<T> {
    fn sexp_parse(input: &str) -> IResult<&str, Self, SexpyError<&str>>
    where
        Self: Sized,
    {
        let (next, res) = opt(T::sexp_parse)(input)?;
        Ok((next, res))
    }
}

/// Parses 0 or more instances of `T` seperated by whitespace
impl<T: Sexpy> Sexpy for Vec<T> {
    fn sexp_parse(input: &str) -> IResult<&str, Self, SexpyError<&str>>
    where
        Self: Sized,
    {
        let (next, res) = many0(preceded(wordbreak0, T::sexp_parse))(input)?;
        Ok((next, res))
    }
}

/// Just parses `T` but puts the result in a `Box<T>`
impl<T: Sexpy> Sexpy for Box<T> {
    fn sexp_parse(input: &str) -> IResult<&str, Self, SexpyError<&str>>
    where
        Self: Sized,
    {
        let (next, res) = T::sexp_parse(input)?;
        Ok((next, Box::new(res)))
    }
}

/// Just parses `T` but puts the result in an `Rc<T>`
impl<T: Sexpy> Sexpy for Rc<T> {
    fn sexp_parse(input: &str) -> IResult<&str, Self, SexpyError<&str>>
    where
        Self: Sized,
    {
        let (next, res) = T::sexp_parse(input)?;
        Ok((next, Rc::new(res)))
    }
}
