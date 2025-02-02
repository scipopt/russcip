use crate::builder::CanBeAddedToModel;
use crate::{Model, Pricer, ProblemCreated};

/// A builder for easily creating pricers. It can be created using the `pricer` function.
pub struct PricerBuilder<P: Pricer> {
    name: Option<String>,
    desc: Option<String>,
    priority: i32,
    delay: bool,
    pricer: P,
}

impl<P: Pricer> PricerBuilder<P> {
    /// Creates a new `PricerBuilder` with the given pricer.
    ///
    /// # Defaults
    /// - `priority`: 0
    /// - `delay`: false
    pub fn new(pricer: P) -> Self {
        PricerBuilder {
            name: None,
            desc: None,
            priority: 0,
            delay: false,
            pricer,
        }
    }

    /// Sets the name of the pricer.
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    /// Sets the description of the pricer.
    pub fn desc(mut self, desc: &str) -> Self {
        self.desc = Some(desc.to_string());
        self
    }

    /// Sets the priority of the pricer.
    ///
    /// When SCIP decides which pricer to call, it considers their priorities.
    /// A higher value indicates a higher priority.
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Sets whether the pricer should be delayed.
    ///
    /// If true, the pricer is only called when no other pricers or already existing problem
    /// variables with negative reduced costs are found. Otherwise, the pricer may produce columns
    /// that already exist in the problem.
    pub fn delay(mut self, delay: bool) -> Self {
        self.delay = delay;
        self
    }
}

/// Creates a new default `PricerBuilder` from a pricer.
///
/// This function allows you to start the chain:
/// ```rust
///
/// use russcip::{Model, Pricer, PricerResult, SCIPPricer, Solving};///
///
/// use russcip::prelude::pricer;
///
/// struct MyPricer;
/// impl Pricer for MyPricer {fn generate_columns(&mut self, model: Model<Solving>, pricer: SCIPPricer, farkas: bool) -> PricerResult {
///         todo!()
///     }
/// }
///
/// let pricer = pricer(MyPricer)
///     .name("MyPricer")
///     .desc("A custom pricer");
///
/// let mut model = Model::default();
/// model.add(pricer);
/// ```
pub fn pricer<P: Pricer>(pricer: P) -> PricerBuilder<P> {
    PricerBuilder::new(pricer)
}

impl<P: Pricer + 'static> CanBeAddedToModel for PricerBuilder<P> {
    type Return = ();

    fn add(self, model: &mut Model<ProblemCreated>) {
        let name = self.name.unwrap_or_else(|| "".into());
        let desc = self.desc.unwrap_or_else(|| "".into());
        let pricer_box = Box::new(self.pricer);

        model.include_pricer(&name, &desc, self.priority, self.delay, pricer_box);
    }
}
