use crate::ast::{Expr, ExprKind};
use crate::env::{Environment, FunctionDef};
use crate::value::Value;
use chrono::{Datelike, Local, Timelike};
use std::cell::RefCell;
use std::io::{self, Write};
use std::process::Command;
use std::rc::Rc;
use std::time::Duration;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Builtin {
    // Variables & Control
    Set,
    If,
    While,
    For,
    Def,
    Try,
    // Output & Input
    Print,
    Println,
    Read,
    // Files
    FileRead,
    FileWrite,
    FileAppend,
    FileExists,
    DirList,
    DirCreate,
    // Process & Environment
    Shell,
    ShellStatus,
    EnvGet,
    EnvSet,
    // Network
    HttpGet,
    HttpPost,
    // Time
    TimeNow,
    Sleep,
    // String
    StrJoin,
    StrSplit,
    StrLength,
    // Math
    Add,
    Sub,
    Mult,
    Div,
    Rem,
    Pow,
    Floor,
    Ceil,
    Round,
    Sqrt,
    // Comparison
    Lt,
    Gt,
    Le,
    Ge,
    Eq,
    Neq,
    // Logic
    And,
    Or,
    Not,
    // Arrays
    ArrayPush,
    ArrayPop,
    ArrayGet,
    ArraySet,
    ArrayLength,
    // Objects
    ObjSet,
    ObjGet,
    ObjKeys,
    // JSON
    JsonParse,
    JsonStringify,
    // Type checks
    IsInt,
    IsFloat,
    IsNumber,
    IsString,
    IsBool,
    IsArray,
    IsObject,
    IsVoid,
}

#[inline(always)]
fn resolve_builtin(name: &str) -> Option<Builtin> {
    match name {
        "set" => Some(Builtin::Set),
        "if" => Some(Builtin::If),
        "while" => Some(Builtin::While),
        "for" => Some(Builtin::For),
        "def" => Some(Builtin::Def),
        "try" => Some(Builtin::Try),
        "print" => Some(Builtin::Print),
        "println" => Some(Builtin::Println),
        "read" => Some(Builtin::Read),
        "file-read" => Some(Builtin::FileRead),
        "file-write" => Some(Builtin::FileWrite),
        "file-append" => Some(Builtin::FileAppend),
        "file-exists" => Some(Builtin::FileExists),
        "dir-list" => Some(Builtin::DirList),
        "dir-create" => Some(Builtin::DirCreate),
        "shell" => Some(Builtin::Shell),
        "shell-status" => Some(Builtin::ShellStatus),
        "env-get" => Some(Builtin::EnvGet),
        "env-set" => Some(Builtin::EnvSet),
        "http-get" => Some(Builtin::HttpGet),
        "http-post" => Some(Builtin::HttpPost),
        "time-now" => Some(Builtin::TimeNow),
        "sleep" => Some(Builtin::Sleep),
        "str-join" => Some(Builtin::StrJoin),
        "str-split" => Some(Builtin::StrSplit),
        "str-length" => Some(Builtin::StrLength),
        "add" => Some(Builtin::Add),
        "sub" => Some(Builtin::Sub),
        "mult" => Some(Builtin::Mult),
        "div" => Some(Builtin::Div),
        "rem" => Some(Builtin::Rem),
        "pow" => Some(Builtin::Pow),
        "floor" => Some(Builtin::Floor),
        "ceil" => Some(Builtin::Ceil),
        "round" => Some(Builtin::Round),
        "sqrt" => Some(Builtin::Sqrt),
        "lt" => Some(Builtin::Lt),
        "gt" => Some(Builtin::Gt),
        "le" => Some(Builtin::Le),
        "ge" => Some(Builtin::Ge),
        "eq" => Some(Builtin::Eq),
        "neq" => Some(Builtin::Neq),
        "and" => Some(Builtin::And),
        "or" => Some(Builtin::Or),
        "not" => Some(Builtin::Not),
        "array-push" => Some(Builtin::ArrayPush),
        "array-pop" => Some(Builtin::ArrayPop),
        "array-get" => Some(Builtin::ArrayGet),
        "array-set" => Some(Builtin::ArraySet),
        "array-length" => Some(Builtin::ArrayLength),
        "obj-set" => Some(Builtin::ObjSet),
        "obj-get" => Some(Builtin::ObjGet),
        "obj-keys" => Some(Builtin::ObjKeys),
        "json-parse" => Some(Builtin::JsonParse),
        "json-stringify" => Some(Builtin::JsonStringify),
        "isInt" => Some(Builtin::IsInt),
        "isFloat" => Some(Builtin::IsFloat),
        "isNumber" => Some(Builtin::IsNumber),
        "isString" => Some(Builtin::IsString),
        "isBool" => Some(Builtin::IsBool),
        "isArray" => Some(Builtin::IsArray),
        "isObject" => Some(Builtin::IsObject),
        "isVoid" => Some(Builtin::IsVoid),
        _ => None,
    }
}

pub struct Evaluator {
    pub env: Rc<RefCell<Environment>>,
}

impl Evaluator {
    pub fn new() -> Self {
        Self {
            env: Rc::new(RefCell::new(Environment::new())),
        }
    }

