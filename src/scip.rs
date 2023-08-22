use crate::branchrule::{BranchRule, BranchingCandidate};
use crate::pricer::{Pricer, PricerResultState};
use crate::{ffi, scip_call_panic, BranchingResult, Constraint, Eventhdlr, HeurResult, Node, ObjSense, ParamSetting, Retcode, Solution, Status, VarType, Variable, SharedMut};
use crate::{scip_call, HeurTiming, Heuristic};
use core::panic;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::ffi::{c_int, CStr, CString};
use std::mem::MaybeUninit;
use std::rc::Rc;

#[non_exhaustive]
#[derive(Debug)]
pub(crate) struct ScipPtr {
    pub(crate) raw: *mut ffi::SCIP,
    consumed: bool,
    vars_added_in_solving: Vec<*mut ffi::SCIP_VAR>
}

impl<'a> ScipPtr {
    pub(crate) fn new() -> Self {
        let mut scip_ptr = MaybeUninit::uninit();
        scip_call_panic!(ffi::SCIPcreate(scip_ptr.as_mut_ptr()));
        let scip_ptr = unsafe { scip_ptr.assume_init() };
        ScipPtr {
            raw: scip_ptr,
            consumed: false,
            vars_added_in_solving: Vec::new(),
        }
    }

    pub(crate) fn clone(&self) -> Self {
        ScipPtr {
            raw: self.raw,
            consumed: true,
            vars_added_in_solving: Vec::new(),
        }
    }

    pub(crate) fn set_str_param(&mut self, param: &str, value: &str) -> Result<(), Retcode> {
        let param = CString::new(param).unwrap();
        let value = CString::new(value).unwrap();
        scip_call! { ffi::SCIPsetStringParam(self.raw, param.as_ptr(), value.as_ptr()) };
        Ok(())
    }

    pub(crate) fn set_int_param(&mut self, param: &str, value: i32) -> Result<(), Retcode> {
        let param = CString::new(param).unwrap();
        scip_call! { ffi::SCIPsetIntParam(self.raw, param.as_ptr(), value) };
        Ok(())
    }

    pub(crate) fn set_longint_param(&mut self, param: &str, value: i64) -> Result<(), Retcode> {
        let param = CString::new(param).unwrap();
        scip_call! { ffi::SCIPsetLongintParam(self.raw, param.as_ptr(), value) };
        Ok(())
    }

    pub(crate) fn set_real_param(&mut self, param: &str, value: f64) -> Result<(), Retcode> {
        let param = CString::new(param).unwrap();
        scip_call! { ffi::SCIPsetRealParam(self.raw, param.as_ptr(), value) };
        Ok(())
    }

    pub(crate) fn set_presolving(&mut self, presolving: ParamSetting) -> Result<(), Retcode> {
        scip_call! { ffi::SCIPsetPresolving(self.raw, presolving.into(), true.into()) };
        Ok(())
    }

    pub(crate) fn set_separating(&mut self, separating: ParamSetting) -> Result<(), Retcode> {
        scip_call! { ffi::SCIPsetSeparating(self.raw, separating.into(), true.into()) };
        Ok(())
    }

    pub(crate) fn set_heuristics(&mut self, heuristics: ParamSetting) -> Result<(), Retcode> {
        scip_call! { ffi::SCIPsetHeuristics(self.raw, heuristics.into(), true.into()) };
        Ok(())
    }

    pub(crate) fn create_prob(&mut self, name: &str) -> Result<(), Retcode> {
        let name = CString::new(name).unwrap();
        scip_call!(ffi::SCIPcreateProbBasic(self.raw, name.as_ptr()));
        Ok(())
    }

    pub(crate) fn read_prob(&mut self, filename: &str) -> Result<(), Retcode> {
        let filename = CString::new(filename).unwrap();
        scip_call!(ffi::SCIPreadProb(
            self.raw,
            filename.as_ptr(),
            std::ptr::null_mut()
        ));
        Ok(())
    }

    pub(crate) fn set_obj_sense(&mut self, sense: ObjSense) -> Result<(), Retcode> {
        scip_call!(ffi::SCIPsetObjsense(self.raw, sense.into()));
        Ok(())
    }

    pub(crate) fn set_obj_integral(&mut self) -> Result<(), Retcode> {
        scip_call!(ffi::SCIPsetObjIntegral(self.raw));
        Ok(())
    }

    pub(crate) fn n_vars(&self) -> usize {
        unsafe { ffi::SCIPgetNVars(self.raw) as usize }
    }

    pub(crate) fn n_conss(&self) -> usize {
        unsafe { ffi::SCIPgetNConss(self.raw) as usize }
    }

    pub(crate) fn status(&self) -> Status {
        let status = unsafe { ffi::SCIPgetStatus(self.raw) };
        status.try_into().expect("Unknown SCIP status")
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

    pub(crate) fn include_default_plugins(&mut self) -> Result<(), Retcode> {
        scip_call!(ffi::SCIPincludeDefaultPlugins(self.raw));
        Ok(())
    }

    pub(crate) fn vars(&self) -> BTreeMap<usize, Variable> {
        // NOTE: this method should only be called once per SCIP instance
        let n_vars = self.n_vars();
        let mut vars = BTreeMap::new();
        let scip_vars = unsafe { ffi::SCIPgetVars(self.raw) };
        for i in 0..n_vars {
            let scip_var = unsafe { *scip_vars.add(i) };
            unsafe {
                ffi::SCIPcaptureVar(self.raw, scip_var);
            }
            let var = Variable { raw: scip_var, data: None };
            let var_id = var.index();
            vars.insert(var_id, var);
        }
        vars
    }

    pub(crate) fn conss(&self) -> Vec<Constraint> {
        // NOTE: this method should only be called once per SCIP instance
        let n_conss = self.n_conss();
        let mut conss = Vec::with_capacity(n_conss);
        let scip_conss = unsafe { ffi::SCIPgetConss(self.raw) };
        for i in 0..n_conss {
            let scip_cons = unsafe { *scip_conss.add(i) };
            unsafe {
                ffi::SCIPcaptureCons(self.raw, scip_cons);
            }
            let cons = Constraint { raw: scip_cons };
            conss.push(cons);
        }
        conss
    }

    pub(crate) fn solve(&mut self) -> Result<(), Retcode> {
        scip_call!(ffi::SCIPsolve(self.raw));
        Ok(())
    }

    pub(crate) fn n_sols(&self) -> usize {
        unsafe { ffi::SCIPgetNSols(self.raw) as usize }
    }

    pub(crate) fn best_sol(&self) -> Solution {
        let sol = unsafe { ffi::SCIPgetBestSol(self.raw) };

        Solution {
            scip_ptr: self.raw,
            raw: sol,
        }
    }

    pub(crate) fn obj_val(&self) -> f64 {
        unsafe { ffi::SCIPgetPrimalbound(self.raw) }
    }

    pub(crate) fn create_var(
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
        Ok(Variable { raw: var_ptr, data: None })
    }

    pub(crate) fn create_var_solving(
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
        Ok(Variable { raw: trans_var_ptr, data: None })
    }

    pub(crate) fn create_priced_var(
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
        Ok(Variable { raw: trans_var_ptr, data: None })
    }

    pub(crate) fn create_cons(
        &'a mut self,
        vars: Vec<&'a Variable>,
        coefs: &[f64],
        lhs: f64,
        rhs: f64,
        name: &str,
    ) -> Result<Constraint, Retcode> {
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
        let scip_cons = unsafe { scip_cons.assume_init() };
        for (i, var) in vars.iter().enumerate() {
            scip_call! { ffi::SCIPaddCoefLinear(self.raw, scip_cons, var.raw, coefs[i]) };
        }
        scip_call! { ffi::SCIPaddCons(self.raw, scip_cons) };
        Ok(Constraint { raw: scip_cons })
    }

    /// Create set partitioning constraint
    pub(crate) fn create_cons_set_part(
        &'a mut self,
        vars: Vec<&'a Variable>,
        name: &str,
    ) -> Result<Constraint, Retcode> {
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
        Ok(Constraint { raw: scip_cons })
    }

    /// Create set cover constraint
    pub(crate) fn create_cons_set_cover(
        &'a mut self,
        vars: Vec<&'a Variable>,
        name: &str,
    ) -> Result<Constraint, Retcode> {
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
        Ok(Constraint { raw: scip_cons })
    }

    pub(crate) fn create_cons_quadratic(
        &'a mut self,
        lin_vars: Vec<&'a Variable>,
        lin_coefs: &mut [f64],
        quad_vars_1: Vec<&'a Variable>,
        quad_vars_2: Vec<&'a Variable>,
        quad_coefs: &mut [f64],
        lhs: f64,
        rhs: f64,
        name: &str,
    ) -> Result<Constraint, Retcode> {
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

        let get_ptrs = |vars: Vec<&'a Variable>| {
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
        Ok(Constraint { raw: scip_cons })
    }

    /// Create set packing constraint
    pub(crate) fn create_cons_set_pack(
        &'a mut self,
        vars: Vec<&'a Variable>,
        name: &str,
    ) -> Result<Constraint, Retcode> {
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
        Ok(Constraint { raw: scip_cons })
    }

    /// Create solution
    pub(crate) fn create_sol(&self) -> Result<Solution, Retcode> {
        let mut sol = MaybeUninit::uninit();
        scip_call! { ffi::SCIPcreateSol(self.raw, sol.as_mut_ptr(), std::ptr::null_mut()) }
        let sol = unsafe { sol.assume_init() };
        Ok(Solution {
            scip_ptr: self.raw,
            raw: sol,
        })
    }

    /// Add coefficient to set packing/partitioning/covering constraint
    pub(crate) fn add_cons_coef_setppc(
        &'a mut self,
        cons: &Constraint,
        var: &'a Variable,
    ) -> Result<(), Retcode> {
        scip_call! { ffi::SCIPaddCoefSetppc(self.raw, cons.raw, var.raw) };
        Ok(())
    }

    pub(crate) fn lp_branching_cands(scip: *mut ffi::SCIP) -> Vec<BranchingCandidate> {
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
            let var = Rc::new(Variable { raw: var_ptr, data: None });
            let lp_sol_val = unsafe { *lpcandssol.add(i as usize) };
            let frac = lp_sol_val.fract();
            cands.push(BranchingCandidate {
                var,
                lp_sol_val,
                frac,
            });
        }
        cands
    }

    pub(crate) fn branch_var_val(
        scip: *mut ffi::SCIP,
        var: *mut ffi::SCIP_VAR,
        val: f64,
    ) -> Result<(), Retcode> {
        scip_call! { ffi::SCIPbranchVarVal(scip, var, val, std::ptr::null_mut(), std::ptr::null_mut(),std::ptr::null_mut()) };
        Ok(())
    }

    pub(crate) fn include_eventhdlr(
        &self,
        name: &str,
        desc: &str,
        eventhdlr: Box<dyn Eventhdlr>,
    ) -> Result<(), Retcode> {
        extern "C" fn eventhdlrexec(
            _scip: *mut ffi::SCIP,
            eventhdlr: *mut ffi::SCIP_EVENTHDLR,
            _event: *mut ffi::SCIP_EVENT,
            _event_data: *mut ffi::SCIP_EVENTDATA,
        ) -> ffi::SCIP_Retcode {
            let data_ptr = unsafe { ffi::SCIPeventhdlrGetData(eventhdlr) };
            assert!(!data_ptr.is_null());
            let eventhdlr_ptr = data_ptr as *mut Box<dyn Eventhdlr>;
            unsafe { (*eventhdlr_ptr).execute() };
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
            let cands = ScipPtr::lp_branching_cands(scip);
            let branching_res = unsafe { (*rule_ptr).execute(cands) };

            if let BranchingResult::BranchOn(cand) = branching_res.clone() {
                ScipPtr::branch_var_val(scip, cand.var.raw, cand.lp_sol_val).unwrap();
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
            let pricing_res = unsafe { (*pricer_ptr).generate_columns(farkas) };

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
                assert!(n_vars_before < n_vars_after);
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
            let heur_res = unsafe { (*rule_ptr).execute(heurtiming.into(), nodeinfeasible != 0) };
            if heur_res == HeurResult::FoundSol {
                let new_n_sols = unsafe { ffi::SCIPgetNSols(scip) };

                if new_n_sols <= current_n_sols {
                    let heur_name =
                        unsafe { CStr::from_ptr(ffi::SCIPheurGetName(heur)).to_str().unwrap() };
                    panic!(
                        "Heuristic {} returned result {:?}, but no solutions were added",
                        heur_name, heur_res
                    );
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

    pub(crate) fn add_cons_coef(
        &mut self,
        cons: &Constraint,
        var: &'a Variable,
        coef: f64,
    ) -> Result<(), Retcode> {
        let cons_is_transformed = unsafe { ffi::SCIPconsIsTransformed(cons.raw) } == 1;
        let var_is_transformed = unsafe { ffi::SCIPvarIsTransformed(var.raw) } == 1;
        let cons_ptr = if !cons_is_transformed && var_is_transformed {
            let mut transformed_cons = MaybeUninit::<*mut ffi::SCIP_Cons>::uninit();
            scip_call!(ffi::SCIPgetTransformedCons(
                self.raw,
                cons.raw,
                transformed_cons.as_mut_ptr()
            ));
            unsafe { transformed_cons.assume_init() }
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

        scip_call! { ffi::SCIPaddCoefLinear(self.raw, cons_ptr, var_ptr, coef) };
        Ok(())
    }

    pub(crate) fn set_cons_modifiable(
        &mut self,
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

    pub(crate) fn n_nodes(&self) -> usize {
        unsafe { ffi::SCIPgetNNodes(self.raw) as usize }
    }

    pub(crate) fn solving_time(&self) -> f64 {
        unsafe { ffi::SCIPgetSolvingTime(self.raw) }
    }

    pub(crate) fn n_lp_iterations(&self) -> usize {
        unsafe { ffi::SCIPgetNLPIterations(self.raw) as usize }
    }

    pub(crate) fn focus_node(&self) -> Node {
        Node {
            raw: unsafe { ffi::SCIPgetFocusNode(self.raw) },
        }
    }

    pub(crate) fn create_child(&mut self) -> Result<Node, Retcode> {
        let mut node_ptr = MaybeUninit::uninit();
        scip_call!(ffi::SCIPcreateChild(
            self.raw,
            node_ptr.as_mut_ptr(),
            0.,
            ffi::SCIPgetLocalTransEstimate(self.raw), // TODO: pass that as an argument
        ));

        let node_ptr = unsafe { node_ptr.assume_init() };
        Ok(Node { raw: node_ptr })
    }

    pub(crate) fn add_sol(&self, mut sol: Solution) -> Result<bool, Retcode> {
        let mut stored = MaybeUninit::uninit();
        scip_call!(ffi::SCIPaddSolFree(self.raw, &mut sol.raw, stored.as_mut_ptr()));
        let stored = unsafe { stored.assume_init() };
        Ok(stored != 0)
    }
}

impl Drop for ScipPtr {
    fn drop(&mut self) {
        if self.consumed {
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