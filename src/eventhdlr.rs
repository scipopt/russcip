use std::ops::{BitOr, BitOrAssign};

/// Trait used to define custom event handlers.
pub trait Eventhdlr {
    /// Returns the type of the event handler.
    fn get_type(&self) -> EventMask;
    /// Executes the event handler.
    fn execute(&mut self);
}

/// The EventMask represents different states or actions within an optimization problem.
#[derive(Debug, Copy, Clone)]
pub struct EventMask(u64);

impl EventMask {
    /// The event was disabled and has no effect any longer.
    pub const DISABLED: Self = EventMask(0x000000000);
    /// A variable has been added to the transformed problem.
    pub const VAR_ADDED: Self = EventMask(0x000000001);
    /// A variable will be deleted from the transformed problem.
    pub const VAR_DELETED: Self = EventMask(0x000000002);
    /// A variable has been fixed, aggregated, or multi-aggregated.
    pub const VAR_FIXED: Self = EventMask(0x000000004);
    /// The number of rounding locks of a variable was reduced to zero or one.
    pub const VAR_UNLOCKED: Self = EventMask(0x000000008);
    /// The objective value of a variable has been changed.
    pub const OBJ_CHANGED: Self = EventMask(0x000000010);
    /// The global lower bound of a variable has been changed.
    pub const GLB_CHANGED: Self = EventMask(0x000000020);
    /// The global upper bound of a variable has been changed.
    pub const GUB_CHANGED: Self = EventMask(0x000000040);
    /// The local lower bound of a variable has been increased.
    pub const LB_TIGHTENED: Self = EventMask(0x000000080);
    /// The local lower bound of a variable has been decreased.
    pub const LB_RELAXED: Self = EventMask(0x000000100);
    /// The local upper bound of a variable has been decreased.
    pub const UB_TIGHTENED: Self = EventMask(0x000000200);
    /// The local upper bound of a variable has been increased.
    pub const UB_RELAXED: Self = EventMask(0x000000400);
    /// A global hole has been added to the hole list of a variable's domain.
    pub const GHOLE_ADDED: Self = EventMask(0x000000800);
    /// A global hole has been removed from the hole list of a variable's domain.
    pub const GHOLE_REMOVED: Self = EventMask(0x000001000);
    /// A local hole has been added to the hole list of a variable's domain.
    pub const LHOLE_ADDED: Self = EventMask(0x000002000);
    /// A local hole has been removed from the hole list of a variable's domain.
    pub const LHOLE_REMOVED: Self = EventMask(0x000004000);
    /// The variable's implication list, variable bound, or clique information was extended.
    pub const IMPL_ADDED: Self = EventMask(0x000008000);
    /// The type of a variable has changed.
    pub const TYPE_CHANGED: Self = EventMask(0x000010000);
    /// A presolving round has been finished.
    pub const PRESOLVE_ROUND: Self = EventMask(0x000020000);
    /// A node has been focused and is now the focus node.
    pub const NODE_FOCUSED: Self = EventMask(0x000040000);
    /// The LP/pseudo solution of the node was feasible.
    pub const NODE_FEASIBLE: Self = EventMask(0x000080000);
    /// The focus node has been proven to be infeasible or was bounded.
    pub const NODE_INFEASIBLE: Self = EventMask(0x000100000);
    /// The focus node has been solved by branching.
    pub const NODE_BRANCHED: Self = EventMask(0x000200000);
    /// A node is about to be deleted from the tree.
    pub const NODE_DELETE: Self = EventMask(0x000400000);
    /// The node's initial LP was solved.
    pub const FIRST_LP_SOLVED: Self = EventMask(0x000800000);
    /// The node's LP was completely solved with cut & price.
    pub const LP_SOLVED: Self = EventMask(0x001000000);
    /// A good enough primal feasible (but not new best) solution was found.
    pub const POOR_SOL_FOUND: Self = EventMask(0x002000000);
    /// A new best primal feasible solution was found.
    pub const BEST_SOL_FOUND: Self = EventMask(0x004000000);
    /// A row has been added to SCIP's separation storage.
    pub const ROW_ADDED_SEPA: Self = EventMask(0x008000000);
    /// A row has been removed from SCIP's separation storage.
    pub const ROW_DELETED_SEPA: Self = EventMask(0x010000000);
    /// A row has been added to the LP.
    pub const ROW_ADDED_LP: Self = EventMask(0x020000000);
    /// A row has been removed from the LP.
    pub const ROW_DELETED_LP: Self = EventMask(0x040000000);
    /// A coefficient of a row has been changed (row specific event).
    pub const ROW_COEF_CHANGED: Self = EventMask(0x080000000);
    /// The constant of a row has been changed (row specific event).
    pub const ROW_CONST_CHANGED: Self = EventMask(0x100000000);
    /// A side of a row has been changed (row specific event).
    pub const ROW_SIDE_CHANGED: Self = EventMask(0x200000000);
    /// Synchronization event.
    pub const SYNC: Self = EventMask(0x400000000);
    /// Event mask for the change of both global lower bound and global upper bound of a variable.
    /// Event mask for the change of both global lower bound and global upper bound of a variable.
    pub const GBD_CHANGED: Self = Self(Self::GLB_CHANGED.0 | Self::GUB_CHANGED.0);
    /// Event mask for the change of the local lower bound of a variable.
    pub const LB_CHANGED: Self = Self(Self::LB_TIGHTENED.0 | Self::LB_RELAXED.0);
    /// Event mask for the change of the local upper bound of a variable.
    pub const UB_CHANGED: Self = Self(Self::UB_TIGHTENED.0 | Self::UB_RELAXED.0);
    /// Event mask for the tightening of both bounds of a variable.
    pub const BOUND_TIGHTENED: Self = Self(Self::LB_TIGHTENED.0 | Self::UB_TIGHTENED.0);
    /// Event mask for the relaxing of both bounds of a variable.
    pub const BOUND_RELAXED: Self = Self(Self::LB_RELAXED.0 | Self::UB_RELAXED.0);
    /// Event mask for the change of both bounds of a variable.
    pub const BOUND_CHANGED: Self = Self(Self::LB_CHANGED.0 | Self::UB_CHANGED.0);
    /// Event mask for the change of both global holes of a variable's domain.
    pub const GHOLE_CHANGED: Self = Self(Self::GHOLE_ADDED.0 | Self::GHOLE_REMOVED.0);
    /// Event mask for the change of both local holes of a variable's domain.
    pub const LHOLE_CHANGED: Self = Self(Self::LHOLE_ADDED.0 | Self::LHOLE_REMOVED.0);
    /// Event mask for the change of any hole of a variable's domain.
    pub const HOLE_CHANGED: Self = Self(Self::GHOLE_CHANGED.0 | Self::LHOLE_CHANGED.0);
    /// Event mask for the change of the domain of a variable.
    pub const DOM_CHANGED: Self = Self(Self::BOUND_CHANGED.0 | Self::HOLE_CHANGED.0);
    /// Event mask for the change of a variable's properties.
    pub const VAR_CHANGED: Self = Self(
        Self::VAR_FIXED.0
            | Self::VAR_UNLOCKED.0
            | Self::OBJ_CHANGED.0
            | Self::GBD_CHANGED.0
            | Self::DOM_CHANGED.0
            | Self::IMPL_ADDED.0
            | Self::VAR_DELETED.0
            | Self::TYPE_CHANGED.0,
    );
    /// Event mask for variable-related events.
    pub const VAR_EVENT: Self =
        Self(Self::VAR_ADDED.0 | Self::VAR_CHANGED.0 | Self::TYPE_CHANGED.0);
    /// Event mask for node-related events.
    pub const NODE_SOLVED: Self =
        Self(Self::NODE_FEASIBLE.0 | Self::NODE_INFEASIBLE.0 | Self::NODE_BRANCHED.0);
    /// Event mask for node-related events.
    pub const NODE_EVENT: Self = Self(Self::NODE_FOCUSED.0 | Self::NODE_SOLVED.0);
    /// Event mask for LP-related events.
    pub const LP_EVENT: Self = Self(Self::FIRST_LP_SOLVED.0 | Self::LP_SOLVED.0);
    /// Event mask for primal solution-related events.
    pub const SOL_FOUND: Self = Self(Self::POOR_SOL_FOUND.0 | Self::BEST_SOL_FOUND.0);
    /// Event mask for primal solution-related events.
    pub const SOL_EVENT: Self = Self(Self::SOL_FOUND.0);
    /// Event mask for row-related events.
    pub const ROW_CHANGED: Self =
        Self(Self::ROW_COEF_CHANGED.0 | Self::ROW_CONST_CHANGED.0 | Self::ROW_SIDE_CHANGED.0);
    /// Event mask for row-related events.
    pub const ROW_EVENT: Self = Self(
        Self::ROW_ADDED_SEPA.0
            | Self::ROW_DELETED_SEPA.0
            | Self::ROW_ADDED_LP.0
            | Self::ROW_DELETED_LP.0
            | Self::ROW_CHANGED.0,
    );
}

impl BitOr for EventMask {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        EventMask(self.0 | rhs.0)
    }
}

impl BitOrAssign for EventMask {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl From<EventMask> for u64 {
    fn from(mask: EventMask) -> Self {
        mask.0
    }
}

#[cfg(test)]
mod tests {
    use crate::eventhdlr::{EventMask, Eventhdlr};
    use crate::model::Model;
    use std::cell::RefCell;
    use std::rc::Rc;

    struct CountingEventHdlr {
        counter: Rc<RefCell<usize>>,
    }

    impl Eventhdlr for CountingEventHdlr {
        fn get_type(&self) -> EventMask {
            EventMask::LP_EVENT | EventMask::NODE_EVENT
        }

        fn execute(&mut self) {
            *self.counter.borrow_mut() += 1;
        }
    }

    #[test]
    fn test_eventhdlr() {
        let counter = Rc::new(RefCell::new(0));
        let eh = CountingEventHdlr {
            counter: counter.clone(),
        };

        Model::new()
            .hide_output()
            .include_default_plugins()
            .read_prob("data/test/simple.lp")
            .unwrap()
            .include_eventhdlr("PanickingEventHdlr", "", Box::new(eh))
            .solve();

        assert!(*counter.borrow() > 1);
    }
}
