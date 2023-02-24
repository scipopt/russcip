use core::panic;
use std::collections::BTreeMap;
use std::mem::{self, take, MaybeUninit};

use crate::constraint::Constraint;
use crate::retcode::Retcode;
use crate::solution::Solution;
use crate::status::Status;
use crate::variable::{VarId, VarType, Variable};
use crate::{ffi, scip_call_panic};
use crate::{scip_call, scip_call_expect};
use std::ffi::CString;

struct ScipPtr(*mut ffi::SCIP);

impl ScipPtr {
    fn new() -> Self {
        let mut scip_ptr = MaybeUninit::uninit();
        scip_call_panic!(ffi::SCIPcreate(scip_ptr.as_mut_ptr()));
        let scip_ptr = unsafe { scip_ptr.assume_init() };
        ScipPtr(scip_ptr)
    }

    fn ptr(&self) -> *mut ffi::SCIP {
        self.0
    }
}

impl Default for ScipPtr {
    fn default() -> Self {
        Self(std::ptr::null_mut())
    }
}

impl Drop for ScipPtr {
    fn drop(&mut self) {
        if self.0.is_null() {
            return;
        } else {
            // Rust Model struct keeps at most one copy of each variable and constraint pointers
            // so we need to release them before freeing the SCIP instance

            // release variables
            let n_vars = unsafe { ffi::SCIPgetNVars(self.0) };
            let vars = unsafe { ffi::SCIPgetOrigVars(self.0) };
            for i in 0..n_vars {
                let mut var = unsafe { *vars.add(i as usize) };
                scip_call_panic!(ffi::SCIPreleaseVar(self.0, &mut var));
            }

            // release constraints
            let n_conss = unsafe { ffi::SCIPgetNConss(self.0) };
            let conss = unsafe { ffi::SCIPgetOrigConss(self.0) };
            for i in 0..n_conss {
                let mut cons = unsafe { *conss.add(i as usize) };
                scip_call_panic!(ffi::SCIPreleaseCons(self.0, &mut cons));
            }

            // free SCIP instance
            unsafe { ffi::SCIPfree(&mut self.0) };
        }
    }
}

#[non_exhaustive]
pub struct Model<State> {
    scip: ScipPtr,
    pub(crate) vars: BTreeMap<VarId, Variable>,
    pub(crate) conss: Vec<Constraint>,
    pub(crate) best_sol: Option<Solution>,
    state: State,
}

pub struct Unsolved;
pub struct PluginsIncluded;
pub struct ProblemCreated;

pub struct Solved;

impl Model<Unsolved> {
    pub fn new() -> Self {
        Self::try_new().expect("Failed to create SCIP instance")
    }

    pub fn try_new() -> Result<Self, Retcode> {
        let scip_ptr = ScipPtr::new();
        Ok(Model {
            scip: scip_ptr,
            vars: BTreeMap::new(),
            conss: Vec::new(),
            best_sol: None,
            state: Unsolved,
        })
    }

    pub fn include_default_plugins(&mut self) -> Model<PluginsIncluded> {
        scip_call_expect!(
            ffi::SCIPincludeDefaultPlugins(self.scip.ptr()),
            "Failed to include default plugins"
        );

        self.move_to_state(PluginsIncluded)
    }

    pub fn set_str_param(self, param: &str, value: &str) -> Result<Self, Retcode> {
        let param = CString::new(param).unwrap();
        let value = CString::new(value).unwrap();
        scip_call! { ffi::SCIPsetStringParam(self.scip.ptr(), param.as_ptr(), value.as_ptr()) };
        Ok(self)
    }

    pub fn set_int_param(self, param: &str, value: i32) -> Result<Self, Retcode> {
        let param = CString::new(param).unwrap();
        scip_call! { ffi::SCIPsetIntParam(self.scip.ptr(), param.as_ptr(), value) };
        Ok(self)
    }

    pub fn set_real_param(self, param: &str, value: f64) -> Result<Self, Retcode> {
        let param = CString::new(param).unwrap();
        scip_call! { ffi::SCIPsetRealParam(self.scip.ptr(), param.as_ptr(), value) };
        Ok(self)
    }

    pub fn hide_output(self) -> Result<Self, Retcode> {
        let muted_model = self.set_int_param("display/verblevel", 0)?;
        Ok(muted_model)
    }

    pub fn set_presolving(self, presolving: ParamSetting) -> Result<Self, Retcode> {
        scip_call! { ffi::SCIPsetPresolving(self.scip.ptr(), presolving.into(), true.into()) };
        Ok(self)
    }

    pub fn set_separating(self, separating: ParamSetting) -> Result<Self, Retcode> {
        scip_call! { ffi::SCIPsetSeparating(self.scip.ptr(), separating.into(), true.into()) };
        Ok(self)
    }

