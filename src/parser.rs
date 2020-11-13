use crate::tokenizer::{MathToken, MathTokenizer};
use std::cmp::Ordering;

#[derive(PartialEq, Debug)]
pub enum Assoc {
    Left,
    Right,
    None,
}

pub fn precedence(mt: &MathToken) -> (usize, Assoc) {
    // You can play with the relation between exponentiation an unary - by
    // a. switching order in which the lexer tokenizes, if it tries
    // operators first then '-' will never be the negative part of number,
    // else if numbers are tried before operators, - can only be unary
    // for non-numeric tokens (eg: -(3)).
    // b. changing the precedence of '-' respect to '^'
    // If '-' has lower precedence then 2^-3 will fail to evaluate if the
    // '-' isn't part of the number because ^ will only find 1 operator
    match *mt {
        MathToken::OParen => (1, Assoc::Left), // keep at bottom
        MathToken::BOp(ref o) if o == "+" => (2, Assoc::Left),
        MathToken::BOp(ref o) if o == "-" => (2, Assoc::Left),
        MathToken::BOp(ref o) if o == "*" => (3, Assoc::Left),
        MathToken::BOp(ref o) if o == "/" => (3, Assoc::Left),
        MathToken::BOp(ref o) if o == "%" => (3, Assoc::Left),
        MathToken::UOp(ref o) if o == "-" => (5, Assoc::Right), // unary minus
        MathToken::BOp(ref o) if o == "^" => (5, Assoc::Right),
        MathToken::UOp(ref o) if o == "!" => (6, Assoc::Left), // factorial
        MathToken::Function(_, _) => (7, Assoc::Left),
        _ => (99, Assoc::None),
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct RPNExpr(pub Vec<MathToken>);

pub struct ShuntingParser;

impl ShuntingParser {
    pub fn parse_str(expr: &str) -> Result<RPNExpr, String> {
        Self::parse(&mut MathTokenizer::new(expr.chars()))
    }

    pub fn parse(lex: &mut impl Iterator<Item = MathToken>) -> Result<RPNExpr, String> {
        let mut out = Vec::new();
        let mut stack = Vec::new();
        let mut arity = Vec::<usize>::new();

        for token in lex {
            match token {
                MathToken::Number(_) => out.push(token),
                MathToken::Variable(_) => out.push(token),
                MathToken::OParen => stack.push(token),
                MathToken::Function(_, _) => {
                    stack.push(token);
                    arity.push(1);
                }
                MathToken::Comma | MathToken::CParen => {
                    while !stack.is_empty() && stack.last() != Some(&MathToken::OParen) {
                        out.push(stack.pop().unwrap());
                    }
                    if stack.is_empty() {
                        return Err("Missing Opening Paren".to_string());
                    }
                    // end of grouping: check if this is a function call
                    if token == MathToken::CParen {
                        stack.pop(); // peel matching OParen
                        match stack.pop() {
                            Some(MathToken::Function(func, _)) => {
                                out.push(MathToken::Function(func, arity.pop().unwrap()))
                            }
                            Some(other) => stack.push(other),
                            None => (),
                        }
                    } else if let Some(a) = arity.last_mut() {
                        *a += 1;
                    } // Comma
                }
                MathToken::UOp(_) | MathToken::BOp(_) => {
                    let (prec_rhs, assoc_rhs) = precedence(&token);
                    while !stack.is_empty() {
                        let (prec_lhs, _) = precedence(stack.last().unwrap());
                        match prec_lhs.cmp(&prec_rhs) {
                            Ordering::Greater => out.push(stack.pop().unwrap()),
                            Ordering::Less => break,
                            Ordering::Equal => match assoc_rhs {
                                Assoc::Left => out.push(stack.pop().unwrap()),
                                Assoc::None => return Err("No Associativity".to_string()),
                                Assoc::Right => break,
                            },
                        }
                    }
                    stack.push(token);
                }
                MathToken::Unknown(lexeme) => return Err(format!("Bad token: {}", lexeme)),
            }
        }
        while let Some(top) = stack.pop() {
            match top {
                MathToken::OParen => return Err("Missing Closing Paren".to_string()),
                token => out.push(token),
            }
        }
        Ok(RPNExpr(out))
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::{RPNExpr, ShuntingParser};
    use crate::tokenizer::MathToken;

    #[test]
    fn test_parse1() {
        let rpn = ShuntingParser::parse_str("3+4*2/-(1-5)^2^3").unwrap();
        let expect = vec![
            MathToken::Number(3.0),
            MathToken::Number(4.0),
            MathToken::Number(2.0),
            MathToken::BOp(format!("*")),
            MathToken::Number(1.0),
            MathToken::Number(5.0),
            MathToken::BOp(format!("-")),
            MathToken::Number(2.0),
            MathToken::Number(3.0),
            MathToken::BOp(format!("^")),
            MathToken::BOp(format!("^")),
            MathToken::UOp(format!("-")),
            MathToken::BOp(format!("/")),
            MathToken::BOp(format!("+")),
        ];
        assert_eq!(rpn, RPNExpr(expect));
    }
    #[test]
    fn test_parse2() {
        let rpn = ShuntingParser::parse_str("3.4e-2 * sin(x)/(7! % -4) * max(2, x)").unwrap();
        let expect = vec![
            MathToken::Number(3.4e-2),
            MathToken::Variable(format!("x")),
            MathToken::Function(format!("sin"), 1),
            MathToken::BOp(format!("*")),
            MathToken::Number(7.0),
            MathToken::UOp(format!("!")),
            MathToken::Number(4.0),
            MathToken::UOp(format!("-")),
            MathToken::BOp(format!("%")),
            MathToken::BOp(format!("/")),
            MathToken::Number(2.0),
            MathToken::Variable(format!("x")),
            MathToken::Function(format!("max"), 2),
            MathToken::BOp(format!("*")),
        ];
        assert_eq!(rpn, RPNExpr(expect));
    }

    #[test]
    fn test_parse3() {
        let rpn = ShuntingParser::parse_str("sqrt(-(1-x^2) / (1 + x^2))").unwrap();
        let expect = vec![
            MathToken::Number(1.0),
            MathToken::Variable(format!("x")),
            MathToken::Number(2.0),
            MathToken::BOp(format!("^")),
            MathToken::BOp(format!("-")),
            MathToken::UOp(format!("-")),
            MathToken::Number(1.0),
            MathToken::Variable(format!("x")),
            MathToken::Number(2.0),
            MathToken::BOp(format!("^")),
            MathToken::BOp(format!("+")),
            MathToken::BOp(format!("/")),
            MathToken::Function(format!("sqrt"), 1),
        ];
        assert_eq!(rpn, RPNExpr(expect));
    }

    #[test]
    fn bad_parse() {
        let rpn = ShuntingParser::parse_str("sqrt(-(1-x^2) / (1 + x^2)");
        assert_eq!(rpn, Err(format!("Missing Closing Paren")));

        let rpn = ShuntingParser::parse_str("-(1-x^2) / (1 + x^2))");
        assert_eq!(rpn, Err(format!("Missing Opening Paren")));

        let rpn = ShuntingParser::parse_str("max 4, 6, 4)");
        assert_eq!(rpn, Err(format!("Missing Opening Paren")));
    }

    #[test]
    fn check_arity() {
        use std::collections::HashMap;
        let rpn = ShuntingParser::parse_str("sin(1)+(max(2, gamma(3.5), gcd(24, 8))+sum(i,0,10))")
            .unwrap();
        let mut expect = HashMap::new();
        expect.insert("sin", 1);
        expect.insert("max", 3);
        expect.insert("gamma", 1);
        expect.insert("gcd", 2);
        expect.insert("sum", 3);

        for token in rpn.0.iter() {
            match *token {
                MathToken::Function(ref func, arity) => {
                    let expected_arity = expect.get(&func[..]);
                    assert_eq!(*expected_arity.unwrap(), arity);
                }
                _ => (),
            }
        }
    }
}
