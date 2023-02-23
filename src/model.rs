use core::panic;
use std::collections::BTreeMap;
use std::mem::MaybeUninit;

use crate::constraint::Constraint;
use crate::retcode::Retcode;
use crate::scip_call;
use crate::solution::Solution;
use crate::status::Status;
use crate::variable::{VarId, VarType, Variable};
use crate::{ffi, scip_call_panic};
use std::ffi::CString;

#[derive(Debug, PartialEq, Eq)]
pub enum State {
    Created,
    PluginsIncluded,
    ProblemCreated,
    Solved,
}

#[non_exhaustive]
pub struct Model {
    pub(crate) scip: *mut ffi::SCIP,
    pub(crate) vars: BTreeMap<VarId, Variable>,
    pub(crate) conss: Vec<Constraint>,
    pub(crate) best_sol: Option<Solution>,
    pub(crate) state: State,
}

impl Model {
    pub fn new() -> Result<Self, Retcode> {
        let mut scip_ptr = MaybeUninit::uninit();
        scip_call!(ffi::SCIPcreate(scip_ptr.as_mut_ptr()));
        let scip_ptr = unsafe { scip_ptr.assume_init() };
        Ok(Model {
            scip: scip_ptr,
            vars: BTreeMap::new(),
            conss: Vec::new(),
            best_sol: None,
            state: State::Created,
        })
    }

    pub fn include_default_plugins(&mut self) -> Result<(), Retcode> {
        scip_call! { ffi::SCIPincludeDefaultPlugins(self.scip)};
        self.state = State::PluginsIncluded;
        Ok(())
    }

    pub fn read_prob(&mut self, filename: &str) -> Result<(), Retcode> {
        let filename = CString::new(filename).unwrap();
        scip_call! { ffi::SCIPreadProb(self.scip, filename.as_ptr(), std::ptr::null_mut()) };
        self._set_vars();
        self._set_conss();
        self.state = State::ProblemCreated;
        Ok(())
    }

    fn _set_vars(&mut self) {
        let n_vars = self.get_n_vars();
        let scip_vars = unsafe { ffi::SCIPgetVars(self.scip) };
        for i in 0..n_vars {
            let scip_var = unsafe { *scip_vars.add(i) };
            unsafe {
                ffi::SCIPcaptureVar(self.scip, scip_var);
            }
            let var = Variable { raw: scip_var };
            self.vars.insert(var.get_index(), var);
        }
    }

    fn _set_conss(&mut self) {
        let n_conss = self.get_n_conss();
        let scip_conss = unsafe { ffi::SCIPgetConss(self.scip) };
        for i in 0..n_conss {
            let scip_cons = unsafe { *scip_conss.add(i) };
            unsafe {
                ffi::SCIPcaptureCons(self.scip, scip_cons);
            }
            let cons = Constraint { raw: scip_cons };
            self.conss.push(cons);
        }
    }

    fn _set_best_sol(&mut self) {
        let sol = unsafe { ffi::SCIPgetBestSol(self.scip) };
        let sol = Solution {
            scip_ptr: self.scip,
            raw: sol,
        };
        self.best_sol = Some(sol);
    }

    pub fn solve(&mut self) -> Result<(), Retcode> {
        if self.state != State::ProblemCreated {
            return Err(Retcode::InvalidCall);
        }
        scip_call! { ffi::SCIPsolve(self.scip) };
        if self.get_status() == Status::OPTIMAL {
            self._set_best_sol();
        }
        Ok(())
    }

    pub fn get_status(&mut self) -> Status {
        let status = unsafe { ffi::SCIPgetStatus(self.scip) };
        Status::from_c_scip_status(status).unwrap()
    }

    pub fn get_obj_val(&mut self) -> f64 {
        unsafe { ffi::SCIPgetPrimalbound(self.scip) }
    }

    pub fn get_n_vars(&self) -> usize {
        unsafe { ffi::SCIPgetNVars(self.scip) as usize }
    }

    pub fn print_version(&mut self) {
        unsafe { ffi::SCIPprintVersion(self.scip, std::ptr::null_mut()) };
    }

    pub fn get_best_sol(&self) -> Option<Box<&Solution>> {
        self.best_sol.as_ref().map(Box::new)
    }

    pub fn get_vars(&self) -> Vec<Box<&Variable>> {
        self.vars.values().map(Box::new).collect()
    }

    pub fn get_var(&self, var_id: VarId) -> Option<Box<&Variable>> {
        self.vars.get(&var_id).map(Box::new)
    }

    pub fn set_str_param(&mut self, param: &str, value: &str) -> Result<(), Retcode> {
        let param = CString::new(param).unwrap();
        let value = CString::new(value).unwrap();
        scip_call! { ffi::SCIPsetStringParam(self.scip, param.as_ptr(), value.as_ptr()) };
        Ok(())
    }

    pub fn set_int_param(&mut self, param: &str, value: i32) -> Result<(), Retcode> {
        let param = CString::new(param).unwrap();
        scip_call! { ffi::SCIPsetIntParam(self.scip, param.as_ptr(), value) };
        Ok(())
    }

