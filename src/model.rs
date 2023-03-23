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

#[non_exhaustive]

struct ScipPtr(*mut ffi::SCIP);

impl ScipPtr {
    fn new() -> Self {
        let mut scip_ptr = MaybeUninit::uninit();
        scip_call_panic!(ffi::SCIPcreate(scip_ptr.as_mut_ptr()));
        let scip_ptr = unsafe { scip_ptr.assume_init() };
        ScipPtr(scip_ptr)
    }

    fn set_str_param(&mut self, param: &str, value: &str) -> Result<(), Retcode> {
        let param = CString::new(param).unwrap();
        let value = CString::new(value).unwrap();
        scip_call! { ffi::SCIPsetStringParam(self.0, param.as_ptr(), value.as_ptr()) };
        Ok(())
    }

    fn set_int_param(&mut self, param: &str, value: i32) -> Result<(), Retcode> {
        let param = CString::new(param).unwrap();
        scip_call! { ffi::SCIPsetIntParam(self.0, param.as_ptr(), value) };
        Ok(())
    }

    fn set_real_param(&mut self, param: &str, value: f64) -> Result<(), Retcode> {
        let param = CString::new(param).unwrap();
        scip_call! { ffi::SCIPsetRealParam(self.0, param.as_ptr(), value) };
        Ok(())
    }

    fn set_presolving(&mut self, presolving: ParamSetting) -> Result<(), Retcode> {
        scip_call! { ffi::SCIPsetPresolving(self.0, presolving.into(), true.into()) };
        Ok(())
    }

    fn set_separating(&mut self, separating: ParamSetting) -> Result<(), Retcode> {
        scip_call! { ffi::SCIPsetSeparating(self.0, separating.into(), true.into()) };
        Ok(())
    }

    fn set_heuristics(&mut self, heuristics: ParamSetting) -> Result<(), Retcode> {
        scip_call! { ffi::SCIPsetHeuristics(self.0, heuristics.into(), true.into()) };
        Ok(())
    }

    fn create_prob(&mut self, name: &str) -> Result<(), Retcode> {
        let name = CString::new(name).unwrap();
        scip_call!(ffi::SCIPcreateProbBasic(self.0, name.as_ptr()));
        Ok(())
    }

    fn read_prob(&mut self, filename: &str) -> Result<(), Retcode> {
        let filename = CString::new(filename).unwrap();
        scip_call!(ffi::SCIPreadProb(
            self.0,
            filename.as_ptr(),
            std::ptr::null_mut()
        ));
        Ok(())
    }

    fn set_obj_sense(&mut self, sense: ObjSense) -> Result<(), Retcode> {
        scip_call!(ffi::SCIPsetObjsense(self.0, sense.into()));
        Ok(())
    }

    fn get_n_vars(&self) -> usize {
        unsafe { ffi::SCIPgetNVars(self.0) as usize }
    }

    fn get_n_conss(&self) -> usize {
        unsafe { ffi::SCIPgetNConss(self.0) as usize }
    }

    fn get_status(&self) -> Status {
        let status = unsafe { ffi::SCIPgetStatus(self.0) };
        status.try_into().expect("Unknown SCIP status")
    }

    fn print_version(&self) {
        unsafe { ffi::SCIPprintVersion(self.0, std::ptr::null_mut()) };
    }

    fn write(&self, path: &str, ext: &str) -> Result<(), Retcode> {
        let c_path = CString::new(path).unwrap();
        let c_ext = CString::new(ext).unwrap();
        scip_call! { ffi::SCIPwriteOrigProblem(
            self.0,
            c_path.as_ptr(),
            c_ext.as_ptr(),
            true.into(),
        ) };
        Ok(())
    }

    fn include_default_plugins(&mut self) -> Result<(), Retcode> {
        scip_call!(ffi::SCIPincludeDefaultPlugins(self.0));
        Ok(())
    }

