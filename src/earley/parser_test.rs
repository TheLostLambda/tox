use earley::types::{Symbol, Rule, Item, StateSet};
use earley::grammar::{GrammarBuilder, Grammar};
use earley::tree1::build_tree;
use earley::trees::build_trees;
use earley::{Lexer, EarleyParser, ParseError};
use std::rc::Rc;

#[test]
fn symbol_uniqueness() {
    // test equalty operators
    fn testfn(o: &str) -> bool { o.len() == 1 && "+-".contains(o) }
    assert_eq!(Symbol::nonterm("sym1"), Symbol::nonterm("sym1"));
    // comparing Fn trait (data, vtable) so shouldn't match
    assert!(Symbol::terminal("+-", testfn) != Symbol::terminal("+-", testfn));
    assert!(Symbol::terminal("X", |_| true) != Symbol::terminal("X", |_| true));

    let rule = {
        let s = Rc::new(Symbol::nonterm("S"));
        let add_op = Rc::new(Symbol::terminal("+-", testfn));
        let num = Rc::new(Symbol::terminal("[0-9]", |n: &str| {
                            n.len() == 1 && "1234567890".contains(n)}));
        Rc::new(Rule::new(s.clone(), vec![s, add_op, num]))
    };

    // test item comparison
    assert_eq!(Item::new(rule.clone(), 0, 0, 0), Item::new(rule.clone(), 0, 0, 0));
    assert!(Item::new(rule.clone(), 0, 0, 0) != Item::new(rule.clone(), 0, 1, 0));

    // check that items are deduped in statesets
    let mut ss = StateSet::new();
    ss.push(Item::new(rule.clone(), 0, 0, 0));
    ss.push(Item::new(rule.clone(), 0, 0, 0));
    assert_eq!(ss.len(), 1);
    ss.push(Item::new(rule.clone(), 1, 0, 1));
    assert_eq!(ss.len(), 2);

    let ix = Item::new(rule.clone(), 2, 3, 3);
    let vi = vec![ix.clone(), ix.clone(), ix.clone(), ix.clone()];
    ss.extend(vi.into_iter());
    assert_eq!(ss.len(), 3);
}

#[test]
fn symbol_nullable() {
    let mut gb = GrammarBuilder::new();
    gb.symbol(Symbol::nonterm("A"))
      .symbol(Symbol::nonterm("B"));
    gb.rule("A", Vec::new())
      .rule("A", vec!["B"])
      .rule("B", vec!["A"]);
    let g = gb.into_grammar("A");
    assert!(g.is_nullable("A"));
    assert!(g.is_nullable("B"));
}

// Sum -> Sum + Mul | Mul
// Mul -> Mul * Pow | Pow
// Pow -> Num ^ Pow | Num
// Num -> Number | ( Sum )

fn grammar_math() -> Grammar {
    let mut gb = GrammarBuilder::new();
    // register some symbols
    gb.symbol(Symbol::nonterm("Sum"))
      .symbol(Symbol::nonterm("Mul"))
      .symbol(Symbol::nonterm("Pow"))
      .symbol(Symbol::nonterm("Num"))
      .symbol(Symbol::terminal("Number", |n: &str| {
          n.chars().all(|c| "1234567890".contains(c))
        }))
      .symbol(Symbol::terminal("[+-]", |n: &str| {
          n.len() == 1 && "+-".contains(n)
        }))
      .symbol(Symbol::terminal("[*/]", |n: &str| {
          n.len() == 1 && "*/".contains(n)
        }))
      .symbol(Symbol::terminal("[^]", |n: &str| { n == "^" }))
      .symbol(Symbol::terminal("(", |n: &str| { n == "(" }))
      .symbol(Symbol::terminal(")", |n: &str| { n == ")" }));
    // add grammar rules
    gb.rule("Sum", vec!["Sum", "[+-]", "Mul"])
      .rule("Sum", vec!["Mul"])
      .rule("Mul", vec!["Mul", "[*/]", "Pow"])
      .rule("Mul", vec!["Pow"])
      .rule("Pow", vec!["Num", "[^]", "Pow"])
      .rule("Pow", vec!["Num"])
      .rule("Num", vec!["(", "Sum", ")"])
      .rule("Num", vec!["Number"]);

    let grammar = gb.into_grammar("Sum");
    assert_eq!(grammar.rules("Sum").count(), 2);
    assert_eq!(grammar.rules("Mul").count(), 2);
    assert_eq!(grammar.rules("Pow").count(), 2);
    assert_eq!(grammar.rules("Num").count(), 2);
    grammar
}

