use jonrod::eval::Evaluator;
use jonrod::lexer::tokenize;
use jonrod::parser::parse;
use jonrod::value::Value;

fn eval_str(code: &str) -> Result<Value, String> {
    let tokens = tokenize(code)?;
    let ast = parse(tokens)?;
    let mut ev = Evaluator::new();
    let mut last = Value::new_void();
    for expr in &ast {
        last = ev.eval(expr)?;
    }
    Ok(last)
}

#[test]
fn test_math_operations() {
    assert_eq!(eval_str("[add 10 20]").unwrap(), Value::new_int(30));
    assert_eq!(eval_str("[sub 50 15]").unwrap(), Value::new_int(35));
    assert_eq!(eval_str("[mult 6 7]").unwrap(), Value::new_int(42));
    assert_eq!(eval_str("[div 100 4]").unwrap(), Value::new_float(25.0));
    assert_eq!(eval_str("[rem 10 3]").unwrap(), Value::new_int(1));
    assert_eq!(eval_str("[pow 2 8]").unwrap(), Value::new_float(256.0));
    assert_eq!(eval_str("[floor 3.9]").unwrap(), Value::new_int(3));
    assert_eq!(eval_str("[ceil 3.1]").unwrap(), Value::new_int(4));
    assert_eq!(eval_str("[round 3.5]").unwrap(), Value::new_int(4));
    assert_eq!(eval_str("[sqrt 64]").unwrap(), Value::new_float(8.0));
}

#[test]
fn test_comparisons() {
    assert_eq!(eval_str("[lt 5 10]").unwrap(), Value::new_bool(true));
    assert_eq!(eval_str("[gt 5 10]").unwrap(), Value::new_bool(false));
    assert_eq!(eval_str("[le 10 10]").unwrap(), Value::new_bool(true));
    assert_eq!(eval_str("[ge 10 9]").unwrap(), Value::new_bool(true));
    assert_eq!(eval_str("[eq 42 42]").unwrap(), Value::new_bool(true));
    assert_eq!(eval_str("[eq \"abc\" \"abc\"]").unwrap(), Value::new_bool(true));
    assert_eq!(eval_str("[neq 42 100]").unwrap(), Value::new_bool(true));
}

#[test]
fn test_logic() {
    assert_eq!(eval_str("[and true true]").unwrap(), Value::new_bool(true));
    assert_eq!(eval_str("[and true false]").unwrap(), Value::new_bool(false));
    assert_eq!(eval_str("[or false true]").unwrap(), Value::new_bool(true));
    assert_eq!(eval_str("[or false false]").unwrap(), Value::new_bool(false));
    assert_eq!(eval_str("[not true]").unwrap(), Value::new_bool(false));
    assert_eq!(eval_str("[not false]").unwrap(), Value::new_bool(true));
}

#[test]
fn test_control_flow() {
    assert_eq!(
        eval_str("[if [lt 5 10] [add 1 2] [add 3 4]]").unwrap(),
        Value::new_int(3)
    );
    assert_eq!(
        eval_str("[if [gt 5 10] [add 1 2] [add 3 4]]").unwrap(),
        Value::new_int(7)
    );
    assert_eq!(
        eval_str("[if [gt 5 10] [add 1 2] [else [add 10 20]]]").unwrap(),
        Value::new_int(30)
    );

    let while_code = r#"
        [set i 0]
        [set acc 0]
        [while [lt i 5]
            [set acc [add acc i]]
            [set i [add i 1]]]
        acc
    "#;
    assert_eq!(eval_str(while_code).unwrap(), Value::new_int(10));

    let for_code = r#"
        [set sum 0]
        [for [set j 1] [le j 10] [set j [add j 1]]
            [set sum [add sum j]]]
        sum
    "#;
    assert_eq!(eval_str(for_code).unwrap(), Value::new_int(55));
}

#[test]
fn test_functions() {
    let code = r#"
        [def square [x]
            [mult x x]]
        [square 9]
    "#;
    assert_eq!(eval_str(code).unwrap(), Value::new_int(81));

    let multi_code = r#"
        [def calc [a b c]
            [set temp [add a b]]
            [mult temp c]]
        [calc 2 3 4]
    "#;
    assert_eq!(eval_str(multi_code).unwrap(), Value::new_int(20));

    let recur_code = r#"
        [def fact [n]
            [if [le n 1]
                1
                [mult n [fact [sub n 1]]]]]
        [fact 5]
    "#;
    assert_eq!(eval_str(recur_code).unwrap(), Value::new_int(120));
}

#[test]
fn test_arrays() {
    let code = r#"
        [set arr [10 20 30]]
        [array-push arr 40]
        [array-length arr]
    "#;
    assert_eq!(eval_str(code).unwrap(), Value::new_int(4));

    let get_code = r#"
        [set arr [10 20 30]]
        [array-get arr 1]
    "#;
    assert_eq!(eval_str(get_code).unwrap(), Value::new_int(20));

    let pop_code = r#"
        [set arr [10 20 30]]
        [array-pop arr]
    "#;
    assert_eq!(eval_str(pop_code).unwrap(), Value::new_int(30));
}

#[test]
fn test_objects_and_json() {
    let code = r#"
        [set user {"name" "Alice" "age" 30}]
        [obj-get user "name"]
    "#;
    assert_eq!(eval_str(code).unwrap(), Value::new_string("Alice"));

    let json_code = r#"
        [set parsed [json-parse "{\"count\": 42}"]]
        [obj-get parsed "count"]
    "#;
    assert_eq!(eval_str(json_code).unwrap(), Value::new_int(42));
}

#[test]
fn test_strings() {
    let join_code = r#"[str-join ", " ["a" "b" "c"]]"#;
    assert_eq!(eval_str(join_code).unwrap(), Value::new_string("a, b, c"));

    let len_code = r#"[str-length "Antigravity"]"#;
    assert_eq!(eval_str(len_code).unwrap(), Value::new_int(11));
}

#[test]
fn test_try_catch() {
    let code = r#"
        [try
            [div 10 0]
            [catch err
                "caught error"]]
    "#;
    assert_eq!(eval_str(code).unwrap(), Value::new_string("caught error"));
}
