use crate::builder::CanBeAddedToModel;
use crate::{
    Constraint, Model, ModelStageProblemOrSolving, ModelStageWithProblem, Row, SCIPConshdlr,
    SCIPSeparator,
};

/// A builder for creating constraints.
#[derive(Debug)]
pub struct RowBuilder<'a> {
    /// Left-hand side of constraint
    pub(crate) lhs: f64,
    /// Right-hand side of constraint
    pub(crate) rhs: f64,
    /// (Optional) name of constraint
    pub(crate) name: Option<&'a str>,
    /// Modifiable flag of constraint
    pub(crate) modifiable: Option<bool>,
    /// Removable flag of constraint
    pub(crate) removable: Option<bool>,
    /// Local flag of constraint
    pub(crate) local: Option<bool>,
    /// Source of the row
    pub(crate) source: Option<RowSource<'a>>,
}

#[derive(Debug)]
pub enum RowSource<'a> {
    /// The row comes from a separator
    Separator(&'a SCIPSeparator),
    /// The row comes from a constraint handler
    ConstraintHandler(&'a SCIPConshdlr),
    /// The row comes from a specific constraint
    Constraint(&'a Constraint),
}

/// Creates a new default `ConsBuilder`.
pub fn row() -> RowBuilder<'static> {
    RowBuilder::default()
}

impl Default for RowBuilder<'_> {
    fn default() -> Self {
        RowBuilder {
            lhs: f64::NEG_INFINITY,
            rhs: f64::INFINITY,
            name: None,
            modifiable: None,
            removable: None,
            local: None,
            source: None,
        }
    }
}

impl<'a> RowBuilder<'a> {
    /// Creates a row of the form `expr <= val`.
    pub fn le(mut self, val: f64) -> Self {
        self.rhs = val;
        self.lhs = f64::NEG_INFINITY;
        self
    }

    /// Creates a row of the form `val <= expr`.
    pub fn ge(mut self, val: f64) -> Self {
        self.lhs = val;
        self.rhs = f64::INFINITY;
        self
    }

    /// Creates a row of the form `expr = val`.
    pub fn eq(mut self, val: f64) -> Self {
        self.lhs = val;
        self.rhs = val;
        self
    }

    /// Sets both bounds of the row
    pub fn bounds(mut self, lhs: f64, rhs: f64) -> Self {
        self.lhs = lhs;
        self.rhs = rhs;
        self
    }

    /// Sets the name of the row.
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = Some(name);
        self
    }

    /// Sets the modifiable flag of the row
    pub fn modifiable(mut self, modifiable: bool) -> Self {
        self.modifiable = Some(modifiable);
        self
    }

    /// Sets the removable flag of the row
    pub fn removable(mut self, removable: bool) -> Self {
        self.removable = Some(removable);
        self
    }

    /// Sets whether the row is only valid locally
    pub fn local(mut self, separate: bool) -> Self {
        self.local = Some(separate);
        self
    }

    /// sets the source of the row
    pub fn source(mut self, source: RowSource<'a>) -> Self {
        self.source = Some(source);
        self
    }
}

impl<S> CanBeAddedToModel<S> for RowBuilder<'_>
where
    S: ModelStageProblemOrSolving + ModelStageWithProblem,
{
    type Return = Row;
    fn add(self, model: &mut Model<S>) -> Self::Return {
        let row_ptr = model
            .scip
            .create_empty_row(&self)
            .expect("Failed to create row");

        Row {
            raw: row_ptr,
            scip: model.scip.clone(),
        }
    }
}
