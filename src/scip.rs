use crate::branchrule::{BranchRule, BranchingCandidate};
use crate::node::Node;
use crate::pricer::{Pricer, PricerResultState};
use crate::{
    ffi, scip_call_panic, BranchingResult, Conshdlr, Constraint, Event, Eventhdlr, HeurResult,
    Model, ObjSense, ParamSetting, Retcode, Row, SCIPBranchRule, SCIPConshdlr, SCIPEventhdlr,
    SCIPPricer, SCIPSeparator, Separator, Solution, Solving, Status, VarType, Variable,
};
use crate::{scip_call, HeurTiming, Heuristic};
use core::panic;
use scip_sys::{
    SCIP_Cons, SCIP_Var, Scip, SCIP, SCIP_CONS, SCIP_CONSHDLR, SCIP_LOCKTYPE, SCIP_NODE,
    SCIP_RESULT, SCIP_RETCODE, SCIP_SOL,
};
use std::collections::BTreeMap;
use std::ffi::{c_int, CStr, CString};
use std::mem::MaybeUninit;
use std::rc::Rc;

#[non_exhaustive]
#[derive(Debug)]
pub struct ScipPtr {
    /// Pointer to the SCIP instance
    pub(crate) raw: *mut ffi::SCIP,
    /// If true, the SCIP instance will not be freed when the ScipPtr is dropped
    pub(crate) weak: bool,
    /// Variables added during solving (to be released after solving)
    vars_added_in_solving: Vec<*mut ffi::SCIP_VAR>,
}

impl ScipPtr {
    pub(crate) fn new() -> Self {
        let mut scip_ptr = MaybeUninit::uninit();
        scip_call_panic!(ffi::SCIPcreate(scip_ptr.as_mut_ptr()));
        let scip_ptr = unsafe { scip_ptr.assume_init() };
        ScipPtr {
            raw: scip_ptr,
            weak: false,
            vars_added_in_solving: Vec::new(),
        }
    }

    pub(crate) fn from_raw(raw: *mut ffi::SCIP, weak: bool) -> Self {
        ScipPtr {
            raw,
            weak,
            vars_added_in_solving: Vec::new(),
        }
    }

    pub(crate) fn set_str_param(&self, param: &str, value: &str) -> Result<(), Retcode> {
        let param = CString::new(param).unwrap();
        let value = CString::new(value).unwrap();
        scip_call! { ffi::SCIPsetStringParam(self.raw, param.as_ptr(), value.as_ptr()) };
        Ok(())
    }

    pub(crate) fn str_param(&self, param: &str) -> Result<&str, Retcode> {
        let param = CString::new(param).unwrap();
        let mut value_ptr = MaybeUninit::uninit();
        scip_call! { ffi::SCIPgetStringParam(self.raw, param.as_ptr(), value_ptr.as_mut_ptr()) };
        let value_ptr = unsafe { value_ptr.assume_init() };
        let value = unsafe { CStr::from_ptr(value_ptr) };
        Ok(value.to_str().unwrap())
    }

    pub(crate) fn set_bool_param(&self, param: &str, value: bool) -> Result<(), Retcode> {
        let param = CString::new(param).unwrap();
        scip_call! { ffi::SCIPsetBoolParam(self.raw, param.as_ptr(), if value { 1u32 } else { 0u32 }) };
        Ok(())
    }

    pub(crate) fn bool_param(&self, param: &str) -> Result<bool, Retcode> {
        let param = CString::new(param).unwrap();
        let mut value = MaybeUninit::uninit();
        scip_call! { ffi::SCIPgetBoolParam(self.raw, param.as_ptr(), value.as_mut_ptr()) };
        let value = unsafe { value.assume_init() };
        Ok(value != 0)
    }

    pub(crate) fn set_int_param(&self, param: &str, value: i32) -> Result<(), Retcode> {
        let param = CString::new(param).unwrap();
        scip_call! { ffi::SCIPsetIntParam(self.raw, param.as_ptr(), value) };
        Ok(())
    }

    pub(crate) fn int_param(&self, param: &str) -> Result<i32, Retcode> {
        let param = CString::new(param).unwrap();
        let mut value = MaybeUninit::uninit();
        scip_call! { ffi::SCIPgetIntParam(self.raw, param.as_ptr(), value.as_mut_ptr()) };
        let value = unsafe { value.assume_init() };
        Ok(value)
    }

    pub(crate) fn set_longint_param(&self, param: &str, value: i64) -> Result<(), Retcode> {
        let param = CString::new(param).unwrap();
        scip_call! { ffi::SCIPsetLongintParam(self.raw, param.as_ptr(), value) };
        Ok(())
    }

    pub(crate) fn longint_param(&self, param: &str) -> Result<i64, Retcode> {
        let param = CString::new(param).unwrap();
        let mut value = MaybeUninit::uninit();
        scip_call! { ffi::SCIPgetLongintParam(self.raw, param.as_ptr(), value.as_mut_ptr()) };
        let value = unsafe { value.assume_init() };
        Ok(value)
    }

    pub(crate) fn set_real_param(&self, param: &str, value: f64) -> Result<(), Retcode> {
        let param = CString::new(param).unwrap();
        scip_call! { ffi::SCIPsetRealParam(self.raw, param.as_ptr(), value) };
        Ok(())
    }

    pub(crate) fn real_param(&self, param: &str) -> Result<f64, Retcode> {
        let param = CString::new(param).unwrap();
        let mut value = MaybeUninit::uninit();
        scip_call! { ffi::SCIPgetRealParam(self.raw, param.as_ptr(), value.as_mut_ptr()) };
        let value = unsafe { value.assume_init() };
        Ok(value)
    }

    pub(crate) fn set_presolving(&self, presolving: ParamSetting) -> Result<(), Retcode> {
        scip_call! { ffi::SCIPsetPresolving(self.raw, presolving.into(), true.into()) };
        Ok(())
    }

    pub(crate) fn set_separating(&self, separating: ParamSetting) -> Result<(), Retcode> {
        scip_call! { ffi::SCIPsetSeparating(self.raw, separating.into(), true.into()) };
        Ok(())
    }

    pub(crate) fn set_heuristics(&self, heuristics: ParamSetting) -> Result<(), Retcode> {
        scip_call! { ffi::SCIPsetHeuristics(self.raw, heuristics.into(), true.into()) };
        Ok(())
    }

    pub(crate) fn create_prob(&self, name: &str) -> Result<(), Retcode> {
        let name = CString::new(name).unwrap();
        scip_call!(ffi::SCIPcreateProbBasic(self.raw, name.as_ptr()));
        Ok(())
    }

    pub(crate) fn read_prob(&self, filename: &str) -> Result<(), Retcode> {
        let filename = CString::new(filename).unwrap();
        scip_call!(ffi::SCIPreadProb(
            self.raw,
            filename.as_ptr(),
            std::ptr::null_mut()
        ));
        // capture vars and cons since they were not created by the user (and SCIP will free them later)
        self.vars(false, true);
        self.conss(true);
        Ok(())
    }

