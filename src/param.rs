use crate::{Model, Retcode};

pub trait ScipParameter: Sized {
    fn set<T>(model: Model<T>, name: &str, value: Self) -> Result<Model<T>, Retcode>;
    fn get<T>(model: &Model<T>, name: &str) -> Self;
}

impl ScipParameter for f64 {
    fn set<T>(model: Model<T>, name: &str, value: f64) -> Result<Model<T>, Retcode> {
        let model  = model.set_real_param(name, value)?;
        Ok(model)
    }

    fn get<T>(model: &Model<T>, name: &str) -> f64 {
        model.real_param(name)
    }
}


impl ScipParameter for i32 {
    fn set<T>(model: Model<T>, name: &str, value: i32) -> Result<Model<T>, Retcode> {
        let model  = model.set_int_param(name, value)?;
        Ok(model)
    }

    fn get<T>(model: &Model<T>, name: &str) -> i32 {
        model.int_param(name)
    }
}

impl ScipParameter for bool {
    fn set<T>(model: Model<T>, name: &str, value: bool) -> Result<Model<T>, Retcode> {
        let model  = model.set_bool_param(name, value)?;
        Ok(model)
    }

    fn get<T>(model: &Model<T>, name: &str) -> bool {
        model.bool_param(name)
    }
}


impl ScipParameter for i64 {
    fn set<T>(model: Model<T>, name: &str, value: i64) -> Result<Model<T>, Retcode> {
        let model  = model.set_longint_param(name, value)?;
        Ok(model)
    }

    fn get<T>(model: &Model<T>, name: &str) -> i64 {
        model.longint_param(name)
    }
}


impl ScipParameter for String {
    fn set<T>(model: Model<T>, name: &str, value: String) -> Result<Model<T>, Retcode> {
        let model  = model.set_str_param(name, &value)?;
        Ok(model)
    }

    fn get<T>(model: &Model<T>, name: &str) -> String {
        model.str_param(name)
    }
}
