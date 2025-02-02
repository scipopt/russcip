use crate::builder::CanBeAddedToModel;
use crate::{HeurTiming, Heuristic, Model, ProblemCreated};

/// A builder for easily creating primal heuristics. It can be created using the `heur` function.
pub struct HeurBuilder<H: Heuristic> {
    name: Option<String>,
    desc: Option<String>,
    priority: i32,
    dispchar: Option<char>,
    freq: i32,
    freqofs: i32,
    maxdepth: i32,
    timing: Option<HeurTiming>,
    usessubscip: bool,
    heur: H,
}

impl<H: Heuristic> HeurBuilder<H> {
    /// Creates a new `HeurBuilder` with the given heuristic.
    ///
    /// # Defaults
    /// - `priority`: 0
    /// - `freq`: 1 (called at every node)
    /// - `freqofs`: 0
    /// - `maxdepth`: -1 (unlimited depth)
    /// - `usessubscip`: false
    ///
    /// For fields without a provided default (e.g. `name`, `desc`, `dispchar`, `timing`),
    /// empty strings or reasonable defaults are used if not explicitly set.
    pub fn new(heur: H) -> Self {
        HeurBuilder {
            name: None,
            desc: None,
            priority: 0,
            dispchar: None,
            freq: 1,
            freqofs: 0,
            maxdepth: -1,
            timing: None,
            usessubscip: false,
            heur,
        }
    }

    /// Sets the name of the heuristic.
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    /// Sets the description of the heuristic.
    pub fn desc(mut self, desc: &str) -> Self {
        self.desc = Some(desc.to_string());
        self
    }

    /// Sets the priority of the heuristic.
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Sets the display character of the heuristic.
    pub fn dispchar(mut self, dispchar: char) -> Self {
        self.dispchar = Some(dispchar);
        self
    }

    /// Sets the frequency for calling the heuristic.
    pub fn freq(mut self, freq: i32) -> Self {
        self.freq = freq;
        self
    }

    /// Sets the frequency offset for calling the heuristic.
    pub fn freqofs(mut self, freqofs: i32) -> Self {
        self.freqofs = freqofs;
        self
    }

    /// Sets the maximum depth up to which the heuristic is used.
    pub fn maxdepth(mut self, maxdepth: i32) -> Self {
        self.maxdepth = maxdepth;
        self
    }

    /// Sets the timing mask of the heuristic.
    pub fn timing(mut self, timing: HeurTiming) -> Self {
        self.timing = Some(timing);
        self
    }

    /// Sets whether the heuristic should use a secondary SCIP instance.
    pub fn usessubscip(mut self, usessubscip: bool) -> Self {
        self.usessubscip = usessubscip;
        self
    }
}

/// Creates a new default `HeurBuilder` from a heuristic.
/// This function allows you to start chaining:
///
/// ```rust
///
/// use russcip::{HeurResult, HeurTiming, Heuristic, Model, Solving};///
///
/// use russcip::prelude::heur;
///
/// struct MyHeur;
/// impl Heuristic for MyHeur {fn execute(&mut self, model: Model<Solving>, timing: HeurTiming, node_inf: bool) -> HeurResult {
///         todo!()
///     }
/// }
///
/// let my_heur = heur(MyHeur)
///     .name("MyHeuristic")
///     .desc("A custom primal heuristic")
///     .priority(100)
///     .dispchar('H')
///     .freq(1)
///     .freqofs(0)
///     .maxdepth(10)
///     .timing(HeurTiming::BEFORE_NODE | HeurTiming::AFTER_LP_LOOP)
///     .usessubscip(true);
///
/// let mut model = Model::default();
/// model.add(my_heur);
/// ```
pub fn heur<H: Heuristic>(heur: H) -> HeurBuilder<H> {
    HeurBuilder::new(heur)
}

impl<H: Heuristic + 'static> CanBeAddedToModel for HeurBuilder<H> {
    type Return = ();

    fn add(self, model: &mut Model<ProblemCreated>) {
        // Provide default values if not set:
        let name = self.name.unwrap_or_else(|| "".into());
        let desc = self.desc.unwrap_or_else(|| "".into());
        let dispchar = self.dispchar.unwrap_or('?');
        let timing = self.timing.unwrap_or_else(|| HeurTiming::BEFORE_NODE);
        let heur_box = Box::new(self.heur);
        model.include_heur(
            &name,
            &desc,
            self.priority,
            dispchar,
            self.freq,
            self.freqofs,
            self.maxdepth,
            timing,
            self.usessubscip,
            heur_box,
        );
    }
}
