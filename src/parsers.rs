use nom::{
    branch::alt,
    bytes::complete::take_till,
    character::complete::{anychar, char, none_of, one_of},
    combinator::{cut, map, peek},
    error::{context, ParseError, VerboseError},
    multi::{many0, many1},
    sequence::{delimited, preceded},
    Err::Error,
    IResult,
};

pub fn ignore<'a, F, O1>(
    inner: F,
) -> impl Fn(&'a str) -> IResult<&'a str, (), VerboseError<&'a str>>
where
    F: Fn(&'a str) -> IResult<&'a str, O1, VerboseError<&'a str>>,
{
    map(inner, |_| ())
}

pub fn comment<'a>(
    input: &'a str,
) -> IResult<&'a str, (), VerboseError<&'a str>> {
    ignore(preceded(char(';'), many0(none_of("\n"))))(input)
}

pub fn wordbreak0<'a>(
    input: &'a str,
) -> IResult<&'a str, (), VerboseError<&'a str>> {
    ignore(many0(alt((ignore(one_of(" \t\r\n")), comment))))(input)
}

pub fn wordbreak1<'a>(
    input: &'a str,
) -> IResult<&'a str, (), VerboseError<&'a str>> {
    ignore(many1(alt((ignore(one_of(" \t\r\n")), comment))))(input)
}

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
            preceded(wordbreak0, inner),
            context("closing paren", cut(preceded(wordbreak0, char(')')))),
        )(input)
    } else if let Ok((_, '[')) = res {
        // if its open bracket, parse with brackets
        delimited(
            char('['),
            preceded(wordbreak0, inner),
            context("closing bracket", cut(preceded(wordbreak0, char(']')))),
        )(input)
    } else {
        IResult::Err(Error(VerboseError::from_char(input, '(')))
    }
}

/// Takes in a `word` and returns `()` if the first word matches, otherwise
/// returns an Error
pub fn word<'a>(
    word: &'a str,
) -> impl Fn(&'a str) -> IResult<&'a str, (), VerboseError<&'a str>> {
    move |i: &'a str| {
        let chars = " ()[]{}\n\t\r;";
        let (rest, st) =
            // take characters until word boundary
            context("matching word", take_till(|c| chars.contains(c)))(i)?;
        if st == word {
            Ok((rest, ()))
        } else {
            IResult::Err(Error(VerboseError::from_char(
                i,
                word.as_bytes()[0].into(),
            )))
        }
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
    preceded(context("incorrect head", word(head_tag)), cut(inner))
}
