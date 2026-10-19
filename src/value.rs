use indexmap::IndexMap;
use rustc_hash::FxBuildHasher;
use serde_json::Value as JsonValue;
use std::cell::RefCell;
use std::rc::Rc;

pub type ObjectMap = IndexMap<String, Value, FxBuildHasher>;

#[derive(Clone, Debug)]
pub enum Value {
    Void,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(Rc<RefCell<Vec<Value>>>),
    Object(Rc<RefCell<ObjectMap>>),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Void, Value::Void) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Int(a), Value::Float(b)) => (*a as f64) == *b,
            (Value::Float(a), Value::Int(b)) => *a == (*b as f64),
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => *a.borrow() == *b.borrow(),
            (Value::Object(a), Value::Object(b)) => *a.borrow() == *b.borrow(),
            _ => false,
        }
    }
}

impl Value {
    #[inline(always)]
    pub fn new_int(v: i64) -> Self {
        Value::Int(v)
    }

    #[inline(always)]
    pub fn new_float(v: f64) -> Self {
        Value::Float(v)
    }

    #[inline(always)]
    pub fn new_string<S: Into<String>>(v: S) -> Self {
        Value::String(v.into())
    }

    #[inline(always)]
    pub fn new_bool(v: bool) -> Self {
        Value::Bool(v)
    }

    #[inline(always)]
    pub fn new_void() -> Self {
        Value::Void
    }

    #[inline(always)]
    pub fn new_array() -> Self {
        Value::Array(Rc::new(RefCell::new(Vec::new())))
    }

    #[inline(always)]
    pub fn new_array_with(elements: Vec<Value>) -> Self {
        Value::Array(Rc::new(RefCell::new(elements)))
    }

    #[inline(always)]
    pub fn new_object() -> Self {
        Value::Object(Rc::new(RefCell::new(IndexMap::with_hasher(
            FxBuildHasher::default(),
        ))))
    }

    #[inline(always)]
    pub fn is_int(&self) -> bool {
        matches!(self, Value::Int(_))
    }

    #[inline(always)]
    pub fn is_float(&self) -> bool {
        matches!(self, Value::Float(_))
    }

    #[inline(always)]
    pub fn is_number(&self) -> bool {
        matches!(self, Value::Int(_) | Value::Float(_))
    }

    #[inline(always)]
    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    #[inline(always)]
    pub fn is_bool(&self) -> bool {
        matches!(self, Value::Bool(_))
    }

    #[inline(always)]
    pub fn is_void(&self) -> bool {
        matches!(self, Value::Void)
    }

    #[inline(always)]
    pub fn is_array(&self) -> bool {
        matches!(self, Value::Array(_))
    }

    #[inline(always)]
    pub fn is_object(&self) -> bool {
        matches!(self, Value::Object(_))
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Void => "void",
            Value::Bool(_) => "bool",
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
        }
    }

    pub fn as_int(&self) -> Result<i64, String> {
        match self {
            Value::Int(v) => Ok(*v),
            _ => Err(format!("Expected int, got {}", self.type_name())),
        }
    }

    pub fn as_float(&self) -> Result<f64, String> {
        match self {
            Value::Int(v) => Ok(*v as f64),
            Value::Float(v) => Ok(*v),
            _ => Err(format!("Expected number, got {}", self.type_name())),
        }
    }

    pub fn as_string(&self) -> Result<String, String> {
        match self {
            Value::String(s) => Ok(s.clone()),
            _ => Err(format!("Expected string, got {}", self.type_name())),
        }
    }

    pub fn as_bool(&self) -> Result<bool, String> {
        match self {
            Value::Bool(b) => Ok(*b),
            _ => Err(format!("Expected bool, got {}", self.type_name())),
        }
    }

    pub fn as_array(&self) -> Result<Rc<RefCell<Vec<Value>>>, String> {
        match self {
            Value::Array(a) => Ok(Rc::clone(a)),
            _ => Err(format!("Expected array, got {}", self.type_name())),
        }
    }

    pub fn as_object(&self) -> Result<Rc<RefCell<ObjectMap>>, String> {
        match self {
            Value::Object(o) => Ok(Rc::clone(o)),
            // Compatibility: allow empty array to act as empty object if used with object functions
            Value::Array(a) if a.borrow().is_empty() => {
                let o = Rc::new(RefCell::new(IndexMap::with_hasher(
                    FxBuildHasher::default(),
                )));
                Ok(o)
            }
            _ => Err(format!("Expected object, got {}", self.type_name())),
        }
    }

    pub fn to_string_repr(&self) -> String {
        match self {
            Value::Int(v) => v.to_string(),
            Value::Float(v) => {
                if v.fract() == 0.0 && !v.is_infinite() && !v.is_nan() {
                    // For whole numbers formatted in Nim, floats often display as "3.0" or similar
                    format!("{:.1}", v)
                } else {
                    v.to_string()
                }
            }
            Value::String(s) => s.clone(),
            Value::Bool(b) => b.to_string(),
            Value::Void => "void".to_string(),
            Value::Array(a) => format!("array[{}]", a.borrow().len()),
            Value::Object(_) => "object".to_string(),
        }
    }

    pub fn check_type(&self, expected: &str) -> bool {
        match expected {
            "int" => self.is_int(),
            "float" => self.is_float(),
            "number" => self.is_number(),
            "str" | "string" => self.is_string(),
            "bool" => self.is_bool(),
            "array" => self.is_array(),
            "object" => self.is_object(),
            "void" => self.is_void(),
            _ => true,
        }
    }

    pub fn to_json(&self) -> JsonValue {
        match self {
            Value::Void => JsonValue::Null,
            Value::Bool(b) => JsonValue::Bool(*b),
            Value::Int(i) => JsonValue::Number((*i).into()),
            Value::Float(f) => {
                if let Some(n) = serde_json::Number::from_f64(*f) {
                    JsonValue::Number(n)
                } else {
                    JsonValue::Null
                }
            }
            Value::String(s) => JsonValue::String(s.clone()),
            Value::Array(arr) => {
                let list: Vec<JsonValue> = arr.borrow().iter().map(|item| item.to_json()).collect();
                JsonValue::Array(list)
            }
            Value::Object(obj) => {
                let mut map = serde_json::Map::new();
                for (k, v) in obj.borrow().iter() {
                    map.insert(k.clone(), v.to_json());
                }
                JsonValue::Object(map)
            }
        }
    }

    pub fn from_json(json: &JsonValue) -> Self {
        match json {
            JsonValue::Null => Value::new_void(),
            JsonValue::Bool(b) => Value::new_bool(*b),
            JsonValue::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Value::new_int(i)
                } else if let Some(f) = n.as_f64() {
                    Value::new_float(f)
                } else {
                    Value::new_float(0.0)
                }
            }
            JsonValue::String(s) => Value::new_string(s.clone()),
            JsonValue::Array(arr) => {
                let values = arr.iter().map(Value::from_json).collect();
                Value::new_array_with(values)
            }
            JsonValue::Object(map) => {
                let obj = Value::new_object();
                if let Value::Object(ref o) = obj {
                    let mut b = o.borrow_mut();
                    for (k, v) in map {
                        b.insert(k.clone(), Value::from_json(v));
                    }
                }
                obj
            }
        }
    }
}
