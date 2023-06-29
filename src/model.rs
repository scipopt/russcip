use core::panic;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::ffi::{c_int, CStr, CString};
use std::mem::MaybeUninit;
use std::rc::Rc;

use crate::branchrule::{BranchRule, BranchingCandidate, BranchingResult};
use crate::constraint::Constraint;
use crate::eventhdlr::Eventhdlr;
use crate::node::Node;
use crate::pricer::{Pricer, PricerResultState};
use crate::retcode::Retcode;
use crate::solution::{SolError, Solution};
use crate::status::Status;
use crate::variable::{VarId, VarType, Variable};
use crate::{ffi, scip_call_panic};
use crate::{scip_call, Heuristic, HeurResult, HeurTiming};

#[non_exhaustive]
#[derive(Debug)]
struct ScipPtr {
    raw: *mut ffi::SCIP,
    consumed: bool,
}

impl ScipPtr {
    fn new() -> Self {
        let mut scip_ptr = MaybeUninit::uninit();
        scip_call_panic!(ffi::SCIPcreate(scip_ptr.as_mut_ptr()));
        let scip_ptr = unsafe { scip_ptr.assume_init() };
        ScipPtr {
            raw: scip_ptr,
            consumed: false,
        }
    }

    fn clone(&self) -> Self {
        ScipPtr {
            raw: self.raw,
            consumed: true,
        }
    }

    fn set_str_param(&mut self, param: &str, value: &str) -> Result<(), Retcode> {
        let param = CString::new(param).unwrap();
        let value = CString::new(value).unwrap();
        scip_call! { ffi::SCIPsetStringParam(self.raw, param.as_ptr(), value.as_ptr()) };
        Ok(())
    }

    fn set_int_param(&mut self, param: &str, value: i32) -> Result<(), Retcode> {
        let param = CString::new(param).unwrap();
        scip_call! { ffi::SCIPsetIntParam(self.raw, param.as_ptr(), value) };
        Ok(())
    }

    fn set_longint_param(&mut self, param: &str, value: i64) -> Result<(), Retcode> {
        let param = CString::new(param).unwrap();
        scip_call! { ffi::SCIPsetLongintParam(self.raw, param.as_ptr(), value) };
        Ok(())
    }

    fn set_real_param(&mut self, param: &str, value: f64) -> Result<(), Retcode> {
        let param = CString::new(param).unwrap();
        scip_call! { ffi::SCIPsetRealParam(self.raw, param.as_ptr(), value) };
        Ok(())
    }

    fn set_presolving(&mut self, presolving: ParamSetting) -> Result<(), Retcode> {
        scip_call! { ffi::SCIPsetPresolving(self.raw, presolving.into(), true.into()) };
        Ok(())
    }

    fn set_separating(&mut self, separating: ParamSetting) -> Result<(), Retcode> {
        scip_call! { ffi::SCIPsetSeparating(self.raw, separating.into(), true.into()) };
        Ok(())
    }

    fn set_heuristics(&mut self, heuristics: ParamSetting) -> Result<(), Retcode> {
        scip_call! { ffi::SCIPsetHeuristics(self.raw, heuristics.into(), true.into()) };
        Ok(())
    }

    fn create_prob(&mut self, name: &str) -> Result<(), Retcode> {
        let name = CString::new(name).unwrap();
        scip_call!(ffi::SCIPcreateProbBasic(self.raw, name.as_ptr()));
        Ok(())
    }

    fn read_prob(&mut self, filename: &str) -> Result<(), Retcode> {
        let filename = CString::new(filename).unwrap();
        scip_call!(ffi::SCIPreadProb(
            self.raw,
            filename.as_ptr(),
            std::ptr::null_mut()
        ));
        Ok(())
    }

    fn set_obj_sense(&mut self, sense: ObjSense) -> Result<(), Retcode> {
        scip_call!(ffi::SCIPsetObjsense(self.raw, sense.into()));
        Ok(())
    }

    fn n_vars(&self) -> usize {
        unsafe { ffi::SCIPgetNVars(self.raw) as usize }
    }

    fn n_conss(&self) -> usize {
        unsafe { ffi::SCIPgetNConss(self.raw) as usize }
    }

    fn status(&self) -> Status {
        let status = unsafe { ffi::SCIPgetStatus(self.raw) };
        status.try_into().expect("Unknown SCIP status")
    }

    fn print_version(&self) {
        unsafe { ffi::SCIPprintVersion(self.raw, std::ptr::null_mut()) };
    }

