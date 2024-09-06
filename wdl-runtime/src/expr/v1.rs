//! Implementation of an expression evaluator for 1.x WDL documents.

use std::collections::HashMap;

use wdl_analysis::diagnostics::{
    ambiguous_argument, argument_type_mismatch, cannot_coerce_to_string, map_key_not_primitive,
    too_few_arguments, too_many_arguments, type_mismatch, unknown_function, unknown_name,
    unsupported_function,
};
use wdl_analysis::stdlib::FunctionBindError;
use wdl_analysis::types::{Coercible, Type};
use wdl_ast::v1::{
    CallExpr, Expr, LiteralArray, LiteralExpr, LiteralMap, LiteralObject, LiteralPair,
    LiteralStringKind, LiteralStruct, Placeholder, StringPart,
};
use wdl_ast::{AstNode, AstNodeExt, AstToken, Diagnostic, Ident, Span, SyntaxKind, TokenStrHash};

use crate::util::strip_leading_whitespace;
use crate::{read_string, Runtime, Value};
use std::fmt::Write;

/// Creates an "integer not in range" diagnostic
fn integer_not_in_range(span: Span) -> Diagnostic {
    Diagnostic::error(format!(
        "literal integer exceeds the range for a 64-bit signed integer ({min}..={max})",
        min = i64::MIN,
        max = i64::MAX,
    ))
    .with_label("this literal integer is not in range", span)
}

/// Creates a "float not in range" diagnostic
fn float_not_in_range(span: Span) -> Diagnostic {
    Diagnostic::error(format!(
        "literal float exceeds the range for a 64-bit float ({min:+e}..={max:+e})",
        min = f64::MIN,
        max = f64::MAX,
    ))
    .with_label("this literal float is not in range", span)
}

/// Creates a "cannot call" diagnostic.
fn cannot_call(target: &Ident) -> Diagnostic {
    Diagnostic::error(format!(
        "function `{target}` can only be called from task outputs",
        target = target.as_str()
    ))
    .with_highlight(target.span())
}

/// Creates a "call failed" diagnostic.
fn call_failed(target: &Ident, error: &anyhow::Error) -> Diagnostic {
    Diagnostic::error(format!(
        "function `{target}` failed: {error:#}",
        target = target.as_str()
    ))
    .with_highlight(target.span())
}

/// Represents a WDL expression evaluator.
#[derive(Debug)]
pub struct ExprEvaluator<'a> {
    /// The scope to use for the evaluation.
    scope: &'a HashMap<TokenStrHash<Ident>, Value>,
    /// The value to return from a call to `stdout`.
    ///
    /// This is `Some` only when evaluating task outputs.
    stdout: Option<Value>,
    /// The value to return from a call to `stderr`.
    ///
    /// This is `Some` only when evaluating task outputs.
    stderr: Option<Value>,
}

impl<'a> ExprEvaluator<'a> {
    /// Creates a new expression evaluator.
    pub fn new(scope: &'a HashMap<TokenStrHash<Ident>, Value>) -> Self {
        Self {
            scope,
            stdout: None,
            stderr: None,
        }
    }

    /// Creates a new expression evaluator with the given stdout/stderr output.
    pub fn new_with_output(
        scope: &'a HashMap<TokenStrHash<Ident>, Value>,
        stdout: Value,
        stderr: Value,
    ) -> Self {
        Self {
            scope,
            stdout: Some(stdout),
            stderr: Some(stderr),
        }
    }

    /// Evaluates the given expression.
    pub fn evaluate_expr(
        &self,
        runtime: &mut Runtime<'_>,
        expr: &Expr,
    ) -> Result<Value, Diagnostic> {
        match expr {
            Expr::Literal(expr) => self.evaluate_literal_expr(runtime, expr),
            Expr::Name(r) => {
                let name = r.name();
                self.scope
                    .get(name.as_str())
                    .copied()
                    .ok_or_else(|| unknown_name(name.as_str(), name.span()))
            }
            Expr::Parenthesized(expr) => self.evaluate_expr(runtime, &expr.inner()),
            Expr::If(_) => todo!(),
            Expr::LogicalNot(_) => todo!(),
            Expr::Negation(_) => todo!(),
            Expr::LogicalOr(_) => todo!(),
            Expr::LogicalAnd(_) => todo!(),
            Expr::Equality(_) => todo!(),
            Expr::Inequality(_) => todo!(),
            Expr::Less(_) => todo!(),
            Expr::LessEqual(_) => todo!(),
            Expr::Greater(_) => todo!(),
            Expr::GreaterEqual(_) => todo!(),
            Expr::Addition(_) => todo!(),
            Expr::Subtraction(_) => todo!(),
            Expr::Multiplication(_) => todo!(),
            Expr::Division(_) => todo!(),
            Expr::Modulo(_) => todo!(),
            Expr::Exponentiation(_) => todo!(),
            Expr::Call(expr) => self.evaluate_call_expr(runtime, expr),
            Expr::Index(_) => todo!(),
            Expr::Access(_) => todo!(),
        }
    }

