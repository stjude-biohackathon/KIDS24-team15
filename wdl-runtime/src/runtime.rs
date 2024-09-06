//! Implementation of the WDL runtime and values.

use std::collections::HashMap;
use std::fmt;

use id_arena::{Arena, Id};
use ordered_float::OrderedFloat;
use string_interner::{symbol::SymbolU32, DefaultStringInterner};
use wdl_analysis::{
    diagnostics::unknown_type,
    scope::DocumentScope,
    types::{ArrayType, MapType, PairType, PrimitiveTypeKind, Type, TypeEq, Types},
};
use wdl_ast::{AstToken, Diagnostic, Ident};

/// Represents a WDL runtime value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Value {
    /// The value is a `Boolean`.
    Boolean(bool),
    /// The value is an `Int`.
    Integer(i64),
    /// The value is a `Float`.
    Float(OrderedFloat<f64>),
    /// The value is a `String`.
    String(SymbolU32),
    /// The value is a `File`.
    File(SymbolU32),
    /// The value is a `Directory`.
    Directory(SymbolU32),
    /// The value is a literal `None` value.
    None,
    /// The value is stored in the runtime.
    Stored(Type, StoredValueId),
}

impl Value {
    /// Gets the type of the value.
    pub fn ty(&self) -> Type {
        match self {
            Value::Boolean(_) => PrimitiveTypeKind::Boolean.into(),
            Value::Integer(_) => PrimitiveTypeKind::Integer.into(),
            Value::Float(_) => PrimitiveTypeKind::Float.into(),
            Value::String(_) => PrimitiveTypeKind::String.into(),
            Value::File(_) => PrimitiveTypeKind::File.into(),
            Value::Directory(_) => PrimitiveTypeKind::Directory.into(),
            Value::None => Type::None,
            Value::Stored(ty, _) => *ty,
        }
    }

    /// Unwraps the value into a boolean.
    ///
    /// # Panics
    ///
    /// Panics if the value is not a boolean.
    pub fn unwrap_boolean(self) -> bool {
        match self {
            Self::Boolean(v) => v,
            _ => panic!("value is not a boolean"),
        }
    }

    /// Unwraps the value into an integer.
    ///
    /// # Panics
    ///
    /// Panics if the value is not an integer.
    pub fn unwrap_integer(self) -> i64 {
        match self {
            Self::Integer(v) => v,
            _ => panic!("value is not an integer"),
        }
    }

    /// Unwraps the value into a float.
    ///
    /// # Panics
    ///
    /// Panics if the value is not a float.
    pub fn unwrap_float(self) -> f64 {
        match self {
            Self::Float(v) => v.into(),
            _ => panic!("value is not a float"),
        }
    }

    /// Unwraps the value into a string.
    ///
    /// # Panics
    ///
    /// Panics if the value is not a string.
    pub fn unwrap_string<'a>(self, runtime: &'a Runtime<'_>) -> &'a str {
        match self {
            Self::String(sym) => runtime.resolve_str(sym),
            _ => panic!("value is not a string"),
        }
    }

    /// Unwraps the value into a file.
    ///
    /// # Panics
    ///
    /// Panics if the value is not a file.
    pub fn unwrap_file<'a>(self, runtime: &'a Runtime<'_>) -> &'a str {
        match self {
            Self::File(sym) => runtime.resolve_str(sym),
            _ => panic!("value is not a file"),
        }
    }

    /// Unwraps the value into a directory.
    ///
    /// # Panics
    ///
    /// Panics if the value is not a directory.
    pub fn unwrap_directory<'a>(self, runtime: &'a Runtime<'_>) -> &'a str {
        match self {
            Self::Directory(sym) => runtime.resolve_str(sym),
            _ => panic!("value is not a directory"),
        }
    }

