use std::mem;

use crate::constraint::Constraint;
use crate::ffi;
use crate::scip_call;
use crate::solution::Solution;
use crate::status::Status;
use crate::variable::{VarType, Variable};
use std::ffi::CString;

#[non_exhaustive]
pub struct Model {
    pub(crate) scip: *mut ffi::SCIP,
}

impl Model {
    pub fn new() -> Self {
        let mut model: *mut ffi::SCIP = unsafe { mem::zeroed() };
        scip_call!(ffi::SCIPcreate(&mut model));
        Model { scip: model }
    }

    pub fn include_default_plugins(&mut self) {
        scip_call! { ffi::SCIPincludeDefaultPlugins(self.scip)};
    }

    pub fn read_prob(&mut self, filename: &str) {
        let filename = CString::new(filename).unwrap();
        scip_call! { ffi::SCIPreadProb(self.scip, filename.as_ptr(), std::ptr::null_mut()) };
    }

    pub fn solve(&self) {
        scip_call! { ffi::SCIPsolve(self.scip) };
    }

    pub fn get_status(&self) -> Status {
        let status = unsafe { ffi::SCIPgetStatus(self.scip) };
        Status::from_c_scip_status(status).unwrap()
    }

    pub fn get_obj_val(&self) -> f64 {
        unsafe { ffi::SCIPgetPrimalbound(self.scip) }
    }

    pub fn get_n_vars(&self) -> usize {
        unsafe { ffi::SCIPgetNVars(self.scip) as usize }
    }

    pub fn print_version(&self) {
        unsafe { ffi::SCIPprintVersion(self.scip, std::ptr::null_mut()) };
    }

    pub fn get_best_sol(&self) -> Solution {
        let sol = unsafe { ffi::SCIPgetBestSol(self.scip) };
        Solution {
            model: &self,
            raw: sol,
        }
    }

    pub fn get_vars(&self) -> Vec<Variable> {
        let n_vars = self.get_n_vars();
        let mut vars = Vec::with_capacity(n_vars);
        let scip_vars = unsafe { ffi::SCIPgetVars(self.scip) };
        for i in 0..n_vars {
            let scip_var = unsafe { *scip_vars.offset(i as isize) };
            // increase scip var's reference count
            scip_call!(ffi::SCIPcaptureVar(self.scip, scip_var));
            vars.push(Variable {
                model: &self,
                raw: scip_var,
            });
        }
        vars
    }

    pub fn set_str_param(&mut self, param: &str, value: &str) {
        let param = CString::new(param).unwrap();
        let value = CString::new(value).unwrap();
        scip_call! { ffi::SCIPsetStringParam(self.scip, param.as_ptr(), value.as_ptr()) };
    }

    pub fn set_int_param(&mut self, param: &str, value: i32) {
        let param = CString::new(param).unwrap();
        scip_call! { ffi::SCIPsetIntParam(self.scip, param.as_ptr(), value) };
    }

    pub fn set_real_param(&mut self, param: &str, value: f64) {
        let param = CString::new(param).unwrap();
        scip_call! { ffi::SCIPsetRealParam(self.scip, param.as_ptr(), value) };
    }

    pub fn hide_output(&mut self) {
        self.set_int_param("display/verblevel", 0);
    }

    pub fn add_var(
        &self,
        lb: f64,
        ub: f64,
        obj: f64,
        name: &str,
        var_type: VarType,
    ) -> Variable {
        Variable::new(&self, lb, ub, obj, name, var_type.into())
    }