    /// Evaluates a literal expression.
    fn evaluate_literal_expr(
        &self,
        runtime: &mut Runtime<'_>,
        expr: &LiteralExpr,
    ) -> Result<Value, Diagnostic> {
        match expr {
            LiteralExpr::Boolean(lit) => Ok(lit.value().into()),
            LiteralExpr::Integer(lit) => {
                let parent = lit
                    .syntax()
                    .parent()
                    .expect("should have a parent expression");

                // Check to see if this literal is a direct child of a negation expression; if so, we want to negate the literal
                let (value, span) = if parent.kind() == SyntaxKind::NegationExprNode {
                    let start = parent.text_range().start().into();
                    (lit.negate(), Span::new(start, lit.span().end() - start))
                } else {
                    (lit.value(), lit.span())
                };

                Ok(value.ok_or_else(|| integer_not_in_range(span))?.into())
            }
            LiteralExpr::Float(lit) => Ok(lit
                .value()
                .ok_or_else(|| float_not_in_range(lit.span()))?
                .into()),
            LiteralExpr::String(lit) => {
                // An optimization if the literal is just text; don't bother building a new string
                if let Some(text) = lit.text() {
                    return Ok(runtime.new_string(text.as_str()));
                }

                let mut s = String::new();
                for p in lit.parts() {
                    match p {
                        StringPart::Text(t) => s.push_str(t.as_str()),
                        StringPart::Placeholder(placeholder) => {
                            self.evaluate_placeholder(runtime, &placeholder, &mut s)?;
                        }
                    }
                }

                if let LiteralStringKind::Multiline = lit.kind() {
                    s = strip_leading_whitespace(&s, false);
                }

                Ok(runtime.new_string(s))
            }
            LiteralExpr::Array(lit) => self.evaluate_literal_array(runtime, lit),
            LiteralExpr::Pair(lit) => self.evaluate_literal_pair(runtime, lit),
            LiteralExpr::Map(lit) => self.evaluate_literal_map(runtime, lit),
            LiteralExpr::Object(lit) => self.evaluate_literal_object(runtime, lit),
            LiteralExpr::Struct(lit) => self.evaluate_literal_struct(runtime, lit),
            LiteralExpr::None(_) => Ok(Value::None),
            LiteralExpr::Hints(_) | LiteralExpr::Input(_) | LiteralExpr::Output(_) => {
                todo!("implement for WDL 1.2 support")
            }
        }
    }

    /// Evaluates a placeholder into the given string buffer.
    pub(crate) fn evaluate_placeholder(
        &self,
        runtime: &mut Runtime<'_>,
        placeholder: &Placeholder,
        buffer: &mut String,
    ) -> Result<(), Diagnostic> {
        let expr = placeholder.expr();
        match self.evaluate_expr(runtime, &expr)? {
            Value::Boolean(v) => buffer.push_str(if v { "true" } else { "false" }),
            Value::Integer(v) => write!(buffer, "{v}").unwrap(),
            Value::Float(v) => write!(buffer, "{v}").unwrap(),
            Value::String(v) | Value::File(v) | Value::Directory(v) => {
                buffer.push_str(runtime.resolve_str(v))
            }
            Value::None => {}
            Value::Stored(ty, _) => {
                return Err(cannot_coerce_to_string(runtime.types(), ty, expr.span()));
            }
        }

        Ok(())
    }

    /// Evaluates a literal array expression.
    fn evaluate_literal_array(
        &self,
        runtime: &mut Runtime<'_>,
        expr: &LiteralArray,
    ) -> Result<Value, Diagnostic> {
        // Look at the first array element to determine the element type
        // The remaining elements must match the first type
        let mut elements = expr.elements();
        match elements.next() {
            Some(expr) => {
                let mut values = Vec::new();
                let value = self.evaluate_expr(runtime, &expr)?;
                let expected = value.ty();
                let expected_span = expr.span();
                values.push(value);

                // Ensure the remaining element types are the same as the first
                for expr in elements {
                    let value = self.evaluate_expr(runtime, &expr)?;
                    let actual = value.ty();
                    values.push(value);

                    if !actual.is_coercible_to(runtime.types(), &expected) {
                        return Err(type_mismatch(
                            runtime.types(),
                            expected,
                            expected_span,
                            actual,
                            expr.span(),
                        ));
                    }
                }

                Ok(runtime.new_array(values))
            }
            None => Ok(runtime.new_array(Vec::new())),
        }
    }

    /// Evaluates a literal pair expression.
    fn evaluate_literal_pair(
        &self,
        runtime: &mut Runtime<'_>,
        expr: &LiteralPair,
    ) -> Result<Value, Diagnostic> {
        let (left, right) = expr.exprs();
        let left = self.evaluate_expr(runtime, &left)?;
        let right = self.evaluate_expr(runtime, &right)?;
        Ok(runtime.new_pair(left, right))
    }