    pub(crate) fn set_obj_sense(&self, sense: ObjSense) -> Result<(), Retcode> {
        scip_call!(ffi::SCIPsetObjsense(self.raw, sense.into()));
        Ok(())
    }

    pub(crate) fn set_obj_integral(&self) -> Result<(), Retcode> {
        scip_call!(ffi::SCIPsetObjIntegral(self.raw));
        Ok(())
    }

    pub(crate) fn n_vars(&self) -> usize {
        unsafe { ffi::SCIPgetNVars(self.raw) as usize }
    }

    pub(crate) fn n_conss(&self) -> usize {
        unsafe { ffi::SCIPgetNConss(self.raw) as usize }
    }

    pub(crate) fn find_cons(&self, name: &str) -> Option<*mut SCIP_Cons> {
        let c_name = CString::new(name).unwrap();
        let scip_cons = unsafe { ffi::SCIPfindCons(self.raw, c_name.as_ptr()) };
        if scip_cons.is_null() {
            None
        } else {
            Some(scip_cons)
        }
    }

    pub(crate) fn get_transformed_cons(
        &self,
        cons: &Constraint,
    ) -> Result<Option<*mut SCIP_Cons>, Retcode> {
        let mut transformed_cons = std::mem::MaybeUninit::<*mut ffi::SCIP_Cons>::uninit();
        scip_call! {
            ffi::SCIPgetTransformedCons(self.raw, cons.raw, transformed_cons.as_mut_ptr())
        };
        let ptr = unsafe { transformed_cons.assume_init() };
        if ptr.is_null() {
            Ok(None)
        } else {
            Ok(Some(ptr))
        }
    }

    pub(crate) fn status(&self) -> Status {
        let status = unsafe { ffi::SCIPgetStatus(self.raw) };
        status.into()
    }

    pub(crate) fn print_version(&self) {
        unsafe { ffi::SCIPprintVersion(self.raw, std::ptr::null_mut()) };
    }

    pub(crate) fn write(&self, path: &str, ext: &str) -> Result<(), Retcode> {
        let c_path = CString::new(path).unwrap();
        let c_ext = CString::new(ext).unwrap();
        scip_call! { ffi::SCIPwriteOrigProblem(
            self.raw,
            c_path.as_ptr(),
            c_ext.as_ptr(),
            true.into(),
        ) };
        Ok(())
    }

    pub(crate) fn include_default_plugins(&self) -> Result<(), Retcode> {
        scip_call!(ffi::SCIPincludeDefaultPlugins(self.raw));
        Ok(())
    }

    pub(crate) fn vars(&self, original: bool, capture: bool) -> BTreeMap<usize, *mut SCIP_Var> {
        // NOTE: this method should only be called once per SCIP instance
        let n_vars = self.n_vars();
        let mut vars = BTreeMap::new();
        let scip_vars = if original {
            unsafe { ffi::SCIPgetOrigVars(self.raw) }
        } else {
            unsafe { ffi::SCIPgetVars(self.raw) }
        };
        for i in 0..n_vars {
            let scip_var = unsafe { *scip_vars.add(i) };
            if capture {
                unsafe {
                    ffi::SCIPcaptureVar(self.raw, scip_var);
                }
            }
            let var = scip_var;
            let var_id = unsafe { ffi::SCIPvarGetIndex(var) } as usize;
            vars.insert(var_id, var);
        }
        vars
    }

    pub(crate) fn conss(&self, capture: bool) -> Vec<*mut SCIP_Cons> {
        // NOTE: this method should only be called once per SCIP instance
        let n_conss = self.n_conss();
        let mut conss = Vec::with_capacity(n_conss);
        let scip_conss = unsafe { ffi::SCIPgetConss(self.raw) };
        for i in 0..n_conss {
            let scip_cons = unsafe { *scip_conss.add(i) };
            if capture {
                unsafe {
                    ffi::SCIPcaptureCons(self.raw, scip_cons);
                }
            }
            conss.push(scip_cons);
        }
        conss
    }

    pub(crate) fn solve(&self) -> Result<(), Retcode> {
        scip_call!(ffi::SCIPsolve(self.raw));
        Ok(())
    }

    pub(crate) fn n_sols(&self) -> usize {
        unsafe { ffi::SCIPgetNSols(self.raw) as usize }
    }

    pub(crate) fn best_sol(&self) -> Option<*mut SCIP_SOL> {
        if self.n_sols() == 0 {
            return None;
        }
        Some(unsafe { ffi::SCIPgetBestSol(self.raw) })
    }

    pub(crate) fn obj_val(&self) -> f64 {
        unsafe { ffi::SCIPgetPrimalbound(self.raw) }
    }

    pub(crate) fn best_bound(&self) -> f64 {
        unsafe { ffi::SCIPgetDualbound(self.raw) }
    }

    pub(crate) fn create_var(
        &self,
        lb: f64,
        ub: f64,
        obj: f64,
        name: &str,
        var_type: VarType,
    ) -> Result<*mut SCIP_Var, Retcode> {
        let name = CString::new(name).unwrap();
        let mut var_ptr = MaybeUninit::uninit();
        scip_call! { ffi::SCIPcreateVarBasic(
            self.raw,
            var_ptr.as_mut_ptr(),
            name.as_ptr(),
            lb,
            ub,
            obj,
            var_type.into(),
        ) };
        let var_ptr = unsafe { var_ptr.assume_init() };
        scip_call! { ffi::SCIPaddVar(self.raw, var_ptr) };
        Ok(var_ptr)
    }

    pub(crate) fn create_var_solving(
        &self,
        lb: f64,
        ub: f64,
        obj: f64,
        name: &str,
        var_type: VarType,
    ) -> Result<*mut SCIP_Var, Retcode> {
        let name = CString::new(name).unwrap();
        let mut var_ptr = MaybeUninit::uninit();
        scip_call! { ffi::SCIPcreateVarBasic(
            self.raw,
            var_ptr.as_mut_ptr(),
            name.as_ptr(),
            lb,
            ub,
            obj,
            var_type.into(),
        ) };
        let mut var_ptr = unsafe { var_ptr.assume_init() };
        scip_call! { ffi::SCIPaddVar(self.raw, var_ptr) }
        let mut transformed_var = MaybeUninit::uninit();
        scip_call! { ffi::SCIPgetTransformedVar(self.raw, var_ptr, transformed_var.as_mut_ptr()) };
        let trans_var_ptr = unsafe { transformed_var.assume_init() };
        scip_call! { ffi::SCIPreleaseVar(self.raw, &mut var_ptr) };
        Ok(trans_var_ptr)
    }

    pub(crate) fn is_lp_constructed(&self) -> bool {
        unsafe { ffi::SCIPisLPConstructed(self.raw) != 0 }
    }

    pub(crate) fn construct_lp(&self) -> Result<Option<bool>, Retcode> {
        let mut cutoff = 0;
        scip_call! { ffi::SCIPconstructLP(self.raw, &mut cutoff) }
        Ok(Some(cutoff != 0))
    }

