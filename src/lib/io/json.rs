use crate::lang::execution_context::CommandContext;
use crate::{
    lang::errors::CrushError,
    lang::{table::Row, value::Value, value::ValueType},
};
use std::io::{BufReader, Write};

use crate::lang::argument::ArgumentHandler;
use crate::lang::command::OutputType::Unknown;
use crate::lang::errors::{error, mandate, to_crush_error, CrushResult};
use crate::lang::files::Files;
use crate::lang::scope::ScopeLoader;
use crate::lang::table::ColumnType;
use crate::lang::{list::List, r#struct::Struct, table::Table};
use signature::signature;
use std::collections::HashSet;
use std::convert::TryFrom;

fn from_json(json_value: &serde_json::Value) -> CrushResult<Value> {
    match json_value {
        serde_json::Value::Null => Ok(Value::Empty()),
        serde_json::Value::Bool(b) => Ok(Value::Bool(*b)),
        serde_json::Value::Number(f) => {
            if f.is_u64() {
                Ok(Value::Integer(f.as_u64().expect("") as i128))
            } else if f.is_i64() {
                Ok(Value::Integer(f.as_i64().expect("") as i128))
            } else {
                Ok(Value::Float(f.as_f64().ok_or(CrushError::InvalidData(
                    "Not a valid number".to_string(),
                ))?))
            }
        }
        serde_json::Value::String(s) => Ok(Value::string(s.as_str())),
        serde_json::Value::Array(arr) => {
            let mut lst = arr
                .iter()
                .map(|v| from_json(v))
                .collect::<CrushResult<Vec<Value>>>()?;
            let types: HashSet<ValueType> = lst.iter().map(|v| v.value_type()).collect();
            let struct_types: HashSet<Vec<ColumnType>> = lst
                .iter()
                .flat_map(|v| match v {
                    Value::Struct(r) => vec![r.local_signature()],
                    _ => vec![],
                })
                .collect();

            match types.len() {
                0 => Ok(Value::Empty()),
                1 => {
                    let list_type = types.iter().next().unwrap();
                    match (list_type, struct_types.len()) {
                        (ValueType::Struct, 1) => {
                            let row_list = lst
                                .drain(..)
                                .map(|v| match v {
                                    Value::Struct(r) => Ok(r.to_row()),
                                    _ => error("Impossible!"),
                                })
                                .collect::<CrushResult<Vec<Row>>>()?;
                            Ok(Value::Table(Table::new(
                                struct_types.iter().next().unwrap().clone(),
                                row_list,
                            )))
                        }
                        _ => Ok(Value::List(List::new(list_type.clone(), lst))),
                    }
                }
                _ => Ok(Value::List(List::new(ValueType::Any, lst))),
            }
        }
        serde_json::Value::Object(o) => Ok(Value::Struct(Struct::new(
            o.iter()
                .map(|(k, v)| (k.to_string(), from_json(v)))
                .map(|(k, v)| match v {
                    Ok(vv) => Ok((k, vv)),
                    Err(e) => Err(e),
                })
                .collect::<Result<Vec<(String, Value)>, CrushError>>()?,
            None,
        ))),
    }
}

fn to_json(value: Value) -> CrushResult<serde_json::Value> {
    match value.materialize() {
        Value::File(s) => Ok(serde_json::Value::from(mandate(
            s.to_str(),
            "Invalid filename",
        )?)),

        Value::String(s) => Ok(serde_json::Value::from(s)),

        Value::Integer(i) => Ok(serde_json::Value::from(to_crush_error(i64::try_from(i))?)),

        Value::List(l) => Ok(serde_json::Value::Array(
            l.dump()
                .drain(..)
                .map(to_json)
                .collect::<CrushResult<Vec<_>>>()?,
        )),

        Value::Table(t) => {
            let types = t.types().to_vec();
            let structs = t
                .rows()
                .iter()
                .map(|r| r.clone().into_struct(&types))
                .map(|s| to_json(Value::Struct(s)))
                .collect::<CrushResult<Vec<_>>>()?;
            Ok(serde_json::Value::Array(structs))
        }

        Value::Bool(b) => Ok(serde_json::Value::from(b)),

        Value::Float(f) => Ok(serde_json::Value::from(f)),

        Value::Struct(s) => {
            let mut map = serde_json::map::Map::new();
            for (k, v) in s.local_elements() {
                map.insert(k.to_string(), to_json(v)?);
            }
            Ok(serde_json::Value::Object(map))
        }

        Value::Duration(d) => Ok(serde_json::Value::from(d.num_seconds())),

        Value::Time(t) => Ok(serde_json::Value::from(t.to_rfc3339())),

        Value::Binary(b) => Ok(serde_json::Value::from(b)),

        Value::BinaryStream(_) => panic!("Impossible"),

        Value::TableStream(_) => panic!("Impossible"),

        v => error(&format!("Unsupported data type {}", v.value_type())),
    }
}

#[signature(
from,
can_block = true,
output = Unknown,
short = "Parse json format",
example = "(http \"https://jsonplaceholder.typicode.com/todos/3\"):body | json:from")]
struct From {
    #[unnamed()]
    files: Files,
}

pub fn from(context: CommandContext) -> CrushResult<()> {
    let cfg: From = From::parse(context.arguments, &context.printer)?;
    let reader = BufReader::new(cfg.files.reader(context.input)?);
    let serde_value = to_crush_error(serde_json::from_reader(reader))?;
    let crush_value = from_json(&serde_value)?;
    context.output.send(crush_value)
}

#[signature(
to,
can_block = true,
output = Unknown,
short = "Serialize to json format",
example = "ls | json:to")]
struct To {
    #[unnamed()]
    file: Files,
}

fn to(context: CommandContext) -> CrushResult<()> {
    let cfg: To = To::parse(context.arguments, &context.printer)?;
    let mut writer = cfg.file.writer(context.output)?;
    let value = context.input.recv()?;
    let json_value = to_json(value)?;
    to_crush_error(writer.write(json_value.to_string().as_bytes()))?;
    Ok(())
}

pub fn declare(root: &mut ScopeLoader) -> CrushResult<()> {
    root.create_namespace(
        "json",
        Box::new(move |env| {
            let _ = From::declare(env); // TODO: why unused?
            let _ = To::declare(env); // TODO: why unused?
            Ok(())
        }),
    )?;
    Ok(())
}