    pub fn set_heuristics(self, heuristics: ParamSetting) -> Result<Self, Retcode> {
        scip_call! { ffi::SCIPsetHeuristics(self.scip.ptr(), heuristics.into(), true.into()) };
        Ok(self)
    }
}

impl Model<PluginsIncluded> {
    pub fn create_prob(&mut self, name: &str) -> Model<ProblemCreated> {
        let name = CString::new(name).unwrap();
        scip_call_expect!(
            ffi::SCIPcreateProbBasic(self.scip.ptr(), name.as_ptr()),
            "Unexpected fail to create problem from state PluginsIncluded"
        );
        self.move_to_state(ProblemCreated)
    }

    pub fn read_prob(&mut self, filename: &str) -> Result<Model<ProblemCreated>, Retcode> {
        let filename = CString::new(filename).unwrap();
        scip_call! { ffi::SCIPreadProb(self.scip.ptr(), filename.as_ptr(), std::ptr::null_mut()) };
        let mut new_model = self.move_to_state(ProblemCreated);
        new_model._set_vars();
        new_model._set_conss();
        Ok(new_model)
    }
}

impl Model<ProblemCreated> {
    pub fn set_obj_sense(self, sense: ObjSense) -> Self {
        scip_call_expect!(
            ffi::SCIPsetObjsense(self.scip.ptr(), sense.into()),
            "Unexpected fail to set objective sense from state ProblemCreated"
        );
        self
    }

    fn _set_vars(&mut self) {
        let n_vars = self.get_n_vars();
        let scip_vars = unsafe { ffi::SCIPgetVars(self.scip.ptr()) };
        for i in 0..n_vars {
            let scip_var = unsafe { *scip_vars.add(i) };
            unsafe {
                ffi::SCIPcaptureVar(self.scip.ptr(), scip_var);
            }
            let var = Variable { raw: scip_var };
            self.vars.insert(var.get_index(), var);
        }
    }

    fn _set_conss(&mut self) {
        let n_conss = self.get_n_conss();
        let scip_conss = unsafe { ffi::SCIPgetConss(self.scip.ptr()) };
        for i in 0..n_conss {
            let scip_cons = unsafe { *scip_conss.add(i) };
            unsafe {
                ffi::SCIPcaptureCons(self.scip.ptr(), scip_cons);
            }
            let cons = Constraint { raw: scip_cons };
            self.conss.push(cons);
        }
    }

    pub fn add_var(&mut self, lb: f64, ub: f64, obj: f64, name: &str, var_type: VarType) -> VarId {
        let var = Variable::new(self.scip.ptr(), lb, ub, obj, name, var_type)
            .expect("Unexpected fail to create variable from state ProblemCreated");
        let var_id = var.get_index();
        self.vars.insert(var_id, var);
        var_id
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
            .map(|var_id| {
                self.vars
                    .get(var_id)
                    .expect(&format!("Variable with id {var_id} was not found"))
            })
            .collect::<Vec<_>>();
        let c_name = CString::new(name).unwrap();
        let mut scip_cons = MaybeUninit::uninit();
        scip_call! { ffi::SCIPcreateConsBasicLinear(
            self.scip.ptr(),
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
            scip_call! { ffi::SCIPaddCoefLinear(self.scip.ptr(), scip_cons, var.raw, coefs[i]) };
        }
        scip_call! { ffi::SCIPaddCons(self.scip.ptr(), scip_cons) };
        let cons = Constraint { raw: scip_cons };
        self.conss.push(cons);
        Ok(())
    }

    pub fn solve(&mut self) -> Model<Solved> {
        scip_call_expect!(
            ffi::SCIPsolve(self.scip.ptr()),
            "Unexpected fail to call solve from state ProblemCreated."
        );
        let mut new_model = self.move_to_state(Solved);
        new_model._set_best_sol();
        new_model
    }
}

impl Model<Solved> {
    fn _set_best_sol(&mut self) {
        let sol = unsafe { ffi::SCIPgetBestSol(self.scip.ptr()) };
        let sol = Solution {
            scip_ptr: self.scip.ptr(),
            raw: sol,
        };
        self.best_sol = Some(sol);
    }

    pub fn get_best_sol(&self) -> Option<Box<&Solution>> {
        self.best_sol.as_ref().map(Box::new)
    }

