use nom::{
    error::{ErrorKind, ParseError},
    Err, IResult,
};
use std::iter::repeat;

#[derive(Debug)]
pub struct SexpyError<Input> {
    pub errors: Vec<(Input, SexpyErrorKind)>,
}

#[derive(Clone, Debug, PartialEq)]
/// error context for `VerboseError`
pub enum SexpyErrorKind {
    /// static string added by the `context` function
    Context(&'static str),
    /// indicates which character was expected by the `char` function
    Char(char),
    /// indicates which word was expected by the `word` function
    Word(String),
    /// indicates an error occurred while parsing a number
    Number,
    /// error kind given by various nom parsers
    Nom(ErrorKind),
}

impl<Input> ParseError<Input> for SexpyError<Input> {
    fn from_error_kind(input: Input, kind: ErrorKind) -> Self {
        SexpyError {
            errors: vec![(input, SexpyErrorKind::Nom(kind))],
        }
    }

    fn append(input: Input, kind: ErrorKind, mut other: Self) -> Self {
        other.errors.push((input, SexpyErrorKind::Nom(kind)));
        other
    }

    fn from_char(input: Input, c: char) -> Self {
        SexpyError {
            errors: vec![(input, SexpyErrorKind::Char(c))],
        }
    }

    fn add_context(input: Input, ctx: &'static str, mut other: Self) -> Self {
        other.errors.push((input, SexpyErrorKind::Context(ctx)));
        other
    }
}

impl<Input> SexpyError<Input> {
    /// Make a `SexpyError` from an Input and a String
    pub fn from_word(input: Input, w: String) -> Self {
        SexpyError {
            errors: vec![(input, SexpyErrorKind::Word(w))],
        }
    }

    /// Make a `SexpyErrorKind::Number` from an Input
    pub fn number(input: Input) -> Self {
        SexpyError {
            errors: vec![(input, SexpyErrorKind::Number)],
        }
    }
}

impl SexpyError<&str> {
    /// Converts a `SexpyError` into a formatted string.
    /// Only shows the topmost parsing error. Use `convert_error_verbose`
    /// to get the whole error stack
    pub fn convert_error(&self, input: &str) -> String {
        if self.errors.is_empty() {
            panic!("No errors found")
        } else {
            format_error(input, 0, &self.errors[0])
        }
    }

    /// Converts a `SexpyError` into a formated string.
    /// Shows the entire error stack. Use `convert_error` to show
    /// only the topmost error
    #[allow(unused)]
    pub fn convert_error_verbose(&self, input: &str) -> String {
        let mut result = String::new();

        for (i, error) in self.errors.iter().enumerate() {
            result += &format_error(input, i, error);
        }

        result
    }
}

/// create a new error from an input position, a static string and an existing error.
/// This is used mainly in the [context] combinator, to add user friendly information
/// to errors when backtracking through a parse tree
pub fn context<I: Clone, E: ParseError<I>, F, O>(
    context: &'static str,
    f: F,
) -> impl Fn(I) -> IResult<I, O, E>
where
    F: Fn(I) -> IResult<I, O, E>,
{
    move |i: I| match f(i.clone()) {
        Ok(o) => Ok(o),
        Err(Err::Incomplete(i)) => Err(Err::Incomplete(i)),
        Err(Err::Error(e)) => Err(Err::Error(E::add_context(i, context, e))),
        Err(Err::Failure(e)) => {
            Err(Err::Failure(E::add_context(i, context, e)))
        }
    }
}

fn offset(first: &str, second: &str) -> usize {
    let fst = first.as_ptr();
    let snd = second.as_ptr();

    snd as usize - fst as usize
}

fn format_error(input: &str, num: usize, e: &(&str, SexpyErrorKind)) -> String {
    let lines: Vec<_> = input.lines().map(String::from).collect();
    let (substring, kind) = e;
    let mut offset = offset(input, substring);
    let mut result = String::new();

    if lines.is_empty() {
        match kind {
            SexpyErrorKind::Char(c) => {
                result +=
                    &format!("{}: expected '{}', got empty input\n\n", num, c);
            }
            SexpyErrorKind::Word(_w) => {
                result += &format!(
                    "{}: expected a keyword, got empty input\n\n",
                    num
                );
            }
            SexpyErrorKind::Number => {
                result +=
                    &format!("{}: expected a number, got empty input\n\n", num);
            }
            SexpyErrorKind::Context(s) => {
                result += &format!("{}: in {}, got empty input\n\n", num, s);
            }
            SexpyErrorKind::Nom(e) => {
                result += &format!("{}: in {:?}, got empty input\n\n", num, e);
            }
        }
    } else {
        let mut line = 0;
        let mut column = 0;

        for (j, l) in lines.iter().enumerate() {
            if offset <= l.len() {
                line = j;
                column = offset;
                break;
            } else {
                offset = offset - l.len() - 1;
            }
        }

        match kind {
            SexpyErrorKind::Char(c) => {
                result += &format!("{}: at line {}:\n", num, line);
                result += &lines[line];
                result += "\n";

                if column > 0 {
                    result += &repeat(' ').take(column).collect::<String>();
                }
                result += "^\n";
                let found = match substring.chars().next() {
                    Some(x) => format!("'{}'", x),
                    None => "<eof>".to_string(),
                };
                result += &format!("expected '{}', found {}\n\n", c, found);
            }
            SexpyErrorKind::Word(w) => {
                result += &format!("{}: at line {}:\n", num, line);
                result += &lines[line];
                result += "\n";

                if column > 0 {
                    result += &repeat(' ').take(column).collect::<String>();
                }
                result += "^\n";
                result += &format!("expected a keyword, found \"{}\"\n\n", w);
            }
            SexpyErrorKind::Number => {
                result += &format!("{}: at line {}:\n", num, line);
                result += &lines[line];
                result += "\n";

                if column > 0 {
                    result += &repeat(' ').take(column).collect::<String>();
                }
                result += "^\n";
                result += "unable to parse number\n\n";
            }
            SexpyErrorKind::Context(s) => {
                result += &format!("{}: at line {}, in {}:\n", num, line, s);
                result += &lines[line];
                result += "\n";
                if column > 0 {
                    result += &repeat(' ').take(column).collect::<String>();
                }
                result += "^\n\n";
            }
            SexpyErrorKind::Nom(e) => {
                result += &format!("{}: at line {}, in {:?}:\n", num, line, e);
                result += &lines[line];
                result += "\n";
                if column > 0 {
                    result += &repeat(' ').take(column).collect::<String>();
                }
                result += "^\n\n";
            }
        }
    }
    result
}
