use crate::{Model, Retcode};

pub trait ScipParameter: Sized {
    fn set<T>(model: Model<T>, name: &str, value: Self) -> Result<Model<T>, Retcode>;
    fn get<T>(model: &Model<T>, name: &str) -> Self;
}

impl ScipParameter for f64 {
    fn set<T>(model: Model<T>, name: &str, value: f64) -> Result<Model<T>, Retcode> {
        let model = model.set_real_param(name, value)?;
        Ok(model)
    }

    fn get<T>(model: &Model<T>, name: &str) -> f64 {
        model.real_param(name)
    }
}

impl ScipParameter for i32 {
    fn set<T>(model: Model<T>, name: &str, value: i32) -> Result<Model<T>, Retcode> {
        let model = model.set_int_param(name, value)?;
        Ok(model)
    }

    fn get<T>(model: &Model<T>, name: &str) -> i32 {
        model.int_param(name)
    }
}

impl ScipParameter for bool {
    fn set<T>(model: Model<T>, name: &str, value: bool) -> Result<Model<T>, Retcode> {
        let model = model.set_bool_param(name, value)?;
        Ok(model)
    }

    fn get<T>(model: &Model<T>, name: &str) -> bool {
        model.bool_param(name)
    }
}

impl ScipParameter for i64 {
    fn set<T>(model: Model<T>, name: &str, value: i64) -> Result<Model<T>, Retcode> {
        let model = model.set_longint_param(name, value)?;
        Ok(model)
    }

    fn get<T>(model: &Model<T>, name: &str) -> i64 {
        model.longint_param(name)
    }
}

impl ScipParameter for String {
    fn set<T>(model: Model<T>, name: &str, value: String) -> Result<Model<T>, Retcode> {
        let model = model.set_str_param(name, &value)?;
        Ok(model)
    }

    fn get<T>(model: &Model<T>, name: &str) -> String {
        model.str_param(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bool() {
        let model = Model::default();
        let model = bool::set(model, "display/allviols", true).unwrap();
        assert!(model.param::<bool>("display/allviols"));
        let model = model.set_param("display/allviols", false);
        assert!(!bool::get(&model, "display/allviols"));
    }

    #[test]
    fn test_i64() {
        let model = Model::default();
        assert_eq!(
            model.param::<i64>("constraints/components/nodelimit"),
            10000i64
        );
        let model = model.set_param("constraints/components/nodelimit", 100i64);
        assert_eq!(
            model.param::<i64>("constraints/components/nodelimit"),
            100i64
        );
    }

    #[test]
    fn test_i32() {
        let model = Model::default();
        assert_eq!(model.param::<i32>("conflict/minmaxvars"), 0i32);
        let model = model.set_param("conflict/minmaxvars", 100i32);
        assert_eq!(model.param::<i32>("conflict/minmaxvars"), 100i32);
    }

    #[test]
    fn test_f64() {
        let model = Model::default();
        assert_eq!(model.param::<f64>("limits/time"), 1e+20);
        let model = model.set_param("limits/time", 100.0);
        assert_eq!(model.param::<f64>("limits/time"), 100.0);
    }

    #[test]
    fn test_str() {
        let model = Model::default();
        assert_eq!(model.param::<String>("visual/vbcfilename"), "-".to_string());
        let model = model.set_param("visual/vbcfilename", "test".to_string());
        assert_eq!(
            model.param::<String>("visual/vbcfilename"),
            "test".to_string()
        );
    }
}
