use crate::builder::CanBeAddedToModel;
use crate::{Model, ProblemCreated, Separator};

/// A builder for easily creating separators. It can be easily created using the `sepa` function.
pub struct SepaBuilder<S: Separator> {
    name: Option<String>,
    desc: Option<String>,
    priority: i32,
    freq: i32,
    maxbounddist: f64,
    usesubscip: bool,
    delay: bool,
    sepa: S,
}

impl<S: Separator> SepaBuilder<S> {
    /// Create a new `SepaBuilder` with the given separator.
    /// 
    /// # Defaults
    /// - `name`: empty string
    /// - `desc`: empty string
    /// - `priority`: 100000
    /// - `freq`: 1 (called at every node)
    /// - `maxbounddist`: 1.0
    /// - `usesubscip`: false
    /// - `delay`: false
    pub fn new(sepa: S) -> SepaBuilder<S> {
        SepaBuilder {
            name: None,
            desc: None,
            priority: 100000,
            freq: 1,
            maxbounddist: 1.0,
            usesubscip: false,
            delay: false,
            sepa,
        }
    }
}
/// Creates a new default `SepaBuilder`. It can be chained with other methods to set the properties of the separator.
/// # Example
///
/// ```rust
/// use russcip::prelude::*;
///
/// use russcip::{ProblemCreated, SCIPSeparator, SeparationResult, Separator, Solving};
///
/// struct MySeparator;
/// impl Separator for MySeparator {
///     fn execute_lp(&mut self, model: Model<Solving>, sep: SCIPSeparator) -> SeparationResult {
///         todo!()
///     }
/// }
/// let sepa = sepa(MySeparator{}).name("My Separator");
///
/// let mut model = Model::default();
/// model.add(sepa);
/// ```
pub fn sepa<S: Separator>(sepa: S) -> SepaBuilder<S> {
    SepaBuilder::new(sepa)
}

impl<S: Separator> SepaBuilder<S> {
    /// Sets the name of the separator.
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    /// Sets the description of the separator.
    pub fn desc(mut self, desc: &str) -> Self {
        self.desc = Some(desc.to_string());
        self
    }

    /// Sets the priority of the separator.
    /// When SCIP decides which separator to call, it considers their priorities.
    /// A higher value indicates a higher priority.
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Sets the frequency of the separator. 1 means at every node, 2 means at every other node and so on, -1 turns off the separator.
    pub fn freq(mut self, freq: i32) -> Self {
        self.freq = freq;
        self
    }

    /// Sets the maximum relative distance from the current node's dual bound to primal bound compared to the best node's dual bound for applying the separator.
    /// A value of 0.0 means the separator can only be applied on the current best node, while 1.0 means it can be applied on all nodes.
    pub fn maxbounddist(mut self, maxbounddist: f64) -> Self {
        self.maxbounddist = maxbounddist;
        self
    }

    /// Sets whether the separator uses a secondary SCIP instance.
    pub fn usesubscip(mut self, usesubscip: bool) -> Self {
        self.usesubscip = usesubscip;
        self
    }

    /// Sets whether the separator should be delayed.
    pub fn delay(mut self, delay: bool) -> Self {
        self.delay = delay;
        self
    }
}

impl<S: Separator + 'static> CanBeAddedToModel for SepaBuilder<S> {
    type Return = ();
    fn add(self, model: &mut Model<ProblemCreated>) {
        let name = self.name.clone().unwrap_or("".into());
        let desc = self.desc.clone().unwrap_or("".into());

        let sepa = self.sepa;
        let sepa = Box::new(sepa);

        model.include_separator(
            &name,
            &desc,
            self.priority,
            self.freq,
            self.maxbounddist,
            self.usesubscip,
            self.delay,
            sepa,
        );
    }
}