fn print_statesets(ss: &Vec<StateSet>) {
    for (idx, stateset) in ss.iter().enumerate() {
        println!("=== {} ===", idx);
        for item in stateset.iter() { println!("{:?}", item); }
    }
}

///////////////////////////////////////////////////////////////////////////////

#[test]
fn test_badparse() {
    let mut input = Lexer::from_str("1+", "+*");
    let out = EarleyParser::new(grammar_math()).parse(&mut input);
    assert_eq!(out.unwrap_err(), ParseError::BadInput);
}

#[test]
fn test_partialparse() {
    let mut gb = GrammarBuilder::new();
    gb.symbol(Symbol::nonterm("Start"))
      .symbol(Symbol::terminal("+", |n: &str| n == "+"))
      .rule("Start", vec!["+", "+"]);
    let mut input = Lexer::from_str("+++", "+");
    let out = EarleyParser::new(gb.into_grammar("Start")).parse(&mut input);
    assert_eq!(out.unwrap_err(), ParseError::PartialParse);
}

#[test]
fn grammar_ambiguous() {
    // S -> SS | b
    let mut gb = GrammarBuilder::new();
    gb.symbol(Symbol::nonterm("S"))
      .symbol(Symbol::terminal("b", |n: &str| n == "b"))
      .rule("S", vec!["S", "S"])
      .rule("S", vec!["b"]);
    // Earley's corner case that generates spurious trees for bbb
    let mut input = Lexer::from_str("b b b", " ");
    let p = EarleyParser::new(gb.into_grammar("S"));
    let ps = p.parse(&mut input).unwrap();
    assert_eq!(ps.states.len(), 4);
    print_statesets(&ps.states);
    println!("=== tree ===");
    for t in build_trees(&p.g, &ps) { println!("{:?}", t); }
}

#[test]
fn math_grammar_test() {
    let p = EarleyParser::new(grammar_math());
    let mut input = Lexer::from_str("1+(2*3-4)", "+*-/()");
    let ps = p.parse(&mut input).unwrap();
    assert_eq!(ps.states.len(), 10);
    print_statesets(&ps.states);
    println!("=== tree ===");
    println!("{:?}", build_tree(&p.g, &ps));
}

#[test]
fn test_left_recurse() {
    // S -> S + N | N
    // N -> [0-9]
    let mut gb = GrammarBuilder::new();
    gb.symbol(Symbol::nonterm("S"))
      .symbol(Symbol::nonterm("N"))
      .symbol(Symbol::terminal("[+]", |n: &str| n == "+"))
      .symbol(Symbol::terminal("[0-9]", |n: &str| "1234567890".contains(n)))
      .rule("S", vec!["S", "[+]", "N"])
      .rule("S", vec!["N"])
      .rule("N", vec!["[0-9]"]);
    let p = EarleyParser::new(gb.into_grammar("S"));
    let mut input = Lexer::from_str("1+2", "+");
    let ps = p.parse(&mut input).unwrap();
    print_statesets(&ps.states);
    println!("=== tree ===");
    println!("{:?}", build_tree(&p.g, &ps));
}