    fn get_vars(&mut self) -> BTreeMap<usize, Variable> {
        // NOTE: this method should only be called once per SCIP instance
        let n_vars = self.get_n_vars();
        let mut vars = BTreeMap::new();
        let scip_vars = unsafe { ffi::SCIPgetVars(self.0) };
        for i in 0..n_vars {
            let scip_var = unsafe { *scip_vars.add(i) };
            unsafe {
                ffi::SCIPcaptureVar(self.0, scip_var);
            }
            let var = Variable { raw: scip_var };
            vars.insert(var.get_index(), var);
        }
        vars
    }

    fn get_conss(&mut self) -> Vec<Constraint> {
        // NOTE: this method should only be called once per SCIP instance
        let n_conss = self.get_n_conss();
        let mut conss = Vec::with_capacity(n_conss);
        let scip_conss = unsafe { ffi::SCIPgetConss(self.0) };
        for i in 0..n_conss {
            let scip_cons = unsafe { *scip_conss.add(i) };
            unsafe {
                ffi::SCIPcaptureCons(self.0, scip_cons);
            }
            let cons = Constraint { raw: scip_cons };
            conss.push(cons);
        }
        conss
    }

    fn solve(&mut self) -> Result<(), Retcode> {
        scip_call!(ffi::SCIPsolve(self.0));
        Ok(())
    }

    fn get_n_sols(&self) -> usize {
        unsafe { ffi::SCIPgetNSols(self.0) as usize }
    }

    fn get_best_sol(&self) -> Solution {
        let sol = unsafe { ffi::SCIPgetBestSol(self.0) };

        Solution {
            scip_ptr: self.0,
            raw: sol,
        }
    }

    fn get_obj_val(&self) -> f64 {
        unsafe { ffi::SCIPgetPrimalbound(self.0) }
    }

    fn create_var(
        &mut self,
        lb: f64,
        ub: f64,
        obj: f64,
        name: &str,
        var_type: VarType,
    ) -> Result<Variable, Retcode> {
        let name = CString::new(name).unwrap();
        let mut var_ptr = MaybeUninit::uninit();
        scip_call! { ffi::SCIPcreateVarBasic(
            self.0,
            var_ptr.as_mut_ptr(),
            name.as_ptr(),
            lb,
            ub,
            obj,
            var_type.into(),
        ) };
        let var_ptr = unsafe { var_ptr.assume_init() };
        scip_call! { ffi::SCIPaddVar(self.0, var_ptr) };
        Ok(Variable { raw: var_ptr })
    }

