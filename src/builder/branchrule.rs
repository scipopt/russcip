use crate::builder::CanBeAddedToModel;
use crate::{BranchRule, Model, ProblemCreated};

/// A builder for easily creating branch rules. It can be created using the `branch_rule` function.
pub struct BranchRuleBuilder<R: BranchRule> {
    name: Option<String>,
    desc: Option<String>,
    priority: i32,
    maxdepth: i32,
    maxbounddist: f64,
    rule: R,
}

impl<R: BranchRule> BranchRuleBuilder<R> {
    /// Creates a new `BranchRuleBuilder` with the given branch rule.
    ///
    /// Defaults:
    /// - `name`: empty string
    /// - `desc`: empty string
    /// - `priority`: 100000
    /// - `maxdepth`: -1 (unlimited)
    /// - `maxbounddist`: 1.0 (applies on all nodes)
    pub fn new(rule: R) -> Self {
        BranchRuleBuilder {
            name: None,
            desc: None,
            priority: 100000,
            maxdepth: -1,
            maxbounddist: 1.0,
            rule,
        }
    }

    /// Sets the name of the branch rule.
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    /// Sets the description of the branch rule.
    pub fn desc(mut self, desc: &str) -> Self {
        self.desc = Some(desc.to_string());
        self
    }

    /// Sets the priority of the branch rule.
    ///
    /// When SCIP decides which branch rule to call, it considers their priorities.
    /// A higher value indicates a higher priority.
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Sets the maximum depth level up to which this branch rule should be used.
    ///
    /// If this is -1, the branch rule can be used at any depth.
    pub fn maxdepth(mut self, maxdepth: i32) -> Self {
        self.maxdepth = maxdepth;
        self
    }

    /// Sets the maximum relative bound distance from the current node's dual bound to
    /// primal bound compared to the best node's dual bound for applying the branch rule.
    ///
    /// A value of 0.0 means the rule can only be applied on the current best node,
    /// while 1.0 means it can be applied on all nodes.
    pub fn maxbounddist(mut self, maxbounddist: f64) -> Self {
        self.maxbounddist = maxbounddist;
        self
    }
}

/// Creates a new default `BranchRuleBuilder` from a branch rule.
/// This function allows you to write:
/// ```rust
/// use russcip::{BranchRule, BranchingCandidate, BranchingResult, SCIPBranchRule, Solving};
/// use russcip::prelude::*;
///
/// struct MyBranchRule;
/// impl BranchRule for MyBranchRule {fn execute(&mut self, model: Model<Solving>, branchrule: SCIPBranchRule, candidates: Vec<BranchingCandidate>) -> BranchingResult {
///         todo!()
///     }
/// }
///
/// let rule = branchrule(MyBranchRule)
///     .name("My Branch Rule")
///     .desc("A custom branch rule")
///     .priority(100)
///     .maxdepth(10)
///     .maxbounddist(0.5);
///
/// let mut model = Model::default();
/// model.add(rule);
/// ```
pub fn branchrule<R: BranchRule>(rule: R) -> BranchRuleBuilder<R> {
    BranchRuleBuilder::new(rule)
}

impl<R: BranchRule + 'static> CanBeAddedToModel<ProblemCreated> for BranchRuleBuilder<R> {
    type Return = ();

    fn add(self, model: &mut Model<ProblemCreated>) {
        // Use empty strings as defaults if name or description are not provided.
        let name = self.name.unwrap_or_else(|| "".into());
        let desc = self.desc.unwrap_or_else(|| "".into());
        let rule_box = Box::new(self.rule);
        model.include_branch_rule(
            &name,
            &desc,
            self.priority,
            self.maxdepth,
            self.maxbounddist,
            rule_box,
        );
    }
}

impl<R: BranchRule> From<R> for BranchRuleBuilder<R> {
    fn from(rule: R) -> Self {
        BranchRuleBuilder::new(rule)
    }
}
