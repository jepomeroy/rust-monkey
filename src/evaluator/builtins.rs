use crate::{
    evaluator::evaluator::new_error,
    object::{
        BuiltinFunction, Object,
        ObjectType::{self, Integer},
    },
};

pub(crate) fn get_builtin(func: &str) -> Option<BuiltinFunction> {
    match func {
        "len" => Some(len_function),
        _ => None,
    }
}

fn len_function(args: &[ObjectType]) -> ObjectType {
    match args.len() {
        1 => {
            let obj = args.first().unwrap();

            match obj {
                ObjectType::StringObj(s) => Integer(s.len() as i64),
                other => new_error(format!(
                    "argument to `len` not supported. got {}",
                    other.objtype()
                )),
            }
        }
        count => new_error(format!("wrong number of arguments. got={count:?} want=1")),
    }
}