    fn create_cons(
        &mut self,
        vars: Vec<&Variable>,
        coefs: &[f64],
        lhs: f64,
        rhs: f64,
        name: &str,
    ) -> Result<Constraint, Retcode> {
        assert_eq!(vars.len(), coefs.len());
        let c_name = CString::new(name).unwrap();
        let mut scip_cons = MaybeUninit::uninit();
        scip_call! { ffi::SCIPcreateConsBasicLinear(
            self.0,
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
            scip_call! { ffi::SCIPaddCoefLinear(self.0, scip_cons, var.raw, coefs[i]) };
        }
        scip_call! { ffi::SCIPaddCons(self.0, scip_cons) };
        Ok(Constraint { raw: scip_cons })
    }
}

impl Default for ScipPtr {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ScipPtr {
    fn drop(&mut self) {
        // Rust Model struct keeps at most one copy of each variable and constraint pointers
        // so we need to release them before freeing the SCIP instance

        // first check if we are in a stage where we have variables and constraints
        let scip_stage = unsafe { ffi::SCIPgetStage(self.0) };
        if scip_stage == ffi::SCIP_Stage_SCIP_STAGE_PROBLEM
            || scip_stage == ffi::SCIP_Stage_SCIP_STAGE_TRANSFORMED
            || scip_stage == ffi::SCIP_Stage_SCIP_STAGE_INITPRESOLVE
            || scip_stage == ffi::SCIP_Stage_SCIP_STAGE_PRESOLVING
            || scip_stage == ffi::SCIP_Stage_SCIP_STAGE_EXITPRESOLVE
            || scip_stage == ffi::SCIP_Stage_SCIP_STAGE_PRESOLVED
            || scip_stage == ffi::SCIP_Stage_SCIP_STAGE_INITSOLVE
            || scip_stage == ffi::SCIP_Stage_SCIP_STAGE_SOLVING
            || scip_stage == ffi::SCIP_Stage_SCIP_STAGE_SOLVED
            || scip_stage == ffi::SCIP_Stage_SCIP_STAGE_EXITSOLVE
        {
            // release variables
            let n_vars = unsafe { ffi::SCIPgetNOrigVars(self.0) };
            let vars = unsafe { ffi::SCIPgetOrigVars(self.0) };
            for i in 0..n_vars {
                let mut var = unsafe { *vars.add(i as usize) };
                scip_call_panic!(ffi::SCIPreleaseVar(self.0, &mut var));
            }

            // release constraints
            let n_conss = unsafe { ffi::SCIPgetNOrigConss(self.0) };
            let conss = unsafe { ffi::SCIPgetOrigConss(self.0) };
            for i in 0..n_conss {
                let mut cons = unsafe { *conss.add(i as usize) };
                scip_call_panic!(ffi::SCIPreleaseCons(self.0, &mut cons));
            }
        }

        // free SCIP instance
        unsafe { ffi::SCIPfree(&mut self.0) };
    }
}

#[non_exhaustive]
pub struct Model<State> {
    scip: ScipPtr,
    state: State,
}

pub struct Unsolved;
pub struct PluginsIncluded;
pub struct ProblemCreated {
    pub(crate) vars: BTreeMap<VarId, Variable>,
    pub(crate) conss: Vec<Constraint>,
}

pub struct Solved {
    pub(crate) vars: BTreeMap<VarId, Variable>,
    pub(crate) conss: Vec<Constraint>,
    pub(crate) best_sol: Option<Solution>,
}

impl Model<Unsolved> {
    pub fn new() -> Self {
        Self::try_new().expect("Failed to create SCIP instance")
    }

    pub fn try_new() -> Result<Self, Retcode> {
        let scip_ptr = ScipPtr::new();
        Ok(Model {
            scip: scip_ptr,
            state: Unsolved {},
        })
    }

    pub fn include_default_plugins(mut self) -> Model<PluginsIncluded> {
        self.scip
            .include_default_plugins()
            .expect("Failed to include default plugins");
        Model {
            scip: self.scip,
            state: PluginsIncluded {},
        }
    }

    pub fn set_str_param(mut self, param: &str, value: &str) -> Result<Self, Retcode> {
        self.scip.set_str_param(param, value)?;
        Ok(self)
    }

    pub fn set_int_param(mut self, param: &str, value: i32) -> Result<Self, Retcode> {
        self.scip.set_int_param(param, value)?;
        Ok(self)
    }

    pub fn set_real_param(mut self, param: &str, value: f64) -> Result<Self, Retcode> {
        self.scip.set_real_param(param, value)?;
        Ok(self)
    }

    pub fn set_presolving(mut self, presolving: ParamSetting) -> Self {
        self.scip
            .set_presolving(presolving)
            .expect("Failed to set presolving with valid value");
        self
    }

    pub fn set_separating(mut self, separating: ParamSetting) -> Self {
        self.scip
            .set_separating(separating)
            .expect("Failed to set separating with valid value");
        self
    }

    pub fn set_heuristics(mut self, heuristics: ParamSetting) -> Self {
        self.scip
            .set_heuristics(heuristics)
            .expect("Failed to set heuristics with valid value");
        self
    }
}

impl Model<PluginsIncluded> {
    pub fn create_prob(mut self, name: &str) -> Model<ProblemCreated> {
        self.scip
            .create_prob(name)
            .expect("Failed to create problem in state PluginsIncluded");
        Model {
            scip: self.scip,
            state: ProblemCreated {
                vars: BTreeMap::new(),
                conss: Vec::new(),
            },
        }
    }

