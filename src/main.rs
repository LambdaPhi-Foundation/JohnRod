use jonrod::ast::dump_ast;
use jonrod::eval::Evaluator;
use jonrod::lexer::tokenize;
use jonrod::parser::parse;
use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use std::process;

fn print_help() {
    println!(
        r#"JonRod Language Interpreter - Version 1.0

USAGE:
  jonrod [options] [file.jr]

OPTIONS:
  -h, --help       Show this help message
  -v, --version    Show version information
  -e, --eval       Evaluate expression directly from command line
  -d, --dump       Dump AST before execution (debug)
  -c, --check      Check syntax only (no execution)
  --no-color       Disable colored output

EXAMPLES:
  jonrod script.jr                    # Run a script file
  jonrod -e '[println "Hello World"]'  # Evaluate expression directly
  jonrod -d script.jr                 # Dump AST and run
  jonrod -c script.jr                 # Check syntax only

FILE EXTENSIONS:
  .jr     - JohnRod source file

ENVIRONMENT VARIABLES:
  JOHNROD_PATH  - Additional directories to search for modules
  JOHNROD_DEBUG - Set to 1 to enable debug output"#
    );
    process::exit(0);
}

fn print_version() {
    println!(
        r#"JohnRod Language Interpreter - Version 1.0
Built with Rust (Aggressively Optimized)"#
    );
    process::exit(0);
}

fn display_error(source: &str, err_msg: &str, filename: &str) {
    let mut line_num: Option<usize> = None;
    let mut clean_msg = err_msg.to_string();

    let lower = err_msg.to_ascii_lowercase();
    if let Some(pos) = lower.find("line") {
        let remainder = &err_msg[pos + 4..];
        let trimmed = remainder.trim_start_matches(|c: char| c.is_whitespace() || c == ':');
        let num_str: String = trimmed.chars().take_while(|c| c.is_ascii_digit()).collect();
        if let Ok(num) = num_str.parse::<usize>() {
            line_num = Some(num);
            if let Some(colon_pos) = trimmed.find(':') {
                clean_msg = trimmed[colon_pos + 1..].trim().to_string();
            }
        }
    }

    if let Some(l) = line_num {
        let lines: Vec<&str> = source.lines().collect();
        if l > 0 && l <= lines.len() {
            println!("Error at line {}: {}", l, clean_msg);
            let line_content = lines[l - 1].trim();
            println!("{}", line_content);
            println!("{}^", " ".repeat(line_content.len()));
            return;
        }
    }

    if filename != "<stdin>" {
        println!("Error in {}: {}", filename, err_msg);
    } else {
        println!("Error: {}", err_msg);
    }
}

fn run_script(filename: &str, source: &str, dump_ast_flag: bool, check_only: bool) {
    // 1. Lexing
    let tokens = match tokenize(source) {
        Ok(toks) => toks,
        Err(e) => {
            display_error(source, &e, filename);
            process::exit(1);
        }
    };

    // 2. Parsing
    let ast = match parse(tokens) {
        Ok(a) => a,
        Err(e) => {
            display_error(source, &e, filename);
            process::exit(1);
        }
    };

    // 3. Dump AST
    if dump_ast_flag {
        println!("=== Abstract Syntax Tree ===");
        for node in &ast {
            dump_ast(node, 0);
        }
        println!("============================");
        if check_only {
            println!("Syntax check passed");
            return;
        }
    }

    // 4. Check only
    if check_only {
        println!("Syntax check passed");
        return;
    }

    // 5. Evaluate
    let mut ev = Evaluator::new();
    for node in &ast {
        if let Err(e) = ev.eval(node) {
            let msg = if node.line > 0 {
                format!("Error at line {}: {}", node.line, e)
            } else {
                e
            };
            display_error(source, &msg, filename);
            process::exit(1);
        }
    }
}