    pub(crate) fn create_priced_var(
        &self,
        lb: f64,
        ub: f64,
        obj: f64,
        name: &str,
        var_type: VarType,
    ) -> Result<*mut SCIP_Var, Retcode> {
        let name = CString::new(name).unwrap();
        let mut var_ptr = MaybeUninit::uninit();
        scip_call! { ffi::SCIPcreateVarBasic(
            self.raw,
            var_ptr.as_mut_ptr(),
            name.as_ptr(),
            lb,
            ub,
            obj,
            var_type.into(),
        ) };
        let mut var_ptr = unsafe { var_ptr.assume_init() };
        scip_call! { ffi::SCIPaddPricedVar(self.raw, var_ptr, 1.0) }; // 1.0 is used as a default score for now
        let mut transformed_var = MaybeUninit::uninit();
        scip_call! { ffi::SCIPgetTransformedVar(self.raw, var_ptr, transformed_var.as_mut_ptr()) };
        let trans_var_ptr = unsafe { transformed_var.assume_init() };
        scip_call! { ffi::SCIPreleaseVar(self.raw, &mut var_ptr) };
        Ok(trans_var_ptr)
    }

    pub(crate) fn create_cons(
        &self,
        node: Option<&Node>,
        vars: Vec<&Variable>,
        coefs: &[f64],
        lhs: f64,
        rhs: f64,
        name: &str,
        local: bool,
    ) -> Result<*mut SCIP_Cons, Retcode> {
        assert_eq!(vars.len(), coefs.len());
        let c_name = CString::new(name).unwrap();
        let mut scip_cons = MaybeUninit::uninit();
        scip_call! { ffi::SCIPcreateConsBasicLinear(
            self.raw,
            scip_cons.as_mut_ptr(),
            c_name.as_ptr(),
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            lhs,
            rhs,
        ) };
        let mut scip_cons = unsafe { scip_cons.assume_init() };
        for (i, var) in vars.iter().enumerate() {
            scip_call! { ffi::SCIPaddCoefLinear(self.raw, scip_cons, var.raw, coefs[i]) };
        }
        if local {
            if node.is_none() {
                // adding to current node
                scip_call! { ffi::SCIPaddConsLocal(self.raw, scip_cons, std::ptr::null_mut()) };
            } else {
                // adding to given node
                scip_call! { ffi::SCIPaddConsNode(self.raw, node.unwrap().raw, scip_cons, std::ptr::null_mut()) };
            }
        } else {
            scip_call! { ffi::SCIPaddCons(self.raw, scip_cons) };
        }

        let stage = unsafe { ffi::SCIPgetStage(self.raw) };
        if stage == ffi::SCIP_Stage_SCIP_STAGE_SOLVING {
            scip_call! { ffi::SCIPreleaseCons(self.raw, &mut scip_cons) };
        }
        Ok(scip_cons)
    }

    /// Create set partitioning constraint
    pub(crate) fn create_cons_set_part(
        &self,
        vars: Vec<&Variable>,
        name: &str,
    ) -> Result<*mut SCIP_Cons, Retcode> {
        let c_name = CString::new(name).unwrap();
        let mut scip_cons = MaybeUninit::uninit();
        scip_call! { ffi::SCIPcreateConsBasicSetpart(
            self.raw,
            scip_cons.as_mut_ptr(),
            c_name.as_ptr(),
            0,
            std::ptr::null_mut(),
        ) };
        let scip_cons = unsafe { scip_cons.assume_init() };
        for var in vars.iter() {
            scip_call! { ffi::SCIPaddCoefSetppc(self.raw, scip_cons, var.raw) };
        }
        scip_call! { ffi::SCIPaddCons(self.raw, scip_cons) };
        Ok(scip_cons)
    }

    /// Create set cover constraint
    pub(crate) fn create_cons_set_cover(
        &self,
        vars: Vec<&Variable>,
        name: &str,
    ) -> Result<*mut SCIP_Cons, Retcode> {
        let c_name = CString::new(name).unwrap();
        let mut scip_cons = MaybeUninit::uninit();
        scip_call! { ffi::SCIPcreateConsBasicSetcover(
            self.raw,
            scip_cons.as_mut_ptr(),
            c_name.as_ptr(),
            0,
            std::ptr::null_mut(),
        ) };
        let scip_cons = unsafe { scip_cons.assume_init() };
        for var in vars.iter() {
            scip_call! { ffi::SCIPaddCoefSetppc(self.raw, scip_cons, var.raw) };
        }
        scip_call! { ffi::SCIPaddCons(self.raw, scip_cons) };
        Ok(scip_cons)
    }

    pub(crate) fn create_cons_quadratic(
        &self,
        lin_vars: Vec<&Variable>,
        lin_coefs: &mut [f64],
        quad_vars_1: Vec<&Variable>,
        quad_vars_2: Vec<&Variable>,
        quad_coefs: &mut [f64],
        lhs: f64,
        rhs: f64,
        name: &str,
    ) -> Result<*mut SCIP_Cons, Retcode> {
        assert_eq!(lin_vars.len(), lin_coefs.len());
        assert!(
            lin_vars.len() <= c_int::MAX as usize,
            "Number of linear variables exceeds SCIP capabilities"
        );
        assert_eq!(quad_vars_1.len(), quad_vars_2.len());
        assert_eq!(quad_vars_1.len(), quad_coefs.len());
        assert!(
            quad_vars_1.len() <= c_int::MAX as usize,
            "Number of quadratic terms exceeds SCIP capabilities"
        );

        let c_name = CString::new(name).unwrap();
        let mut scip_cons = MaybeUninit::uninit();

        let get_ptrs = |vars: Vec<&Variable>| {
            vars.into_iter()
                .map(|var_rc| var_rc.raw)
                .collect::<Vec<_>>()
        };
        let mut lin_var_ptrs = get_ptrs(lin_vars);
        let mut quad_vars_1_ptrs = get_ptrs(quad_vars_1);
        let mut quad_vars_2_ptrs = get_ptrs(quad_vars_2);
        scip_call! { ffi::SCIPcreateConsBasicQuadraticNonlinear(
            self.raw,
            scip_cons.as_mut_ptr(),
            c_name.as_ptr(),
            lin_var_ptrs.len() as c_int,
            lin_var_ptrs.as_mut_ptr(),
            lin_coefs.as_mut_ptr(),
            quad_vars_1_ptrs.len() as c_int,
            quad_vars_1_ptrs.as_mut_ptr(),
            quad_vars_2_ptrs.as_mut_ptr(),
            quad_coefs.as_mut_ptr(),
            lhs,
            rhs,
        ) };

        let scip_cons = unsafe { scip_cons.assume_init() };
        scip_call! { ffi::SCIPaddCons(self.raw, scip_cons) };
        Ok(scip_cons)
    }