    pub fn eval(&mut self, expr: &Expr) -> Result<Value, String> {
        match &expr.kind {
            ExprKind::Int(v) => Ok(Value::new_int(*v)),
            ExprKind::Float(v) => Ok(Value::new_float(*v)),
            ExprKind::String(v) => Ok(Value::new_string(v.clone())),
            ExprKind::Bool(v) => Ok(Value::new_bool(*v)),
            ExprKind::Comment(_) => Ok(Value::new_void()),

            ExprKind::Id(name) => {
                if let Some(val) = self.env.borrow().lookup_var(name) {
                    Ok(val)
                } else {
                    Err(format!("Undefined variable '{}'", name))
                }
            }

            ExprKind::TypedId(name, ty) => Err(format!(
                "Unexpected type annotation '|{}' on variable '{}' at line {}. Type annotations can only be used in variable declarations or function parameters",
                ty, name, expr.line
            )),

            ExprKind::Array(elements) => {
                let mut vals = Vec::with_capacity(elements.len());
                for el in elements {
                    vals.push(self.eval(el)?);
                }
                Ok(Value::new_array_with(vals))
            }

            ExprKind::Object(pairs) => {
                let obj = Value::new_object();
                if let Value::Object(ref o) = obj {
                    let mut b = o.borrow_mut();
                    for (k_expr, v_expr) in pairs {
                        let k = match &k_expr.kind {
                            ExprKind::String(s) => s.clone(),
                            ExprKind::Id(s) => s.clone(),
                            _ => self.eval(k_expr)?.to_string_repr(),
                        };
                        let v = self.eval(v_expr)?;
                        b.insert(k, v);
                    }
                }
                Ok(obj)
            }

            ExprKind::List(children) => {
                if children.is_empty() {
                    return Ok(Value::new_array());
                }

                let first = &children[0];

                // If first element is an identifier, try command or user function
                if let ExprKind::Id(ref cmd) = first.kind {
                    let args = &children[1..];

                    if let Some(builtin) = resolve_builtin(cmd) {
                        return self.eval_builtin(builtin, args, expr.line);
                    }

                    // Check if it is a user-defined function
                    let maybe_func = self.env.borrow().lookup_func(cmd);
                    if let Some(func_def) = maybe_func {
                        let mut arg_values = Vec::with_capacity(args.len());
                        for arg in args {
                            arg_values.push(self.eval(arg)?);
                        }

                        if func_def.params.len() != arg_values.len() {
                            return Err(format!(
                                "Function {} expects {} arguments, got {}",
                                cmd,
                                func_def.params.len(),
                                arg_values.len()
                            ));
                        }

                        // Type check parameters if annotated
                        for (i, ptype) in func_def.param_types.iter().enumerate() {
                            if let Some(expected) = ptype {
                                if !arg_values[i].check_type(expected) {
                                    return Err(format!(
                                        "Type mismatch for parameter '{}': expected {}, got {}",
                                        func_def.params[i],
                                        expected,
                                        arg_values[i].type_name()
                                    ));
                                }
                            }
                        }

                        // Create local function scope
                        let local_env = Rc::new(RefCell::new(Environment::with_parent(Rc::clone(
                            &self.env,
                        ))));
                        for (i, param_name) in func_def.params.iter().enumerate() {
                            local_env
                                .borrow_mut()
                                .set_var(param_name.clone(), arg_values[i].clone());
                            if let Some(ref ptype) = func_def.param_types[i] {
                                local_env
                                    .borrow_mut()
                                    .set_type(param_name.clone(), ptype.clone());
                            }
                        }

                        let old_env = Rc::clone(&self.env);
                        self.env = local_env;

                        let mut result = Value::new_void();
                        for body_expr in &func_def.body {
                            result = self.eval(body_expr)?;
                        }

                        self.env = old_env;
                        return Ok(result);
                    }
                }

                // If not a command or function, evaluate as an array literal: [1 2 3], ["a" "b"], etc.
                let mut elements = Vec::with_capacity(children.len());
                for child in children {
                    elements.push(self.eval(child)?);
                }
                Ok(Value::new_array_with(elements))
            }
        }
    }

    fn eval_block(&mut self, exprs: &[Expr]) -> Result<Value, String> {
        let mut last = Value::new_void();
        for expr in exprs {
            last = self.eval(expr)?;
        }
        Ok(last)
    }

    fn eval_branch(&mut self, expr: &Expr) -> Result<Value, String> {
        // If expr is a list whose first element is another list, it's a block of statements
        if let ExprKind::List(children) = &expr.kind {
            if !children.is_empty() && matches!(children[0].kind, ExprKind::List(_)) {
                return self.eval_block(children);
            }
        }
        self.eval(expr)
    }