    fn write(&self, path: &str, ext: &str) -> Result<(), Retcode> {
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

    fn include_default_plugins(&mut self) -> Result<(), Retcode> {
        scip_call!(ffi::SCIPincludeDefaultPlugins(self.raw));
        Ok(())
    }

    fn vars(&self) -> BTreeMap<usize, Rc<Variable>> {
        // NOTE: this method should only be called once per SCIP instance
        let n_vars = self.n_vars();
        let mut vars = BTreeMap::new();
        let scip_vars = unsafe { ffi::SCIPgetVars(self.raw) };
        for i in 0..n_vars {
            let scip_var = unsafe { *scip_vars.add(i) };
            unsafe {
                ffi::SCIPcaptureVar(self.raw, scip_var);
            }
            let var = Rc::new(Variable { raw: scip_var });
            vars.insert(var.index(), var);
        }
        vars
    }

    fn conss(&self) -> Vec<Rc<Constraint>> {
        // NOTE: this method should only be called once per SCIP instance
        let n_conss = self.n_conss();
        let mut conss = Vec::with_capacity(n_conss);
        let scip_conss = unsafe { ffi::SCIPgetConss(self.raw) };
        for i in 0..n_conss {
            let scip_cons = unsafe { *scip_conss.add(i) };
            unsafe {
                ffi::SCIPcaptureCons(self.raw, scip_cons);
            }
            let cons = Rc::new(Constraint { raw: scip_cons });
            conss.push(cons);
        }
        conss
    }

    fn solve(&mut self) -> Result<(), Retcode> {
        scip_call!(ffi::SCIPsolve(self.raw));
        Ok(())
    }

    fn n_sols(&self) -> usize {
        unsafe { ffi::SCIPgetNSols(self.raw) as usize }
    }

    fn best_sol(&self) -> Solution {
        let sol = unsafe { ffi::SCIPgetBestSol(self.raw) };

        Solution {
            scip_ptr: self.raw,
            raw: sol,
        }
    }

    fn obj_val(&self) -> f64 {
        unsafe { ffi::SCIPgetPrimalbound(self.raw) }
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
        Ok(Variable { raw: var_ptr })
    }

    fn create_priced_var(
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
        Ok(Variable { raw: trans_var_ptr })
    }

    fn create_cons(
        &mut self,
        vars: Vec<Rc<Variable>>,
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
    fn create_cons_set_part(
        &mut self,
        vars: Vec<Rc<Variable>>,
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
    fn create_cons_set_cover(
        &mut self,
        vars: Vec<Rc<Variable>>,
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

    fn create_cons_quadratic(
        &mut self,
        lin_vars: Vec<Rc<Variable>>,
        lin_coefs: &mut [f64],
        quad_vars_1: Vec<Rc<Variable>>,
        quad_vars_2: Vec<Rc<Variable>>,
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

        let get_ptrs = |vars: Vec<Rc<Variable>>| {
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
    fn create_cons_set_pack(
        &mut self,
        vars: Vec<Rc<Variable>>,
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
    fn create_sol(&self) -> Result<Solution, Retcode> {
        let mut sol = MaybeUninit::uninit();
        scip_call! { ffi::SCIPcreateSol(self.raw, sol.as_mut_ptr(), std::ptr::null_mut()) }
        let sol = unsafe { sol.assume_init() };
        Ok(Solution {
            scip_ptr: self.raw,
            raw: sol,
        })
    }

    /// Add coefficient to set packing/partitioning/covering constraint
    fn add_cons_coef_setppc(
        &mut self,
        cons: Rc<Constraint>,
        var: Rc<Variable>,
    ) -> Result<(), Retcode> {
        scip_call! { ffi::SCIPaddCoefSetppc(self.raw, cons.raw, var.raw) };
        Ok(())
    }

    fn lp_branching_cands(scip: *mut ffi::SCIP) -> Vec<BranchingCandidate> {
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
            let var = Rc::new(Variable { raw: var_ptr });
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

    fn branch_var_val(
        scip: *mut ffi::SCIP,
        var: *mut ffi::SCIP_VAR,
        val: f64,
    ) -> Result<(), Retcode> {
        scip_call! { ffi::SCIPbranchVarVal(scip, var, val, std::ptr::null_mut(), std::ptr::null_mut(),std::ptr::null_mut()) };
        Ok(())
    }

    fn include_eventhdlr(
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
            let event_type = unsafe { (*eventhdlr_ptr).var_type() };
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

    fn include_branch_rule(
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

    fn include_pricer(
        &self,
        name: &str,
        desc: &str,
        priority: i32,
        delay: bool,
        pricer: Box<dyn Pricer>,
    ) -> Result<(), Retcode> {
        let c_name = CString::new(name).unwrap();
        let c_desc = CString::new(desc).unwrap();

        fn call_pricer(
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

    fn include_heur(
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

    fn add_cons_coef(
        &mut self,
        cons: Rc<Constraint>,
        var: Rc<Variable>,
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

    fn set_cons_modifiable(
        &mut self,
        cons: Rc<Constraint>,
        modifiable: bool,
    ) -> Result<(), Retcode> {
        scip_call!(ffi::SCIPsetConsModifiable(
            self.raw,
            cons.raw,
            modifiable.into()
        ));
        Ok(())
    }

    fn n_nodes(&self) -> usize {
        unsafe { ffi::SCIPgetNNodes(self.raw) as usize }
    }

    fn solving_time(&self) -> f64 {
        unsafe { ffi::SCIPgetSolvingTime(self.raw) }
    }

    fn n_lp_iterations(&self) -> usize {
        unsafe { ffi::SCIPgetNLPIterations(self.raw) as usize }
    }

    fn focus_node(&self) -> Node {
        Node {
            raw: unsafe { ffi::SCIPgetFocusNode(self.raw) },
        }
    }

    fn create_child(&mut self) -> Result<Node, Retcode> {
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

    fn add_sol(&self, sol: Solution) -> Result<bool, Retcode> {
        let mut stored = MaybeUninit::uninit();
        scip_call!(ffi::SCIPaddSol(self.raw, sol.raw, stored.as_mut_ptr()));
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

/// Represents an optimization model.
#[non_exhaustive]
#[derive(Debug)]
pub struct Model<State> {
    scip: ScipPtr,
    state: State,
}

/// Represents the state of an optimization model that has not yet been solved.
#[derive(Debug)]
pub struct Unsolved;

/// Represents the state of an optimization model where all plugins have been included.
#[derive(Debug)]
pub struct PluginsIncluded;

/// Represents the state of an optimization model where the problem has been created.
#[derive(Debug, Clone)]
pub struct ProblemCreated {
    pub(crate) vars: Rc<RefCell<BTreeMap<VarId, Rc<Variable>>>>,
    pub(crate) conss: Rc<RefCell<Vec<Rc<Constraint>>>>,
}

/// Represents the state of an optimization model that has been solved.
#[derive(Debug)]
pub struct Solved {
    pub(crate) vars: Rc<RefCell<BTreeMap<VarId, Rc<Variable>>>>,
    pub(crate) conss: Rc<RefCell<Vec<Rc<Constraint>>>>,
    pub(crate) best_sol: Option<Solution>,
}

impl Model<Unsolved> {
    /// Creates a new `Model` instance with an `Unsolved` state.
    pub fn new() -> Self {
        Self::try_new().expect("Failed to create SCIP instance")
    }

    /// Tries to create a new `Model` instance with an `Unsolved` state.
    ///
    /// Returns a `Result` with the new `Model` instance on success, or a `Retcode` error on failure.
    pub fn try_new() -> Result<Self, Retcode> {
        let scip_ptr = ScipPtr::new();
        Ok(Model {
            scip: scip_ptr,
            state: Unsolved {},
        })
    }

    /// Includes all default plugins in the SCIP instance and returns a new `Model` instance with a `PluginsIncluded` state.
    pub fn include_default_plugins(mut self) -> Model<PluginsIncluded> {
        self.scip
            .include_default_plugins()
            .expect("Failed to include default plugins");
        Model {
            scip: self.scip,
            state: PluginsIncluded {},
        }
    }

    /// Sets a SCIP string parameter and returns a new `Model` instance with the parameter set.
    pub fn set_str_param(mut self, param: &str, value: &str) -> Result<Self, Retcode> {
        self.scip.set_str_param(param, value)?;
        Ok(self)
    }

    /// Sets a SCIP integer parameter and returns a new `Model` instance with the parameter set.
    pub fn set_int_param(mut self, param: &str, value: i32) -> Result<Self, Retcode> {
        self.scip.set_int_param(param, value)?;
        Ok(self)
    }

    /// Sets a SCIP long integer parameter and returns a new `Model` instance with the parameter set.
    pub fn set_longint_param(mut self, param: &str, value: i64) -> Result<Self, Retcode> {
        self.scip.set_longint_param(param, value)?;
        Ok(self)
    }

    /// Sets a SCIP real parameter and returns a new `Model` instance with the parameter set.
    pub fn set_real_param(mut self, param: &str, value: f64) -> Result<Self, Retcode> {
        self.scip.set_real_param(param, value)?;
        Ok(self)
    }

    /// Sets the presolving parameter of the SCIP instance and returns the same `Model` instance.
    pub fn set_presolving(mut self, presolving: ParamSetting) -> Self {
        self.scip
            .set_presolving(presolving)
            .expect("Failed to set presolving with valid value");
        self
    }

    /// Sets the separating parameter of the SCIP instance and returns the same `Model` instance.
    pub fn set_separating(mut self, separating: ParamSetting) -> Self {
        self.scip
            .set_separating(separating)
            .expect("Failed to set separating with valid value");
        self
    }

    /// Sets the heuristics parameter of the SCIP instance and returns the same `Model` instance.
    pub fn set_heuristics(mut self, heuristics: ParamSetting) -> Self {
        self.scip
            .set_heuristics(heuristics)
            .expect("Failed to set heuristics with valid value");
        self
    }
}

impl Model<PluginsIncluded> {
    /// Creates a new problem in the SCIP instance with the given name and returns a new `Model` instance with a `ProblemCreated` state.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the problem to create.
    ///
    /// # Panics
    ///
    /// This method panics if the problem cannot be created in the current state.
    pub fn create_prob(mut self, name: &str) -> Model<ProblemCreated> {
        self.scip
            .create_prob(name)
            .expect("Failed to create problem in state PluginsIncluded");
        Model {
            scip: self.scip,
            state: ProblemCreated {
                vars: Rc::new(RefCell::new(BTreeMap::new())),
                conss: Rc::new(RefCell::new(Vec::new())),
            },
        }
    }

    /// Reads a problem from the given file and returns a new `Model` instance with a `ProblemCreated` state.
    ///
    /// # Arguments
    ///
    /// * `filename` - The name of the file to read the problem from.
    ///
    /// # Errors
    ///
    /// This method returns a `Retcode` error if the problem cannot be read from the file.
    pub fn read_prob(mut self, filename: &str) -> Result<Model<ProblemCreated>, Retcode> {
        self.scip.read_prob(filename)?;
        let vars = Rc::new(RefCell::new(self.scip.vars()));
        let conss = Rc::new(RefCell::new(self.scip.conss()));
        let new_model = Model {
            scip: self.scip,
            state: ProblemCreated { vars, conss },
        };
        Ok(new_model)
    }
}

impl Model<ProblemCreated> {
    /// Sets the objective sense of the model to the given value and returns the same `Model` instance.
    ///
    /// # Arguments
    ///
    /// * `sense` - The objective sense to set.
    ///
    /// # Panics
    ///
    /// This method panics if the objective sense cannot be set in the current state.
    pub fn set_obj_sense(mut self, sense: ObjSense) -> Self {
        self.scip
            .set_obj_sense(sense)
            .expect("Failed to set objective sense in state ProblemCreated");
        self
    }

    /// Returns a clone of the current model.
    /// The clone is meant for use in implementing custom plugins.
    pub fn clone_for_plugins(&self) -> Self {
        Model {
            scip: self.scip.clone(),
            state: self.state.clone(),
        }
    }

    /// Sets the constraint as modifiable or not.
    pub fn set_cons_modifiable(&mut self, cons: Rc<Constraint>, modifiable: bool) {
        self.scip
            .set_cons_modifiable(cons, modifiable)
            .expect("Failed to set constraint modifiable");
    }

    /// Returns the current node of the model.
    ///
    /// # Panics
    ///
    /// This method panics if not called in the `Solving` state, it should only be used from plugins implementations.
    pub fn focus_node(&self) -> Node {
        self.scip.focus_node()
    }

    /// Creates a new child node of the current node and returns it.
    ///
    /// # Panics
    ///
    /// This method panics if not called from plugins implementations.
    pub fn create_child(&mut self) -> Node {
        self.scip
            .create_child()
            .expect("Failed to create child node in state ProblemCreated")
    }

    /// Adds a new variable to the model with the given lower bound, upper bound, objective coefficient, name, and type.
    ///
    /// # Arguments
    ///
    /// * `lb` - The lower bound of the variable.
    /// * `ub` - The upper bound of the variable.
    /// * `obj` - The objective coefficient of the variable.
    /// * `name` - The name of the variable.
    /// * `var_type` - The type of the variable.
    ///
    /// # Returns
    ///
    /// A reference-counted pointer to the new variable.
    ///
    /// # Panics
    ///
    /// This method panics if the variable cannot be created in the current state.
    pub fn add_var(
        &mut self,
        lb: f64,
        ub: f64,
        obj: f64,
        name: &str,
        var_type: VarType,
    ) -> Rc<Variable> {
        let var = self
            .scip
            .create_var(lb, ub, obj, name, var_type)
            .expect("Failed to create variable in state ProblemCreated");
        let var_id = var.index();
        let var = Rc::new(var);
        self.state.vars.borrow_mut().insert(var_id, var.clone());
        var
    }

    /// Adds a new priced variable to the SCIP data structure.
    ///
    /// # Arguments
    ///
    /// * `lb` - The lower bound of the variable.
    /// * `ub` - The upper bound of the variable.
    /// * `obj` - The objective function coefficient for the variable.
    /// * `name` - The name of the variable. This should be a unique identifier.
    /// * `var_type` - The type of the variable, specified as an instance of the `VarType` enum.
    ///
    /// # Returns
    ///
    /// This function returns a reference-counted smart pointer (`Rc`) to the created `Variable` instance.
    pub fn add_priced_var(
        &mut self,
        lb: f64,
        ub: f64,
        obj: f64,
        name: &str,
        var_type: VarType,
    ) -> Rc<Variable> {
        let var = self
            .scip
            .create_priced_var(lb, ub, obj, name, var_type)
            .expect("Failed to create variable in state ProblemCreated");
        let var = Rc::new(var);
        let var_id = var.index();
        self.state.vars.borrow_mut().insert(var_id, var.clone());
        var
    }

    /// Adds a new constraint to the model with the given variables, coefficients, left-hand side, right-hand side, and name.
    ///
    /// # Arguments
    ///
    /// * `vars` - The variables in the constraint.
    /// * `coefs` - The coefficients of the variables in the constraint.
    /// * `lhs` - The left-hand side of the constraint.
    /// * `rhs` - The right-hand side of the constraint.
    /// * `name` - The name of the constraint.
    ///
    /// # Returns
    ///
    /// A reference-counted pointer to the new constraint.
    ///
    /// # Panics
    ///
    /// This method panics if the constraint cannot be created in the current state.
    pub fn add_cons(
        &mut self,
        vars: Vec<Rc<Variable>>,
        coefs: &[f64],
        lhs: f64,
        rhs: f64,
        name: &str,
    ) -> Rc<Constraint> {
        assert_eq!(vars.len(), coefs.len());
        let cons = self
            .scip
            .create_cons(vars, coefs, lhs, rhs, name)
            .expect("Failed to create constraint in state ProblemCreated");
        let cons = Rc::new(cons);
        self.state.conss.borrow_mut().push(cons.clone());
        cons
    }

    /// Adds a new set partitioning constraint to the model with the given variables and name.
    ///
    /// # Arguments
    ///
    /// * `vars` - The binary variables in the constraint.
    /// * `name` - The name of the constraint.
    ///
    /// # Returns
    ///
    /// A reference-counted pointer to the new constraint.
    ///
    /// # Panics
    ///
    /// This method panics if the constraint cannot be created in the current state, or if any of the variables are not binary.
    pub fn add_cons_set_part(&mut self, vars: Vec<Rc<Variable>>, name: &str) -> Rc<Constraint> {
        assert!(vars.iter().all(|v| v.var_type() == VarType::Binary));
        let cons = self
            .scip
            .create_cons_set_part(vars, name)
            .expect("Failed to add constraint set partition in state ProblemCreated");
        let cons = Rc::new(cons);
        self.state.conss.borrow_mut().push(cons.clone());
        cons
    }

    /// Adds a new set cover constraint to the model with the given variables and name.
    ///
    /// # Arguments
    ///
    /// * `vars` - The binary variables in the constraint.
    /// * `name` - The name of the constraint.
    ///
    /// # Returns
    ///
    /// A reference-counted pointer to the new constraint.
    ///
    /// # Panics
    ///
    /// This method panics if the constraint cannot be created in the current state, or if any of the variables are not binary.
    pub fn add_cons_set_cover(&mut self, vars: Vec<Rc<Variable>>, name: &str) -> Rc<Constraint> {
        assert!(vars.iter().all(|v| v.var_type() == VarType::Binary));
        let cons = self
            .scip
            .create_cons_set_cover(vars, name)
            .expect("Failed to add constraint set cover in state ProblemCreated");
        let cons = Rc::new(cons);
        self.state.conss.borrow_mut().push(cons.clone());
        cons
    }

    /// Adds a new set packing constraint to the model with the given variables and name.
    ///
    /// # Arguments
    ///
    /// * `vars` - The binary variables in the constraint.
    /// * `name` - The name of the constraint.
    ///
    /// # Returns
    ///
    /// A reference-counted pointer to the new constraint.
    ///
    /// # Panics
    ///
    /// This method panics if the constraint cannot be created in the current state, or if any of the variables are not binary.
    pub fn add_cons_set_pack(&mut self, vars: Vec<Rc<Variable>>, name: &str) -> Rc<Constraint> {
        assert!(vars.iter().all(|v| v.var_type() == VarType::Binary));
        let cons = self
            .scip
            .create_cons_set_pack(vars, name)
            .expect("Failed to add constraint set packing in state ProblemCreated");
        let cons = Rc::new(cons);
        self.state.conss.borrow_mut().push(cons.clone());
        cons
    }

    /// Adds a new quadratic constraint to the model with the given variables, coefficients, left-hand side, right-hand side, and name.
    ///
    /// # Arguments
    ///
    /// * `lin_vars` - The linear variables in the constraint.
    /// * `lin_coefs` - The coefficients of the linear variables in the constraint.
    /// * `quad_vars_1` - The first variable in the quadratic constraints.
    /// * `quad_vars_2` - The second variable in the quadratic constraints.
    /// * `quad_coefs` - The coefficients of the quadratic terms in the constraint.
    /// * `lhs` - The left-hand side of the constraint.
    /// * `rhs` - The right-hand side of the constraint.
    /// * `name` - The name of the constraint.
    ///
    /// # Returns
    ///
    /// A reference-counted pointer to the new constraint.
    ///
    /// # Panics
    ///
    /// This method panics if the constraint cannot be created in the current state.
    pub fn add_cons_quadratic(
        &mut self,
        lin_vars: Vec<Rc<Variable>>,
        lin_coefs: &mut [f64],
        quad_vars_1: Vec<Rc<Variable>>,
        quad_vars_2: Vec<Rc<Variable>>,
        quad_coefs: &mut [f64],
        lhs: f64,
        rhs: f64,
        name: &str,
    ) -> Rc<Constraint> {
        assert_eq!(lin_vars.len(), lin_coefs.len());
        assert_eq!(quad_vars_1.len(), quad_vars_2.len());
        assert_eq!(quad_vars_1.len(), quad_coefs.len());
        let cons = self
            .scip
            .create_cons_quadratic(
                lin_vars,
                lin_coefs,
                quad_vars_1,
                quad_vars_2,
                quad_coefs,
                lhs,
                rhs,
                name,
            )
            .expect("Failed to create constraint in state ProblemCreated");
        let cons = Rc::new(cons);
        self.state.conss.borrow_mut().push(cons.clone());
        cons
    }

    /// Adds a coefficient to the given constraint for the given variable and coefficient value.
    ///
    /// # Arguments
    ///
    /// * `cons` - The constraint to add the coefficient to.
    /// * `var` - The variable to add the coefficient for.
    /// * `coef` - The coefficient value to add.
    ///
    /// # Panics
    ///
    /// This method panics if the coefficient cannot be added in the current state.
    pub fn add_cons_coef(&mut self, cons: Rc<Constraint>, var: Rc<Variable>, coef: f64) {
        self.scip
            .add_cons_coef(cons, var, coef)
            .expect("Failed to add constraint coefficient in state ProblemCreated");
    }

    /// Adds a binary variable to the given set partitioning constraint.
    ///
    /// # Arguments
    ///
    /// * `cons` - The constraint to add the variable to.
    /// * `var` - The binary variable to add.
    ///
    /// # Panics
    ///
    /// This method panics if the variable cannot be added in the current state, or if the variable is not binary.
    pub fn add_cons_coef_setppc(&mut self, cons: Rc<Constraint>, var: Rc<Variable>) {
        assert_eq!(var.var_type(), VarType::Binary);
        self.scip
            .add_cons_coef_setppc(cons, var)
            .expect("Failed to add constraint coefficient in state ProblemCreated");
    }

    /// Includes a new branch rule in the model with the given name, description, priority, maximum depth, maximum bound distance, and implementation.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the branching rule. This should be a unique identifier.
    /// * `desc` - A brief description of the branching rule. This is used for informational purposes.
    /// * `priority` - The priority of the branching rule. When SCIP decides which branching rule to call, it considers their priorities. A higher value indicates a higher priority.
    /// * `maxdepth` - The maximum depth level up to which this branching rule should be used. If this is -1, the branching rule can be used at any depth.
    /// * `maxbounddist` - The maximum relative distance from the current node's dual bound to primal bound compared to the best node's dual bound for applying the branching rule. A value of 0.0 means the rule can only be applied on the current best node, while 1.0 means it can be applied on all nodes.
    /// * `rule` - The branching rule to be included. This should be a mutable reference to an object that implements the `BranchRule` trait, and represents the branching rule data.
    ///
    /// # Returns
    ///
    /// This function returns the `Model` instance for which the branching rule was included. This allows for method chaining.
    ///
    /// # Panics
    ///
    /// This method will panic if the inclusion of the branching rule fails. This can happen if another branching rule with the same name already exists.
    pub fn include_branch_rule(
        self,
        name: &str,
        desc: &str,
        priority: i32,
        maxdepth: i32,
        maxbounddist: f64,
        rule: Box<dyn BranchRule>,
    ) -> Self {
        self.scip
            .include_branch_rule(name, desc, priority, maxdepth, maxbounddist, rule)
            .expect("Failed to include branch rule at state ProblemCreated");
        self
    }

    /// Include a new primal heuristic in the model.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the heuristic. This should be a unique identifier.
    /// * `desc` - A brief description of the heuristic. This is used for informational purposes.
    /// * `priority` - The priority of the heuristic. When SCIP decides which heuristic to call, it considers their priorities. A higher value indicates a higher priority.
    /// * `dispchar` - The display character of the heuristic (used in logs).
    /// * `freq` - The frequency for calling the heuristic in the tree; 1 means at every node, 2 means at every other node and so on, -1 turns off the heuristic.
    /// * `freqofs` - The frequency offset for calling the heuristic in the tree; it defines the depth of the branching tree at which the primal heuristic is executed for the first time.
    /// * `maxdepth` - The maximum depth level up to which this heuristic should be used. If this is -1, the heuristic can be used at any depth.
    /// * `timing` - The timing mask of the heuristic.
    /// * `usessubscip` - Should the heuristic use a secondary SCIP instance?
    /// * `heur` - The heuristic to be included. This should be a Box of an object that implements the `Heur` trait, and represents the heuristic data.
    ///
    /// # Returns
    ///
    /// This function returns the `Model` instance for which the heuristic was included. This allows for method chaining.
    pub fn include_heur(
        self,
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
    ) -> Self {
        self.scip
            .include_heur(
                name,
                desc,
                priority,
                dispchar,
                freq,
                freqofs,
                maxdepth,
                timing,
                usessubscip,
                heur,
            )
            .expect("Failed to include heuristic at state ProblemCreated");
        self
    }

    /// Includes a new event handler in the model.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the event handler. This should be a unique identifier.
    /// * `desc` - A brief description of the event handler. This is used for informational purposes.
    /// * `eventhdlr` - The event handler to be included. This should be a mutable reference to an object that implements the `EventHdlr` trait, and represents the event handling logic.
    ///
    /// # Returns
    ///
    /// This function returns the `Model` instance for which the event handler was included. This allows for method chaining.
    pub fn include_eventhdlr(self, name: &str, desc: &str, eventhdlr: Box<dyn Eventhdlr>) -> Self {
        self.scip
            .include_eventhdlr(name, desc, eventhdlr)
            .expect("Failed to include event handler at state ProblemCreated");
        self
    }

    /// Includes a new pricer in the SCIP data structure.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the variable pricer. This should be a unique identifier.
    /// * `desc` - A brief description of the variable pricer.
    /// * `priority` - The priority of the variable pricer. When SCIP decides which pricer to call, it considers their priorities. A higher value indicates a higher priority.
    /// * `delay` - A boolean indicating whether the pricer should be delayed. If true, the pricer is only called when no other pricers or already existing problem variables with negative reduced costs are found. If this is set to false, the pricer may produce columns that already exist in the problem.
    /// * `pricer` - The pricer to be included. This should be a mutable reference to an object that implements the `Pricer` trait.
    ///
    /// # Returns
    ///
    /// This function returns the `Model` instance for which the pricer was included. This allows for method chaining.
    ///
    /// # Panics
    ///
    /// This method will panic if the inclusion of the pricer fails. This can happen if another pricer with the same name already exists.
    pub fn include_pricer(
        self,
        name: &str,
        desc: &str,
        priority: i32,
        delay: bool,
        pricer: Box<dyn Pricer>,
    ) -> Self {
        self.scip
            .include_pricer(name, desc, priority, delay, pricer)
            .expect("Failed to include pricer at state ProblemCreated");
        self
    }

    /// Solves the model and returns a new `Model` instance with a `Solved` state.
    ///
    /// # Returns
    ///
    /// A new `Model` instance with a `Solved` state.
    ///
    /// # Panics
    ///
    /// This method panics if the problem cannot be solved in the current state.
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

    /// Creates a new solution initialized to zero.
    pub fn create_sol(&mut self) -> Solution {
        self.scip
            .create_sol()
            .expect("Failed to create solution in state ProblemCreated")
    }

    /// Adds a solution to the model
    ///
    /// # Returns
    /// A `Result` indicating whether the solution was added successfully.
    pub fn add_sol(&self, sol: Solution) -> Result<(), SolError> {
        let succesfully_stored = self.scip.add_sol(sol).expect("Failed to add solution");
        if succesfully_stored {
            Ok(())
        } else {
            Err(SolError::Infeasible)
        }
    }
}

impl Model<Solved> {
    /// Sets the best solution for the optimization model if one exists.
    fn _set_best_sol(&mut self) {
        if self.scip.n_sols() > 0 {
            self.state.best_sol = Some(self.scip.best_sol());
        }
    }

    /// Returns the best solution for the optimization model, if one exists.
    pub fn best_sol(&self) -> Option<Box<&Solution>> {
        self.state.best_sol.as_ref().map(Box::new)
    }

    /// Returns the number of solutions found by the optimization model.
    pub fn n_sols(&self) -> usize {
        self.scip.n_sols()
    }

    /// Returns the objective value of the best solution found by the optimization model.
    pub fn obj_val(&self) -> f64 {
        self.scip.obj_val()
    }

    /// Returns the number of nodes explored by the optimization model.
    pub fn n_nodes(&self) -> usize {
        self.scip.n_nodes()
    }

    /// Returns the total solving time of the optimization model.
    pub fn solving_time(&self) -> f64 {
        self.scip.solving_time()
    }

    /// Returns the number of LP iterations performed by the optimization model.
    pub fn n_lp_iterations(&self) -> usize {
        self.scip.n_lp_iterations()
    }
}

/// A trait for optimization models with a problem created.
pub trait ModelWithProblem {
    /// Returns a vector of all variables in the optimization model.
    fn vars(&self) -> Vec<Rc<Variable>>;

    /// Returns the variable with the given ID, if it exists.
    fn var(&self, var_id: VarId) -> Option<Rc<Variable>>;

    /// Returns the number of variables in the optimization model.
    fn n_vars(&self) -> usize;

    /// Returns the number of constraints in the optimization model.
    fn n_conss(&mut self) -> usize;

    /// Returns a vector of all constraints in the optimization model.
    fn conss(&mut self) -> Vec<Rc<Constraint>>;

    /// Writes the optimization model to a file with the given path and extension.
    fn write(&self, path: &str, ext: &str) -> Result<(), Retcode>;
}

macro_rules! impl_ModelWithProblem {
    (for $($t:ty),+) => {
        $(impl ModelWithProblem for $t {

            /// Returns a vector of all variables in the optimization model.
            fn vars(&self) -> Vec<Rc<Variable>> {
                self.state.vars.borrow().values().map(Rc::clone).collect()
            }

            /// Returns the variable with the given ID, if it exists.
            fn var(&self, var_id: VarId) -> Option<Rc<Variable>> {
                self.state.vars.borrow().get(&var_id).map(Rc::clone)
            }

            /// Returns the number of variables in the optimization model.
            fn n_vars(&self) -> usize {
                self.scip.n_vars()
            }

            /// Returns the number of constraints in the optimization model.
            fn n_conss(&mut self) -> usize {
                self.scip.n_conss()
            }

            /// Returns a vector of all constraints in the optimization model.
            fn conss(&mut self) -> Vec<Rc<Constraint>> {
                self.state.conss.borrow().iter().map(Rc::clone).collect()
            }

            /// Writes the optimization model to a file with the given path and extension.
            fn write(&self, path: &str, ext: &str) -> Result<(), Retcode> {
                self.scip.write(path, ext)?;
                Ok(())
            }

        })*
    }
}

impl_ModelWithProblem!(for Model<ProblemCreated>, Model<Solved>);

impl<T> Model<T> {
    /// Returns a pointer to the underlying SCIP instance.
    ///
    /// # Safety
    ///
    /// This method is unsafe because it returns a raw pointer to the underlying SCIP instance.
    /// The caller must ensure that the pointer is used safely and correctly.
    #[cfg(feature = "raw")]
    pub unsafe fn scip_ptr(&self) -> *mut ffi::SCIP {
        self.scip.raw
    }

    /// Returns the status of the optimization model.
    pub fn status(&self) -> Status {
        self.scip.status()
    }

    /// Prints the version of SCIP used by the optimization model.
    pub fn print_version(&self) {
        self.scip.print_version()
    }

    /// Hides the output of the optimization model by setting the `display/verblevel` parameter to 0.
    pub fn hide_output(mut self) -> Self {
        self.scip
            .set_int_param("display/verblevel", 0)
            .expect("Failed to set display/verblevel to 0");
        self
    }

    /// Sets the time limit for the optimization model.
    ///
    /// # Arguments
    ///
    /// * `time_limit` - The time limit in seconds.
    pub fn set_time_limit(mut self, time_limit: usize) -> Self {
        self.scip
            .set_real_param("limits/time", time_limit as f64)
            .expect("Failed to set time limit");
        self
    }
}

/// The default implementation for a `Model` instance in the `ProblemCreated` state.
impl Default for Model<ProblemCreated> {
    /// Creates a new `Model` instance with the default plugins included and a problem named "problem".
    fn default() -> Self {
        Model::new()
            .include_default_plugins()
            .create_prob("problem")
    }
}

/// An enum representing the possible settings for a SCIP parameter.
#[derive(Debug)]
pub enum ParamSetting {
    /// Use default values.
    Default,
    /// Set to aggressive settings.
    Aggressive,
    /// Set to fast settings.
    Fast,
    /// Turn off.
    Off,
}

impl From<ParamSetting> for ffi::SCIP_PARAMSETTING {
    /// Converts a `ParamSetting` enum variant into its corresponding `ffi::SCIP_PARAMSETTING` value.
    fn from(val: ParamSetting) -> Self {
        match val {
            ParamSetting::Default => ffi::SCIP_ParamSetting_SCIP_PARAMSETTING_DEFAULT,
            ParamSetting::Aggressive => ffi::SCIP_ParamSetting_SCIP_PARAMSETTING_AGGRESSIVE,
            ParamSetting::Fast => ffi::SCIP_ParamSetting_SCIP_PARAMSETTING_FAST,
            ParamSetting::Off => ffi::SCIP_ParamSetting_SCIP_PARAMSETTING_OFF,
        }
    }
}

/// An enum representing the objective sense of a SCIP optimization model.
#[derive(Debug)]
pub enum ObjSense {
    /// The problem is a minimization problem.
    Minimize,
    /// The problem is a maximization problem.
    Maximize,
}

impl From<ObjSense> for ffi::SCIP_OBJSENSE {
    /// Converts an `ObjSense` enum variant into its corresponding `ffi::SCIP_OBJSENSE` value.
    fn from(val: ObjSense) -> Self {
        match val {
            ObjSense::Maximize => ffi::SCIP_Objsense_SCIP_OBJSENSE_MAXIMIZE,
            ObjSense::Minimize => ffi::SCIP_Objsense_SCIP_OBJSENSE_MINIMIZE,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use crate::status::Status;

    use super::*;

    #[test]
    fn solve_from_lp_file() {
        let mut model = Model::new()
            .hide_output()
            .include_default_plugins()
            .read_prob("data/test/simple.lp")
            .unwrap()
            .solve();
        let status = model.status();
        assert_eq!(status, Status::Optimal);

        //test objective value
        let obj_val = model.obj_val();
        assert_eq!(obj_val, 200.);

        //test constraints
        let conss = model.conss();
        assert_eq!(conss.len(), 2);

        //test solution values
        let sol = model.best_sol().unwrap();
        let vars = model.vars();
        assert_eq!(vars.len(), 2);
        assert_eq!(sol.val(vars[0].clone()), 40.);
        assert_eq!(sol.val(vars[1].clone()), 20.);

        assert_eq!(sol.obj_val(), model.obj_val());
    }

    #[test]
    fn set_time_limit() {
        let model = Model::new()
            .hide_output()
            .set_time_limit(0)
            .include_default_plugins()
            .read_prob("data/test/simple.lp")
            .unwrap()
            .solve();
        let status = model.status();
        assert_eq!(status, Status::TimeLimit);
        assert!(model.solving_time() < 0.5);
        assert_eq!(model.n_nodes(), 0);
        assert_eq!(model.n_lp_iterations(), 0);
    }

    #[test]
    fn add_variable() {
        let mut model = Model::new()
            .hide_output()
            .include_default_plugins()
            .create_prob("test")
            .set_obj_sense(ObjSense::Maximize);
        let x1_id = model
            .add_var(0., f64::INFINITY, 3., "x1", VarType::Integer)
            .index();
        let x2_id = model
            .add_var(0., f64::INFINITY, 4., "x2", VarType::Continuous)
            .index();
        let x1 = model.var(x1_id).unwrap();
        let x2 = model.var(x2_id).unwrap();
        assert_eq!(model.n_vars(), 2);
        assert_eq!(model.vars().len(), 2);
        assert!(x1.raw != x2.raw);
        assert!(x1.var_type() == VarType::Integer);
        assert!(x2.var_type() == VarType::Continuous);
        assert!(x1.name() == "x1");
        assert!(x2.name() == "x2");
        assert!(x1.obj() == 3.);
        assert!(x2.obj() == 4.);
    }

    fn create_model() -> Model<ProblemCreated> {
        let mut model = Model::new()
            .hide_output()
            .include_default_plugins()
            .create_prob("test")
            .set_obj_sense(ObjSense::Maximize);

        let x1 = model.add_var(0., f64::INFINITY, 3., "x1", VarType::Integer);
        let x2 = model.add_var(0., f64::INFINITY, 4., "x2", VarType::Integer);
        model.add_cons(
            vec![x1.clone(), x2.clone()],
            &[2., 1.],
            -f64::INFINITY,
            100.,
            "c1",
        );
        model.add_cons(vec![x1, x2], &[1., 2.], -f64::INFINITY, 80., "c2");

        model
    }

    #[test]
    fn build_model_with_functions() {
        let mut model = create_model();
        assert_eq!(model.vars().len(), 2);
        assert_eq!(model.n_conss(), 2);

        let conss = model.conss();
        assert_eq!(conss.len(), 2);
        assert_eq!(conss[0].name(), "c1");
        assert_eq!(conss[1].name(), "c2");

        let solved_model = model.solve();

        let status = solved_model.status();
        assert_eq!(status, Status::Optimal);

        let obj_val = solved_model.obj_val();
        assert_eq!(obj_val, 200.);

        let sol = solved_model.best_sol().unwrap();
        let vars = solved_model.vars();
        assert_eq!(vars.len(), 2);
        assert_eq!(sol.val(vars[0].clone()), 40.);
        assert_eq!(sol.val(vars[1].clone()), 20.);
    }

    #[test]
    fn unbounded_model() {
        let mut model = Model::default()
            .set_obj_sense(ObjSense::Maximize)
            .hide_output();

        model.add_var(0., f64::INFINITY, 1., "x1", VarType::Integer);
        model.add_var(0., f64::INFINITY, 1., "x2", VarType::Integer);

        let solved_model = model.solve();

        let status = solved_model.status();
        assert_eq!(status, Status::Unbounded);

        let sol = solved_model.best_sol();
        assert!(sol.is_some());
    }

    #[test]
    fn infeasible_model() {
        let mut model = Model::default()
            .set_obj_sense(ObjSense::Maximize)
            .hide_output();

        let var = model.add_var(0., 1., 1., "x1", VarType::Integer);

        model.add_cons(vec![var], &[1.], -f64::INFINITY, -1., "c1");

        let solved_model = model.solve();

        let status = solved_model.status();
        assert_eq!(status, Status::Infeasible);

        assert_eq!(solved_model.n_sols(), 0);
        let sol = solved_model.best_sol();
        assert!(sol.is_none());
    }

    #[cfg(feature = "raw")]
    #[test]
    fn scip_ptr() {
        let mut model = Model::new()
            .hide_output()
            .include_default_plugins()
            .create_prob("test")
            .set_obj_sense(ObjSense::Maximize);

        let x1 = model.add_var(0., f64::INFINITY, 3., "x1", VarType::Integer);
        let x2 = model.add_var(0., f64::INFINITY, 4., "x2", VarType::Integer);
        model.add_cons(
            vec![x1.clone(), x2.clone()],
            &[2., 1.],
            -f64::INFINITY,
            100.,
            "c1",
        );
        model.add_cons(
            vec![x1.clone(), x2.clone()],
            &[1., 2.],
            -f64::INFINITY,
            80.,
            "c2",
        );

        let scip_ptr = unsafe { model.scip_ptr() };
        assert!(!scip_ptr.is_null());
    }

    #[test]
    fn add_cons_coef() {
        let mut model = Model::new()
            .hide_output()
            .include_default_plugins()
            .create_prob("test")
            .set_obj_sense(ObjSense::Maximize);

        let x1 = model.add_var(0., f64::INFINITY, 3., "x1", VarType::Integer);
        let x2 = model.add_var(0., f64::INFINITY, 4., "x2", VarType::Integer);
        let cons = model.add_cons(vec![], &[], -f64::INFINITY, 10., "c1");

        model.add_cons_coef(cons.clone(), x1, 0.); // x1 is unconstrained
        model.add_cons_coef(cons, x2, 10.); // x2 can't be be used

        let solved_model = model.solve();
        let status = solved_model.status();
        assert_eq!(status, Status::Unbounded);
    }

    #[test]
    fn set_cover_partitioning_and_packing() {
        let mut model = Model::new()
            .hide_output()
            .include_default_plugins()
            .create_prob("test")
            .set_obj_sense(ObjSense::Minimize);

        let x1 = model.add_var(0., 1., 3., "x1", VarType::Binary);
        let x2 = model.add_var(0., 1., 4., "x2", VarType::Binary);
        let cons1 = model.add_cons_set_part(vec![], "c");
        model.add_cons_coef_setppc(cons1, x1);

        model.add_cons_set_cover(vec![x2.clone()], "c");
        model.add_cons_set_pack(vec![x2], "c");

        let solved_model = model.solve();
        let status = solved_model.status();
        assert_eq!(status, Status::Optimal);
        assert_eq!(solved_model.obj_val(), 7.);
    }

    #[test]
    fn create_sol() {
        let mut model = Model::new()
            .hide_output()
            .include_default_plugins()
            .create_prob("test")
            .set_obj_sense(ObjSense::Minimize);

        let x1 = model.add_var(0., 1., 3., "x1", VarType::Binary);
        let x2 = model.add_var(0., 1., 4., "x2", VarType::Binary);
        let cons1 = model.add_cons_set_part(vec![], "c");
        model.add_cons_coef_setppc(cons1, x1.clone());

        model.add_cons_set_pack(vec![x2.clone()], "c");

        let sol = model.create_sol();
        assert_eq!(sol.obj_val(), 0.);

        sol.set_val(x1, 1.);
        sol.set_val(x2, 1.);
        assert_eq!(sol.obj_val(), 7.);

        assert!(model.add_sol(sol).is_ok());

        let model = model.solve();
        assert_eq!(model.n_sols(), 2);
    }

    #[test]
    fn quadratic_constraint() {
        // this model should find the maximum manhattan distance a point in a unit-circle can have.
        // This should be 2*sin(pi/4) = sqrt(2).
        let mut model = Model::new()
            .hide_output()
            .include_default_plugins()
            .create_prob("test")
            .set_obj_sense(ObjSense::Maximize);

        let x1 = model.add_var(0., 1., 1., "x1", VarType::Continuous);
        let x2 = model.add_var(0., 1., 1., "x2", VarType::Continuous);

        let _cons = model.add_cons_quadratic(
            vec![],
            &mut [],
            vec![x1.clone(), x2.clone()],
            vec![x1, x2],
            &mut [1., 1.],
            0.,
            1.,
            "circle",
        );

        let solved_model = model.solve();
        let status = solved_model.status();
        assert_eq!(status, Status::Optimal);

        assert!((2f64.sqrt() - solved_model.obj_val()).abs() < 1e-3);
    }

    #[test]
    fn set_str_param() {
        let mut model = Model::new()
            .hide_output()
            .set_str_param("visual/vbcfilename", "test.vbc")
            .unwrap()
            .include_default_plugins()
            .create_prob("test")
            .set_obj_sense(ObjSense::Minimize);

        let x1 = model.add_var(0., 1., 3., "x1", VarType::Binary);
        let x2 = model.add_var(0., 1., 4., "x2", VarType::Binary);
        model.add_cons_set_part(vec![x1, x2], "c");

        let solved_model = model.solve();
        let status = solved_model.status();
        assert_eq!(status, Status::Optimal);
        assert_eq!(solved_model.obj_val(), 3.);

        assert!(Path::new("test.vbc").exists());
        fs::remove_file("test.vbc").unwrap();
    }

    #[test]
    fn set_heurs_presolving_separation() {
        let model = Model::new()
            .hide_output()
            .set_heuristics(ParamSetting::Aggressive)
            .set_presolving(ParamSetting::Fast)
            .set_separating(ParamSetting::Off)
            .include_default_plugins()
            .read_prob("data/test/simple.lp")
            .unwrap()
            .solve();

        assert_eq!(model.status(), Status::Optimal);
    }

    #[test]
    fn write_and_read_lp() {
        let model = create_model();

        model.write("test.lp", "lp").unwrap();

        let read_model = Model::new()
            .include_default_plugins()
            .read_prob("test.lp")
            .unwrap();

        let solved = model.solve();
        let read_solved = read_model.solve();

        assert_eq!(solved.status(), read_solved.status());
        assert_eq!(solved.obj_val(), read_solved.obj_val());

        fs::remove_file("test.lp").unwrap();
    }

    #[test]
    fn print_version() {
        Model::new().print_version();
    }

    #[test]
    fn set_int_param() {
        let res = Model::new()
            .hide_output()
            .set_int_param("display/verblevel", -1)
            .unwrap_err();

        assert_eq!(res, Retcode::ParameterWrongVal);
    }

    #[test]
    fn set_real_param() {
        let model = Model::new()
            .hide_output()
            .set_real_param("limits/time", 0.)
            .unwrap()
            .include_default_plugins()
            .read_prob("data/test/simple.lp")
            .unwrap()
            .solve();

        assert_eq!(model.status(), Status::TimeLimit);
    }
}