    /// Create set packing constraint
    pub(crate) fn create_cons_set_pack(
        &self,
        vars: Vec<&Variable>,
        name: &str,
    ) -> Result<*mut SCIP_Cons, Retcode> {
        let c_name = CString::new(name).unwrap();
        let mut scip_cons = MaybeUninit::uninit();
        scip_call! { ffi::SCIPcreateConsBasicSetpack(
            self.raw,
            scip_cons.as_mut_ptr(),
            c_name.as_ptr(),
            0,
            std::ptr::null_mut(),
        ) };
        let scip_cons = unsafe { scip_cons.assume_init() };
        for var in vars.iter() {
            scip_call! { ffi::SCIPaddCoefSetppc(self.raw, scip_cons, var.raw) };
        }
        scip_call! { ffi::SCIPaddCons(self.raw, scip_cons) };
        Ok(scip_cons)
    }

    /// Create cardinality constraint
    pub(crate) fn create_cons_cardinality(
        &self,
        vars: Vec<&Variable>,
        cardinality: usize,
        name: &str,
    ) -> Result<*mut SCIP_Cons, Retcode> {
        let c_name = CString::new(name).unwrap();
        let mut scip_cons = MaybeUninit::uninit();
        scip_call! { ffi::SCIPcreateConsBasicCardinality(
            self.raw,
            scip_cons.as_mut_ptr(),
            c_name.as_ptr(),
            0,
            std::ptr::null_mut(),
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        ) };
        let scip_cons = unsafe { scip_cons.assume_init() };
        for (ind, var) in vars.iter().enumerate() {
            scip_call! { ffi::SCIPaddVarCardinality(self.raw, scip_cons, var.raw, std::ptr::null_mut(), ind as f64) };
        }
        scip_call! { ffi:: SCIPchgCardvalCardinality(self.raw, scip_cons, cardinality as i32) };
        scip_call! { ffi::SCIPaddCons(self.raw, scip_cons) };
        Ok(scip_cons)
    }

    /// Get number of constraints added in node
    pub(crate) fn node_get_n_added_conss(&self, node: &Node) -> usize {
        unsafe { ffi::SCIPnodeGetNAddedConss(node.raw) as usize }
    }

    pub(crate) unsafe fn var_from_id(scip: *mut Scip, var_prob_id: usize) -> Option<*mut SCIP_Var> {
        let n_vars = ffi::SCIPgetNVars(scip) as usize;
        let var = *ffi::SCIPgetVars(scip).add(var_prob_id);
        if var_prob_id >= n_vars {
            None
        } else {
            Some(var)
        }
    }

    /// Create indicator constraint
    pub(crate) fn create_cons_indicator(
        &self,
        bin_var: &Variable,
        vars: Vec<&Variable>,
        coefs: &mut [f64],
        rhs: f64,
        name: &str,
    ) -> Result<*mut SCIP_Cons, Retcode> {
        assert_eq!(vars.len(), coefs.len());
        let c_name = CString::new(name).unwrap();
        let mut scip_cons = MaybeUninit::uninit();

        scip_call! { ffi::SCIPcreateConsBasicIndicator(
            self.raw,
            scip_cons.as_mut_ptr(),
            c_name.as_ptr(),
            bin_var.raw,
            vars.len() as c_int,
            (vars.into_iter()
              .map(|var_rc| var_rc.raw)
                .collect::<Vec<_>>()).as_mut_ptr(),
            coefs.as_mut_ptr(),
            rhs,
        ) };

        let scip_cons = unsafe { scip_cons.assume_init() };
        scip_call! { ffi::SCIPaddCons(self.raw, scip_cons) };
        Ok(scip_cons)
    }

    /// Create solution
    pub(crate) fn create_sol(&self, original: bool) -> Result<*mut SCIP_SOL, Retcode> {
        let mut sol = MaybeUninit::uninit();
        if original {
            scip_call! { ffi::SCIPcreateOrigSol(self.raw, sol.as_mut_ptr(), std::ptr::null_mut()) }
        } else {
            scip_call! { ffi::SCIPcreateSol(self.raw, sol.as_mut_ptr(), std::ptr::null_mut()) }
        }
        let sol = unsafe { sol.assume_init() };
        assert!(!sol.is_null());
        Ok(sol)
    }

    /// Add coefficient to set packing/partitioning/covering constraint
    pub(crate) fn add_cons_coef_setppc(
        &self,
        cons: &Constraint,
        var: &Variable,
    ) -> Result<(), Retcode> {
        scip_call! { ffi::SCIPaddCoefSetppc(self.raw, cons.raw, var.raw) };
        Ok(())
    }

    pub(crate) unsafe fn lp_branching_cands(
        scip: *mut ffi::SCIP,
    ) -> Vec<(*mut SCIP_Var, f64, f64)> {
        let mut lpcands = MaybeUninit::uninit();
        let mut lpcandssol = MaybeUninit::uninit();
        // let mut lpcandsfrac = MaybeUninit::uninit();
        let mut nlpcands = MaybeUninit::uninit();
        // let mut npriolpcands = MaybeUninit::uninit();
        let mut nfracimplvars = MaybeUninit::uninit();
        unsafe {
            ffi::SCIPgetLPBranchCands(
                scip,
                lpcands.as_mut_ptr(),
                lpcandssol.as_mut_ptr(),
                std::ptr::null_mut(),
                nlpcands.as_mut_ptr(),
                std::ptr::null_mut(),
                nfracimplvars.as_mut_ptr(),
            );
        }
        let lpcands = unsafe { lpcands.assume_init() };
        let lpcandssol = unsafe { lpcandssol.assume_init() };
        // let lpcandsfrac = unsafe { lpcandsfrac.assume_init() };
        let nlpcands = unsafe { nlpcands.assume_init() };
        // let npriolpcands = unsafe { npriolpcands.assume_init() };
        let mut cands = Vec::with_capacity(nlpcands as usize);
        for i in 0..nlpcands {
            let var_ptr = unsafe { *lpcands.add(i as usize) };
            let var = var_ptr;
            let lp_sol_val = unsafe { *lpcandssol.add(i as usize) };
            let frac = lp_sol_val.fract();
            cands.push((var, lp_sol_val, frac));
        }
        cands
    }

    pub(crate) unsafe fn branch_var_val(
        scip: *mut ffi::SCIP,
        var_prob_id: usize,
        val: f64,
    ) -> Result<(), Retcode> {
        let var = ScipPtr::var_from_id(scip, var_prob_id);
        if var.is_none() {
            return Err(Retcode::Error);
        }
        let var = var.unwrap();
        scip_call! { ffi::SCIPbranchVarVal(scip, var, val, std::ptr::null_mut(), std::ptr::null_mut(),std::ptr::null_mut()) }
        Ok(())
    }

