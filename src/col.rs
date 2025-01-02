use std::rc::Rc;
use crate::{ffi, Variable};
use crate::row::BasisStatus;
use crate::scip::ScipPtr;
use crate::row::Row;

struct Col {
    pub(crate) raw: *mut ffi::SCIP_COL,
    pub(crate) scip: Rc<ScipPtr>,
}


impl Col {
    #[cfg(feature = "raw")]
    /// Returns a raw pointer to the underlying `ffi::SCIP_COL` struct.
    pub fn inner(&self) -> *mut ffi::SCIP_COL {
        self.raw
    }

    /// Returns the number of non-zero entries in the column.
    pub fn len(&self) -> usize {
        let len = unsafe { ffi::SCIPcolGetNNonz(self.raw) };
        assert!(len >= 0);
        len as usize
    }

    /// Returns the index of the column.
    pub fn index(&self) -> usize {
        let id = unsafe { ffi::SCIPcolGetIndex(self.raw) };
        assert!(id >= 0);
        id as usize
    }

    /// Returns the objective coefficient of the column.
    pub fn obj(&self) -> f64 {
        unsafe { ffi::SCIPcolGetObj(self.raw) }
    }

    /// Returns the lower bound of the column.
    pub fn lb(&self) -> f64 {
        unsafe { ffi::SCIPcolGetLb(self.raw) }
    }

    /// Returns the upper bound of the column.
    pub fn ub(&self) -> f64 {
        unsafe { ffi::SCIPcolGetUb(self.raw) }
    }

    /// Returns the best bound of the column with respect to the objective function.
    pub fn best_bound(&self) -> f64 {
        unsafe { ffi::SCIPcolGetBestBound(self.raw) }
    }

    /// Returns the variable associated with the column.
    pub fn var(&self) -> Variable {
        let var_ptr = unsafe { ffi::SCIPcolGetVar(self.raw) };
        let var = Variable {
            raw: var_ptr,
            scip: Rc::clone(&self.scip),
        };
        var
    }

    // extern "C" {
    //     #[doc = " gets the primal LP solution of a column"]
    //     pub fn SCIPcolGetPrimsol(col: *mut SCIP_COL) -> f64;
    // }

    /// Returns the primal LP solution of the column.
    pub fn primal_sol(&self) -> f64 {
        unsafe { ffi::SCIPcolGetPrimsol(self.raw) }
    }

    /// Returns the minimal LP solution value, this column ever assumed.
    pub fn min_primal_sol(&self) -> f64 {
        unsafe { ffi::SCIPcolGetMinPrimsol(self.raw) }
    }


    /// Returns the maximal LP solution value, this column ever assumed.
    pub fn max_primal_sol(&self) -> f64 {
        unsafe { ffi::SCIPcolGetMaxPrimsol(self.raw) }
    }


    /// Returns the basis status of a column in the LP solution.
    pub fn basis_status(&self) -> BasisStatus {
        unsafe { ffi::SCIPcolGetBasisStatus(self.raw) }.into()
    }

    /// Returns the probindex of the corresponding variable.
    pub fn var_probindex(&self) -> Option<usize> {
        let probindex = unsafe { ffi::SCIPcolGetVarProbindex(self.raw) };
        if probindex < 0 {
            None
        } else {
            Some(probindex as usize)
        }
    }


    /// Returns whether the column is of integral type.
    pub fn is_integral(&self) -> bool {
        unsafe { ffi::SCIPcolIsIntegral(self.raw) }.into()
    }


    /// Returns whether the column is removable from the LP.
    pub fn is_removable(&self) -> bool {
        unsafe { ffi::SCIPcolIsRemovable(self.raw) }.into()
    }

    /// Returns the position of the column in the current LP.
    pub fn lp_pos(&self) -> Option<usize> {
        let pos = unsafe { ffi::SCIPcolGetLPPos(self.raw) };
        if pos < 0 {
            None
        } else {
            Some(pos as usize)
        }
    }

    /// Returns the depth in the tree where the column entered the LP.
    pub fn lp_depth(&self) -> Option<usize> {
        let depth = unsafe { ffi::SCIPcolGetLPDepth(self.raw) };
        if depth < 0 {
            None
        } else {
            Some(depth as usize)
        }
    }


    /// Returns whether the column is in the current LP.
    pub fn is_in_lp(&self) -> bool {
        unsafe { ffi::SCIPcolIsInLP(self.raw) }.into()
    }

    /// Returns the number of non-zero entries.
    pub fn n_non_zeros(&self) -> usize {
        let n_non_zeros = unsafe { ffi::SCIPcolGetNNonz(self.raw) };
        assert!(n_non_zeros >= 0);
        n_non_zeros as usize
    }

    /// Returns the number of non-zero entries that correspond to rows currently in the LP.
    pub fn n_lp_non_zeros(&self) -> usize {
        let n_lp_non_zeros = unsafe { ffi::SCIPcolGetNLPNonz(self.raw) };
        assert!(n_lp_non_zeros >= 0);
        n_lp_non_zeros as usize
    }


    /// Returns the rows of non-zero entries.
    pub fn rows(&self) -> Vec<Row> {
        let n_non_zeros = self.n_non_zeros();
        let rows_ptr = unsafe { ffi::SCIPcolGetRows(self.raw) };
        let rows = unsafe { std::slice::from_raw_parts(rows_ptr, n_non_zeros) };
        rows.iter().map(|&row_ptr| {
            Row {
                raw: row_ptr,
                scip: Rc::clone(&self.scip),
            }
        }).collect()
    }

    /// Returns the coefficients of non-zero entries.
    pub fn vals(&self) -> Vec<f64> {
        let n_non_zeros = self.n_non_zeros();
        let vals_ptr = unsafe { ffi::SCIPcolGetVals(self.raw) };
        let vals = unsafe { std::slice::from_raw_parts(vals_ptr, n_non_zeros) };
        vals.to_vec()
    }


    /// Returns the node number of the last node in current branch and bound run, where strong branching was used on the given column.
    pub fn strong_branching_node(&self) -> Option<i64> {
        let node = unsafe { ffi::SCIPcolGetStrongbranchNode(self.raw) };
        if node < 0 {
            None
        } else {
            Some(node)
        }
    }


    /// Returns the number of times, strong branching was applied in current run on the given column.
    pub fn n_strong_branches(&self) -> usize {
        let n_strong_branches = unsafe { ffi::SCIPcolGetNStrongbranchs(self.raw) };
        assert!(n_strong_branches >= 0);
        n_strong_branches as usize
    }

    /// Returns the age of a column, i.e., the total number of successive times a column was in the LP and was 0.0 in the solution.
    pub fn age(&self) -> usize {
        let age = unsafe { ffi::SCIPcolGetAge(self.raw) };
        assert!(age >= 0);
        age as usize
    }
}

impl PartialEq for Col {
    fn eq(&self, other: &Self) -> bool {
        self.index() == other.index() && self.raw == other.raw
    }
}