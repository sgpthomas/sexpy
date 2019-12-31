use nom::{
    bytes::complete::tag,
    character::complete::{anychar, char, multispace0},
    combinator::{cut, peek},
    error::{context, ParseError, VerboseError},
    sequence::{delimited, preceded},
    Err::Error,
    IResult,
};

/// Create a parser that surrounds whatever `inner` parses
/// with brackets or parentheses
pub fn surround<'a, O1, F>(
    inner: F,
    input: &'a str,
) -> IResult<&'a str, O1, VerboseError<&'a str>>
where
    F: Fn(&'a str) -> IResult<&'a str, O1, VerboseError<&'a str>>,
{
    // look the first char without consuming it
    let res: IResult<&'a str, char, VerboseError<&'a str>> =
        peek(anychar)(input);

    if let Ok((_, '(')) = res {
        // if its open paren, parse with parens
        delimited(
            char('('),
            preceded(multispace0, inner),
            context("closing paren", cut(preceded(multispace0, char(')')))),
        )(input)
    } else if let Ok((_, '[')) = res {
        // if its open paren, parse with brackets
        delimited(
            char('['),
            preceded(multispace0, inner),
            context("closing bracket", cut(preceded(multispace0, char(']')))),
        )(input)
    } else {
        IResult::Err(Error(VerboseError::from_char(input, '(')))
    }
}

/// Parses a `head` pattern. Takes a string `head_tag` and a parser, `inner`
/// and creates a parser for [`head tag` `inner`]
pub fn head<'a, O1, F>(
    head_tag: &'a str,
    inner: F,
) -> impl Fn(&'a str) -> IResult<&'a str, O1, VerboseError<&'a str>>
where
    F: Fn(&'a str) -> IResult<&'a str, O1, VerboseError<&'a str>>,
{
    preceded(context("incorrect head", tag(head_tag)), cut(inner))
}
