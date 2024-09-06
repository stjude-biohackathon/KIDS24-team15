//! Implementation of stdlib functions.

use std::fs;

use anyhow::Result;

use anyhow::Context;
use wdl_analysis::types::PrimitiveTypeKind;

use crate::{Runtime, Value};

/// Implements the `read_string` stdlib function.
pub fn read_string(runtime: &mut Runtime<'_>, args: &[Value]) -> Result<Value> {
    let path = args[0]
        .coerce(runtime, PrimitiveTypeKind::File.into())
        .unwrap()
        .unwrap_file(runtime);
    Ok(runtime.new_string(
        fs::read_to_string(path).with_context(|| format!("failed to read file `{path}`"))?,
    ))
}