#[test]
fn test_right_recurse() {
    // P -> N ^ P | N
    // N -> [0-9]
    let mut gb = GrammarBuilder::new();
    gb.symbol(Symbol::nonterm("P"))
      .symbol(Symbol::nonterm("N"))
      .symbol(Symbol::terminal("[^]", |n: &str| n == "^"))
      .symbol(Symbol::terminal("[0-9]", |n: &str| "1234567890".contains(n)))
      .rule("P", vec!["N", "[^]", "P"])
      .rule("P", vec!["N"])
      .rule("N", vec!["[0-9]"]);
    let p = EarleyParser::new(gb.into_grammar("P"));
    let mut input = Lexer::from_str("1^2", "^");
    let ps = p.parse(&mut input).unwrap();
    print_statesets(&ps.states);
    println!("=== tree ===");
    println!("{:?}", build_tree(&p.g, &ps));
}

#[test]
fn grammar_empty() {
    // A -> <empty> | B
    // B -> A
    let mut gb = GrammarBuilder::new();
    gb.symbol(Symbol::nonterm("A"))
      .symbol(Symbol::nonterm("B"))
      .rule("A", Vec::new())
      .rule("A", vec!["B"])
      .rule("B", vec!["A"]);
    let g = gb.into_grammar("A");
    let p = EarleyParser::new(g);
    let mut input = Lexer::from_str("", "-");
    let ps = p.parse(&mut input).unwrap();
    print_statesets(&ps.states);
    println!("=== tree ===");
    println!("{:?}", build_tree(&p.g, &ps));
}

#[test]
fn math_ambiguous() {
    // E -> E + E | E * E | n
    let mut gb = GrammarBuilder::new();
    gb.symbol(Symbol::nonterm("E"))
      .symbol(Symbol::terminal("+", |n: &str| n == "+"))
      .symbol(Symbol::terminal("*", |n: &str| n == "*"))
      .symbol(Symbol::terminal("n", |n: &str|
          n.chars().all(|c| "1234567890".contains(c))))
      .rule("E", vec!["E", "+", "E"])
      .rule("E", vec!["E", "*", "E"])
      .rule("E", vec!["n"]);
    // parse something ... should return 2 parse trees
    let p = EarleyParser::new(gb.into_grammar("E"));
    let mut input = Lexer::from_str("0*1*2*3*4*5", "+*");
    let ps = p.parse(&mut input).unwrap();
    print_statesets(&ps.states);
    println!("=== tree ===");
    for t in build_trees(&p.g, &ps) { println!("{:?}", t); }
}

#[test]
fn math_various() {
    let p = EarleyParser::new(grammar_math());
    let inputs = vec![
        "1+2^3^4*5/6+7*8^9",
        "(1+2^3)^4*5/6+7*8^9",
        "1+2^3^4*5",
        "(1+2)*3",
    ];
    for input in inputs.iter() {
        println!("============ input: {}", input);
        let mut input = Lexer::from_str(input, "+*-/()^");
        let ps = p.parse(&mut input).unwrap();
        print_statesets(&ps.states);
        println!("=== tree ===");
        println!("{:?}", build_tree(&p.g, &ps));
    }
}

#[test]
fn chained_terminals() {
    let rule_variants = vec![
        vec!["X", "+"],
        vec!["+", "X"],
        vec!["X", "+", "+"],
        vec!["+", "+", "X"],
        vec!["+", "X", "+"],
    ];

    for variant in rule_variants {
        let tokens = match variant.len() {
            2 => "+", 3 => "++", _ => unreachable!()
        };
        let mut gb = GrammarBuilder::new();
        gb.symbol(Symbol::nonterm("E"))
          .symbol(Symbol::nonterm("X"))
          .symbol(Symbol::terminal("+", |n: &str| n == "+"))
          .rule("E", variant.clone())
          .rule("X", Vec::new());
        let p = EarleyParser::new(gb.into_grammar("E"));
        let mut input = Lexer::from_str(tokens, "+");
        let ps = p.parse(&mut input).unwrap();
        print_statesets(&ps.states);
        println!("=== tree === variant {:?} === input {}", variant, tokens);
        println!("{:?}", build_tree(&p.g, &ps));
    }
}