    pub fn set_real_param(&mut self, param: &str, value: f64) -> Result<(), Retcode> {
        let param = CString::new(param).unwrap();
        scip_call! { ffi::SCIPsetRealParam(self.scip, param.as_ptr(), value) };
        Ok(())
    }

    pub fn hide_output(&mut self) -> Result<(), Retcode> {
        self.set_int_param("display/verblevel", 0)?;
        Ok(())
    }

    pub fn add_var(
        &mut self,
        lb: f64,
        ub: f64,
        obj: f64,
        name: &str,
        var_type: VarType,
    ) -> Result<VarId, Retcode> {
        let var = Variable::new(self.scip, lb, ub, obj, name, var_type)?;
        let var_id = var.get_index();
        self.vars.insert(var_id, var);
        Ok(var_id)
    }

    pub fn add_cons(
        &mut self,
        var_ids: &[VarId],
        coefs: &[f64],
        lhs: f64,
        rhs: f64,
        name: &str,
    ) -> Result<(), Retcode> {
        assert_eq!(var_ids.len(), coefs.len());
        let vars = var_ids
            .iter()
            .map(|var_id| self.vars.get(var_id).unwrap())
            .collect::<Vec<_>>();
        let c_name = CString::new(name).unwrap();
        let mut scip_cons = MaybeUninit::uninit();
        scip_call! { ffi::SCIPcreateConsBasicLinear(
            self.scip,
            scip_cons.as_mut_ptr(),
            c_name.as_ptr(),
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            lhs,
            rhs,
        ) };
        let scip_cons = unsafe { scip_cons.assume_init() };
        for (i, var) in vars.iter().enumerate() {
            scip_call! { ffi::SCIPaddCoefLinear(self.scip, scip_cons, var.raw, coefs[i]) };
        }
        scip_call! { ffi::SCIPaddCons(self.scip, scip_cons) };
        let cons = Constraint { raw: scip_cons };
        self.conss.push(cons);
        Ok(())
    }

    pub fn create_prob(&mut self, name: &str) -> Result<(), Retcode> {
        let name = CString::new(name).unwrap();
        scip_call! { ffi::SCIPcreateProbBasic(
            self.scip,
            name.as_ptr(),
        ) };
        self.state = State::ProblemCreated;
        Ok(())
    }

    pub fn set_obj_sense(&mut self, sense: ObjSense) -> Result<(), Retcode> {
        scip_call! { ffi::SCIPsetObjsense(self.scip, sense.into()) };
        Ok(())
    }

    pub fn get_n_conss(&mut self) -> usize {
        unsafe { ffi::SCIPgetNConss(self.scip) as usize }
    }

    pub fn get_conss(&mut self) -> &Vec<Constraint> {
        &self.conss
    }

    pub fn set_presolving(&mut self, presolving: ParamSetting) -> Result<(), Retcode> {
        scip_call! { ffi::SCIPsetPresolving(self.scip, presolving.into(), true.into()) };
        Ok(())
    }

    pub fn set_separating(&mut self, separating: ParamSetting) -> Result<(), Retcode> {
        scip_call! { ffi::SCIPsetSeparating(self.scip, separating.into(), true.into()) };
        Ok(())
    }

    pub fn set_heuristics(&mut self, heuristics: ParamSetting) -> Result<(), Retcode> {
        scip_call! { ffi::SCIPsetHeuristics(self.scip, heuristics.into(), true.into()) };
        Ok(())
    }

    pub fn write(&mut self, path: &str, ext: &str) -> Result<(), Retcode> {
        let c_path = CString::new(path).unwrap();
        let c_ext = CString::new(ext).unwrap();
        scip_call! { ffi::SCIPwriteOrigProblem(
            self.scip,
            c_path.as_ptr(),
            c_ext.as_ptr(),
            true.into(),
        ) };
        Ok(())
    }
}

impl Default for Model {
    fn default() -> Self {
        let mut model = Model::new().expect("Failed to create SCIP model");
        model
            .include_default_plugins()
            .expect("Failed to include default plugins");
        model
            .create_prob("problem")
            .expect("Failed to create problem");
        model
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        for var in self.vars.values_mut() {
            scip_call_panic!(ffi::SCIPreleaseVar(self.scip, &mut var.raw));
        }
        for cons in self.conss.iter_mut() {
            scip_call_panic!(ffi::SCIPreleaseCons(self.scip, &mut cons.raw));
        }
        self.vars.clear();
        self.conss.clear();
        scip_call_panic!(ffi::SCIPfree(&mut self.scip));
    }
}

pub enum ParamSetting {
    Default,
    Aggressive,
    Fast,
    Off,
}

