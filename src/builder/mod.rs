/// This module contains `BranchRuleBuilder` for easily creating branch rules.
pub mod branchrule;
/// This module contains `ConsBuilder` for easily creating constraints.
pub mod cons;
/// This module contains `EventHdlrBuilder` for easily creating event handlers.
pub mod eventhdlr;
/// This module contains `HeurBuilder` for easily creating heuristics.
pub mod heur;
/// This module contains `PricerBuilder` for easily creating pricers.
pub mod pricer;
/// This module contains `SepaBuilder` for easily creating separators.
pub mod sepa;
/// This module contains `VarBuilder` for easily creating variables.
pub mod var;

use crate::Model;

/// A trait for adding two values together.
pub trait CanBeAddedToModel<Stage> {
    /// The return type after adding to the model (e.g. `Variable` / `Constraint` ).
    type Return;
    /// How to add the value to the model.
    fn add(self, model: &mut Model<Stage>) -> Self::Return;
}

impl<T, I, Stage> CanBeAddedToModel<Stage> for I
where
    T: CanBeAddedToModel<Stage>,
    I: IntoIterator<Item = T>,
{
    type Return = Vec<T::Return>;
    fn add(self, model: &mut Model<Stage>) -> Self::Return {
        self.into_iter().map(|x| x.add(model)).collect()
    }
}
