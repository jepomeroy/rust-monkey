use std::process;

use crate::{
    evaluator::evaluator::new_error,
    object::{
        BuiltinFunction, Object,
        ObjectType::{self, Integer, Null},
    },
};

pub(crate) fn get_builtin(func: &str) -> Option<BuiltinFunction> {
    match func {
        "exit" => Some(exit_function),
        "first" => Some(first_function),
        "last" => Some(last_function),
        "len" => Some(len_function),
        "push" => Some(push_function),
        "rest" => Some(rest_function),
        _ => None,
    }
}

fn exit_function(args: &[ObjectType]) -> ObjectType {
    match args.len() {
        0 => process::exit(0),
        1 => match args.first().unwrap() {
            Integer(code) => process::exit(*code as i32),
            other => new_error(format!(
                "argument to `exit` must be an INTEGER. got {}",
                other.objtype()
            )),
        },

        count => new_error(format!(
            "wrong number of arguments. got={count:?} want=0 or 1"
        )),
    }
}

fn first_function(args: &[ObjectType]) -> ObjectType {
    match args.len() {
        1 => {
            let obj = args.first().unwrap();

            match obj {
                ObjectType::Array(arr) => {
                    if !arr.is_empty() {
                        return arr.first().unwrap().clone();
                    }

                    Null
                }
                other => new_error(format!(
                    "argument to `first` must be an ARRAY. got {}",
                    other.objtype()
                )),
            }
        }
        count => new_error(format!("wrong number of arguments. got={count:?} want=1")),
    }
}

fn last_function(args: &[ObjectType]) -> ObjectType {
    match args.len() {
        1 => {
            let obj = args.first().unwrap();

            match obj {
                ObjectType::Array(arr) => {
                    if !arr.is_empty() {
                        return arr.last().unwrap().clone();
                    }

                    Null
                }
                other => new_error(format!(
                    "argument to `last` must be an ARRAY. got {}",
                    other.objtype()
                )),
            }
        }
        count => new_error(format!("wrong number of arguments. got={count:?} want=1")),
    }
}

fn len_function(args: &[ObjectType]) -> ObjectType {
    match args.len() {
        1 => {
            let obj = args.first().unwrap();

            match obj {
                ObjectType::StringObj(s) => Integer(s.len() as i64),
                ObjectType::Array(arr) => Integer(arr.len() as i64),
                other => new_error(format!(
                    "argument to `len` not supported. got {}",
                    other.objtype()
                )),
            }
        }
        count => new_error(format!("wrong number of arguments. got={count:?} want=1")),
    }
}

fn push_function(args: &[ObjectType]) -> ObjectType {
    match args.len() {
        2 => {
            let obj = args.first().unwrap();
            let appended = args.get(1).unwrap().clone();

            match obj {
                ObjectType::Array(arr) => {
                    let mut new_arr = vec![];
                    new_arr.extend_from_slice(&arr);
                    new_arr.push(appended);
                    ObjectType::Array(new_arr)
                }
                other => new_error(format!(
                    "argument to `push` must be an ARRAY. got {}",
                    other.objtype()
                )),
            }
        }
        count => new_error(format!("wrong number of arguments. got={count:?} want=2")),
    }
}

fn rest_function(args: &[ObjectType]) -> ObjectType {
    match args.len() {
        1 => {
            let obj = args.first().unwrap();

            match obj {
                ObjectType::Array(arr) => {
                    if !arr.is_empty() {
                        let mut new_arr = vec![];
                        new_arr.extend_from_slice(&arr[1..]);
                        return ObjectType::Array(new_arr);
                    }

                    Null
                }
                other => new_error(format!(
                    "argument to `rest` must be an ARRAY. got {}",
                    other.objtype()
                )),
            }
        }
        count => new_error(format!("wrong number of arguments. got={count:?} want=1")),
    }
}