    /// Evaluates a literal map expression.
    fn evaluate_literal_map(
        &self,
        runtime: &mut Runtime<'_>,
        expr: &LiteralMap,
    ) -> Result<Value, Diagnostic> {
        let mut items = expr.items();
        match items.next() {
            Some(item) => {
                let mut elements = HashMap::new();
                let (key, value) = item.key_value();
                let expected_key = self.evaluate_expr(runtime, &key)?;
                let expected_key_span = key.span();
                let expected_value = self.evaluate_expr(runtime, &value)?;
                let expected_value_span = value.span();
                match expected_key.ty() {
                    Type::Primitive(_) => {
                        // OK
                    }
                    ty => {
                        return Err(map_key_not_primitive(runtime.types(), ty, key.span()));
                    }
                }

                elements.insert(expected_key, expected_value);

                // Ensure the remaining items types are the same as the first
                for item in items {
                    let (key, value) = item.key_value();
                    let actual_key = self.evaluate_expr(runtime, &key)?;
                    let actual_value = self.evaluate_expr(runtime, &value)?;

                    if !actual_key
                        .ty()
                        .is_coercible_to(runtime.types(), &expected_key.ty())
                    {
                        return Err(type_mismatch(
                            runtime.types(),
                            expected_key.ty(),
                            expected_key_span,
                            actual_key.ty(),
                            key.span(),
                        ));
                    }

                    if !actual_value
                        .ty()
                        .is_coercible_to(runtime.types(), &expected_value.ty())
                    {
                        return Err(type_mismatch(
                            runtime.types(),
                            expected_value.ty(),
                            expected_value_span,
                            actual_value.ty(),
                            value.span(),
                        ));
                    }

                    elements.insert(actual_key, actual_value);
                }

                Ok(runtime.new_map(elements))
            }
            // Treat as `Map[Union, Union]`
            None => Ok(runtime.new_map(HashMap::new())),
        }
    }

    /// Evaluates a literal object expression.
    fn evaluate_literal_object(
        &self,
        runtime: &mut Runtime<'_>,
        expr: &LiteralObject,
    ) -> Result<Value, Diagnostic> {
        let items = expr
            .items()
            .map(|item| {
                let (name, value) = item.name_value();
                Ok((
                    name.as_str().to_string(),
                    self.evaluate_expr(runtime, &value)?,
                ))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;

        Ok(runtime.new_object(items))
    }

    /// Evaluates a literal struct expression.
    fn evaluate_literal_struct(
        &self,
        _runtime: &mut Runtime<'_>,
        _expr: &LiteralStruct,
    ) -> Result<Value, Diagnostic> {
        todo!()
    }

    /// Evaluates a call expression.
    fn evaluate_call_expr(
        &self,
        runtime: &mut Runtime<'_>,
        expr: &CallExpr,
    ) -> Result<Value, Diagnostic> {
        let target = expr.target();
        match wdl_analysis::stdlib::STDLIB.function(target.as_str()) {
            Some(f) => {
                let minimum_version = f.minimum_version();
                if minimum_version
                    > runtime
                        .document()
                        .version()
                        .expect("document should have a version")
                {
                    return Err(unsupported_function(
                        minimum_version,
                        target.as_str(),
                        target.span(),
                    ));
                }

                let (arguments, types) = expr.arguments().try_fold(
                    (Vec::new(), Vec::new()),
                    |(mut args, mut types), expr| {
                        let value = self.evaluate_expr(runtime, &expr)?;
                        types.push(value.ty());
                        args.push(value);
                        Ok((args, types))
                    },
                )?;

                // TODO: implement a `can_bind` which doesn't mutate the types collection
                match f.bind(runtime.types_mut(), &types) {
                    Ok(_) => {
                        // TODO: dispatch the function call in a better way
                        let r = match target.as_str() {
                            "read_string" => read_string(runtime, &arguments),
                            "stdout" => return self.stdout.ok_or_else(|| cannot_call(&target)),
                            "stderr" => return self.stderr.ok_or_else(|| cannot_call(&target)),
                            _ => unreachable!("unknown function"),
                        };

                        r.map_err(|e| call_failed(&target, &e))
                    }
                    Err(FunctionBindError::TooFewArguments(minimum)) => Err(too_few_arguments(
                        target.as_str(),
                        target.span(),
                        minimum,
                        arguments.len(),
                    )),
                    Err(FunctionBindError::TooManyArguments(maximum)) => Err(too_many_arguments(
                        target.as_str(),
                        target.span(),
                        maximum,
                        arguments.len(),
                        expr.arguments().skip(maximum).map(|e| e.span()),
                    )),
                    Err(FunctionBindError::ArgumentTypeMismatch { index, expected }) => {
                        Err(argument_type_mismatch(
                            runtime.types(),
                            &expected,
                            types[index],
                            expr.arguments()
                                .nth(index)
                                .map(|e| e.span())
                                .expect("should have span"),
                        ))
                    }
                    Err(FunctionBindError::Ambiguous { first, second }) => Err(ambiguous_argument(
                        target.as_str(),
                        target.span(),
                        &first,
                        &second,
                    )),
                }
            }
            None => Err(unknown_function(target.as_str(), target.span())),
        }
    }
}