impl From<ParamSetting> for ffi::SCIP_PARAMSETTING {
    fn from(val: ParamSetting) -> Self {
        match val {
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

impl From<ObjSense> for ffi::SCIP_OBJSENSE {
    fn from(val: ObjSense) -> Self {
        match val {
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
    fn call_solve_without_problem() {
        assert!(Model::new().unwrap().solve().is_err());
    }

    #[test]
    fn call_solve_on_empty_problem() {
        assert!(Model::new().unwrap().solve().is_err());
    }

    #[test]
    fn solution_without_problem() {
        let model = Model::new().unwrap();
        let sol = model.get_best_sol();
        assert!(sol.is_none());
    }

    // #[test] does not compile anymore
    // fn drop_problem_before_solution() {
    //     let sol = {
    //         let mut model = Model::new().unwrap();
    //         model.hide_output().unwrap();
    //         model.include_default_plugins().unwrap();
    //         model.read_prob("data/test/simple.lp").unwrap();
    //         model.solve().unwrap();
    //         model.get_best_sol()
    //     };
    //     assert_eq!(sol.get_obj_val(), 200.);
    // }

    // #[test]  does not compile anymore
    // fn drop_variable_after_problem() {
    //     let mut model = Model::new().unwrap();
    //     let var_id = model.add_var(0., 0., 0., "", VarType::Binary).unwrap();
    //     let var = model.get_var(var_id).unwrap();
    //     drop(model);
    //     drop(var);
    // }

    #[test]
    fn solve_from_lp_file() -> Result<(), Retcode> {
        let mut model = Model::new()?;
        model.include_default_plugins()?;
        model.read_prob("data/test/simple.lp")?;
        model.hide_output()?;
        model.solve()?;
        let status = model.get_status();
        assert_eq!(status, Status::OPTIMAL);

        //test objective value
        let obj_val = model.get_obj_val();
        assert_eq!(obj_val, 200.);

        //test constraints
        let conss = model.get_conss();
        assert_eq!(conss.len(), 2);

        //test solution values
        let sol = model.get_best_sol().unwrap();
        let vars = model.get_vars();
        assert_eq!(vars.len(), 2);
        assert_eq!(sol.get_var_val(&vars[0]), 40.);
        assert_eq!(sol.get_var_val(&vars[1]), 20.);
        Ok(())
    }

    #[test]
    fn set_time_limit() -> Result<(), Retcode> {
        let mut model = Model::new()?;
        model.include_default_plugins()?;
        model.hide_output()?;
        model.read_prob("data/test/simple.lp")?;
        model.set_real_param("limits/time", 0.)?;
        model.solve()?;
        let status = model.get_status();
        assert_eq!(status, Status::TIMELIMIT);
        Ok(())
    }

    #[test]
    fn add_variable() -> Result<(), Retcode> {
        let mut model = Model::new()?;
        model.include_default_plugins()?;
        model.create_prob("test")?;
        model.set_obj_sense(ObjSense::Maximize)?;
        model.hide_output()?;
        let x1_id = model.add_var(0., f64::INFINITY, 3., "x1", VarType::Integer)?;
        let x2_id = model.add_var(0., f64::INFINITY, 4., "x2", VarType::Continuous)?;
        let x1 = model.get_var(x1_id).unwrap();
        let x2 = model.get_var(x2_id).unwrap();
        assert_eq!(model.get_n_vars(), 2);
        assert_eq!(model.get_vars().len(), 2);
        assert!(x1.raw != x2.raw);
        assert!(x1.get_type() == VarType::Integer);
        assert!(x2.get_type() == VarType::Continuous);
        assert!(x1.get_name() == "x1");
        assert!(x2.get_name() == "x2");
        assert!(x1.get_obj() == 3.);
        assert!(x2.get_obj() == 4.);
        Ok(())
    }

    fn create_model() -> Result<Model, Retcode> {
        let mut model = Model::new()?;
        model.include_default_plugins()?;
        model.create_prob("test")?;
        model.set_obj_sense(ObjSense::Maximize)?;
        model.hide_output()?;

        let x1_id = model.add_var(0., f64::INFINITY, 3., "x1", VarType::Integer)?;
        let x2_id = model.add_var(0., f64::INFINITY, 4., "x2", VarType::Integer)?;
        model.add_cons(&[x1_id, x2_id], &[2., 1.], -f64::INFINITY, 100., "c1")?;
        model.add_cons(&[x1_id, x2_id], &[1., 2.], -f64::INFINITY, 80., "c2")?;

        Ok(model)
    }

    #[test]
    fn build_model_with_functions() -> Result<(), Retcode> {
        let mut model = create_model()?;
        assert_eq!(model.get_vars().len(), 2);
        assert_eq!(model.get_n_conss(), 2);

        let conss = model.get_conss();
        assert_eq!(conss.len(), 2);
        assert_eq!(conss[0].get_name(), "c1");
        assert_eq!(conss[1].get_name(), "c2");

        model.solve()?;

        let status = model.get_status();
        assert_eq!(status, Status::OPTIMAL);

        let obj_val = model.get_obj_val();
        assert_eq!(obj_val, 200.);

        let sol = model.get_best_sol().unwrap();
        let vars = model.get_vars();
        assert_eq!(vars.len(), 2);
        assert_eq!(sol.get_var_val(&vars[0]), 40.);
        assert_eq!(sol.get_var_val(&vars[1]), 20.);
        Ok(())
    }
}