    fn eval_builtin(
        &mut self,
        builtin: Builtin,
        args: &[Expr],
        _line: usize,
    ) -> Result<Value, String> {
        match builtin {
            // ------- Variable Operations ----------
            Builtin::Set => {
                if args.len() != 2 {
                    return Err(
                        "Usage: [set name value] or [set name|type value]".to_string()
                    );
                }

                let (var_name, declared_type) = match &args[0].kind {
                    ExprKind::Id(s) => (s.clone(), None),
                    ExprKind::TypedId(name, ty) => (name.clone(), Some(ty.clone())),
                    _ => {
                        return Err(
                            "Variable name must be an identifier or typed identifier (e.g. x|int)".to_string()
                        )
                    }
                };

                let existing_type = self.env.borrow().lookup_type(&var_name);
                let (target_type, is_new_type) = match (existing_type, declared_type) {
                    (Some(existing), Some(new_decl)) => {
                        if existing != new_decl {
                            return Err(format!(
                                "Cannot re-declare variable '{}' of type {} as {}",
                                var_name, existing, new_decl
                            ));
                        }
                        (existing, false)
                    }
                    (Some(existing), None) => (existing, false),
                    (None, Some(new_decl)) => (new_decl, true),
                    (None, None) => (String::new(), false),
                };

                let val = self.eval(&args[1])?;

                if !target_type.is_empty() && !val.check_type(&target_type) {
                    return Err(format!(
                        "Type mismatch for '{}': expected {}, got {}",
                        var_name,
                        target_type,
                        val.type_name()
                    ));
                }

                if is_new_type {
                    self.env
                        .borrow_mut()
                        .set_type(var_name.clone(), target_type);
                }
                self.env.borrow_mut().set_var(var_name, val.clone());
                Ok(val)
            }

            // ------- Control Flow ----------
            Builtin::If => {
                if args.len() < 2 {
                    return Err(
                        "Usage: [if condition true-branch [else false-branch]]".to_string()
                    );
                }

                let cond = self.eval(&args[0])?;
                let cond_bool = cond.as_bool()?;

                if cond_bool {
                    self.eval_branch(&args[1])
                } else if args.len() > 2 {
                    // Check if false branch is [else ...]
                    let else_expr = &args[2];
                    if let ExprKind::List(children) = &else_expr.kind {
                        if !children.is_empty() {
                            if let ExprKind::Id(ref s) = children[0].kind {
                                if s == "else" {
                                    return self.eval_block(&children[1..]);
                                }
                            }
                        }
                    }
                    self.eval_branch(else_expr)
                } else {
                    Ok(Value::new_void())
                }
            }

            Builtin::While => {
                if args.len() < 2 {
                    return Err("Usage: [while condition body...]".to_string());
                }

                let cond_expr = &args[0];
                let body = &args[1..];

                loop {
                    let cond = self.eval(cond_expr)?;
                    if !cond.as_bool()? {
                        break;
                    }
                    for b in body {
                        self.eval_branch(b)?;
                    }
                }
                Ok(Value::new_void())
            }

            Builtin::For => {
                if args.len() < 4 {
                    return Err("Usage: [for init condition update body...]".to_string());
                }

                self.eval(&args[0])?; // init
                let cond_expr = &args[1];
                let update_expr = &args[2];
                let body = &args[3..];

                loop {
                    let cond = self.eval(cond_expr)?;
                    if !cond.as_bool()? {
                        break;
                    }
                    for b in body {
                        self.eval_branch(b)?;
                    }
                    self.eval(update_expr)?;
                }
                Ok(Value::new_void())
            }

            Builtin::Def => {
                if args.len() < 3 {
                    return Err("Usage: [def name params body...]".to_string());
                }

                let name = match &args[0].kind {
                    ExprKind::Id(s) => s.clone(),
                    _ => return Err("Function name must be an identifier".to_string()),
                };

                let mut params = Vec::new();
                let mut param_types = Vec::new();

                match &args[1].kind {
                    ExprKind::Id(p) => {
                        params.push(p.clone());
                        param_types.push(None);
                    }
                    ExprKind::TypedId(p, t) => {
                        params.push(p.clone());
                        param_types.push(Some(t.clone()));
                    }
                    ExprKind::List(children) => {
                        for child in children {
                            match &child.kind {
                                ExprKind::Id(p) => {
                                    params.push(p.clone());
                                    param_types.push(None);
                                }
                                ExprKind::TypedId(p, t) => {
                                    params.push(p.clone());
                                    param_types.push(Some(t.clone()));
                                }
                                _ => {
                                    return Err(
                                        "Parameter names must be identifiers or typed identifiers (e.g. x|int)".to_string()
                                    );
                                }
                            }
                        }
                    }
                    _ => {
                        return Err(
                            "Parameters must be an identifier, typed identifier, or list: [p|int q|float r]".to_string()
                        )
                    }
                }

                // Function body: if body is a single list of lists [ [s1] [s2] ], unpack
                let body = if args.len() == 3 {
                    if let ExprKind::List(children) = &args[2].kind {
                        if !children.is_empty() && matches!(children[0].kind, ExprKind::List(_)) {
                            children.clone()
                        } else {
                            vec![args[2].clone()]
                        }
                    } else {
                        vec![args[2].clone()]
                    }
                } else {
                    args[2..].to_vec()
                };

                self.env.borrow_mut().set_func(
                    name,
                    FunctionDef {
                        params,
                        param_types,
                        body,
                    },
                );
                Ok(Value::new_void())
            }

            // ------- Console Output & Input ----------
            Builtin::Print => {
                let stdout = io::stdout();
                let mut handle = stdout.lock();
                for arg in args {
                    let val = self.eval(arg)?;
                    write!(handle, "{}", val.to_string_repr()).map_err(|e| e.to_string())?;
                }
                handle.flush().map_err(|e| e.to_string())?;
                Ok(Value::new_void())
            }

            Builtin::Println => {
                let mut parts = Vec::with_capacity(args.len());
                for arg in args {
                    let val = self.eval(arg)?;
                    parts.push(val.to_string_repr());
                }
                println!("{}", parts.join(" "));
                Ok(Value::new_void())
            }

            Builtin::Read => {
                if !args.is_empty() {
                    let prompt = self.eval(&args[0])?.to_string_repr();
                    print!("{}", prompt);
                    io::stdout().flush().map_err(|e| e.to_string())?;
                }
                let mut buffer = String::new();
                io::stdin().read_line(&mut buffer).map_err(|e| e.to_string())?;
                // Strip trailing newline / carriage return
                if buffer.ends_with('\n') {
                    buffer.pop();
                    if buffer.ends_with('\r') {
                        buffer.pop();
                    }
                }
                Ok(Value::new_string(buffer))
            }

            // ------- File I/O ----------
            Builtin::FileRead => {
                if args.len() != 1 {
                    return Err("Usage: [file-read path]".to_string());
                }
                let path = self.eval(&args[0])?.as_string()?;
                let content = std::fs::read_to_string(&path)
                    .map_err(|_| format!("Failed to read file: {}", path))?;
                Ok(Value::new_string(content))
            }

            Builtin::FileWrite => {
                if args.len() != 2 {
                    return Err("Usage: [file-write path content]".to_string());
                }
                let path = self.eval(&args[0])?.as_string()?;
                let content = self.eval(&args[1])?.to_string_repr();
                std::fs::write(&path, content)
                    .map_err(|_| format!("Failed to write file: {}", path))?;
                Ok(Value::new_void())
            }

            Builtin::FileAppend => {
                if args.len() != 2 {
                    return Err("Usage: [file-append path content]".to_string());
                }
                let path = self.eval(&args[0])?.as_string()?;
                let content = self.eval(&args[1])?.to_string_repr();
                use std::fs::OpenOptions;
                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&path)
                    .map_err(|_| format!("Failed to append to file: {}", path))?;
                file.write_all(content.as_bytes())
                    .map_err(|_| format!("Failed to append to file: {}", path))?;
                Ok(Value::new_void())
            }

            Builtin::FileExists => {
                if args.len() != 1 {
                    return Err("Usage: [file-exists path]".to_string());
                }
                let path = self.eval(&args[0])?.as_string()?;
                Ok(Value::new_bool(std::path::Path::new(&path).exists()))
            }

            Builtin::DirList => {
                if args.len() != 1 {
                    return Err("Usage: [dir-list path]".to_string());
                }
                let path = self.eval(&args[0])?.as_string()?;
                let entries = std::fs::read_dir(&path)
                    .map_err(|_| format!("Failed to list directory: {}", path))?;
                let mut list = Vec::new();
                for entry in entries {
                    if let Ok(entry) = entry {
                        list.push(Value::new_string(entry.path().display().to_string()));
                    }
                }
                Ok(Value::new_array_with(list))
            }

            Builtin::DirCreate => {
                if args.len() != 1 {
                    return Err("Usage: [dir-create path]".to_string());
                }
                let path = self.eval(&args[0])?.as_string()?;
                std::fs::create_dir_all(&path)
                    .map_err(|_| format!("Failed to create directory: {}", path))?;
                Ok(Value::new_void())
            }

            // ------- Shell & System Commands ----------
            Builtin::Shell => {
                if args.len() != 1 {
                    return Err("Usage: [shell command]".to_string());
                }
                let cmd_str = match &args[0].kind {
                    ExprKind::Id(ref id) => {
                        if let Some(v) = self.env.borrow().lookup_var(id) {
                            v.as_string()?
                        } else {
                            id.clone()
                        }
                    }
                    _ => self.eval(&args[0])?.as_string()?,
                };

                let output = if cfg!(windows) {
                    Command::new("cmd").args(["/C", &cmd_str]).output()
                } else {
                    Command::new("sh").args(["-c", &cmd_str]).output()
                }
                .map_err(|_| format!("Failed to execute command: {}", cmd_str))?;

                let res = String::from_utf8_lossy(&output.stdout).into_owned()
                    + &String::from_utf8_lossy(&output.stderr);
                Ok(Value::new_string(res))
            }

            Builtin::ShellStatus => {
                if args.len() != 1 {
                    return Err("Usage: [shell-status command]".to_string());
                }
                let cmd_str = match &args[0].kind {
                    ExprKind::Id(ref id) => {
                        if let Some(v) = self.env.borrow().lookup_var(id) {
                            v.as_string()?
                        } else {
                            id.clone()
                        }
                    }
                    _ => self.eval(&args[0])?.as_string()?,
                };

                let output = if cfg!(windows) {
                    Command::new("cmd").args(["/C", &cmd_str]).status()
                } else {
                    Command::new("sh").args(["-c", &cmd_str]).status()
                }
                .map_err(|_| format!("Failed to execute command: {}", cmd_str))?;

                Ok(Value::new_int(output.code().unwrap_or(-1) as i64))
            }

            // ------- Environment Variables ----------
            Builtin::EnvGet => {
                if args.len() != 1 {
                    return Err("Usage: [env-get name]".to_string());
                }
                let name = self.eval(&args[0])?.as_string()?;
                match std::env::var(&name) {
                    Ok(val) => {
                        if val.is_empty() {
                            Ok(Value::new_void())
                        } else {
                            Ok(Value::new_string(val))
                        }
                    }
                    Err(_) => Ok(Value::new_void()),
                }
            }

            Builtin::EnvSet => {
                if args.len() != 2 {
                    return Err("Usage: [env-set name value]".to_string());
                }
                let name = self.eval(&args[0])?.as_string()?;
                let val = self.eval(&args[1])?.as_string()?;
                std::env::set_var(name, val);
                Ok(Value::new_void())
            }

            // ------- HTTP Client ----------
            Builtin::HttpGet => {
                if args.is_empty() {
                    return Err("Usage: [http-get url [headers]]".to_string());
                }
                let url = self.eval(&args[0])?.as_string()?;
                let mut req = ureq::get(&url);

                if args.len() >= 2 {
                    let headers_val = self.eval(&args[1])?;
                    if let Ok(headers_obj) = headers_val.as_object() {
                        for (k, v) in headers_obj.borrow().iter() {
                            req = req.set(k, &v.to_string_repr());
                        }
                    }
                }

                let response = req
                    .call()
                    .map_err(|_| format!("HTTP request failed: {}", url))?;
                let status = response.status() as i64;
                let status_text = format!("{} {}", status, response.status_text());

                // Read headers
                let headers_obj = Value::new_object();
                if let Value::Object(ref ho) = headers_obj {
                    let mut b = ho.borrow_mut();
                    for name in response.headers_names() {
                        if let Some(val) = response.header(&name) {
                            b.insert(name.clone(), Value::new_string(val));
                        }
                    }
                }

                let body = response
                    .into_string()
                    .map_err(|_| format!("HTTP request failed: {}", url))?;

                let res_obj = Value::new_object();
                if let Value::Object(ref ro) = res_obj {
                    let mut b = ro.borrow_mut();
                    b.insert("status".to_string(), Value::new_int(status));
                    b.insert("statusText".to_string(), Value::new_string(status_text));
                    b.insert("body".to_string(), Value::new_string(body));
                    b.insert("headers".to_string(), headers_obj);
                }

                // Also populate 'response' in current environment for script compatibility
                self.env.borrow_mut().set_var("response".to_string(), res_obj.clone());
                Ok(res_obj)
            }

            Builtin::HttpPost => {
                if args.len() < 2 {
                    return Err("Usage: [http-post url body [headers]]".to_string());
                }
                let url = self.eval(&args[0])?.as_string()?;
                let body_str = self.eval(&args[1])?.as_string()?;
                let mut req = ureq::post(&url);

                if args.len() >= 3 {
                    let headers_val = self.eval(&args[2])?;
                    if let Ok(headers_obj) = headers_val.as_object() {
                        for (k, v) in headers_obj.borrow().iter() {
                            req = req.set(k, &v.to_string_repr());
                        }
                    }
                }

                let response = req
                    .send_string(&body_str)
                    .map_err(|_| format!("HTTP POST failed: {}", url))?;
                let status = response.status() as i64;
                let status_text = format!("{} {}", status, response.status_text());
                let resp_body = response
                    .into_string()
                    .map_err(|_| format!("HTTP POST failed: {}", url))?;

                let res_obj = Value::new_object();
                if let Value::Object(ref ro) = res_obj {
                    let mut b = ro.borrow_mut();
                    b.insert("status".to_string(), Value::new_int(status));
                    b.insert("statusText".to_string(), Value::new_string(status_text));
                    b.insert("body".to_string(), Value::new_string(resp_body));
                }

                self.env.borrow_mut().set_var("response".to_string(), res_obj.clone());
                Ok(res_obj)
            }

            // ------- Time Operations ----------
            Builtin::TimeNow => {
                let now = Local::now();
                let res = Value::new_object();
                if let Value::Object(ref ro) = res {
                    let mut b = ro.borrow_mut();
                    b.insert("year".to_string(), Value::new_int(now.year() as i64));
                    b.insert("month".to_string(), Value::new_int(now.month() as i64));
                    b.insert("day".to_string(), Value::new_int(now.day() as i64));
                    b.insert("hour".to_string(), Value::new_int(now.hour() as i64));
                    b.insert("minute".to_string(), Value::new_int(now.minute() as i64));
                    b.insert("second".to_string(), Value::new_int(now.second() as i64));
                    b.insert(
                        "weekday".to_string(),
                        Value::new_int(now.weekday().num_days_from_sunday() as i64),
                    );
                    b.insert("unix".to_string(), Value::new_int(now.timestamp()));
                }
                Ok(res)
            }

            Builtin::Sleep => {
                if args.len() != 1 {
                    return Err("Usage: [sleep milliseconds]".to_string());
                }
                let ms = self.eval(&args[0])?.as_float()?;
                std::thread::sleep(Duration::from_millis(ms as u64));
                Ok(Value::new_void())
            }

            // ------- String Operations ----------
            Builtin::StrJoin => {
                if args.len() != 2 {
                    return Err("Usage: [str-join separator list]".to_string());
                }
                let sep = self.eval(&args[0])?.as_string()?;
                let list = self.eval(&args[1])?.as_array()?;
                let parts: Vec<String> = list
                    .borrow()
                    .iter()
                    .map(|v| v.to_string_repr())
                    .collect();
                Ok(Value::new_string(parts.join(&sep)))
            }

            Builtin::StrSplit => {
                if args.len() != 2 {
                    return Err("Usage: [str-split string separator]".to_string());
                }
                let s = self.eval(&args[0])?.as_string()?;
                let sep = self.eval(&args[1])?.as_string()?;
                let parts: Vec<Value> = s
                    .split(&sep)
                    .map(|p| Value::new_string(p.to_string()))
                    .collect();
                Ok(Value::new_array_with(parts))
            }

            Builtin::StrLength => {
                if args.len() != 1 {
                    return Err("Usage: [str-length string]".to_string());
                }
                let s = self.eval(&args[0])?.as_string()?;
                Ok(Value::new_int(s.chars().count() as i64))
            }

            // ------- Math Operations ----------
            Builtin::Add => {
                if args.len() != 2 {
                    return Err("Operator add requires exactly 2 arguments".to_string());
                }
                let a = self.eval(&args[0])?;
                let b = self.eval(&args[1])?;
                if a.is_int() && b.is_int() {
                    Ok(Value::new_int(a.as_int()? + b.as_int()?))
                } else {
                    Ok(Value::new_float(a.as_float()? + b.as_float()?))
                }
            }

            Builtin::Sub => {
                if args.len() != 2 {
                    return Err("Operator sub requires exactly 2 arguments".to_string());
                }
                let a = self.eval(&args[0])?;
                let b = self.eval(&args[1])?;
                if a.is_int() && b.is_int() {
                    Ok(Value::new_int(a.as_int()? - b.as_int()?))
                } else {
                    Ok(Value::new_float(a.as_float()? - b.as_float()?))
                }
            }

            Builtin::Mult => {
                if args.len() != 2 {
                    return Err("Operator mult requires exactly 2 arguments".to_string());
                }
                let a = self.eval(&args[0])?;
                let b = self.eval(&args[1])?;
                if a.is_int() && b.is_int() {
                    Ok(Value::new_int(a.as_int()? * b.as_int()?))
                } else {
                    Ok(Value::new_float(a.as_float()? * b.as_float()?))
                }
            }

            Builtin::Div => {
                if args.len() != 2 {
                    return Err("Operator div requires exactly 2 arguments".to_string());
                }
                let a = self.eval(&args[0])?.as_float()?;
                let b = self.eval(&args[1])?.as_float()?;
                if b == 0.0 {
                    return Err("Division by zero".to_string());
                }
                Ok(Value::new_float(a / b))
            }

            Builtin::Rem => {
                if args.len() != 2 {
                    return Err("Operator rem requires exactly 2 arguments".to_string());
                }
                let a = self.eval(&args[0])?;
                let b = self.eval(&args[1])?;
                if a.is_int() && b.is_int() {
                    let b_int = b.as_int()?;
                    if b_int == 0 {
                        return Err("Modulo by zero".to_string());
                    }
                    Ok(Value::new_int(a.as_int()? % b_int))
                } else {
                    let b_float = b.as_float()?;
                    if b_float == 0.0 {
                        return Err("Modulo by zero".to_string());
                    }
                    Ok(Value::new_float(a.as_float()? % b_float))
                }
            }

            Builtin::Pow => {
                if args.len() != 2 {
                    return Err("Operator pow requires exactly 2 arguments".to_string());
                }
                let a = self.eval(&args[0])?.as_float()?;
                let b = self.eval(&args[1])?.as_float()?;
                Ok(Value::new_float(a.powf(b)))
            }

            Builtin::Floor => {
                if args.len() != 1 {
                    return Err("floor requires exactly 1 argument".to_string());
                }
                let a = self.eval(&args[0])?.as_float()?;
                Ok(Value::new_int(a.floor() as i64))
            }

            Builtin::Ceil => {
                if args.len() != 1 {
                    return Err("ceil requires exactly 1 argument".to_string());
                }
                let a = self.eval(&args[0])?.as_float()?;
                Ok(Value::new_int(a.ceil() as i64))
            }

            Builtin::Round => {
                if args.len() != 1 {
                    return Err("round requires exactly 1 argument".to_string());
                }
                let a = self.eval(&args[0])?.as_float()?;
                Ok(Value::new_int(a.round() as i64))
            }

            Builtin::Sqrt => {
                if args.len() != 1 {
                    return Err("sqrt requires exactly 1 argument".to_string());
                }
                let a = self.eval(&args[0])?.as_float()?;
                if a < 0.0 {
                    return Err("Cannot take square root of negative number".to_string());
                }
                Ok(Value::new_float(a.sqrt()))
            }

            // ------- Comparison Operations ----------
            Builtin::Lt => {
                if args.len() != 2 {
                    return Err("Operator lt requires exactly 2 arguments".to_string());
                }
                let a = self.eval(&args[0])?.as_float()?;
                let b = self.eval(&args[1])?.as_float()?;
                Ok(Value::new_bool(a < b))
            }

            Builtin::Gt => {
                if args.len() != 2 {
                    return Err("Operator gt requires exactly 2 arguments".to_string());
                }
                let a = self.eval(&args[0])?.as_float()?;
                let b = self.eval(&args[1])?.as_float()?;
                Ok(Value::new_bool(a > b))
            }

            Builtin::Le => {
                if args.len() != 2 {
                    return Err("Operator le requires exactly 2 arguments".to_string());
                }
                let a = self.eval(&args[0])?.as_float()?;
                let b = self.eval(&args[1])?.as_float()?;
                Ok(Value::new_bool(a <= b))
            }

            Builtin::Ge => {
                if args.len() != 2 {
                    return Err("Operator ge requires exactly 2 arguments".to_string());
                }
                let a = self.eval(&args[0])?.as_float()?;
                let b = self.eval(&args[1])?.as_float()?;
                Ok(Value::new_bool(a >= b))
            }

            Builtin::Eq => {
                if args.len() != 2 {
                    return Err("Operator eq requires exactly 2 arguments".to_string());
                }
                let a = self.eval(&args[0])?;
                let b = self.eval(&args[1])?;
                Ok(Value::new_bool(a == b))
            }

            Builtin::Neq => {
                if args.len() != 2 {
                    return Err("Operator neq requires exactly 2 arguments".to_string());
                }
                let a = self.eval(&args[0])?;
                let b = self.eval(&args[1])?;
                Ok(Value::new_bool(a != b))
            }

            // ------- Logical Operations ----------
            Builtin::And => {
                if args.len() != 2 {
                    return Err("and requires exactly 2 arguments".to_string());
                }
                let a = self.eval(&args[0])?.as_bool()?;
                let b = self.eval(&args[1])?.as_bool()?;
                Ok(Value::new_bool(a && b))
            }

            Builtin::Or => {
                if args.len() != 2 {
                    return Err("or requires exactly 2 arguments".to_string());
                }
                let a = self.eval(&args[0])?.as_bool()?;
                let b = self.eval(&args[1])?.as_bool()?;
                Ok(Value::new_bool(a || b))
            }

            Builtin::Not => {
                if args.len() != 1 {
                    return Err("not requires exactly 1 argument".to_string());
                }
                let a = self.eval(&args[0])?.as_bool()?;
                Ok(Value::new_bool(!a))
            }

            // ------- Array Operations ----------
            Builtin::ArrayPush => {
                if args.len() != 2 {
                    return Err("Usage: [array-push array value]".to_string());
                }
                let arr = self.eval(&args[0])?.as_array()?;
                let val = self.eval(&args[1])?;
                arr.borrow_mut().push(val.clone());
                Ok(val)
            }

            Builtin::ArrayPop => {
                if args.len() != 1 {
                    return Err("Usage: [array-pop array]".to_string());
                }
                let arr = self.eval(&args[0])?.as_array()?;
                let mut b = arr.borrow_mut();
                if b.is_empty() {
                    return Err("Cannot pop from empty array".to_string());
                }
                Ok(b.pop().unwrap())
            }

            Builtin::ArrayGet => {
                if args.len() != 2 {
                    return Err("Usage: [array-get array index]".to_string());
                }
                let arr = self.eval(&args[0])?.as_array()?;
                let idx = self.eval(&args[1])?.as_int()?;
                let b = arr.borrow();
                if idx < 0 || idx as usize >= b.len() {
                    return Err(format!("Array index out of bounds: {}", idx));
                }
                Ok(b[idx as usize].clone())
            }

            Builtin::ArraySet => {
                if args.len() != 3 {
                    return Err("Usage: [array-set array index value]".to_string());
                }
                let arr = self.eval(&args[0])?.as_array()?;
                let idx = self.eval(&args[1])?.as_int()?;
                let val = self.eval(&args[2])?;
                let mut b = arr.borrow_mut();
                if idx < 0 || idx as usize >= b.len() {
                    return Err(format!("Array index out of bounds: {}", idx));
                }
                b[idx as usize] = val.clone();
                Ok(val)
            }

            Builtin::ArrayLength => {
                if args.len() != 1 {
                    return Err("Usage: [array-length array]".to_string());
                }
                let arr = self.eval(&args[0])?.as_array()?;
                let len = arr.borrow().len();
                Ok(Value::new_int(len as i64))
            }

            Builtin::ObjSet => {
                if args.len() != 3 {
                    return Err("Usage: [obj-set object key value]".to_string());
                }
                let target = self.eval(&args[0])?;
                let (obj, needs_rebind) = match target {
                    Value::Object(o) => (o, false),
                    Value::Array(a) if a.borrow().is_empty() => {
                        use indexmap::IndexMap;
                        use rustc_hash::FxBuildHasher;
                        let new_obj = Rc::new(RefCell::new(IndexMap::with_hasher(
                            FxBuildHasher::default(),
                        )));
                        (new_obj, true)
                    }
                    _ => return Err(format!("Expected object, got {}", target.type_name())),
                };
                let key = self.eval(&args[1])?.as_string()?;
                let val = self.eval(&args[2])?;
                obj.borrow_mut().insert(key, val.clone());
                if needs_rebind {
                    if let ExprKind::Id(ref var_name) = args[0].kind {
                        self.env
                            .borrow_mut()
                            .set_var(var_name.clone(), Value::Object(Rc::clone(&obj)));
                    }
                }
                Ok(val)
            }

            Builtin::ObjGet => {
                if args.len() != 2 {
                    return Err("Usage: [obj-get object key]".to_string());
                }
                let target = self.eval(&args[0])?;
                if let Value::String(ref s) = target {
                    return Ok(Value::new_string(s.clone()));
                }
                let obj = target.as_object()?;
                let key = self.eval(&args[1])?.as_string()?;
                let b = obj.borrow();
                if let Some(val) = b.get(&key) {
                    Ok(val.clone())
                } else {
                    Err(format!("Key not found in object: {}", key))
                }
            }

            Builtin::ObjKeys => {
                if args.len() != 1 {
                    return Err("Usage: [obj-keys object]".to_string());
                }
                let obj = self.eval(&args[0])?.as_object()?;
                let keys: Vec<Value> = obj
                    .borrow()
                    .keys()
                    .map(|k| Value::new_string(k.clone()))
                    .collect();
                Ok(Value::new_array_with(keys))
            }

            // ------- JSON Operations ----------
            Builtin::JsonParse => {
                if args.len() != 1 {
                    return Err("Usage: [json-parse string]".to_string());
                }
                let s = self.eval(&args[0])?.as_string()?;
                let json: serde_json::Value = serde_json::from_str(&s)
                    .map_err(|_| "Failed to parse JSON".to_string())?;
                Ok(Value::from_json(&json))
            }

            Builtin::JsonStringify => {
                if args.len() != 1 {
                    return Err("Usage: [json-stringify value]".to_string());
                }
                let val = self.eval(&args[0])?;
                let json = val.to_json();
                let s = serde_json::to_string(&json)
                    .map_err(|e| format!("JSON serialization error: {}", e))?;
                Ok(Value::new_string(s))
            }

            // ------- Error Handling ----------
            Builtin::Try => {
                if args.is_empty() {
                    return Err("Usage: [try body [catch handler]]".to_string());
                }

                match self.eval(&args[0]) {
                    Ok(val) => Ok(val),
                    Err(err_msg) => {
                        if args.len() >= 2 {
                            let catch_expr = &args[1];
                            if let ExprKind::List(children) = &catch_expr.kind {
                                if !children.is_empty() {
                                    if let ExprKind::Id(ref s) = children[0].kind {
                                        if s == "catch" && children.len() >= 3 {
                                            if let ExprKind::Id(ref err_var) = children[1].kind {
                                                self.env.borrow_mut().set_var(
                                                    err_var.clone(),
                                                    Value::new_string(err_msg),
                                                );
                                                return self.eval_block(&children[2..]);
                                            }
                                        }
                                    }
                                }
                            }
                            self.eval(catch_expr)
                        } else {
                            Ok(Value::new_void())
                        }
                    }
                }
            }

            // ------- Type Checks ----------
            Builtin::IsInt => {
                if args.len() != 1 {
                    return Err("isInt requires 1 argument".to_string());
                }
                let val = self.eval(&args[0])?;
                Ok(Value::new_bool(val.is_int()))
            }
            Builtin::IsFloat => {
                if args.len() != 1 {
                    return Err("isFloat requires 1 argument".to_string());
                }
                let val = self.eval(&args[0])?;
                Ok(Value::new_bool(val.is_float()))
            }
            Builtin::IsNumber => {
                if args.len() != 1 {
                    return Err("isNumber requires 1 argument".to_string());
                }
                let val = self.eval(&args[0])?;
                Ok(Value::new_bool(val.is_number()))
            }
            Builtin::IsString => {
                if args.len() != 1 {
                    return Err("isString requires 1 argument".to_string());
                }
                let val = self.eval(&args[0])?;
                Ok(Value::new_bool(val.is_string()))
            }
            Builtin::IsBool => {
                if args.len() != 1 {
                    return Err("isBool requires 1 argument".to_string());
                }
                let val = self.eval(&args[0])?;
                Ok(Value::new_bool(val.is_bool()))
            }
            Builtin::IsArray => {
                if args.len() != 1 {
                    return Err("isArray requires 1 argument".to_string());
                }
                let val = self.eval(&args[0])?;
                Ok(Value::new_bool(val.is_array()))
            }
            Builtin::IsObject => {
                if args.len() != 1 {
                    return Err("isObject requires 1 argument".to_string());
                }
                let val = self.eval(&args[0])?;
                Ok(Value::new_bool(val.is_object()))
            }
            Builtin::IsVoid => {
                if args.len() != 1 {
                    return Err("isVoid requires 1 argument".to_string());
                }
                let val = self.eval(&args[0])?;
                Ok(Value::new_bool(val.is_void()))
            }
        }
    }
}
