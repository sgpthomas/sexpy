use nom::{
    bytes::complete::tag,
    character::complete::{char, multispace0},
    combinator::cut,
    error::{context, VerboseError},
    sequence::{delimited, preceded},
    IResult,
};

pub fn s_exp<'a, O1, F>(
    inner: F,
) -> impl Fn(&'a str) -> IResult<&'a str, O1, VerboseError<&'a str>>
where
    F: Fn(&'a str) -> IResult<&'a str, O1, VerboseError<&'a str>>,
{
    delimited(
        char('('),
        preceded(multispace0, inner),
        context("closing paren", cut(preceded(multispace0, char(')')))),
    )
}

pub fn head<'a, O1, F>(
    head_tag: &'a str,
    inner: F,
) -> impl Fn(&'a str) -> IResult<&'a str, O1, VerboseError<&'a str>>
where
    F: Fn(&'a str) -> IResult<&'a str, O1, VerboseError<&'a str>>,
{
    s_exp(preceded(context("head", tag(head_tag)), cut(inner)))
}
