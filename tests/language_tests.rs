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

#[test]
fn test_typed_variables() {
    // Valid declarations
    assert_eq!(eval_str("[set x|int 42] x").unwrap(), Value::new_int(42));
    assert_eq!(eval_str("[set f|float 3.14] f").unwrap(), Value::new_float(3.14));
    assert_eq!(eval_str("[set s|string \"hello\"] s").unwrap(), Value::new_string("hello"));
    assert_eq!(eval_str("[set b|bool true] b").unwrap(), Value::new_bool(true));
    assert_eq!(eval_str("[set arr|array [1 2 3]] [array-length arr]").unwrap(), Value::new_int(3));

    // Spaced syntax
    assert_eq!(eval_str("[set y | int 100] y").unwrap(), Value::new_int(100));

    // Initial mismatch fails
    let err = eval_str("[set x|int \"not an int\"]").unwrap_err();
    assert!(err.contains("Type mismatch for 'x': expected int, got string"));

    let err2 = eval_str("[set f|float \"not a float\"]").unwrap_err();
    assert!(err2.contains("Type mismatch for 'f': expected float, got string"));
}

#[test]
fn test_permanent_type_enforcement() {
    // Re-assignment conforming to type succeeds
    let ok_code = r#"
        [set x|int 10]
        [set x 20]
        x
    "#;
    assert_eq!(eval_str(ok_code).unwrap(), Value::new_int(20));

    // Re-assignment violating type fails
    let bad_code = r#"
        [set x|int 10]
        [set x "bad string"]
    "#;
    let err = eval_str(bad_code).unwrap_err();
    assert!(err.contains("Type mismatch for 'x': expected int, got string"));

    // Re-declaration with same type succeeds
    let redecl_same = r#"
        [set x|int 10]
        [set x|int 30]
        x
    "#;
    assert_eq!(eval_str(redecl_same).unwrap(), Value::new_int(30));

    // Re-declaration with different type fails
    let redecl_diff = r#"
        [set x|int 10]
        [set x|string "not allowed"]
    "#;
    let err2 = eval_str(redecl_diff).unwrap_err();
    assert!(err2.contains("Cannot re-declare variable 'x' of type int as string"));

    // Dynamic variable without type allows reassignment across types
    let dyn_code = r#"
        [set d 10]
        [set d "now a string"]
        d
    "#;
    assert_eq!(eval_str(dyn_code).unwrap(), Value::new_string("now a string"));
}

#[test]
fn test_typed_functions() {
    // Multi-parameter typed function
    let code = r#"
        [def my_add [a|int b|int]
            [add a b]]
        [my_add 15 25]
    "#;
    assert_eq!(eval_str(code).unwrap(), Value::new_int(40));

    // Call with invalid argument type fails
    let bad_call = r#"
        [def my_add [a|int b|int]
            [add a b]]
        [my_add 15 "25"]
    "#;
    let err = eval_str(bad_call).unwrap_err();
    assert!(err.contains("Type mismatch for parameter 'b': expected int, got string"));

    // Single typed parameter function
    let single_param = r#"
        [def square x|int
            [mult x x]]
        [square 8]
    "#;
    assert_eq!(eval_str(single_param).unwrap(), Value::new_int(64));

    let single_bad = r#"
        [def square x|int
            [mult x x]]
        [square "eight"]
    "#;
    assert!(eval_str(single_bad).is_err());

    // Permanent type enforcement within function body
    let body_enforce = r#"
        [def bad_mutate [x|int]
            [set x "not an int"]
            x]
        [bad_mutate 42]
    "#;
    let err_body = eval_str(body_enforce).unwrap_err();
    assert!(err_body.contains("Type mismatch for 'x': expected int, got string"));
}

#[test]
fn test_dropped_colon_and_object_colons() {
    // Old colon syntax for types is rejected with clear error
    let old_set = "[set x: int 42]";
    let err = eval_str(old_set).unwrap_err();
    assert!(err.contains("Unexpected colon ':' at line 1. Type annotations must use pipe '|' (e.g. x|int)"));

    // Object literals with colon key-value separator still work
    let obj_code = r#"
        [set user {"name": "Bob", "age": 28}]
        [obj-get user "name"]
    "#;
    assert_eq!(eval_str(obj_code).unwrap(), Value::new_string("Bob"));
}