fn run_repl() {
    println!("JohnRod REPL - Type expressions to evaluate (Ctrl+C to exit)");
    println!("Type 'help' for commands\n");

    let mut ev = Evaluator::new();
    let stdin = io::stdin();
    let mut reader = stdin.lock();

    loop {
        print!("> ");
        if io::stdout().flush().is_err() {
            break;
        }

        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => {
                println!("Goodbye!");
                break;
            }
            Ok(_) => {
                let trimmed = line.trim();
                match trimmed {
                    "help" => {
                        println!(
                            r#"Available REPL commands:
  help    - Show this help
  exit    - Exit REPL
  clear   - Clear environment variables
  env     - Show current environment
  ast     - Show AST before evaluation"#
                        );
                        continue;
                    }
                    "exit" | "quit" => {
                        println!("Goodbye!");
                        process::exit(0);
                    }
                    "clear" => {
                        ev = Evaluator::new();
                        println!("Environment cleared");
                        continue;
                    }
                    "env" => {
                        println!("Variables:");
                        let mut vars: Vec<_> = ev
                            .env
                            .borrow()
                            .vars
                            .iter()
                            .map(|(k, v)| (k.clone(), v.to_string_repr()))
                            .collect();
                        vars.sort_by(|a, b| a.0.cmp(&b.0));
                        for (k, v) in vars {
                            println!("  {} = {}", k, v);
                        }

                        println!("Functions:");
                        let mut funcs: Vec<_> =
                            ev.env.borrow().funcs.keys().cloned().collect();
                        funcs.sort();
                        for f in funcs {
                            println!("  {} (function)", f);
                        }
                        continue;
                    }
                    "" => continue,
                    _ => {}
                }

                // Parse and execute
                match tokenize(trimmed) {
                    Ok(toks) => match parse(toks) {
                        Ok(ast) => {
                            for node in &ast {
                                match ev.eval(node) {
                                    Ok(res) => {
                                        if !res.is_void() {
                                            println!("{}", res.to_string_repr());
                                        }
                                    }
                                    Err(e) => {
                                        println!("Error: {}", e);
                                    }
                                }
                            }
                        }
                        Err(e) => println!("Error: {}", e),
                    },
                    Err(e) => println!("Error: {}", e),
                }
            }
            Err(_) => {
                println!("Goodbye!");
                break;
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        run_repl();
        return;
    }

    let mut i = 0;
    let mut filename = String::new();
    let mut eval_expr = String::new();
    let mut dump = false;
    let mut check = false;
    let mut _no_color = false;

    while i < args.len() {
        let arg = &args[i];
        match arg.as_str() {
            "-h" | "--help" => print_help(),
            "-v" | "--version" => print_version(),
            "-d" | "--dump" => {
                dump = true;
                i += 1;
            }
            "-c" | "--check" => {
                check = true;
                i += 1;
            }
            "--no-color" => {
                _no_color = true;
                i += 1;
            }
            "-e" | "--eval" => {
                if i + 1 < args.len() {
                    eval_expr = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: -e requires an expression");
                    process::exit(1);
                }
            }
            _ => {
                filename = arg.clone();
                i += 1;
            }
        }
    }

    if !eval_expr.is_empty() {
        match tokenize(&eval_expr) {
            Ok(tokens) => match parse(tokens) {
                Ok(ast) => {
                    if dump {
                        println!("=== Abstract Syntax Tree ===");
                        for node in &ast {
                            dump_ast(node, 0);
                        }
                        println!("============================");
                    }
                    if check {
                        println!("Syntax check passed");
                        return;
                    }

                    let mut ev = Evaluator::new();
                    for node in &ast {
                        match ev.eval(node) {
                            Ok(res) => {
                                if !res.is_void() {
                                    println!("{}", res.to_string_repr());
                                }
                            }
                            Err(e) => {
                                eprintln!("Error: {}", e);
                                process::exit(1);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    process::exit(1);
                }
            },
            Err(e) => {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        return;
    }

    if !filename.is_empty() {
        match fs::read_to_string(&filename) {
            Ok(source) => {
                run_script(&filename, &source, dump, check);
            }
            Err(e) => {
                eprintln!("Error: Failed to read file '{}': {}", filename, e);
                process::exit(1);
            }
        }
    } else {
        print_help();
    }
}
