pub mod c_api;
pub mod model;
pub mod variable;
pub mod constraint;
pub mod status;
pub mod solution;
pub mod retcode;

#[macro_export]
macro_rules! scip_call {
    ($res:expr) => {
        let res = unsafe { $res };
        if res != c_api::SCIP_Retcode_SCIP_OKAY {
            let retcode = crate::retcode::Retcode::from_c_scip_retcode(res).expect(format!("Unknown SCIP return code {}", res).as_str());
            panic!("SCIP call failed with return code {:?}", retcode);
        }
    };
}