    pub fn read_prob(mut self, filename: &str) -> Result<Model<ProblemCreated>, Retcode> {
        self.scip.read_prob(filename)?;
        let vars = self.scip.get_vars();
        let conss = self.scip.get_conss();
        let new_model = Model {
            scip: self.scip,
            state: ProblemCreated { vars, conss },
        };
        Ok(new_model)
    }
}

impl Model<ProblemCreated> {
    pub fn set_obj_sense(mut self, sense: ObjSense) -> Self {
        self.scip
            .set_obj_sense(sense)
            .expect("Failed to set objective sense in state ProblemCreated");
        self
    }

    pub fn add_var(&mut self, lb: f64, ub: f64, obj: f64, name: &str, var_type: VarType) -> VarId {
        let var = self
            .scip
            .create_var(lb, ub, obj, name, var_type)
            .expect("Failed to create variable in state ProblemCreated");
        let var_id = var.get_index();
        self.state.vars.insert(var_id, var);
        var_id
    }

    pub fn add_cons(&mut self, var_ids: &[VarId], coefs: &[f64], lhs: f64, rhs: f64, name: &str) {
        assert_eq!(var_ids.len(), coefs.len());
        let vars_in_cons = var_ids
            .iter()
            .map(|var_id| {
                self.state
                    .vars
                    .get(var_id)
                    .unwrap_or_else(|| panic!("Variable with id {var_id} was not found"))
            })
            .collect::<Vec<_>>();
        let cons = self
            .scip
            .create_cons(vars_in_cons, coefs, lhs, rhs, name)
            .expect("Failed to create constraint in state ProblemCreated");
        self.state.conss.push(cons);
    }

    pub fn solve(mut self) -> Model<Solved> {
        self.scip
            .solve()
            .expect("Failed to solve problem in state ProblemCreated");
        let mut new_model = Model {
            scip: self.scip,
            state: Solved {
                vars: self.state.vars,
                conss: self.state.conss,
                best_sol: None,
            },
        };
        new_model._set_best_sol();
        new_model
    }
}

impl Model<Solved> {
    fn _set_best_sol(&mut self) {
        if self.scip.get_n_sols() > 0 {
            self.state.best_sol = Some(self.scip.get_best_sol());
        }
    }

    pub fn get_best_sol(&self) -> Option<Box<&Solution>> {
        self.state.best_sol.as_ref().map(Box::new)
    }

    pub fn get_n_sols(&self) -> usize {
        self.scip.get_n_sols()
    }

    pub fn get_obj_val(&self) -> f64 {
        self.scip.get_obj_val()
    }
}

pub trait ModelWithProblem {
    fn get_vars(&self) -> Vec<Box<&Variable>>;
    fn get_var(&self, var_id: VarId) -> Option<Box<&Variable>>;
    fn get_n_vars(&self) -> usize;
    fn get_n_conss(&mut self) -> usize;
    fn get_conss(&mut self) -> &Vec<Constraint>;
    fn write(&self, path: &str, ext: &str) -> Result<(), Retcode>;
}

macro_rules! impl_ModelWithProblem {
    (for $($t:ty),+) => {
        $(impl ModelWithProblem for $t {
            fn get_vars(&self) -> Vec<Box<&Variable>> {
            self.state.vars.values().map(Box::new).collect()
        }

    fn get_n_vars(&self) -> usize {
        self.scip.get_n_vars()
    }

    fn get_var(&self, var_id: VarId) -> Option<Box<&Variable>> {
        self.state.vars.get(&var_id).map(Box::new)
    }

    fn get_n_conss(&mut self) -> usize {
        self.scip.get_n_conss()
    }

    fn get_conss(&mut self) -> &Vec<Constraint> {
        &self.state.conss
    }

    fn write(&self, path: &str, ext: &str) -> Result<(), Retcode> {
        self.scip.write(path, ext)?;
        Ok(())
    }

        })*
    }
}

impl_ModelWithProblem!(for Model<ProblemCreated>, Model<Solved>);

impl<T> Model<T> {
    pub fn get_status(&self) -> Status {
        self.scip.get_status()
    }

