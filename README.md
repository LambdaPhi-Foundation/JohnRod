# JohnRod (jonrod)

> **A fast, expressive Lisp/Tcl-inspired scripting language and reference foundation for [JR-JIT](JRJIT.md).**

---

## Table of Contents
- [Overview](#overview)
- [Design Philosophy & The Stepping Stone to JR-JIT](#design-philosophy--the-stepping-stone-to-jr-jit)
- [Language Syntax & Features](#language-syntax--features)
  - [Prefix S-Expression Syntax](#prefix-s-expression-syntax)
  - [Pipe-Bound Type Annotations (`x|int`)](#pipe-bound-type-annotations-xint)
  - [Permanent Type Enforcement](#permanent-type-enforcement)
  - [Data Types](#data-types)
  - [Variables & Scope](#variables--scope)
  - [Functions](#functions)
  - [Control Flow](#control-flow)
  - [Arrays & Objects](#arrays--objects)
  - [System I/O & Built-ins](#system-io--built-ins)
  - [Error Handling](#error-handling)
- [CLI & REPL Usage](#cli--repl-usage)
- [Project Architecture](#project-architecture)
- [Building & Testing](#building--testing)

---

## Overview

**JohnRod** is a lightweight, high-performance interpreted scripting language written in Rust. It combines the clean, unburdened simplicity of Lisp/Tcl prefix expressions with modern type safety, dynamic evaluation, and native operating system capabilities.

```scheme
# Quick JohnRod sample
[def factorial [n|int]
  [if [le n 1]
    1
    [mult n [factorial [sub n 1]]]]]

[set result|int [factorial 5]]
[println "Factorial of 5 is: " result] # 120
```

---

## Design Philosophy & The Stepping Stone to JR-JIT

JohnRod serves a dual purpose:
1. **A practical, batteries-included interpreter** for system automation, rapid file manipulation, JSON processing, and web scripting.
2. **The semantic reference implementation and testbed for [JR-JIT](JRJIT.md)**.

### Why JohnRod is the Stepping Stone to JR-JIT

```
┌────────────────────────────────┐
│   JohnRod Script (.jr / CLI)   │
└───────────────┬────────────────┘
                │ Fast Byte-Level Lexer & Parser
                ▼
┌────────────────────────────────┐
│       Typed AST (ast.rs)       │
├────────────────────────────────┤
│ • ExprKind::TypedId("x","int") │ ───┐
│ • ExprKind::List([...])        │    │
└───────────────┬────────────────┘    │
                │                     │
       ┌────────┴────────┐            │
       ▼                 ▼            ▼
┌──────────────┐  ┌───────────────────────────────────┐
│ Current Tree-│  │              JR-JIT               │
│ Walk Runtime │  │    (Native Machine Code Emitter)  │
│  (eval.rs)   │  ├───────────────────────────────────┤
│              │  │ • Maps x|int directly to RAX/RDI  │
│ • Portable   │  │ • Sub-microsecond JIT compilation │
│ • Safe VM    │  │ • Zero-allocation native ABI      │
│ • Full I/O   │  │ • Direct C FFI (libcurl, libc)    │
└──────────────┘  └───────────────────────────────────┘
```

The design of JohnRod was engineered from the ground up to prepare for native machine code compilation:
- **Pipe-Bound Types (`x|int`)**: By binding variable names directly to hardware-mappable types (`int`, `float`, `bool`, `ptr`), the AST provides the exact type signatures needed by the JIT compiler without requiring complex type-inference passes.
- **Permanent Type Enforcement**: In JohnRod, declaring `[set x|int 42]` locks `x` to `int` for its entire lifetime. This mirrors native compilation, where variables occupy fixed register widths (`i64`, `f64`) or typed stack slots rather than polymorphic variant boxes.
- **Microscopic Core Semantics**: The control flow (`if`, `while`, `for`) and function definitions in this interpreter translate directly to low-level assembly branching instructions (`CMP`, `JMP`, `JNE`, `CALL`).

For the complete technical blueprint of the native machine emitter, refer to [JRJIT.md](JRJIT.md).

---

## Language Syntax & Features

### Prefix S-Expression Syntax
Everything in JohnRod is an expression wrapped in square brackets:
```scheme
[command arg1 arg2 ...]
```
No semicolons, no braces for blocks, and no complex operator precedence rules.

### Pipe-Bound Type Annotations (`x|int`)
JohnRod handles optional type declarations with a pipe (`|`), visually and semantically binding the type to the identifier:

```scheme
# Typed variable declarations
[set x|int 42]
[set rate|float 0.075]
[set name|string "Alice"]
[set is_active|bool true]
[set scores|array [90 85 95]]
[set config|object {"debug": true}]

# Spaced syntax is also supported
[set count | int 10]
```

Colon `:` is reserved strictly for key-value pairs in object literals (`{"key": value}`). Using colon for types is cleanly rejected with a migration diagnostic.

### Permanent Type Enforcement
When a variable is declared with a type, that type constraint is permanently enforced within that scope:

```scheme
[set x|int 10]

# Valid: Re-assignment with a conforming type
[set x 20]
[set x [add x 5]] # x is now 25

# Error: Re-assignment with a mismatched type
[set x "twenty"]  # -> Error: Type mismatch for 'x': expected int, got string

# Error: Attempting to re-declare with another type
[set x|string "test"] # -> Error: Cannot re-declare variable 'x' of type int as string

# Dynamic variables (without type annotations) allow flexible typing
[set dyn 100]
[set dyn "flexible"] # OK
```

### Data Types

| Type | Example | Description |
|---|---|---|
| `int` | `42`, `-10` | 64-bit signed integer |
| `float` | `3.14`, `-0.5` | 64-bit IEEE 754 floating point |
| `string` | `"hello\nworld"` | UTF-8 text with escape characters |
| `bool` | `true`, `false` | Boolean values |
| `void` | `()` / implicit | Unit / absence of value |
| `array` | `[1 2 3]` | Ordered, growable vector of values |
| `object` | `{"k": "v"}` | Key-value dictionary |

Type query built-ins:
```scheme
[isInt 42]       # true
[isFloat 3.14]   # true
[isString "hi"]  # true
[isBool true]    # true
[isArray []]     # true
[isObject {}]    # true
```

### Variables & Scope
Variables are set using the `set` command. JohnRod maintains scoped lexical environments:
```scheme
[set counter 0]
[set counter [add counter 1]]
[println counter] # 1
```

### Functions
Define functions with `def`. Arguments can be untyped or pipe-annotated:

```scheme
# Multiple typed parameters
[def calculate [base|float multiplier|float]
  [mult base multiplier]]

[println [calculate 10.5 2.0]] # 21.0

# Single parameter function
[def square x|int
  [mult x x]]

[println [square 9]] # 81

# Parameter types are permanently enforced in the function scope
[def process [val|int]
  # [set val "bad"] would throw a type error!
  [add val 10]]
```

### Control Flow

#### `if` Expressions
```scheme
[if [gt score 50]
  [println "Passed!"]
  [else [println "Try again!"]]]
```

#### `while` Loops
```scheme
[set i 0]
[while [lt i 5]
  [println "i = " i]
  [set i [add i 1]]]
```

#### `for` Loops
```scheme
[for [set i 0] [lt i 5] [set i [add i 1]]
  [println "loop " i]]
```

### Arrays & Objects

#### Arrays
```scheme
[set items [10 20 30]]
[array-push items 40]
[println [array-get items 1]]    # 20
[println [array-length items]]   # 4
[set last [array-pop items]]     # 40
```

#### Objects
```scheme
[set user {"name": "Bob" "role": "admin"}]
[obj-set user "active" true]
[println [obj-get user "name"]]  # Bob
[println [obj-keys user]]        # [name role active]
```

### System I/O & Built-ins

- **Math**: `add`, `sub`, `mult`, `div`, `rem`, `pow`, `floor`, `ceil`, `round`, `sqrt`
- **Logic & Comparison**: `lt`, `gt`, `le`, `ge`, `eq`, `neq`, `and`, `or`, `not`
- **Strings**: `str-join`, `str-split`, `str-length`
- **JSON**:
  ```scheme
  [set data [json-parse "{\"status\": \"ok\", \"code\": 200}"]]
  [println [obj-get data "status"]] # ok
  [set json_str [json-stringify data]]
  ```
- **File System**:
  ```scheme
  [file-write "log.txt" "Application started\n"]
  [file-append "log.txt" "Event recorded\n"]
  [if [file-exists "log.txt"]
    [println [file-read "log.txt"]]]
  [println [dir-list "."]]
  ```
- **System & Network**:
  ```scheme
  [shell "echo Hello from shell"]
  [set status [shell-status "dir"]]
  [env-set "APP_ENV" "production"]
  [println [env-get "APP_ENV"]]
  [set res [http-get "https://httpbin.org/get"]]
  ```

### Error Handling
Catch runtime exceptions with `try`/`catch`:
```scheme
[try
  [div 10 0]
  [catch err
    [println "Safely caught error: " err]]]
```

---

## CLI & REPL Usage

Build the binary using Cargo:
```bash
cargo build --release
```

The resulting executable is located at `target/release/jonrod`.

### CLI Options

| Flag | Argument | Description |
|---|---|---|
| `-e`, `--eval` | `<expr>` | Evaluate JohnRod code directly |
| `-d`, `--dump` | | Dump Abstract Syntax Tree (AST) before running |
| `-c`, `--check` | | Validate syntax and exit without executing |
| `-h`, `--help` | | Show help menu |
| `-v`, `--version`| | Show version information |

### Examples

```bash
# Execute a script file
jonrod script.jr

# Run inline code
jonrod -e '[set x|int 42] [println [mult x 2]]'

# Inspect the Typed AST
jonrod -d -e '[set x|int 42]'
# Output:
# === Abstract Syntax Tree ===
# List:
#   Id: set
#   TypedId: x|int
#   Int: 42
# ============================

# Interactive REPL
jonrod
```

---

## Project Architecture

```
JohnRod/
├── src/
│   ├── token.rs          # TokenType definitions (including Pipe, Delimiters, Literals)
│   ├── lexer.rs          # Fast zero-copy ASCII/UTF-8 byte scanner
│   ├── ast.rs            # AST nodes (Expr, ExprKind with first-class TypedId)
│   ├── parser.rs         # Recursive-descent S-expression parser
│   ├── env.rs            # Scoped environments with permanent type storage
│   ├── value.rs          # Dynamic Value union and type-checking logic
│   ├── eval.rs           # Core tree-walk evaluator & built-in primitives
│   ├── main.rs           # CLI driver, REPL, and argument routing
│   └── lib.rs            # Public API exports
├── tests/
│   └── language_tests.rs # Full integration test suite
├── architecture.txt      # Full language specification and documentation
├── JRJIT.md              # Technical blueprint for the JR-JIT compiler
├── Cargo.toml            # Project manifest & optimized release profiles
└── README.md             # This document
```

---

## Building & Testing

### Prerequisites
- [Rust](https://www.rust-lang.org/) (2021 Edition, MSRV 1.70+)

### Commands
```bash
# Run the test suite
cargo test

# Build debug binary
cargo build

# Build optimized release binary
cargo build --release
```

---

## License & Credits

Built with Rust. Created as part of the **JohnRod** and **JR-JIT** project. Inspired by Lisp, Tcl, and modern typed systems programming.