    pub fn get_obj_val(&self) -> f64 {
        unsafe { ffi::SCIPgetPrimalbound(self.scip.ptr()) }
    }
}

pub trait ModelWithProblem {
    fn get_vars(&self) -> Vec<Box<&Variable>>;
    fn get_var(&self, var_id: VarId) -> Option<Box<&Variable>>;
    fn get_n_vars(&self) -> usize;
    fn get_n_conss(&mut self) -> usize;
    fn get_conss(&mut self) -> &Vec<Constraint>;
}

macro_rules! impl_ModelWithProblem {
    (for $($t:ty),+) => {
        $(impl ModelWithProblem for $t {
            fn get_vars(&self) -> Vec<Box<&Variable>> {
            self.vars.values().map(Box::new).collect()
        }

     fn get_var(&self, var_id: VarId) -> Option<Box<&Variable>> {
        self.vars.get(&var_id).map(Box::new)
    }

    fn get_n_vars(&self) -> usize {
        unsafe { ffi::SCIPgetNVars(self.scip.ptr()) as usize }
    }

    fn get_n_conss(&mut self) -> usize {
        unsafe { ffi::SCIPgetNConss(self.scip.ptr()) as usize }
    }

    fn get_conss(&mut self) -> &Vec<Constraint> {
        &self.conss
    }

        })*
    }
}

impl_ModelWithProblem!(for Model<ProblemCreated>, Model<Solved>);

impl<T> Model<T> {
    fn move_to_state<S>(&mut self, state: S) -> Model<S> {
        Model {
            state,
            scip: mem::take(&mut self.scip),
            vars: mem::take(&mut self.vars),
            conss: mem::take(&mut self.conss),
            best_sol: mem::take(&mut self.best_sol),
        }
    }
    pub fn get_status(&self) -> Status {
        let status = unsafe { ffi::SCIPgetStatus(self.scip.ptr()) };
        Status::from_c_scip_status(status).unwrap()
    }

    pub fn print_version(&mut self) {
        unsafe { ffi::SCIPprintVersion(self.scip.ptr(), std::ptr::null_mut()) };
    }

    pub fn write(&mut self, path: &str, ext: &str) -> Result<(), Retcode> {
        let c_path = CString::new(path).unwrap();
        let c_ext = CString::new(ext).unwrap();
        scip_call! { ffi::SCIPwriteOrigProblem(
            self.scip.ptr(),
            c_path.as_ptr(),
            c_ext.as_ptr(),
            true.into(),
        ) };
        Ok(())
    }
}

impl Default for Model<ProblemCreated> {
    fn default() -> Self {
        Model::new()
            .include_default_plugins()
            .create_prob("problem")
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

    // #[test] call prevented from type system
    // fn call_solve_without_problem() {
    //     assert!(Model::new().solve().is_err());
    // }

    // #[test] call prevented from type system
    // fn call_solve_on_empty_problem() {
    //     assert!(Model::new().solve().is_err());
    // }

    // #[test] call prevented from type system
    // fn solution_without_problem() {
    //     let model = Model::new();
    //     let sol = model.get_best_sol();
    //     assert!(sol.is_none());
    // }

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
        let mut model = Model::new()
            .hide_output()?
            .include_default_plugins()
            .read_prob("data/test/simple.lp")?
            .solve();
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
        let model = Model::new()
            .hide_output()?
            .set_real_param("limits/time", 0.)?
            .include_default_plugins()
            .read_prob("data/test/simple.lp")?
            .solve();
        let status = model.get_status();
        assert_eq!(status, Status::TIMELIMIT);
        Ok(())
    }

    #[test]
    fn add_variable() -> Result<(), Retcode> {
        let mut model = Model::new()
            .hide_output()?
            .include_default_plugins()
            .create_prob("test")
            .set_obj_sense(ObjSense::Maximize);
        let x1_id = model.add_var(0., f64::INFINITY, 3., "x1", VarType::Integer);
        let x2_id = model.add_var(0., f64::INFINITY, 4., "x2", VarType::Continuous);
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

    fn create_model() -> Result<Model<ProblemCreated>, Retcode> {
        let mut model = Model::new()
            .hide_output()?
            .include_default_plugins()
            .create_prob("test")
            .set_obj_sense(ObjSense::Maximize);

        let x1_id = model.add_var(0., f64::INFINITY, 3., "x1", VarType::Integer);
        let x2_id = model.add_var(0., f64::INFINITY, 4., "x2", VarType::Integer);
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

        let solved_model = model.solve();

        let status = solved_model.get_status();
        assert_eq!(status, Status::OPTIMAL);

        let obj_val = solved_model.get_obj_val();
        assert_eq!(obj_val, 200.);

        let sol = solved_model.get_best_sol().unwrap();
        let vars = solved_model.get_vars();
        assert_eq!(vars.len(), 2);
        assert_eq!(sol.get_var_val(&vars[0]), 40.);
        assert_eq!(sol.get_var_val(&vars[1]), 20.);
        println!("print solution");
        Ok(())
    }
}