    pub fn add_cons(
        &self,
        vars: &[&Variable],
        coefs: &[f64],
        lhs: f64,
        rhs: f64,
        name: &str,
    ) -> Constraint {
        assert_eq!(vars.len(), coefs.len());
        let c_name = CString::new(name).unwrap();
        let mut scip_cons: *mut ffi::SCIP_CONS = unsafe { mem::zeroed() };
        scip_call! { ffi::SCIPcreateConsBasicLinear(
            self.scip,
            &mut scip_cons,
            c_name.as_ptr(),
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            lhs,
            rhs,
        ) };
        for (i, var) in vars.iter().enumerate() {
            scip_call! { ffi::SCIPaddCoefLinear(self.scip, scip_cons, var.raw, coefs[i]) };
        }
        scip_call! { ffi::SCIPaddCons(self.scip, scip_cons) };
        Constraint {
            model: &self,
            raw: scip_cons,
        }
    }

    pub fn create_prob(&mut self, name: &str) {
        let name = CString::new(name).unwrap();
        scip_call! { ffi::SCIPcreateProbBasic(
            self.scip,
            name.as_ptr(),
        ) };
    }

    pub fn set_obj_sense(&mut self, sense: ObjSense) {
        scip_call! { ffi::SCIPsetObjsense(self.scip, sense.into()) };
    }

    pub fn get_n_conss(&self) -> usize {
        unsafe { ffi::SCIPgetNConss(self.scip) as usize }
    }

    pub fn get_conss(&self) -> Vec<Constraint> {
        let n_conss = self.get_n_conss();
        let mut conss = Vec::with_capacity(n_conss);
        let scip_conss = unsafe { ffi::SCIPgetConss(self.scip) };
        for i in 0..n_conss {
            let scip_cons = unsafe { *scip_conss.offset(i as isize) };
            // increase scip cons's reference count
            scip_call!(ffi::SCIPcaptureCons(self.scip, scip_cons));
            conss.push(Constraint {
                model: &self,
                raw: scip_cons,
            });
        }
        conss
    }

    pub fn set_presolving(&mut self, presolving: ParamSetting) {
        scip_call! { ffi::SCIPsetPresolving(self.scip, presolving.into(), true.into()) };
    }

    pub fn set_separating(&mut self, separating: ParamSetting) {
        scip_call! { ffi::SCIPsetSeparating(self.scip, separating.into(), true.into()) };
    }

    pub fn set_heuristics(&mut self, heuristics: ParamSetting) {
        scip_call! { ffi::SCIPsetHeuristics(self.scip, heuristics.into(), true.into()) };
    }

    pub fn write(&self, path: &str, ext: &str) {
        let c_path = CString::new(path).unwrap();
        let c_ext = CString::new(ext).unwrap();
        scip_call! { ffi::SCIPwriteOrigProblem(
            self.scip,
            c_path.as_ptr(),
            c_ext.as_ptr(),
            true.into(),
        ) };
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        unsafe { ffi::SCIPfree(&mut self.scip) };
    }
}

pub enum ParamSetting {
    Default,
    Aggressive,
    Fast,
    Off,
}

impl Into<ffi::SCIP_PARAMSETTING> for ParamSetting {
    fn into(self) -> ffi::SCIP_PARAMSETTING {
        match self {
            ParamSetting::Default => ffi::SCIP_ParamSetting_SCIP_PARAMSETTING_DEFAULT,
            ParamSetting::Aggressive => ffi::SCIP_ParamSetting_SCIP_PARAMSETTING_AGGRESSIVE,
            ParamSetting::Fast => ffi::SCIP_ParamSetting_SCIP_PARAMSETTING_FAST,
            ParamSetting::Off => ffi::SCIP_ParamSetting_SCIP_PARAMSETTING_OFF,
        }
    }
}

pub enum ObjSense {
    Minimize,
    Maximize,
}

impl From<ffi::SCIP_OBJSENSE> for ObjSense {
    fn from(sense: ffi::SCIP_OBJSENSE) -> Self {
        match sense {
            ffi::SCIP_Objsense_SCIP_OBJSENSE_MAXIMIZE => ObjSense::Maximize,
            ffi::SCIP_Objsense_SCIP_OBJSENSE_MINIMIZE => ObjSense::Minimize,
            _ => panic!("Unknown objective sense value {:?}", sense),
        }
    }
}