    pub(crate) fn include_eventhdlr(
        &self,
        name: &str,
        desc: &str,
        eventhdlr: Box<dyn Eventhdlr>,
    ) -> Result<(), Retcode> {
        extern "C" fn eventhdlrexec(
            scip: *mut ffi::SCIP,
            eventhdlr: *mut ffi::SCIP_EVENTHDLR,
            event: *mut ffi::SCIP_EVENT,
            _event_data: *mut ffi::SCIP_EVENTDATA,
        ) -> ffi::SCIP_Retcode {
            let data_ptr = unsafe { ffi::SCIPeventhdlrGetData(eventhdlr) };
            assert!(!data_ptr.is_null());
            let eventhdlr_ptr = data_ptr as *mut Box<dyn Eventhdlr>;
            let scip_ptr = Rc::new(ScipPtr::from_raw(scip, true));
            let model = Model {
                scip: scip_ptr.clone(),
                state: Solving,
            };
            let eventhdlr = SCIPEventhdlr { raw: eventhdlr };
            let event = Event {
                raw: event,
                scip: scip_ptr.clone(),
            };
            unsafe { (*eventhdlr_ptr).execute(model, eventhdlr, event) };
            Retcode::Okay.into()
        }

        extern "C" fn eventhdlrinit(
            scip: *mut ffi::SCIP,
            eventhdlr: *mut ffi::SCIP_EVENTHDLR,
        ) -> ffi::SCIP_Retcode {
            let data_ptr = unsafe { ffi::SCIPeventhdlrGetData(eventhdlr) };
            assert!(!data_ptr.is_null());
            let eventhdlr_ptr = data_ptr as *mut Box<dyn Eventhdlr>;
            let event_type = unsafe { (*eventhdlr_ptr).get_type() };
            unsafe {
                ffi::SCIPcatchEvent(
                    scip,
                    event_type.into(),
                    eventhdlr,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                )
            }
        }

        unsafe extern "C" fn eventhdlrfree(
            _scip: *mut ffi::SCIP,
            eventhdlr: *mut ffi::SCIP_EVENTHDLR,
        ) -> ffi::SCIP_Retcode {
            let data_ptr = unsafe { ffi::SCIPeventhdlrGetData(eventhdlr) };
            assert!(!data_ptr.is_null());
            let eventhdlr_ptr = data_ptr as *mut Box<dyn Eventhdlr>;
            drop(unsafe { Box::from_raw(eventhdlr_ptr) });
            Retcode::Okay.into()
        }

        let c_name = CString::new(name).unwrap();
        let c_desc = CString::new(desc).unwrap();
        let eventhdlr_ptr = Box::into_raw(Box::new(eventhdlr));

        unsafe {
            ffi::SCIPincludeEventhdlr(
                self.raw,
                c_name.as_ptr(),
                c_desc.as_ptr(),
                None,
                Some(eventhdlrfree),
                Some(eventhdlrinit),
                None,
                None,
                None,
                None,
                Some(eventhdlrexec),
                eventhdlr_ptr as *mut ffi::SCIP_EVENTHDLRDATA,
            );
        }

        Ok(())
    }

    pub(crate) fn include_branch_rule(
        &self,
        name: &str,
        desc: &str,
        priority: i32,
        maxdepth: i32,
        maxbounddist: f64,
        rule: Box<dyn BranchRule>,
    ) -> Result<(), Retcode> {
        let c_name = CString::new(name).unwrap();
        let c_desc = CString::new(desc).unwrap();

        // TODO: Add rest of branching rule plugin callbacks

        extern "C" fn branchexeclp(
            scip: *mut ffi::SCIP,
            branchrule: *mut ffi::SCIP_BRANCHRULE,
            _: u32,
            res: *mut ffi::SCIP_RESULT,
        ) -> ffi::SCIP_Retcode {
            let data_ptr = unsafe { ffi::SCIPbranchruleGetData(branchrule) };
            assert!(!data_ptr.is_null());
            let rule_ptr = data_ptr as *mut Box<dyn BranchRule>;
            let cands = unsafe { ScipPtr::lp_branching_cands(scip) }
                .into_iter()
                .map(|(scip_var, lp_sol_val, frac)| BranchingCandidate {
                    var_prob_id: unsafe { ffi::SCIPvarGetProbindex(scip_var) } as usize,
                    lp_sol_val,
                    frac,
                })
                .collect::<Vec<_>>();
            let scip_ptr = ScipPtr::from_raw(scip, true);
            let model = Model {
                scip: Rc::new(scip_ptr),
                state: Solving,
            };
            let branchrule = SCIPBranchRule { raw: branchrule };
            let branching_res = unsafe { (*rule_ptr).execute(model, branchrule, cands) };

            if let BranchingResult::BranchOn(cand) = branching_res.clone() {
                unsafe {
                    ScipPtr::branch_var_val(scip, cand.var_prob_id, cand.lp_sol_val).unwrap();
                }
            };

            if branching_res == BranchingResult::CustomBranching {
                assert!(
                    unsafe { ffi::SCIPgetNChildren(scip) > 0 },
                    "Custom branching rule must create at least one child node"
                )
            }

            unsafe { *res = branching_res.into() };
            Retcode::Okay.into()
        }

        extern "C" fn branchfree(
            _scip: *mut ffi::SCIP,
            branchrule: *mut ffi::SCIP_BRANCHRULE,
        ) -> ffi::SCIP_Retcode {
            let data_ptr = unsafe { ffi::SCIPbranchruleGetData(branchrule) };
            assert!(!data_ptr.is_null());
            drop(unsafe { Box::from_raw(data_ptr as *mut Box<dyn BranchRule>) });
            Retcode::Okay.into()
        }

        let rule_ptr = Box::into_raw(Box::new(rule));
        let branchrule_faker = rule_ptr as *mut ffi::SCIP_BranchruleData;

        scip_call!(ffi::SCIPincludeBranchrule(
            self.raw,
            c_name.as_ptr(),
            c_desc.as_ptr(),
            priority,
            maxdepth,
            maxbounddist,
            None,
            Some(branchfree),
            None,
            None,
            None,
            None,
            Some(branchexeclp),
            None,
            None,
            branchrule_faker,
        ));

        Ok(())
    }

