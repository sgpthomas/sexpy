use crate::error::{context, SexpyError};
use nom::{
    branch::alt,
    bytes::complete::take_till,
    character::complete::{anychar, char, none_of, one_of},
    combinator::{cut, map, peek},
    error::ParseError,
    multi::{many0, many1},
    sequence::{delimited, preceded},
    Err::Error,
    IResult,
};

/// The `inner` parser that ignores the result and returns unit instead
pub fn ignore<'a, F, O1>(
    inner: F,
) -> impl Fn(&'a str) -> IResult<&'a str, (), SexpyError<&'a str>>
where
    F: Fn(&'a str) -> IResult<&'a str, O1, SexpyError<&'a str>>,
{
    map(inner, |_| ())
}

/// Parses an s-expression comment; something that starts with `;` and ends with `\n`
pub fn comment<'a>(
    input: &'a str,
) -> IResult<&'a str, (), SexpyError<&'a str>> {
    ignore(preceded(char(';'), many0(none_of("\n"))))(input)
}

/// Matches a zero or more whitespace characters or comments
pub fn wordbreak0<'a>(
    input: &'a str,
) -> IResult<&'a str, (), SexpyError<&'a str>> {
    ignore(many0(alt((ignore(one_of(" \t\r\n")), comment))))(input)
}

/// Matches a one or more whitespace characters or comments
pub fn wordbreak1<'a>(
    input: &'a str,
) -> IResult<&'a str, (), SexpyError<&'a str>> {
    ignore(many1(alt((ignore(one_of(" \t\r\n")), comment))))(input)
}

/// Create a parser that surrounds whatever `inner` parses
/// with brackets or parentheses
pub fn surround<'a, O1, F>(
    inner: F,
    input: &'a str,
) -> IResult<&'a str, O1, SexpyError<&'a str>>
where
    F: Fn(&'a str) -> IResult<&'a str, O1, SexpyError<&'a str>>,
{
    // look the first char without consuming it
    let res: IResult<&'a str, char, SexpyError<&'a str>> = peek(anychar)(input);

    if let Ok((_, '(')) = res {
        // if its open paren, parse with parens
        delimited(
            char('('),
            preceded(wordbreak0, cut(inner)),
            context("closing paren", preceded(wordbreak0, char(')'))),
        )(input)
    } else if let Ok((_, '[')) = res {
        // if its open bracket, parse with brackets
        delimited(
            char('['),
            preceded(wordbreak0, cut(inner)),
            context("closing bracket", preceded(wordbreak0, char(']'))),
        )(input)
    } else {
        IResult::Err(Error(SexpyError::from_char(input, '(')))
    }
}

/// Takes in a `word` and returns `()` if the first word matches, otherwise
/// returns an Error
pub fn word<'a>(
    word: &'a str,
) -> impl Fn(&'a str) -> IResult<&'a str, (), SexpyError<&'a str>> {
    move |i: &'a str| {
        let chars = " ()[]{}\n\t\r;";
        let (rest, string) =
            // take characters until word boundary
            context("matching word", take_till(|c| chars.contains(c)))(i)?;
        if string == word {
            Ok((rest, ()))
        } else {
            IResult::Err(Error(SexpyError::from_word(i, string.to_string())))
        }
    }
}

/// Parses a `head` pattern. Takes a string `head_tag` and a parser, `inner`
/// and creates a parser for [`head tag` `inner`]
pub fn head<'a, O1, F>(
    head_tag: &'a str,
    inner: F,
) -> impl Fn(&'a str) -> IResult<&'a str, O1, SexpyError<&'a str>>
where
    F: Fn(&'a str) -> IResult<&'a str, O1, SexpyError<&'a str>>,
{
    preceded(context("incorrect head", word(head_tag)), cut(inner))
}