    /// Coerces the value into the given type.
    ///
    /// Returns `None` if the coercion is not supported.
    pub fn coerce(&self, runtime: &mut Runtime<'_>, ty: Type) -> Option<Self> {
        if self.ty().type_eq(runtime.types(), &ty) {
            return Some(*self);
        }

        match (self, ty) {
            (Value::String(sym), ty) => {
                if let Some(ty) = ty.as_primitive() {
                    match ty.kind() {
                        PrimitiveTypeKind::File => Some(Self::File(*sym)),
                        PrimitiveTypeKind::Directory => Some(Self::Directory(*sym)),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            (Value::Integer(v), ty) => {
                if let Some(ty) = ty.as_primitive() {
                    match ty.kind() {
                        PrimitiveTypeKind::Float => Some(Self::Float((*v as f64).into())),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            _ => todo!("implement the remainder coercions"),
        }
    }

    /// Used to display the value.
    pub fn display<'a>(&'a self, runtime: &'a Runtime<'_>) -> impl fmt::Display + 'a {
        /// Helper type for implementing display.
        struct Display<'a> {
            /// A reference to the runtime.
            runtime: &'a Runtime<'a>,
            /// The value to display.
            value: Value,
        }

        impl fmt::Display for Display<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self.value {
                    Value::Boolean(v) => write!(f, "{v}"),
                    Value::Integer(v) => write!(f, "{v}"),
                    Value::Float(v) => write!(f, "{v}"),
                    Value::String(sym) | Value::File(sym) | Value::Directory(sym) => {
                        write!(f, "{v}", v = self.runtime.resolve_str(sym))
                    }
                    Value::None => write!(f, "None"),
                    Value::Stored(_, _) => todo!("implement display of compound types"),
                }
            }
        }

        Display {
            runtime,
            value: *self,
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Float(value.into())
    }
}

/// Represents a value stored in the runtime.
#[derive(Debug)]
pub enum StoredValue {
    /// The value is a `Pair` of values.
    Pair(Value, Value),
    /// The value is an `Array` of values.
    Array(Vec<Value>),
    /// The value is a `Map` of values.
    Map(HashMap<Value, Value>),
    /// The value is an `Object.`
    Object(HashMap<String, Value>),
    /// The value is a struct.
    Struct(Vec<Value>),
}

/// Represents an identifier of a stored value.
pub type StoredValueId = Id<StoredValue>;

/// Represents a WDL runtime.
///
/// The runtime is responsible for storing complex value types and interning strings.
#[derive(Debug)]
pub struct Runtime<'a> {
    /// The reference to the document scope being evaluated.
    document: &'a DocumentScope,
    /// The types collection for values.
    types: Types,
    /// The storage arena for values.
    values: Arena<StoredValue>,
    /// The string interner used to intern string/file/directory values.
    interner: DefaultStringInterner,
    /// The map of known structs to already imported types.
    structs: HashMap<String, Type>,
}

impl<'a> Runtime<'a> {
    /// Constructs a new runtime for the given document being evaluated.
    pub fn new(document: &'a DocumentScope) -> Self {
        Self {
            document,
            types: Types::default(),
            values: Arena::default(),
            interner: DefaultStringInterner::default(),
            structs: HashMap::default(),
        }
    }

    /// Gets the document associated with the runtime.
    pub fn document(&self) -> &DocumentScope {
        self.document
    }

    /// Gets the types collection associated with the runtime.
    pub fn types(&self) -> &Types {
        &self.types
    }

    /// Gets the mutable types collection associated with the runtime.
    pub fn types_mut(&mut self) -> &mut Types {
        &mut self.types
    }

    /// Creates a new `String` value.
    pub fn new_string(&mut self, s: impl AsRef<str>) -> Value {
        Value::String(self.interner.get_or_intern(s))
    }

    /// Creates a new `File` value.
    pub fn new_file(&mut self, s: impl AsRef<str>) -> Value {
        Value::File(self.interner.get_or_intern(s))
    }

    /// Creates a new `Directory` value.
    pub fn new_directory(&mut self, s: impl AsRef<str>) -> Value {
        Value::Directory(self.interner.get_or_intern(s))
    }

    /// Creates a new `Pair` value.
    pub fn new_pair(&mut self, left: Value, right: Value) -> Value {
        let id = self.values.alloc(StoredValue::Pair(left, right));
        let ty = self.types.add_pair(PairType::new(left.ty(), right.ty()));
        Value::Stored(ty, id)
    }

    /// Creates a new `Array` value.
    ///
    /// Note that this expects that the array elements are homogenous.
    pub fn new_array(&mut self, elements: Vec<Value>) -> Value {
        let element_type = elements.first().map(Value::ty).unwrap_or(Type::Union);
        let id = self.values.alloc(StoredValue::Array(elements));
        let ty = self.types.add_array(ArrayType::new(element_type));
        Value::Stored(ty, id)
    }

    /// Creates a new `Map` value.
    ///
    /// Note that this expects the item keys and values to be homogenous, respectively.
    pub fn new_map(&mut self, items: HashMap<Value, Value>) -> Value {
        let mut iter = items.iter().map(|(k, v)| (k.ty(), v.ty()));
        let (key_type, value_type) = iter.next().unwrap_or((Type::Union, Type::Union));
        let id = self.values.alloc(StoredValue::Map(items));
        let ty = self.types.add_map(MapType::new(key_type, value_type));
        Value::Stored(ty, id)
    }

    /// Creates a new `Object` value.
    pub fn new_object(&mut self, items: HashMap<String, Value>) -> Value {
        let id = self.values.alloc(StoredValue::Object(items));
        Value::Stored(Type::Object, id)
    }

    /// Creates a new struct value.
    pub fn new_struct(&mut self, name: &Ident, members: Vec<Value>) -> Result<Value, Diagnostic> {
        // Import the struct type from the document scope if needed
        let ty = if let Some(ty) = self.structs.get(name.as_str()) {
            *ty
        } else {
            let ty = self
                .document
                .struct_by_name(name.as_str())
                .and_then(|s| s.ty())
                .ok_or_else(|| unknown_type(name.as_str(), name.span()))?;
            let ty = self.types.import(self.document.types(), ty);
            self.structs.insert(name.as_str().to_string(), ty);
            ty
        };

        let id = self.values.alloc(StoredValue::Struct(members));
        Ok(Value::Stored(ty, id))
    }

    /// Resolves a previously interned string from a symbol.
    pub fn resolve_str(&self, sym: SymbolU32) -> &str {
        self.interner.resolve(sym).expect("should have symbol")
    }

    /// Imports a type from the document types collection.
    pub(crate) fn import_type(&mut self, ty: Type) -> Type {
        self.types.import(self.document.types(), ty)
    }
}
