
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
    const DISABLED: Self = EventMask(0x000000000);
    /// A variable has been added to the transformed problem.
    const VAR_ADDED: Self = EventMask(0x000000001);
    /// A variable will be deleted from the transformed problem.
    const VAR_DELETED: Self = EventMask(0x000000002);
    /// A variable has been fixed, aggregated, or multi-aggregated.
    const VAR_FIXED: Self = EventMask(0x000000004);
    /// The number of rounding locks of a variable was reduced to zero or one.
    const VAR_UNLOCKED: Self = EventMask(0x000000008);
    /// The objective value of a variable has been changed.
    const OBJ_CHANGED: Self = EventMask(0x000000010);
    /// The global lower bound of a variable has been changed.
    const GLB_CHANGED: Self = EventMask(0x000000020);
    /// The global upper bound of a variable has been changed.
    const GUB_CHANGED: Self = EventMask(0x000000040);
    /// The local lower bound of a variable has been increased.
    const LB_TIGHTENED: Self = EventMask(0x000000080);
    /// The local lower bound of a variable has been decreased.
    const LB_RELAXED: Self = EventMask(0x000000100);
    /// The local upper bound of a variable has been decreased.
    const UB_TIGHTENED: Self = EventMask(0x000000200);
    /// The local upper bound of a variable has been increased.
    const UB_RELAXED: Self = EventMask(0x000000400);
    /// A global hole has been added to the hole list of a variable's domain.
    const GHOLE_ADDED: Self = EventMask(0x000000800);
    /// A global hole has been removed from the hole list of a variable's domain.
    const GHOLE_REMOVED: Self = EventMask(0x000001000);
    /// A local hole has been added to the hole list of a variable's domain.
    const LHOLE_ADDED: Self = EventMask(0x000002000);
    /// A local hole has been removed from the hole list of a variable's domain.
    const LHOLE_REMOVED: Self = EventMask(0x000004000);
    /// The variable's implication list, variable bound, or clique information was extended.
    const IMPL_ADDED: Self = EventMask(0x000008000);
    /// The type of a variable has changed.
    const TYPE_CHANGED: Self = EventMask(0x000010000);
    /// A presolving round has been finished.
    const PRESOLVE_ROUND: Self = EventMask(0x000020000);
    /// A node has been focused and is now the focus node.
    const NODE_FOCUSED: Self = EventMask(0x000040000);
    /// The LP/pseudo solution of the node was feasible.
    const NODE_FEASIBLE: Self = EventMask(0x000080000);
    /// The focus node has been proven to be infeasible or was bounded.
    const NODE_INFEASIBLE: Self = EventMask(0x000100000);
    /// The focus node has been solved by branching.
    const NODE_BRANCHED: Self = EventMask(0x000200000);
    /// A node is about to be deleted from the tree.
    const NODE_DELETE: Self = EventMask(0x000400000);
    /// The node's initial LP was solved.
    const FIRST_LP_SOLVED: Self = EventMask(0x000800000);
    /// The node's LP was completely solved with cut & price.
    const LP_SOLVED: Self = EventMask(0x001000000);
    /// A good enough primal feasible (but not new best) solution was found.
    const POOR_SOL_FOUND: Self = EventMask(0x002000000);
    /// A new best primal feasible solution was found.
    const BEST_SOL_FOUND: Self = EventMask(0x004000000);
    /// A row has been added to SCIP's separation storage.
    const ROW_ADDED_SEPA: Self = EventMask(0x008000000);
    /// A row has been removed from SCIP's separation storage.
    const ROW_DELETED_SEPA: Self = EventMask(0x010000000);
    /// A row has been added to the LP.
    const ROW_ADDED_LP: Self = EventMask(0x020000000);
    /// A row has been removed from the LP.
    const ROW_DELETED_LP: Self = EventMask(0x040000000);
    /// A coefficient of a row has been changed (row specific event).
    const ROW_COEF_CHANGED: Self = EventMask(0x080000000);
    /// The constant of a row has been changed (row specific event).
    const ROW_CONST_CHANGED: Self = EventMask(0x100000000);
    /// A side of a row has been changed (row specific event).
    const ROW_SIDE_CHANGED: Self = EventMask(0x200000000);
    /// Synchronization event.
    const SYNC: Self = EventMask(0x400000000);
    /// Event mask for the change of both global lower bound and global upper bound of a variable.
    /// Event mask for the change of both global lower bound and global upper bound of a variable.
    const GBD_CHANGED: Self = Self(Self::GLB_CHANGED.0 | Self::GUB_CHANGED.0);
    /// Event mask for the change of the local lower bound of a variable.
    const LB_CHANGED: Self = Self(Self::LB_TIGHTENED.0 | Self::LB_RELAXED.0);
    /// Event mask for the change of the local upper bound of a variable.
    const UB_CHANGED: Self = Self(Self::UB_TIGHTENED.0 | Self::UB_RELAXED.0);
    /// Event mask for the tightening of both bounds of a variable.
    const BOUND_TIGHTENED: Self = Self(Self::LB_TIGHTENED.0 | Self::UB_TIGHTENED.0);
    /// Event mask for the relaxing of both bounds of a variable.
    const BOUND_RELAXED: Self = Self(Self::LB_RELAXED.0 | Self::UB_RELAXED.0);
    /// Event mask for the change of both bounds of a variable.
    const BOUND_CHANGED: Self = Self(Self::LB_CHANGED.0 | Self::UB_CHANGED.0);
    /// Event mask for the change of both global holes of a variable's domain.
    const GHOLE_CHANGED: Self = Self(Self::GHOLE_ADDED.0 | Self::GHOLE_REMOVED.0);
    /// Event mask for the change of both local holes of a variable's domain.
    const LHOLE_CHANGED: Self = Self(Self::LHOLE_ADDED.0 | Self::LHOLE_REMOVED.0);
    /// Event mask for the change of any hole of a variable's domain.
    const HOLE_CHANGED: Self = Self(Self::GHOLE_CHANGED.0 | Self::LHOLE_CHANGED.0);
    /// Event mask for the change of the domain of a variable.
    const DOM_CHANGED: Self = Self(Self::BOUND_CHANGED.0 | Self::HOLE_CHANGED.0);
    /// Event mask for the change of a variable's properties.
    const VAR_CHANGED: Self = Self(Self::VAR_FIXED.0 | Self::VAR_UNLOCKED.0 | Self::OBJ_CHANGED.0 | Self::GBD_CHANGED.0 | Self::DOM_CHANGED.0 | Self::IMPL_ADDED.0 | Self::VAR_DELETED.0 | Self::TYPE_CHANGED.0);
    /// Event mask for variable-related events.
    const VAR_EVENT: Self = Self(Self::VAR_ADDED.0 | Self::VAR_CHANGED.0 | Self::TYPE_CHANGED.0);
    /// Event mask for node-related events.
    const NODE_SOLVED: Self = Self(Self::NODE_FEASIBLE.0 | Self::NODE_INFEASIBLE.0 | Self::NODE_BRANCHED.0);
    /// Event mask for node-related events.
    const NODE_EVENT: Self = Self(Self::NODE_FOCUSED.0 | Self::NODE_SOLVED.0);
    /// Event mask for LP-related events.
    const LP_EVENT: Self = Self(Self::FIRST_LP_SOLVED.0 | Self::LP_SOLVED.0);
    /// Event mask for primal solution-related events.
    const SOL_FOUND: Self = Self(Self::POOR_SOL_FOUND.0 | Self::BEST_SOL_FOUND.0);
    /// Event mask for primal solution-related events.
    const SOL_EVENT: Self = Self(Self::SOL_FOUND.0);
    /// Event mask for row-related events.
    const ROW_CHANGED: Self = Self(Self::ROW_COEF_CHANGED.0 | Self::ROW_CONST_CHANGED.0 | Self::ROW_SIDE_CHANGED.0);
    /// Event mask for row-related events.
    const ROW_EVENT: Self = Self(Self::ROW_ADDED_SEPA.0 | Self::ROW_DELETED_SEPA.0 | Self::ROW_ADDED_LP.0 | Self::ROW_DELETED_LP.0 | Self::ROW_CHANGED.0);
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
    use crate::eventhdlr::{Eventhdlr, EventMask };
    use crate::model::Model;

    struct PanickingEventHdlr;

    impl Eventhdlr for PanickingEventHdlr {
        fn get_type(&self) -> EventMask {
            EventMask::LP_EVENT | EventMask::NODE_EVENT
        }

        fn execute(&mut self) {
           panic!("Panic!");
        }
    }

    #[test]
    #[should_panic]
    fn test_eventhdlr() {
        let eh = PanickingEventHdlr {};

        Model::new()
            .hide_output()
            .include_default_plugins()
            .read_prob("data/test/simple.lp")
            .unwrap()
            .include_eventhdlr("PanickingEventHdlr", "",  Box::new(eh))
            .solve();
    }
}