    pub(crate) fn include_pricer(
        &self,
        name: &str,
        desc: &str,
        priority: i32,
        delay: bool,
        pricer: Box<dyn Pricer>,
    ) -> Result<(), Retcode> {
        let c_name = CString::new(name).unwrap();
        let c_desc = CString::new(desc).unwrap();

        pub(crate) fn call_pricer(
            scip: *mut ffi::SCIP,
            pricer: *mut ffi::SCIP_PRICER,
            lowerbound: *mut f64,
            stopearly: *mut ::std::os::raw::c_uint,
            result: *mut ffi::SCIP_RESULT,
            farkas: bool,
        ) -> ffi::SCIP_Retcode {
            let data_ptr = unsafe { ffi::SCIPpricerGetData(pricer) };
            assert!(!data_ptr.is_null());
            let pricer_ptr = data_ptr as *mut Box<dyn Pricer>;

            let n_vars_before = unsafe { ffi::SCIPgetNVars(scip) };

            let scip_ptr = ScipPtr::from_raw(scip, true);
            let model = Model {
                scip: Rc::new(scip_ptr),
                state: Solving,
            };

            let pricer = SCIPPricer { raw: pricer };
            let pricing_res = unsafe { (*pricer_ptr).generate_columns(model, pricer, farkas) };

            if !farkas {
                if let Some(lb) = pricing_res.lower_bound {
                    unsafe { *lowerbound = lb };
                }
                if pricing_res.state == PricerResultState::StopEarly {
                    unsafe { *stopearly = 1 };
                }
            }

            if farkas && pricing_res.state == PricerResultState::StopEarly {
                panic!("Farkas pricing should never stop early as LP would remain infeasible");
            }

            if pricing_res.state == PricerResultState::FoundColumns {
                let n_vars_after = unsafe { ffi::SCIPgetNVars(scip) };
                if n_vars_before >= n_vars_after {
                    return Retcode::Error.into();
                }
            }

            unsafe { *result = pricing_res.state.into() };
            Retcode::Okay.into()
        }

        unsafe extern "C" fn pricerredcost(
            scip: *mut ffi::SCIP,
            pricer: *mut ffi::SCIP_PRICER,
            lowerbound: *mut f64,
            stopearly: *mut ::std::os::raw::c_uint,
            result: *mut ffi::SCIP_RESULT,
        ) -> ffi::SCIP_Retcode {
            call_pricer(scip, pricer, lowerbound, stopearly, result, false)
        }

        unsafe extern "C" fn pricerfakas(
            scip: *mut ffi::SCIP,
            pricer: *mut ffi::SCIP_PRICER,
            result: *mut ffi::SCIP_RESULT,
        ) -> ffi::SCIP_Retcode {
            call_pricer(
                scip,
                pricer,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                result,
                true,
            )
        }

        unsafe extern "C" fn pricerfree(
            _scip: *mut ffi::SCIP,
            pricer: *mut ffi::SCIP_PRICER,
        ) -> ffi::SCIP_Retcode {
            let data_ptr = unsafe { ffi::SCIPpricerGetData(pricer) };
            assert!(!data_ptr.is_null());
            drop(unsafe { Box::from_raw(data_ptr as *mut Box<dyn Pricer>) });
            Retcode::Okay.into()
        }

        let pricer_ptr = Box::into_raw(Box::new(pricer));
        let pricer_faker = pricer_ptr as *mut ffi::SCIP_PricerData;

        scip_call!(ffi::SCIPincludePricer(
            self.raw,
            c_name.as_ptr(),
            c_desc.as_ptr(),
            priority,
            delay.into(),
            None,
            Some(pricerfree),
            None,
            None,
            None,
            None,
            Some(pricerredcost),
            Some(pricerfakas),
            pricer_faker,
        ));

        unsafe {
            ffi::SCIPactivatePricer(self.raw, ffi::SCIPfindPricer(self.raw, c_name.as_ptr()));
        }

        Ok(())
    }

    pub(crate) fn include_heur(
        &self,
        name: &str,
        desc: &str,
        priority: i32,
        dispchar: char,
        freq: i32,
        freqofs: i32,
        maxdepth: i32,
        timing: HeurTiming,
        usessubscip: bool,
        heur: Box<dyn Heuristic>,
    ) -> Result<(), Retcode> {
        let c_name = CString::new(name).unwrap();
        let c_desc = CString::new(desc).unwrap();

        extern "C" fn heurexec(
            scip: *mut ffi::SCIP,
            heur: *mut ffi::SCIP_HEUR,
            heurtiming: ffi::SCIP_HEURTIMING,
            nodeinfeasible: ::std::os::raw::c_uint,
            result: *mut ffi::SCIP_RESULT,
        ) -> ffi::SCIP_RETCODE {
            let data_ptr = unsafe { ffi::SCIPheurGetData(heur) };
            assert!(!data_ptr.is_null());
            let rule_ptr = data_ptr as *mut Box<dyn Heuristic>;

            let current_n_sols = unsafe { ffi::SCIPgetNSols(scip) };
            let scip_ptr = ScipPtr::from_raw(scip, true);
            let model = Model {
                scip: Rc::new(scip_ptr),
                state: Solving,
            };
            let heur_res =
                unsafe { (*rule_ptr).execute(model, heurtiming.into(), nodeinfeasible != 0) };
            if heur_res == HeurResult::FoundSol {
                let new_n_sols = unsafe { ffi::SCIPgetNSols(scip) };

                if new_n_sols <= current_n_sols {
                    let heur_name =
                        unsafe { CStr::from_ptr(ffi::SCIPheurGetName(heur)).to_str().unwrap() };
                    eprintln!(
                        "Heuristic {} returned result {:?}, but no solutions were added",
                        heur_name, heur_res
                    );
                    return Retcode::Error.into();
                }
            }

            unsafe { *result = heur_res.into() };
            Retcode::Okay.into()
        }

        extern "C" fn heurfree(
            _scip: *mut ffi::SCIP,
            heur: *mut ffi::SCIP_HEUR,
        ) -> ffi::SCIP_Retcode {
            let data_ptr = unsafe { ffi::SCIPheurGetData(heur) };
            assert!(!data_ptr.is_null());
            drop(unsafe { Box::from_raw(data_ptr as *mut Box<dyn Heuristic>) });
            Retcode::Okay.into()
        }

        let ptr = Box::into_raw(Box::new(heur));
        let heur_faker = ptr as *mut ffi::SCIP_HEURDATA;

        scip_call!(ffi::SCIPincludeHeur(
            self.raw,
            c_name.as_ptr(),
            c_desc.as_ptr(),
            dispchar as ::std::os::raw::c_char,
            priority,
            freq,
            freqofs,
            maxdepth,
            timing.into(),
            usessubscip.into(),
            None,
            Some(heurfree),
            None,
            None,
            None,
            None,
            Some(heurexec),
            heur_faker,
        ));

        Ok(())
    }

    pub(crate) fn include_separator(
        &self,
        name: &str,
        desc: &str,
        priority: i32,
        freq: i32,
        maxbounddist: f64,
        usesubscip: bool,
        delay: bool,
        separator: Box<dyn Separator>,
    ) -> Result<(), Retcode> {
        let c_name = CString::new(name).unwrap();
        let c_desc = CString::new(desc).unwrap();

        extern "C" fn sepexeclp(
            scip: *mut ffi::SCIP,
            separator: *mut ffi::SCIP_SEPA,
            result: *mut ffi::SCIP_RESULT,
            _allowlocal: ::std::os::raw::c_uint,
            _depth: ::std::os::raw::c_int,
        ) -> ffi::SCIP_Retcode {
            let data_ptr = unsafe { ffi::SCIPsepaGetData(separator) };
            assert!(!data_ptr.is_null());
            let rule_ptr = data_ptr as *mut Box<dyn Separator>;

            let scip_ptr = ScipPtr::from_raw(scip, true);
            let model = Model {
                scip: Rc::new(scip_ptr),
                state: Solving,
            };
            let separator = SCIPSeparator { raw: separator };
            let sep_res = unsafe { (*rule_ptr).execute_lp(model, separator) };

            unsafe { *result = sep_res.into() };

            Retcode::Okay.into()
        }

        extern "C" fn sepexecsol(
            _scip: *mut ffi::SCIP,
            _separator: *mut ffi::SCIP_SEPA,
            _sol: *mut SCIP_SOL,
            _result: *mut ffi::SCIP_RESULT,
            _allowlocal: ::std::os::raw::c_uint,
            _depth: ::std::os::raw::c_int,
        ) -> ffi::SCIP_Retcode {
            Retcode::Okay.into()
        }

        extern "C" fn sepfree(
            _scip: *mut ffi::SCIP,
            separator: *mut ffi::SCIP_SEPA,
        ) -> ffi::SCIP_Retcode {
            let data_ptr = unsafe { ffi::SCIPsepaGetData(separator) };
            assert!(!data_ptr.is_null());
            drop(unsafe { Box::from_raw(data_ptr as *mut Box<dyn Separator>) });
            Retcode::Okay.into()
        }

        let ptr = Box::into_raw(Box::new(separator));
        let sep_faker = ptr as *mut ffi::SCIP_SEPADATA;

        scip_call!(ffi::SCIPincludeSepa(
            self.raw,
            c_name.as_ptr(),
            c_desc.as_ptr(),
            priority,
            freq,
            maxbounddist,
            usesubscip.into(),
            delay.into(),
            None,
            Some(sepfree),
            None,
            None,
            None,
            None,
            Some(sepexeclp),
            Some(sepexecsol),
            sep_faker,
        ));

        Ok(())
    }

