use lexpr;
use lexpr::Value;
use std::fs;

// pub type TerminalParser<T> = Box<dyn Fn(Value) -> Result<T, ()>>;

pub struct InitialParser {
    f: Box<dyn Fn(Value) -> Result<((), Value), Value>>,
}

pub struct Parser<T> {
    f: Box<dyn Fn(Value) -> Result<(T, Value), Value>>,
}

pub struct TerminalParser<A, B> {
    f: Box<dyn Fn(Value) -> Result<A, Value>>,
    action: fn(A) -> B,
}

impl InitialParser {
    pub fn then<A: 'static>(self, parser: Parser<A>) -> Parser<A> {
        let cl = move |v| {
            let (_, v1) = (self.f)(v)?;
            let (b_res, v2) = (parser.f)(v1)?;
            Ok((b_res, v2))
        };
        Parser { f: Box::new(cl) }
    }
}

impl<A: 'static> Parser<A> {
    pub fn then<B: 'static>(self, parser: Parser<B>) -> Parser<(A, B)> {
        let cl = move |v| {
            let (a_res, v1) = (self.f)(v)?;
            let (b_res, v2) = (parser.f)(v1)?;
            Ok(((a_res, b_res), v2))
        };
        Parser { f: Box::new(cl) }
    }

    pub fn list(self) -> Parser<Vec<A>> {
        let cl = move |v| match v {
            Value::Cons(c) => {
                let (head, _cdr) = c.to_vec();
                let mut vals: Vec<A> = vec![];
                let result =
                    head.into_iter().fold(None, |err, item| match err {
                        Some(e) => Some(e),
                        None => match (self.f)(item) {
                            Ok((val, _)) => {
                                vals.push(val);
                                None
                            }
                            Err(e) => Some(Err(e)),
                        },
                    });
                match result {
                    Some(e) => e,
                    None => Ok((vals, Value::Null)),
                }
            }
            _ => Err(v),
        };
        Parser { f: Box::new(cl) }
    }

    pub fn or(self, parser: Parser<A>) -> Parser<A> {
        let cl = move |v: Value| match (self.f)(v.clone()) {
            Ok(x) => Ok(x),
            Err(_) => (parser.f)(v),
        };
        Parser { f: Box::new(cl) }
    }

    pub fn close<B>(self, f: fn(A) -> B) -> TerminalParser<A, B> {
        TerminalParser {
            f: self.f,
            action: f,
        }
    }

    pub fn call(self, v: Value) -> A {
        let (res, _) = (self.f)(v).expect("Parsing failed!");
        res
    }
}

// ====================
// Root parsers
// ====================

pub fn match_head(s: &'static str) -> InitialParser {
    let cl = move |v| match v {
        Value::Cons(c) => {
            let (head, rest) = c.into_pair();
            if head == Value::symbol(s) {
                Ok(((), rest))
            } else {
                Err(head)
            }
        }
        _ => Err(v),
    };
    InitialParser { f: Box::new(cl) }
}

pub fn match_var() -> Parser<String> {
    Parser {
        f: Box::new(|v| match v {
            Value::Cons(c) => {
                let (head, rest) = c.into_pair();
                match head {
                    Value::Symbol(s) => Ok((s.to_string(), rest)),
                    _ => Err(head),
                }
            }
            _ => Err(v),
        }),
    }
}

pub fn match_i64() -> Parser<i64> {
    Parser {
        f: Box::new(|v| match v {
            Value::Cons(c) => {
                let (head, rest) = c.into_pair();
                match head {
                    Value::Number(n) => Ok((n.as_i64().unwrap(), rest)),
                    _ => Err(head),
                }
            }
            _ => Err(v),
        }),
    }
}

pub fn from_file(filename: &str) -> lexpr::Value {
    let content = &fs::read_to_string(filename).expect("Unable to read file");
    lexpr::from_str(content).unwrap()
}
