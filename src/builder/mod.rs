/// This module contains `VarBuilder` for easily creating variables.
pub mod var;

use crate::{Model, ProblemCreated};

/// A trait for adding two values together.
pub trait CanBeAddedToModel {
    /// The return type after adding to the model (e.g. `Variable` / `Constraint` ).
    type Return;
    /// How to add the value to the model.
    fn add(self, model: &mut Model<ProblemCreated>) -> Self::Return;
}

impl<T, I> CanBeAddedToModel for I
where
    T: CanBeAddedToModel,
    I: IntoIterator<Item = T>,
{
    type Return = Vec<T::Return>;
    fn add(self, model: &mut Model<ProblemCreated>) -> Self::Return {
        self.into_iter().map(|x| x.add(model)).collect()
    }
}

