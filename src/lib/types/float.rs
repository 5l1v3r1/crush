use crate::lang::command::Command;
use crate::lang::command::OutputType::Known;
use crate::lang::command::TypeMap;
use crate::lang::errors::{argument_error, CrushResult};
use crate::lang::execution_context::{ArgumentVector, This};
use crate::lang::value::ValueType;
use crate::lang::{execution_context::CommandContext, value::Value};
use lazy_static::lazy_static;
use ordered_map::OrderedMap;

fn full(name: &'static str) -> Vec<&'static str> {
    vec!["global", "types", "float", name]
}

lazy_static! {
    pub static ref METHODS: OrderedMap<String, Command> = {
        let mut res: OrderedMap<String, Command> = OrderedMap::new();
        res.declare(
            full("__add__"),
            add,
            false,
            "float + term:(integer|float)",
            "Add this number and the specified term",
            None,
            Known(ValueType::Float),
        );
        res.declare(
            full("__sub__"),
            sub,
            false,
            "float - term:(integer|float)",
            "Subtract the specified term from this number",
            None,
            Known(ValueType::Float),
        );
        res.declare(
            full("__mul__"),
            mul,
            false,
            "float * factor:(integer|float)",
            "Multiply this number by the specified factor",
            None,
            Known(ValueType::Float),
        );
        res.declare(
            full("__div__"),
            div,
            false,
            "integer / factor:(integer|float)",
            "Divide this number by the specified factor",
            None,
            Known(ValueType::Float),
        );
        res.declare(
            full("__neg__"),
            neg,
            false,
            "neg float",
            "Negate this integer",
            None,
            Known(ValueType::Float),
        );
        res.declare(
            full("is_finite"),
            is_infinite,
            false,
            "float:is_infinite",
            "True if this float is positive or negative infinity",
            None,
            Known(ValueType::Bool),
        );
        res.declare(
            full("is_nan"),
            is_nan,
            false,
            "float:is_nan",
            "True if this float is NaN",
            None,
            Known(ValueType::Bool),
        );
        res
    };
}

binary_op!(
    add,
    float,
    Integer,
    Float,
    |a, b| a + (b as f64),
    Float,
    Float,
    |a, b| a + b
);
binary_op!(
    sub,
    float,
    Integer,
    Float,
    |a, b| a - (b as f64),
    Float,
    Float,
    |a, b| a - b
);
binary_op!(
    mul,
    float,
    Integer,
    Float,
    |a, b| a * (b as f64),
    Float,
    Float,
    |a, b| a * b
);
binary_op!(
    div,
    float,
    Integer,
    Float,
    |a, b| a / (b as f64),
    Float,
    Float,
    |a, b| a / b
);

fn neg(context: CommandContext) -> CrushResult<()> {
    context.arguments.check_len(0)?;
    context.output.send(Value::Float(-context.this.float()?))
}

fn is_nan(context: CommandContext) -> CrushResult<()> {
    context.arguments.check_len(0)?;
    context
        .output
        .send(Value::Bool(context.this.float()?.is_nan()))
}

fn is_infinite(context: CommandContext) -> CrushResult<()> {
    context.arguments.check_len(0)?;
    context
        .output
        .send(Value::Bool(context.this.float()?.is_infinite()))
}
