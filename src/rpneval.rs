use crate::parser::RPNExpr;
use crate::tokenizer::MathToken;
use std::collections::HashMap;

// a shorthand for checking number of arguments before eval_fn
macro_rules! nargs {
    ($argcheck:expr, $ifok:expr) => {
        if $argcheck {
            $ifok
        } else {
            Err("Wrong number of arguments".to_string())
        }
    };
}

#[derive(Debug, Clone)]
pub struct MathContext(pub HashMap<String, f64>);

impl MathContext {
    pub fn new() -> MathContext {
        use std::f64::consts;
        let mut cx = HashMap::new();
        cx.insert("pi".to_string(), consts::PI);
        cx.insert("e".to_string(), consts::E);
        MathContext(cx)
    }

    pub fn setvar(&mut self, var: &str, val: f64) {
        self.0.insert(var.to_string(), val);
    }

    pub fn eval(&self, rpn: &RPNExpr) -> Result<f64, String> {
        let mut operands = Vec::new();

        for token in rpn.0.iter() {
            match *token {
                MathToken::Number(num) => operands.push(num),
                MathToken::Variable(ref var) => match self.0.get(var) {
                    Some(value) => operands.push(*value),
                    None => return Err(format!("Unknown Variable: {}", var.to_string())),
                },
                MathToken::BOp(ref op) => {
                    let r = operands
                        .pop()
                        .ok_or_else(|| "Wrong number of arguments".to_string())?;
                    let l = operands
                        .pop()
                        .ok_or_else(|| "Wrong number of arguments".to_string())?;
                    match &op[..] {
                        "+" => operands.push(l + r),
                        "-" => operands.push(l - r),
                        "*" => operands.push(l * r),
                        "/" => operands.push(l / r),
                        "%" => operands.push(l % r),
                        "^" => operands.push(l.powf(r)),
                        _ => return Err(format!("Bad Token: {}", op.clone())),
                    }
                }
                MathToken::UOp(ref op) => {
                    let o = operands
                        .pop()
                        .ok_or_else(|| "Wrong number of arguments".to_string())?;
                    match &op[..] {
                        "-" => operands.push(-o),
                        "!" => operands.push(Self::eval_fn("tgamma", vec![o + 1.0])?),
                        _ => return Err(format!("Bad Token: {}", op.clone())),
                    }
                }
                MathToken::Function(ref fname, arity) => {
                    if arity > operands.len() {
                        return Err("Wrong number of arguments".to_string());
                    }
                    let cut = operands.len() - arity;
                    let args = operands.split_off(cut);
                    operands.push(Self::eval_fn(fname, args)?)
                }
                _ => return Err(format!("Bad Token: {:?}", *token)),
            }
        }
        operands
            .pop()
            .ok_or_else(|| "Wrong number of arguments".to_string())
    }

    fn eval_fn(fname: &str, args: Vec<f64>) -> Result<f64, String> {
        match fname {
            "sin" => nargs!(args.len() == 1, Ok(args[0].sin())),
            "cos" => nargs!(args.len() == 1, Ok(args[0].cos())),
            "atan2" => nargs!(args.len() == 2, Ok(args[0].atan2(args[1]))),
            "max" => nargs!(
                !args.is_empty(),
                Ok(args[1..].iter().fold(args[0], |a, &item| a.max(item)))
            ),
            "min" => nargs!(
                !args.is_empty(),
                Ok(args[1..].iter().fold(args[0], |a, &item| a.min(item)))
            ),
            "abs" => nargs!(args.len() == 1, Ok(f64::abs(args[0]))),
            "rand" => nargs!(args.len() == 1, Ok(args[0] * rand::random::<f64>())),
            // Order is important
            "nMPr" => nargs!(args.len() == 2, Ok(args[0].powf(args[1]))),
            // Unknown function
            _ => Err(format!("Unknown function: {}", fname)),
        }
    }
}

impl Default for MathContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::MathContext;
    use crate::parser::ShuntingParser;

    macro_rules! fuzzy_eq {
        ($lhs:expr, $rhs:expr) => {
            assert!(($lhs - $rhs).abs() < 1.0e-10)
        };
    }

    #[test]
    fn test_eval1() {
        let expr = ShuntingParser::parse_str("3+4*2/-(1-5)^2^3").unwrap();
        fuzzy_eq!(MathContext::new().eval(&expr).unwrap(), 2.99987792969);
    }

    #[test]
    fn test_eval2() {
        let expr = ShuntingParser::parse_str("3.4e-2 * sin(pi/3)/(541 % -4) * max(2, -7)").unwrap();
        fuzzy_eq!(MathContext::new().eval(&expr).unwrap(), 0.058889727457341);
    }

    #[test]
    fn test_eval3() {
        let expr = ShuntingParser::parse_str("(-(1-9^2) / (1 + 6^2))^0.5").unwrap();
        fuzzy_eq!(
            MathContext::new().eval(&expr).unwrap(),
            1.470429244187615496759
        );
    }

    #[test]
    fn test_eval4() {
        let expr = ShuntingParser::parse_str("sin(0.345)^2 + cos(0.345)^2").unwrap();
        fuzzy_eq!(MathContext::new().eval(&expr).unwrap(), 1.0);
    }

    #[test]
    fn test_eval5() {
        let expr = ShuntingParser::parse_str("sin(e)/cos(e)").unwrap();
        fuzzy_eq!(MathContext::new().eval(&expr).unwrap(), -0.4505495340698074);
    }

    #[test]
    fn test_eval6() {
        let expr = ShuntingParser::parse_str("(3+4)*3").unwrap();
        fuzzy_eq!(MathContext::new().eval(&expr).unwrap(), 21.0);
    }

    #[test]
    fn test_eval7() {
        let expr = ShuntingParser::parse_str("(3+4)*3").unwrap();
        fuzzy_eq!(MathContext::new().eval(&expr).unwrap(), 21.0);
    }

    #[test]
    fn test_eval8() {
        let expr = ShuntingParser::parse_str("2^3").unwrap();
        fuzzy_eq!(MathContext::new().eval(&expr).unwrap(), 8.0);
        let expr = ShuntingParser::parse_str("2^-3").unwrap();
        fuzzy_eq!(MathContext::new().eval(&expr).unwrap(), 0.125);
        let expr = ShuntingParser::parse_str("-2^3").unwrap();
        fuzzy_eq!(MathContext::new().eval(&expr).unwrap(), -8.0);
        let expr = ShuntingParser::parse_str("-2^-3").unwrap();
        fuzzy_eq!(MathContext::new().eval(&expr).unwrap(), -0.125);
    }
}
