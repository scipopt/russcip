#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod variable;
pub mod c_api;
pub mod model;
pub mod status;
pub mod solution;
pub mod retcode;

#[macro_export]
macro_rules! scip_call {
    ($res:expr) => {
        let res = unsafe { $res };
        if res != c_api::SCIP_Retcode_SCIP_OKAY {
            return Err(SCIPRetcode::from_c_scip_retcode(res).unwrap());
        }
    };
}
