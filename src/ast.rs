#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Id(String),
    List(Vec<Expr>),
    Array(Vec<Expr>),
    Object(Vec<(Expr, Expr)>),
    Comment(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub line: usize,
    pub col: usize,
}

impl Expr {
    #[inline(always)]
    pub fn new(kind: ExprKind, line: usize, col: usize) -> Self {
        Self { kind, line, col }
    }
}

pub fn dump_ast(expr: &Expr, indent: usize) {
    let space = "  ".repeat(indent);
    match &expr.kind {
        ExprKind::Int(v) => println!("{}Int: {}", space, v),
        ExprKind::Float(v) => println!("{}Float: {}", space, v),
        ExprKind::String(v) => println!("{}Str: {}", space, v),
        ExprKind::Bool(v) => println!("{}Bool: {}", space, v),
        ExprKind::Id(v) => println!("{}Id: {}", space, v),
        ExprKind::List(children) => {
            println!("{}List:", space);
            for child in children {
                dump_ast(child, indent + 1);
            }
        }
        ExprKind::Array(children) => {
            println!("{}Array:", space);
            for child in children {
                dump_ast(child, indent + 1);
            }
        }
        ExprKind::Object(pairs) => {
            println!("{}Object:", space);
            for (k, v) in pairs {
                dump_ast(k, indent + 1);
                dump_ast(v, indent + 2);
            }
        }
        ExprKind::Comment(v) => println!("{}Comment: {}", space, v),
    }
}