    pub(crate) fn include_conshdlr(
        &self,
        name: &str,
        desc: &str,
        enfopriority: i32,
        checkpriority: i32,
        conshdlr: Box<dyn Conshdlr>,
    ) -> Result<(), Retcode> {
        let c_name = CString::new(name).unwrap();
        let c_desc = CString::new(desc).unwrap();

        extern "C" fn consenfolp(
            scip: *mut SCIP,
            conshdlr: *mut SCIP_CONSHDLR,
            _conss: *mut *mut SCIP_CONS,
            _nconss: std::os::raw::c_int,
            _nusefulconss: std::os::raw::c_int,
            _solinfeasible: std::os::raw::c_uint,
            result: *mut SCIP_RESULT,
        ) -> SCIP_RETCODE {
            let data_ptr = unsafe { ffi::SCIPconshdlrGetData(conshdlr) };
            assert!(!data_ptr.is_null());
            let conshdlr_ptr = data_ptr as *mut Box<dyn Conshdlr>;

            let scip_ptr = Rc::new(ScipPtr::from_raw(scip, true));
            let model = Model {
                scip: scip_ptr.clone(),
                state: Solving,
            };

            let scip_conshdlr = SCIPConshdlr { raw: conshdlr };

            unsafe {
                *result = (*conshdlr_ptr).enforce(model, scip_conshdlr).into();
            }

            Retcode::Okay.into()
        }

        extern "C" fn conscheck(
            scip: *mut SCIP,
            conshdlr: *mut SCIP_CONSHDLR,
            _conss: *mut *mut SCIP_CONS,
            _nconss: ::std::os::raw::c_int,
            sol: *mut SCIP_SOL,
            _checkintegrality: ::std::os::raw::c_uint,
            _checklprows: ::std::os::raw::c_uint,
            _printreason: ::std::os::raw::c_uint,
            _completely: ::std::os::raw::c_uint,
            result: *mut SCIP_RESULT,
        ) -> SCIP_RETCODE {
            let data_ptr = unsafe { ffi::SCIPconshdlrGetData(conshdlr) };
            assert!(!data_ptr.is_null());
            let conshdlr_ptr = data_ptr as *mut Box<dyn Conshdlr>;

            let scip_ptr = Rc::new(ScipPtr::from_raw(scip, true));
            let model = Model {
                scip: scip_ptr.clone(),
                state: Solving,
            };

            let scip_conshdlr = SCIPConshdlr { raw: conshdlr };

            assert!(!sol.is_null());

            let solution = Solution {
                raw: sol,
                scip_ptr: scip_ptr.clone(),
            };

            let feasible = unsafe { (*conshdlr_ptr).check(model, scip_conshdlr, &solution) };

            unsafe {
                *result = if feasible {
                    ffi::SCIP_Result_SCIP_FEASIBLE
                } else {
                    ffi::SCIP_Result_SCIP_INFEASIBLE
                };
            }

            Retcode::Okay.into()
        }

        extern "C" fn conslock(
            _scip: *mut SCIP,
            _conshdlr: *mut SCIP_CONSHDLR,
            _cons: *mut SCIP_CONS,
            _locktype: SCIP_LOCKTYPE,
            _nlockspos: ::std::os::raw::c_int,
            _nlocksneg: ::std::os::raw::c_int,
        ) -> SCIP_RETCODE {
            // // loops over all the variables and runs the lock method on them
            // let data_ptr = unsafe { ffi::SCIPconshdlrGetData(conshdlr) };
            // assert!(!data_ptr.is_null());
            // let conshdlr_ptr = data_ptr as *mut Box<dyn Conshdlr>;
            //
            // let scip_ptr = Rc::new(ScipPtr::from_raw(scip, true));
            // let model = Model {
            //     scip: scip_ptr.clone(),
            //     state: Solving,
            // };
            //
            // let vars = model.vars();
            // for var in vars {
            //     let scip_conshdlr = SCIPConshdlr { raw: conshdlr };
            //
            //     let model = Model {
            //         scip: scip_ptr.clone(),
            //         state: Solving,
            //     };
            //
            //     let lock_type = unsafe { (*conshdlr_ptr).lock(model, scip_conshdlr, &var) };
            //
            //     unsafe {
            //         match lock_type {
            //             LockDirection::Both => {
            //                 ffi::SCIPaddVarLocks(
            //                     scip,
            //                     var.raw,
            //                     nlockspos + nlocksneg,
            //                     nlockspos + nlocksneg,
            //                 );
            //             }
            //             LockDirection::Decrease => {
            //                 ffi::SCIPaddVarLocks(scip, var.raw, nlockspos, nlocksneg);
            //             }
            //             LockDirection::Increase => {
            //                 ffi::SCIPaddVarLocks(scip, var.raw, nlocksneg, nlockspos);
            //             }
            //         }
            //     }
            // }
            Retcode::Okay.into()
        }

        extern "C" fn consfree(
            _scip: *mut ffi::SCIP,
            conshdlr: *mut ffi::SCIP_CONSHDLR,
        ) -> ffi::SCIP_Retcode {
            let data_ptr = unsafe { ffi::SCIPconshdlrGetData(conshdlr) };
            assert!(!data_ptr.is_null());
            drop(unsafe { Box::from_raw(data_ptr as *mut Box<dyn Conshdlr>) });
            Retcode::Okay.into()
        }

        let ptr = Box::into_raw(Box::new(conshdlr));
        let cons_faker = ptr as *mut ffi::SCIP_CONSHDLRDATA;

        let mut conshdlr: *mut SCIP_CONSHDLR = std::ptr::null_mut();
        scip_call!(ffi::SCIPincludeConshdlrBasic(
            self.raw,
            &mut conshdlr,
            c_name.as_ptr(),
            c_desc.as_ptr(),
            enfopriority,
            checkpriority,
            0,
            false.into(),
            Some(consenfolp),
            None,
            Some(conscheck),
            Some(conslock),
            cons_faker,
        ));

        scip_call!(ffi::SCIPsetConshdlrFree(self.raw, conshdlr, Some(consfree)));

        Ok(())
    }