impl Into<ffi::SCIP_OBJSENSE> for ObjSense {
    fn into(self) -> ffi::SCIP_OBJSENSE {
        match self {
            ObjSense::Maximize => ffi::SCIP_Objsense_SCIP_OBJSENSE_MAXIMIZE,
            ObjSense::Minimize => ffi::SCIP_Objsense_SCIP_OBJSENSE_MINIMIZE,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::status::Status;

    #[test]
    fn solve_from_lp_file() {
        let mut model = Model::new();
        model.include_default_plugins();
        model.read_prob("data/test/simple.lp");
        model.hide_output();
        model.solve();
        let status = model.get_status();
        assert_eq!(status, Status::OPTIMAL);

        //test objective value
        let obj_val = model.get_obj_val();
        assert_eq!(obj_val, 200.);

        //test solution values
        let sol = model.get_best_sol();
        let vars = model.get_vars();
        assert_eq!(vars.len(), 2);
        assert_eq!(sol.get_var_val(&vars[0]), 40.);
        assert_eq!(sol.get_var_val(&vars[1]), 20.);
    }

    #[test]
    fn set_time_limit() {
        let mut model = Model::new();
        model.include_default_plugins();
        model.hide_output();
        model.read_prob("data/test/simple.lp");
        model.set_real_param("limits/time", 0.);
        model.solve();
        let status = model.get_status();
        assert_eq!(status, Status::TIMELIMIT);
    }

    #[test]
    fn add_variable() {
        let mut model = Model::new();
        model.include_default_plugins();
        model.create_prob("test");
        model.set_obj_sense(ObjSense::Maximize);
        model.hide_output();
        let x1 = model.add_var(0., f64::INFINITY, 3., "x1", VarType::Integer);
        let x2 = model.add_var(0., f64::INFINITY, 4., "x2", VarType::Continuous);
        assert_eq!(model.get_n_vars(), 2);
        assert_eq!(model.get_vars().len(), 2);
        assert!(x1.raw != x2.raw);
        assert!(x1.get_type() == VarType::Integer);
        assert!(x2.get_type() == VarType::Continuous);
        assert!(x1.get_name() == "x1");
        assert!(x2.get_name() == "x2");
        assert!(x1.get_obj() == 3.);
        assert!(x2.get_obj() == 4.);
    }

    fn create_model() -> Model {
        let mut model = Model::new();
        model.include_default_plugins();
        model.create_prob("test");
        model.set_obj_sense(ObjSense::Maximize);
        model.hide_output();
        {
        let x1 = model.add_var(0., f64::INFINITY, 3., "x1", VarType::Integer);
        let x2 = model.add_var(0., f64::INFINITY, 4., "x2", VarType::Integer);
        model.add_cons(&[&x1, &x2], &[2., 1.], -f64::INFINITY, 100., "c1");
        model.add_cons(&[&x1, &x2], &[1., 2.], -f64::INFINITY, 80., "c2");
        }
        model
    }
    
    #[test]
    fn build_model_with_functions() {
        let model = create_model();
        assert_eq!(model.get_vars().len(), 2);
        assert_eq!(model.get_n_conss(), 2);

        let conss = model.get_conss();
        assert_eq!(conss.len(), 2);
        assert_eq!(conss[0].get_name(), "c1");
        assert_eq!(conss[1].get_name(), "c2");

        model.solve();

        let status = model.get_status();
        assert_eq!(status, Status::OPTIMAL);

        let obj_val = model.get_obj_val();
        assert_eq!(obj_val, 200.);

        let sol = model.get_best_sol();
        let vars = model.get_vars();
        assert_eq!(vars.len(), 2);
        assert_eq!(sol.get_var_val(&vars[0]), 40.);
        assert_eq!(sol.get_var_val(&vars[1]), 20.);
    }
}