    pub fn print_version(&self) {
        self.scip.print_version()
    }

    pub fn hide_output(mut self) -> Self {
        self.scip
            .set_int_param("display/verblevel", 0)
            .expect("Failed to set display/verblevel to 0");
        self
    }
}

impl Default for Model<ProblemCreated> {
    fn default() -> Self {
        Model::new()
            .include_default_plugins()
            .create_prob("problem")
    }
}
#[derive(Debug)]
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
#[derive(Debug)]
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
    fn solve_from_lp_file() {
        let mut model = Model::new()
            .hide_output()
            .include_default_plugins()
            .read_prob("data/test/simple.lp")
            .unwrap()
            .solve();
        let status = model.get_status();
        assert_eq!(status, Status::Optimal);

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
    }

    #[test]
    fn set_time_limit() {
        let model = Model::new()
            .hide_output()
            .set_real_param("limits/time", 0.)
            .unwrap()
            .include_default_plugins()
            .read_prob("data/test/simple.lp")
            .unwrap()
            .solve();
        let status = model.get_status();
        assert_eq!(status, Status::TimeLimit);
    }

    #[test]
    fn add_variable() {
        let mut model = Model::new()
            .hide_output()
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
    }

    fn create_model() -> Model<ProblemCreated> {
        let mut model = Model::new()
            .hide_output()
            .include_default_plugins()
            .create_prob("test")
            .set_obj_sense(ObjSense::Maximize);

        let x1_id = model.add_var(0., f64::INFINITY, 3., "x1", VarType::Integer);
        let x2_id = model.add_var(0., f64::INFINITY, 4., "x2", VarType::Integer);
        model.add_cons(&[x1_id, x2_id], &[2., 1.], -f64::INFINITY, 100., "c1");
        model.add_cons(&[x1_id, x2_id], &[1., 2.], -f64::INFINITY, 80., "c2");

        model
    }

    #[test]
    fn build_model_with_functions() {
        let mut model = create_model();
        assert_eq!(model.get_vars().len(), 2);
        assert_eq!(model.get_n_conss(), 2);

        let conss = model.get_conss();
        assert_eq!(conss.len(), 2);
        assert_eq!(conss[0].get_name(), "c1");
        assert_eq!(conss[1].get_name(), "c2");

        let solved_model = model.solve();

        let status = solved_model.get_status();
        assert_eq!(status, Status::Optimal);

        let obj_val = solved_model.get_obj_val();
        assert_eq!(obj_val, 200.);

        let sol = solved_model.get_best_sol().unwrap();
        let vars = solved_model.get_vars();
        assert_eq!(vars.len(), 2);
        assert_eq!(sol.get_var_val(&vars[0]), 40.);
        assert_eq!(sol.get_var_val(&vars[1]), 20.);
    }

    #[test]
    fn unbounded_model() {
        let mut model = Model::default()
            .set_obj_sense(ObjSense::Maximize)
            .hide_output();

        model.add_var(0., f64::INFINITY, 1., "x1", VarType::Integer);
        model.add_var(0., f64::INFINITY, 1., "x2", VarType::Integer);

        let solved_model = model.solve();

        let status = solved_model.get_status();
        assert_eq!(status, Status::Unbounded);

        let sol = solved_model.get_best_sol();
        assert!(sol.is_some());
    }

    #[test]
    fn infeasible_model() {
        let mut model = Model::default()
            .set_obj_sense(ObjSense::Maximize)
            .hide_output();

        let var_id = model.add_var(0., 1., 1., "x1", VarType::Integer);

        model.add_cons(&[var_id], &[1.], -f64::INFINITY, -1., "c1");

        let solved_model = model.solve();

        let status = solved_model.get_status();
        assert_eq!(status, Status::Infeasible);

        let sol = solved_model.get_best_sol();
        assert!(sol.is_none());
    }
}