    pub(crate) fn add_cons_coef(
        &self,
        cons: &Constraint,
        var: &Variable,
        coef: f64,
    ) -> Result<(), Retcode> {
        let cons_is_transformed = unsafe { ffi::SCIPconsIsTransformed(cons.raw) } == 1;
        let var_is_transformed = unsafe { ffi::SCIPvarIsTransformed(var.raw) } == 1;
        let cons_ptr = if !cons_is_transformed && var_is_transformed {
            self.get_transformed_cons(cons)?.expect("No transformed constraint was found for the passed original constraint, to prevent this you could disable presolving or mark the constraint to be not removable")
        } else {
            cons.raw
        };

        let var_ptr = if cons_is_transformed && !var_is_transformed {
            let mut transformed_var = MaybeUninit::<*mut ffi::SCIP_Var>::uninit();
            scip_call!(ffi::SCIPgetTransformedVar(
                self.raw,
                var.raw,
                transformed_var.as_mut_ptr()
            ));
            unsafe { transformed_var.assume_init() }
        } else {
            var.raw
        };

        scip_call! { ffi::SCIPaddCoefLinear(self.raw, cons_ptr, var_ptr, coef) }

        Ok(())
    }

    pub(crate) fn set_cons_modifiable(
        &self,
        cons: &Constraint,
        modifiable: bool,
    ) -> Result<(), Retcode> {
        scip_call!(ffi::SCIPsetConsModifiable(
            self.raw,
            cons.raw,
            modifiable.into()
        ));
        Ok(())
    }

    pub(crate) fn cons_is_modifiable(&self, cons: &Constraint) -> bool {
        unsafe { ffi::SCIPconsIsModifiable(cons.raw) == ffi::TRUE }
    }

    pub(crate) fn set_cons_removable(
        &self,
        cons: &Constraint,
        removable: bool,
    ) -> Result<(), Retcode> {
        scip_call!(ffi::SCIPsetConsRemovable(
            self.raw,
            cons.raw,
            removable.into()
        ));
        Ok(())
    }

    pub(crate) fn cons_is_removable(&self, cons: &Constraint) -> bool {
        unsafe { ffi::SCIPconsIsRemovable(cons.raw) == ffi::TRUE }
    }

    pub(crate) fn set_cons_separated(
        &self,
        cons: &Constraint,
        separate: bool,
    ) -> Result<(), Retcode> {
        scip_call!(ffi::SCIPsetConsSeparated(
            self.raw,
            cons.raw,
            separate.into()
        ));
        Ok(())
    }

    pub(crate) fn cons_is_separated(&self, cons: &Constraint) -> bool {
        unsafe { ffi::SCIPconsIsSeparated(cons.raw) == ffi::TRUE }
    }

    pub(crate) fn n_nodes(&self) -> usize {
        unsafe { ffi::SCIPgetNNodes(self.raw) as usize }
    }

    pub(crate) fn solving_time(&self) -> f64 {
        unsafe { ffi::SCIPgetSolvingTime(self.raw) }
    }

    pub(crate) fn n_lp_iterations(&self) -> usize {
        unsafe { ffi::SCIPgetNLPIterations(self.raw) as usize }
    }

    pub(crate) fn focus_node(&self) -> Option<*mut SCIP_NODE> {
        let ptr = unsafe { ffi::SCIPgetFocusNode(self.raw) };
        if ptr.is_null() {
            None
        } else {
            Some(ptr)
        }
    }

    pub(crate) fn create_child(&self) -> Result<*mut SCIP_NODE, Retcode> {
        let mut node_ptr = MaybeUninit::uninit();
        scip_call!(ffi::SCIPcreateChild(
            self.raw,
            node_ptr.as_mut_ptr(),
            0.,
            ffi::SCIPgetLocalTransEstimate(self.raw), // TODO: pass that as an argument
        ));

        let node_ptr = unsafe { node_ptr.assume_init() };
        Ok(node_ptr)
    }

    pub(crate) fn add_sol(&self, mut sol: Solution) -> Result<bool, Retcode> {
        let mut feasible = 0;
        assert!(!sol.raw.is_null());
        let is_orig = unsafe { ffi::SCIPsolIsOriginal(sol.raw) } == 1;
        if is_orig {
            scip_call!(ffi::SCIPcheckSolOrig(
                self.raw,
                sol.raw,
                &mut feasible,
                true.into(),
                true.into(),
            ));
            if feasible == 1 {
                scip_call!(ffi::SCIPaddSolFree(self.raw, &mut sol.raw, &mut feasible));
            }
            return Ok(feasible != 0);
        } else {
            scip_call!(ffi::SCIPtrySol(
                self.raw,
                sol.raw,
                true.into(),
                true.into(),
                true.into(),
                true.into(),
                true.into(),
                &mut feasible,
            ));
        }

        Ok(feasible != 0)
    }

    pub(crate) fn free_transform(&self) -> Result<(), Retcode> {
        scip_call!(ffi::SCIPfreeTransform(self.raw));
        Ok(())
    }

    pub(crate) fn add_row(&self, row: Row, force_cut: bool) -> Result<bool, Retcode> {
        let mut infeasible = 0;
        scip_call!(ffi::SCIPaddRow(
            self.raw,
            row.raw,
            force_cut.into(),
            &mut infeasible
        ));
        Ok(infeasible != 0)
    }
}

impl Drop for ScipPtr {
    fn drop(&mut self) {
        if self.weak {
            return;
        }

        // Rust Model struct keeps at most one copy of each variable and constraint pointers
        // so we need to release them before freeing the SCIP instance

        // first check if we are in a stage where we have variables and constraints
        let scip_stage = unsafe { ffi::SCIPgetStage(self.raw) };
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
            // release original variables
            let n_vars = unsafe { ffi::SCIPgetNOrigVars(self.raw) };
            let vars = unsafe { ffi::SCIPgetOrigVars(self.raw) };
            for i in 0..n_vars {
                let mut var = unsafe { *vars.add(i as usize) };
                scip_call_panic!(ffi::SCIPreleaseVar(self.raw, &mut var));
            }

            // release vars added in solving
            for var_ptr in self.vars_added_in_solving.iter_mut() {
                scip_call_panic!(ffi::SCIPreleaseVar(self.raw, var_ptr));
            }

            // release constraints
            let n_conss = unsafe { ffi::SCIPgetNOrigConss(self.raw) };
            let conss = unsafe { ffi::SCIPgetOrigConss(self.raw) };
            for i in 0..n_conss {
                let mut cons = unsafe { *conss.add(i as usize) };
                scip_call_panic!(ffi::SCIPreleaseCons(self.raw, &mut cons));
            }
        }

        // free SCIP instance
        unsafe { ffi::SCIPfree(&mut self.raw) };
    }
}
