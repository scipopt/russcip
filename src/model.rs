use std::rc::Rc;
use scip_sys::SCIP;
use crate::constraint::Constraint;
use crate::eventhdlr::Eventhdlr;
use crate::node::Node;
use crate::retcode::Retcode;
use crate::scip::ScipPtr;
use crate::solution::{SolError, Solution};
use crate::status::Status;
use crate::variable::{VarId, VarType, Variable};
use crate::{ffi, Row, Separator};
use crate::{BranchRule, HeurTiming, Heuristic, Pricer};

/// Represents an optimization model.
#[non_exhaustive]
#[derive(Debug)]
pub struct Model<State> {
    pub(crate) scip: Rc<ScipPtr>,
    #[allow(dead_code)]
    pub(crate) state: State,
}

/// Represents the state of an optimization model that has not yet been solved.
#[derive(Debug)]
pub struct Unsolved;

/// Represents the state of an optimization model where all plugins have been included.
#[derive(Debug)]
pub struct PluginsIncluded;

/// Represents the state of an optimization model where the problem has been created.
#[derive(Debug, Clone)]
pub struct ProblemCreated;

/// Represents the state of an optimization model during the solving process (to be used in plugins).
#[derive(Debug)]
pub struct Solving;

/// Represents the state of an optimization model that has been solved.
#[derive(Debug)]
pub struct Solved;

impl Model<Unsolved> {
    /// Creates a new `Model` instance with an `Unsolved` state.
    pub fn new() -> Self {
        Self::try_new().expect("Failed to create SCIP instance")
    }

    /// Tries to create a new `Model` instance with an `Unsolved` state.
    ///
    /// Returns a `Result` with the new `Model` instance on success, or a `Retcode` error on failure.
    pub fn try_new() -> Result<Self, Retcode> {
        let scip_ptr = ScipPtr::new();
        Ok(Model {
            scip: Rc::new(scip_ptr),
            state: Unsolved {},
        })
    }
}

impl Model<PluginsIncluded> {
    /// Creates a new problem in the SCIP instance with the given name and returns a new `Model` instance with a `ProblemCreated` state.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the problem to create.
    ///
    /// # Panics
    ///
    /// This method panics if the problem cannot be created in the current state.
    #[allow(unused_mut)]
    pub fn create_prob(mut self, name: &str) -> Model<ProblemCreated> {
        let mut scip = self.scip.clone();
        scip.create_prob(name)
            .expect("Failed to create problem in state PluginsIncluded");
        Model {
            scip,
            state: ProblemCreated {},
        }
    }

    /// Reads a problem from the given file and returns a new `Model` instance with a `ProblemCreated` state.
    ///
    /// # Arguments
    ///
    /// * `filename` - The name of the file to read the problem from.
    ///
    /// # Errors
    ///
    /// This method returns a `Retcode` error if the problem cannot be read from the file.
    #[allow(unused_mut)]
    pub fn read_prob(mut self, filename: &str) -> Result<Model<ProblemCreated>, Retcode> {
        let scip = self.scip.clone();
        scip.read_prob(filename)?;
        let new_model = Model {
            scip: self.scip,
            state: ProblemCreated {},
        };
        Ok(new_model)
    }
}

impl Model<ProblemCreated> {
    /// Sets the objective sense of the model to the given value and returns the same `Model` instance.
    ///
    /// # Arguments
    ///
    /// * `sense` - The objective sense to set.
    ///
    /// # Panics
    ///
    /// This method panics if the objective sense cannot be set in the current state.
    pub fn set_obj_sense(mut self, sense: ObjSense) -> Self {
        let scip = self.scip.clone();
        scip.set_obj_sense(sense)
            .expect("Failed to set objective sense in state ProblemCreated");
        self.scip = scip;
        self
    }

    /// Sets the constraint as modifiable or not.
    pub fn set_cons_modifiable(&mut self, cons: Rc<Constraint>, modifiable: bool) {
        self.scip
            .set_cons_modifiable(cons, modifiable)
            .expect("Failed to set constraint modifiable");
    }

    /// Informs the SCIP instance that the objective value is always integral and returns the same `Model` instance.
    #[allow(unused_mut)]
    pub fn set_obj_integral(mut self) -> Self {
        self.scip
            .set_obj_integral()
            .expect("Failed to set the objective value as integral");
        self
    }

    /// Adds a new variable to the model with the given lower bound, upper bound, objective coefficient, name, and type.
    ///
    /// # Arguments
    ///
    /// * `lb` - The lower bound of the variable.
    /// * `ub` - The upper bound of the variable.
    /// * `obj` - The objective coefficient of the variable.
    /// * `name` - The name of the variable.
    /// * `var_type` - The type of the variable.
    ///
    /// # Returns
    ///
    /// A reference-counted pointer to the new variable.
    ///
    /// # Panics
    ///
    /// This method panics if the variable cannot be created in the current state.
    pub fn add_var(
        &mut self,
        lb: f64,
        ub: f64,
        obj: f64,
        name: &str,
        var_type: VarType,
    ) -> Rc<Variable> {
        let var = self
            .scip
            .create_var(lb, ub, obj, name, var_type)
            .expect("Failed to create variable in state ProblemCreated");
        let var = Variable {
            raw: var,
            scip: self.scip.clone(),
        };

        Rc::new(var)
    }

    /// Includes a new branch rule in the model with the given name, description, priority, maximum depth, maximum bound distance, and implementation.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the branching rule. This should be a unique identifier.
    /// * `desc` - A brief description of the branching rule. This is used for informational purposes.
    /// * `priority` - The priority of the branching rule. When SCIP decides which branching rule to call, it considers their priorities. A higher value indicates a higher priority.
    /// * `maxdepth` - The maximum depth level up to which this branching rule should be used. If this is -1, the branching rule can be used at any depth.
    /// * `maxbounddist` - The maximum relative distance from the current node's dual bound to primal bound compared to the best node's dual bound for applying the branching rule. A value of 0.0 means the rule can only be applied on the current best node, while 1.0 means it can be applied on all nodes.
    /// * `rule` - The branching rule to be included. This should be a mutable reference to an object that implements the `BranchRule` trait, and represents the branching rule data.
    ///
    /// # Returns
    ///
    /// This function returns the `Model` instance for which the branching rule was included. This allows for method chaining.
    ///
    /// # Panics
    ///
    /// This method will panic if the inclusion of the branching rule fails. This can happen if another branching rule with the same name already exists.
    pub fn include_branch_rule(
        self,
        name: &str,
        desc: &str,
        priority: i32,
        maxdepth: i32,
        maxbounddist: f64,
        rule: Box<dyn BranchRule>,
    ) -> Self {
        self.scip
            .include_branch_rule(name, desc, priority, maxdepth, maxbounddist, rule)
            .expect("Failed to include branch rule at state ProblemCreated");
        self
    }

    /// Include a new primal heuristic in the model.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the heuristic. This should be a unique identifier.
    /// * `desc` - A brief description of the heuristic. This is used for informational purposes.
    /// * `priority` - The priority of the heuristic. When SCIP decides which heuristic to call, it considers their priorities. A higher value indicates a higher priority.
    /// * `dispchar` - The display character of the heuristic (used in logs).
    /// * `freq` - The frequency for calling the heuristic in the tree; 1 means at every node, 2 means at every other node and so on, -1 turns off the heuristic.
    /// * `freqofs` - The frequency offset for calling the heuristic in the tree; it defines the depth of the branching tree at which the primal heuristic is executed for the first time.
    /// * `maxdepth` - The maximum depth level up to which this heuristic should be used. If this is -1, the heuristic can be used at any depth.
    /// * `timing` - The timing mask of the heuristic.
    /// * `usessubscip` - Should the heuristic use a secondary SCIP instance?
    /// * `heur` - The heuristic to be included. This should be a Box of an object that implements the `Heur` trait, and represents the heuristic data.
    ///
    /// # Returns
    ///
    /// This function returns the `Model` instance for which the heuristic was included. This allows for method chaining.
    pub fn include_heur(
        self,
        name: &str,
        desc: &str,
        priority: i32,
        dispchar: char,
        freq: i32,
        freqofs: i32,
        maxdepth: i32,
        timing: HeurTiming,
        usessubscip: bool,
        heur: Box<dyn Heuristic>,
    ) -> Self {
        self.scip
            .include_heur(
                name,
                desc,
                priority,
                dispchar,
                freq,
                freqofs,
                maxdepth,
                timing,
                usessubscip,
                heur,
            )
            .expect("Failed to include heuristic at state ProblemCreated");
        self
    }

    /// Includes a new separator in the model.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the separator. This should be a unique identifier.
    /// * `desc` - A brief description of the separator. This is used for informational purposes.
    /// * `priority` - The priority of the separator. When SCIP decides which separator to call, it considers their priorities. A higher value indicates a higher priority.
    /// * `freq` - The frequency for calling the separator in the tree; 1 means at every node, 2 means at every other node and so on, -1 turns off the separator.
    /// * `maxbounddist` - The maximum relative distance from the current node's dual bound to primal bound compared to the best node's dual bound for applying the separator. A value of 0.0 means the separator can only be applied on the current best node, while 1.0 means it can be applied on all nodes.
    /// * `usesubscip` - Does the separator use a secondary SCIP instance?
    /// * `delay` - A boolean indicating whether the separator should be delayed.
    /// * `separator`- The separator to be included. This should be a mutable reference to an object that implements the `Separator` trait, and represents the separator data.
    ///
    /// # Returns
    ///
    /// This function returns the `Model` instance for which the separator was included. This allows for method chaining.
    pub fn include_separator(
        self,
        name: &str,
        desc: &str,
        priority: i32,
        freq: i32,
        maxbounddist: f64,
        usesubscip: bool,
        delay: bool,
        separator: Box<dyn Separator>,
    ) -> Self {
        self.scip
            .include_separator(
                name,
                desc,
                priority,
                freq,
                maxbounddist,
                usesubscip,
                delay,
                separator,
            )
            .expect("Failed to include separator at state ProblemCreated");
        self
    }

    /// Includes a new event handler in the model.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the event handler. This should be a unique identifier.
    /// * `desc` - A brief description of the event handler. This is used for informational purposes.
    /// * `eventhdlr` - The event handler to be included. This should be a mutable reference to an object that implements the `EventHdlr` trait, and represents the event handling logic.
    ///
    /// # Returns
    ///
    /// This function returns the `Model` instance for which the event handler was included. This allows for method chaining.
    pub fn include_eventhdlr(self, name: &str, desc: &str, eventhdlr: Box<dyn Eventhdlr>) -> Self {
        self.scip
            .include_eventhdlr(name, desc, eventhdlr)
            .expect("Failed to include event handler at state ProblemCreated");
        self
    }

    /// Includes a new pricer in the SCIP data structure.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the variable pricer. This should be a unique identifier.
    /// * `desc` - A brief description of the variable pricer.
    /// * `priority` - The priority of the variable pricer. When SCIP decides which pricer to call, it considers their priorities. A higher value indicates a higher priority.
    /// * `delay` - A boolean indicating whether the pricer should be delayed. If true, the pricer is only called when no other pricers or already existing problem variables with negative reduced costs are found. If this is set to false, the pricer may produce columns that already exist in the problem.
    /// * `pricer` - The pricer to be included. This should be a mutable reference to an object that implements the `Pricer` trait.
    ///
    /// # Returns
    ///
    /// This function returns the `Model` instance for which the pricer was included. This allows for method chaining.
    ///
    /// # Panics
    ///
    /// This method will panic if the inclusion of the pricer fails. This can happen if another pricer with the same name already exists.
    pub fn include_pricer(
        self,
        name: &str,
        desc: &str,
        priority: i32,
        delay: bool,
        pricer: Box<dyn Pricer>,
    ) -> Self {
        self.scip
            .include_pricer(name, desc, priority, delay, pricer)
            .expect("Failed to include pricer at state ProblemCreated");
        self
    }

    /// Solves the model and returns a new `Model` instance with a `Solved` state.
    ///
    /// # Returns
    ///
    /// A new `Model` instance with a `Solved` state.
    ///
    /// # Panics
    ///
    /// This method panics if the problem cannot be solved in the current state.
    #[allow(unused_mut)]
    pub fn solve(mut self) -> Model<Solved> {
        self.scip
            .solve()
            .expect("Failed to solve problem in state ProblemCreated");
        Model {
            scip: self.scip,
            state: Solved {},
        }
    }
}

impl Model<Solving> {
    /// Adds a new variable to the model with the given lower bound, upper bound, objective coefficient, name, and type.
    ///
    /// # Arguments
    ///
    /// * `lb` - The lower bound of the variable.
    /// * `ub` - The upper bound of the variable.
    /// * `obj` - The objective coefficient of the variable.
    /// * `name` - The name of the variable.
    /// * `var_type` - The type of the variable.
    ///
    /// # Returns
    ///
    /// A reference-counted pointer to the new variable.
    ///
    /// # Panics
    ///
    /// This method panics if the variable cannot be created in the current state.
    pub fn add_var(
        &mut self,
        lb: f64,
        ub: f64,
        obj: f64,
        name: &str,
        var_type: VarType,
    ) -> Rc<Variable> {
        let var = self
            .scip
            .create_var_solving(lb, ub, obj, name, var_type)
            .expect("Failed to create variable in state ProblemCreated");
        let var = Variable {
            raw: var,
            scip: self.scip.clone(),
        };

        Rc::new(var)
    }

    /// Returns the current node of the model.
    ///
    /// # Panics
    ///
    /// This method panics if not called in the `Solving` state, it should only be used from plugins implementations.
    pub fn focus_node(&self) -> Node {
        self.scip.focus_node()
    }

    /// Creates a new child node of the current node and returns it.
    ///
    /// # Panics
    ///
    /// This method panics if not called from plugins implementations.
    pub fn create_child(&mut self) -> Node {
        self.scip
            .create_child()
            .expect("Failed to create child node in state ProblemCreated")
    }

    /// Adds a new priced variable to the SCIP data structure.
    ///
    /// # Arguments
    ///
    /// * `lb` - The lower bound of the variable.
    /// * `ub` - The upper bound of the variable.
    /// * `obj` - The objective function coefficient for the variable.
    /// * `name` - The name of the variable. This should be a unique identifier.
    /// * `var_type` - The type of the variable, specified as an instance of the `VarType` enum.
    ///
    /// # Returns
    ///
    /// This function returns a reference-counted smart pointer (`Rc`) to the created `Variable` instance.
    pub fn add_priced_var(
        &mut self,
        lb: f64,
        ub: f64,
        obj: f64,
        name: &str,
        var_type: VarType,
    ) -> Rc<Variable> {
        let var = self
            .scip
            .create_priced_var(lb, ub, obj, name, var_type)
            .expect("Failed to create variable in state ProblemCreated");
        let var = Variable {
            raw: var,
            scip: self.scip.clone(),
        };

        Rc::new(var)
    }

    /// Gets the variable in current problem given its index (in the problem).
    ///
    /// # Arguments
    /// * `var_prob_id` - The index of the variable in the problem.
    ///
    /// # Returns
    /// A reference-counted pointer to the variable.
    pub fn var_in_prob(&self, var_prob_id: usize) -> Option<Variable> {
        unsafe {
            ScipPtr::var_from_id(self.scip.raw, var_prob_id).map(|v| Variable {
                raw: v,
                scip: self.scip.clone(),
            })
        }
    }

    /// Adds a new cut (row) to the model.
    ///
    /// # Arguments
    /// * `row` - The row to add.
    /// * `force_cut` - If true, the cut (row) is forced to be selected.
    ///
    /// # Returns
    /// A boolean indicating whether the row is infeasible from the local bounds.
    pub fn add_cut(&mut self, cut: Row, force_cut: bool) -> bool {
        self.scip
            .add_row(cut, force_cut)
            .expect("Failed to add row in state ProblemCreated")
    }
}

impl Model<Solved> {
    /// Returns the objective value of the best solution found by the optimization model.
    pub fn obj_val(&self) -> f64 {
        self.scip.obj_val()
    }

    /// Returns the number of nodes explored by the optimization model.
    pub fn n_nodes(&self) -> usize {
        self.scip.n_nodes()
    }

    /// Returns the total solving time of the optimization model.
    pub fn solving_time(&self) -> f64 {
        self.scip.solving_time()
    }

    /// Returns the number of LP iterations performed by the optimization model.
    pub fn n_lp_iterations(&self) -> usize {
        self.scip.n_lp_iterations()
    }

    /// Frees the transformed problem and returns the model the ProblemCreated state where you
    /// can add variables and constraints, useful for iterated solving
    pub fn free_transform(self) -> Model<ProblemCreated> {
        self.scip
            .free_transform()
            .unwrap_or_else(|retcode| panic!("SCIP returned unexpected retcode {:?}", retcode));
        Model {
            scip: self.scip,
            state: ProblemCreated {},
        }
    }
}

/// A trait for optimization models with a problem created.
pub trait ModelWithProblem {
    /// Returns a vector of all variables in the optimization model.
    fn vars(&self) -> Vec<Rc<Variable>>;

    /// Returns the variable with the given ID, if it exists.
    fn var(&self, var_id: VarId) -> Option<Rc<Variable>>;

    /// Returns the number of variables in the optimization model.
    fn n_vars(&self) -> usize;

    /// Returns the number of constraints in the optimization model.
    fn n_conss(&self) -> usize;

    /// Returns a vector of all constraints in the optimization model.
    fn conss(&self) -> Vec<Rc<Constraint>>;

    /// Writes the optimization model to a file with the given path and extension.
    fn write(&self, path: &str, ext: &str) -> Result<(), Retcode>;
}

trait ModelStageWithProblem {}
impl ModelStageWithProblem for ProblemCreated {}
impl ModelStageWithProblem for Solved {}
impl ModelStageWithProblem for Solving {}

impl<S: ModelStageWithProblem> ModelWithProblem for Model<S> {
    /// Returns a vector of all variables in the optimization model.
    fn vars(&self) -> Vec<Rc<Variable>> {
        let scip_vars = self.scip.vars(false);
        scip_vars
            .into_values()
            .map(|v| Variable {
                raw: v,
                scip: self.scip.clone(),
            })
            .map(Rc::new)
            .collect()
    }

    /// Returns the variable with the given ID, if it exists.
    fn var(&self, var_id: VarId) -> Option<Rc<Variable>> {
        let vars = self.scip.vars(false);
        for (i, v) in vars {
            if i == var_id {
                return Some(Rc::new(Variable {
                    raw: v,
                    scip: self.scip.clone(),
                }));
            }
        }

        None
    }

    /// Returns the number of variables in the optimization model.
    fn n_vars(&self) -> usize {
        self.scip.n_vars()
    }

    /// Returns the number of constraints in the optimization model.
    fn n_conss(&self) -> usize {
        self.scip.n_conss()
    }

    /// Returns a vector of all constraints in the optimization model.
    fn conss(&self) -> Vec<Rc<Constraint>> {
        let scip_conss = self.scip.conss(false);
        scip_conss
            .into_iter()
            .map(|c| Constraint {
                raw: c,
                scip: self.scip.clone(),
            })
            .map(Rc::new)
            .collect()
    }

    /// Writes the optimization model to a file with the given path and extension.
    fn write(&self, path: &str, ext: &str) -> Result<(), Retcode> {
        self.scip.write(path, ext)?;
        Ok(())
    }
}

/// A trait for optimization models with a problem created or solved.
pub trait ProblemOrSolving {
    /// Creates a new solution initialized to zero.
    fn create_sol(&self) -> Solution;

    /// Adds a solution to the model
    ///
    /// # Returns
    /// A `Result` indicating whether the solution was added successfully.
    fn add_sol(&self, sol: Solution) -> Result<(), SolError>;

    /// Adds a binary variable to the given set partitioning constraint.
    ///
    /// # Arguments
    ///
    /// * `cons` - The constraint to add the variable to.
    /// * `var` - The binary variable to add.
    ///
    /// # Panics
    ///
    /// This method panics if the variable cannot be added in the current state, or if the variable is not binary.
    fn add_cons_coef_setppc(&mut self, cons: Rc<Constraint>, var: Rc<Variable>);

    /// Adds a coefficient to the given constraint for the given variable and coefficient value.
    ///
    /// # Arguments
    ///
    /// * `cons` - The constraint to add the coefficient to.
    /// * `var` - The variable to add the coefficient for.
    /// * `coef` - The coefficient value to add.
    ///
    /// # Panics
    ///
    /// This method panics if the coefficient cannot be added in the current state.
    fn add_cons_coef(&mut self, cons: Rc<Constraint>, var: Rc<Variable>, coef: f64);

    /// Adds a new quadratic constraint to the model with the given variables, coefficients, left-hand side, right-hand side, and name.
    ///
    /// # Arguments
    ///
    /// * `lin_vars` - The linear variables in the constraint.
    /// * `lin_coefs` - The coefficients of the linear variables in the constraint.
    /// * `quad_vars_1` - The first variable in the quadratic constraints.
    /// * `quad_vars_2` - The second variable in the quadratic constraints.
    /// * `quad_coefs` - The coefficients of the quadratic terms in the constraint.
    /// * `lhs` - The left-hand side of the constraint.
    /// * `rhs` - The right-hand side of the constraint.
    /// * `name` - The name of the constraint.
    ///
    /// # Returns
    ///
    /// A reference-counted pointer to the new constraint.
    ///
    /// # Panics
    ///
    /// This method panics if the constraint cannot be created in the current state.
    fn add_cons_quadratic(
        &mut self,
        lin_vars: Vec<Rc<Variable>>,
        lin_coefs: &mut [f64],
        quad_vars_1: Vec<Rc<Variable>>,
        quad_vars_2: Vec<Rc<Variable>>,
        quad_coefs: &mut [f64],
        lhs: f64,
        rhs: f64,
        name: &str,
    ) -> Rc<Constraint>;

    /// Adds a new constraint to the model with the given variables, coefficients, left-hand side, right-hand side, and name.
    ///
    /// # Arguments
    ///
    /// * `vars` - The variables in the constraint.
    /// * `coefs` - The coefficients of the variables in the constraint.
    /// * `lhs` - The left-hand side of the constraint.
    /// * `rhs` - The right-hand side of the constraint.
    /// * `name` - The name of the constraint.
    ///
    /// # Returns
    ///
    /// A reference-counted pointer to the new constraint.
    ///
    /// # Panics
    ///
    /// This method panics if the constraint cannot be created in the current state.
    fn add_cons(
        &mut self,
        vars: Vec<Rc<Variable>>,
        coefs: &[f64],
        lhs: f64,
        rhs: f64,
        name: &str,
    ) -> Rc<Constraint>;

    /// Adds a new set partitioning constraint to the model with the given variables and name.
    ///
    /// # Arguments
    ///
    /// * `vars` - The binary variables in the constraint.
    /// * `name` - The name of the constraint.
    ///
    /// # Returns
    ///
    /// A reference-counted pointer to the new constraint.
    ///
    /// # Panics
    ///
    /// This method panics if the constraint cannot be created in the current state, or if any of the variables are not binary.
    fn add_cons_set_part(&mut self, vars: Vec<Rc<Variable>>, name: &str) -> Rc<Constraint>;

    /// Adds a new set cover constraint to the model with the given variables and name.
    ///
    /// # Arguments
    ///
    /// * `vars` - The binary variables in the constraint.
    /// * `name` - The name of the constraint.
    ///
    /// # Returns
    ///
    /// A reference-counted pointer to the new constraint.
    ///
    /// # Panics
    ///
    /// This method panics if the constraint cannot be created in the current state, or if any of the variables are not binary.
    fn add_cons_set_cover(&mut self, vars: Vec<Rc<Variable>>, name: &str) -> Rc<Constraint>;

    /// Adds a new set packing constraint to the model with the given variables and name.
    ///
    /// # Arguments
    ///
    /// * `vars` - The binary variables in the constraint.
    /// * `name` - The name of the constraint.
    ///
    /// # Returns
    ///
    /// A reference-counted pointer to the new constraint.
    ///
    /// # Panics
    ///
    /// This method panics if the constraint cannot be created in the current state, or if any of the variables are not binary.
    fn add_cons_set_pack(&mut self, vars: Vec<Rc<Variable>>, name: &str) -> Rc<Constraint>;

    /// Adds a new cardinality constraint to the model with the given variables, cardinality limit, and name.
    ///
    /// # Arguments
    ///
    /// * `vars` - The binary variables in the constraint.
    /// * `cardinality` - The maximum number of non-zero variables this constraint allows
    /// * `name` - The name of the constraint.
    ///
    /// # Returns
    ///
    /// A reference-counted pointer to the new constraint.
    ///
    /// # Panics
    ///
    /// This method panics if the constraint cannot be created in the current state.
    fn add_cons_cardinality(
        &mut self,
        vars: Vec<Rc<Variable>>,
        cardinality: usize,
        name: &str,
    ) -> Rc<Constraint>;

    /// Adds a new indicator constraint to the model with the given variables, coefficients, right-hand side, and name.
    ///
    /// # Arguments
    ///
    /// * `bin_var` - The binary variable in the constraint.
    /// * `vars` - The variables of the constraints.
    /// * `coefs` - The coefficients of the variables in the constraint.
    /// * `rhs` - The right-hand side of the constraint.
    /// * `name` - The name of the constraint.
    ///
    /// # Returns
    ///
    /// A reference-counted pointer to the new constraint.
    ///
    /// # Panics
    ///
    /// This method panics if the constraint cannot be created in the current state.
    fn add_cons_indicator(
        &mut self,
        bin_var: Rc<Variable>,
        vars: Vec<Rc<Variable>>,
        coefs: &mut [f64],
        rhs: f64,
        name: &str,
    ) -> Rc<Constraint>;
}
trait ModelStageProblemOrSolving {}
impl ModelStageProblemOrSolving for ProblemCreated {}
impl ModelStageProblemOrSolving for Solving {}

impl<S: ModelStageProblemOrSolving> ProblemOrSolving for Model<S> {
    /// Creates a new solution initialized to zero.
    fn create_sol(&self) -> Solution {
        let sol_ptr = self
            .scip
            .create_sol()
            .expect("Failed to create solution in state ProblemCreated");
        Solution {
            scip_ptr: self.scip.clone(),
            raw: sol_ptr,
        }
    }

    /// Adds a solution to the model
    ///
    /// # Returns
    /// A `Result` indicating whether the solution was added successfully.
    fn add_sol(&self, sol: Solution) -> Result<(), SolError> {
        let succesfully_stored = self.scip.add_sol(sol).expect("Failed to add solution");
        if succesfully_stored {
            Ok(())
        } else {
            Err(SolError::Infeasible)
        }
    }

    /// Adds a binary variable to the given set partitioning constraint.
    ///
    /// # Arguments
    ///
    /// * `cons` - The constraint to add the variable to.
    /// * `var` - The binary variable to add.
    ///
    /// # Panics
    ///
    /// This method panics if the variable cannot be added in the current state, or if the variable is not binary.
    fn add_cons_coef_setppc(&mut self, cons: Rc<Constraint>, var: Rc<Variable>) {
        assert_eq!(var.var_type(), VarType::Binary);
        self.scip
            .add_cons_coef_setppc(cons, var)
            .expect("Failed to add constraint coefficient in state ProblemCreated");
    }

    /// Adds a coefficient to the given constraint for the given variable and coefficient value.
    ///
    /// # Arguments
    ///
    /// * `cons` - The constraint to add the coefficient to.
    /// * `var` - The variable to add the coefficient for.
    /// * `coef` - The coefficient value to add.
    ///
    /// # Panics
    ///
    /// This method panics if the coefficient cannot be added in the current state.
    fn add_cons_coef(&mut self, cons: Rc<Constraint>, var: Rc<Variable>, coef: f64) {
        self.scip
            .add_cons_coef(cons, var, coef)
            .expect("Failed to add constraint coefficient in state ProblemCreated");
    }

    /// Adds a new quadratic constraint to the model with the given variables, coefficients, left-hand side, right-hand side, and name.
    ///
    /// # Arguments
    ///
    /// * `lin_vars` - The linear variables in the constraint.
    /// * `lin_coefs` - The coefficients of the linear variables in the constraint.
    /// * `quad_vars_1` - The first variable in the quadratic constraints.
    /// * `quad_vars_2` - The second variable in the quadratic constraints.
    /// * `quad_coefs` - The coefficients of the quadratic terms in the constraint.
    /// * `lhs` - The left-hand side of the constraint.
    /// * `rhs` - The right-hand side of the constraint.
    /// * `name` - The name of the constraint.
    ///
    /// # Returns
    ///
    /// A reference-counted pointer to the new constraint.
    ///
    /// # Panics
    ///
    /// This method panics if the constraint cannot be created in the current state.
    fn add_cons_quadratic(
        &mut self,
        lin_vars: Vec<Rc<Variable>>,
        lin_coefs: &mut [f64],
        quad_vars_1: Vec<Rc<Variable>>,
        quad_vars_2: Vec<Rc<Variable>>,
        quad_coefs: &mut [f64],
        lhs: f64,
        rhs: f64,
        name: &str,
    ) -> Rc<Constraint> {
        assert_eq!(lin_vars.len(), lin_coefs.len());
        assert_eq!(quad_vars_1.len(), quad_vars_2.len());
        assert_eq!(quad_vars_1.len(), quad_coefs.len());
        let cons = self
            .scip
            .create_cons_quadratic(
                lin_vars,
                lin_coefs,
                quad_vars_1,
                quad_vars_2,
                quad_coefs,
                lhs,
                rhs,
                name,
            )
            .expect("Failed to create constraint in state ProblemCreated");

        Rc::new(Constraint {
            raw: cons,
            scip: self.scip.clone(),
        })
    }

    /// Adds a new constraint to the model with the given variables, coefficients, left-hand side, right-hand side, and name.
    ///
    /// # Arguments
    ///
    /// * `vars` - The variables in the constraint.
    /// * `coefs` - The coefficients of the variables in the constraint.
    /// * `lhs` - The left-hand side of the constraint.
    /// * `rhs` - The right-hand side of the constraint.
    /// * `name` - The name of the constraint.
    ///
    /// # Returns
    ///
    /// A reference-counted pointer to the new constraint.
    ///
    /// # Panics
    ///
    /// This method panics if the constraint cannot be created in the current state.
    fn add_cons(
        &mut self,
        vars: Vec<Rc<Variable>>,
        coefs: &[f64],
        lhs: f64,
        rhs: f64,
        name: &str,
    ) -> Rc<Constraint> {
        assert_eq!(vars.len(), coefs.len());
        let cons = self
            .scip
            .create_cons(vars, coefs, lhs, rhs, name)
            .expect("Failed to create constraint in state ProblemCreated");

        Rc::new(Constraint {
            raw: cons,
            scip: self.scip.clone(),
        })
    }

    /// Adds a new set partitioning constraint to the model with the given variables and name.
    ///
    /// # Arguments
    ///
    /// * `vars` - The binary variables in the constraint.
    /// * `name` - The name of the constraint.
    ///
    /// # Returns
    ///
    /// A reference-counted pointer to the new constraint.
    ///
    /// # Panics
    ///
    /// This method panics if the constraint cannot be created in the current state, or if any of the variables are not binary.
    fn add_cons_set_part(&mut self, vars: Vec<Rc<Variable>>, name: &str) -> Rc<Constraint> {
        assert!(vars.iter().all(|v| v.var_type() == VarType::Binary));
        let cons = self
            .scip
            .create_cons_set_part(vars, name)
            .expect("Failed to add constraint set partition in state ProblemCreated");

        Rc::new(Constraint {
            raw: cons,
            scip: self.scip.clone(),
        })
    }

    /// Adds a new set cover constraint to the model with the given variables and name.
    ///
    /// # Arguments
    ///
    /// * `vars` - The binary variables in the constraint.
    /// * `name` - The name of the constraint.
    ///
    /// # Returns
    ///
    /// A reference-counted pointer to the new constraint.
    ///
    /// # Panics
    ///
    /// This method panics if the constraint cannot be created in the current state, or if any of the variables are not binary.
    fn add_cons_set_cover(&mut self, vars: Vec<Rc<Variable>>, name: &str) -> Rc<Constraint> {
        assert!(vars.iter().all(|v| v.var_type() == VarType::Binary));
        let cons = self
            .scip
            .create_cons_set_cover(vars, name)
            .expect("Failed to add constraint set cover in state ProblemCreated");

        Rc::new(Constraint {
            raw: cons,
            scip: self.scip.clone(),
        })
    }

    /// Adds a new set packing constraint to the model with the given variables and name.
    ///
    /// # Arguments
    ///
    /// * `vars` - The binary variables in the constraint.
    /// * `name` - The name of the constraint.
    ///
    /// # Returns
    ///
    /// A reference-counted pointer to the new constraint.
    ///
    /// # Panics
    ///
    /// This method panics if the constraint cannot be created in the current state, or if any of the variables are not binary.
    fn add_cons_set_pack(&mut self, vars: Vec<Rc<Variable>>, name: &str) -> Rc<Constraint> {
        assert!(vars.iter().all(|v| v.var_type() == VarType::Binary));
        let cons = self
            .scip
            .create_cons_set_pack(vars, name)
            .expect("Failed to add constraint set packing in state ProblemCreated");

        Rc::new(Constraint {
            raw: cons,
            scip: self.scip.clone(),
        })
    }

    /// Adds a new cardinality constraint to the model with the given variables, cardinality limit, and name.
    ///
    /// # Arguments
    ///
    /// * `vars` - The binary variables in the constraint.
    /// * `cardinality` - The maximum number of non-zero variables this constraint allows
    /// * `name` - The name of the constraint.
    ///
    /// # Returns
    ///
    /// A reference-counted pointer to the new constraint.
    ///
    /// # Panics
    ///
    /// This method panics if the constraint cannot be created in the current state.
    fn add_cons_cardinality(
        &mut self,
        vars: Vec<Rc<Variable>>,
        cardinality: usize,
        name: &str,
    ) -> Rc<Constraint> {
        let cons = self
            .scip
            .create_cons_cardinality(vars, cardinality, name)
            .expect("Failed to add cardinality constraint");

        Rc::new(Constraint {
            raw: cons,
            scip: self.scip.clone(),
        })
    }

    /// Adds a new indicator constraint to the model with the given variables, coefficients, right-hand side, and name.
    ///
    /// # Arguments
    ///
    /// * `bin_var` - The binary variable in the constraint.
    /// * `vars` - The variables of the constraints.
    /// * `coefs` - The coefficients of the variables in the constraint.
    /// * `rhs` - The right-hand side of the constraint.
    /// * `name` - The name of the constraint.
    ///
    /// # Returns
    ///
    /// A reference-counted pointer to the new constraint.
    ///
    /// # Panics
    ///
    /// This method panics if the constraint cannot be created in the current state.
    fn add_cons_indicator(
        &mut self,
        bin_var: Rc<Variable>,
        vars: Vec<Rc<Variable>>,
        coefs: &mut [f64],
        rhs: f64,
        name: &str,
    ) -> Rc<Constraint> {
        assert_eq!(vars.len(), coefs.len());
        assert_eq!(bin_var.var_type(), VarType::Binary);
        let cons = self
            .scip
            .create_cons_indicator(bin_var, vars, coefs, rhs, name)
            .expect("Failed to create constraint in state ProblemCreated");

        Rc::new(Constraint {
            raw: cons,
            scip: self.scip.clone(),
        })
    }
}

/// A trait for optimization models with any state that might have solutions.
pub trait WithSolutions {
    /// Returns the best solution for the optimization model, if one exists.
    fn best_sol(&self) -> Option<Solution>;

    /// Returns the number of solutions found by the optimization model.
    fn n_sols(&self) -> usize;
}

trait ModelStageWithSolutions {}
impl ModelStageWithSolutions for Solved {}
impl ModelStageWithSolutions for Solving {}
impl ModelStageWithSolutions for ProblemCreated {}

impl<S: ModelStageWithSolutions> WithSolutions for Model<S> {
    /// Returns the best solution for the optimization model, if one exists.
    fn best_sol(&self) -> Option<Solution> {
        if self.n_sols() > 0 {
            let sol = Solution {
                scip_ptr: self.scip.clone(),
                raw: self.scip.best_sol().unwrap(),
            };
            Some(sol)
        } else {
            None
        }
    }

    /// Returns the number of solutions found by the optimization model.
    fn n_sols(&self) -> usize {
        self.scip.n_sols()
    }
}

/// A trait for optimization models with any state that might have solving statistics.
pub trait WithSolvingStats {
    /// Returns the objective value of the best solution found by the optimization model.
    fn obj_val(&self) -> f64;

    /// Returns the best bound (dualbound) proven so far.
    fn best_bound(&self) -> f64;

    /// Returns the number of nodes explored by the optimization model.
    fn n_nodes(&self) -> usize;

    /// Returns the total solving time of the optimization model.
    fn solving_time(&self) -> f64;

    /// Returns the number of LP iterations performed by the optimization model.
    fn n_lp_iterations(&self) -> usize;
}

trait ModelStageWithSolvingStats {}
impl ModelStageWithSolvingStats for Solved {}
impl ModelStageWithSolvingStats for Solving {}
impl ModelStageWithSolvingStats for ProblemCreated {}

impl<S: ModelStageWithSolvingStats> WithSolvingStats for Model<S> {
    /// Returns the objective value of the best solution found by the optimization model.
    fn obj_val(&self) -> f64 {
        self.scip.obj_val()
    }

    /// Returns the best bound (dualbound) proven so far.
    fn best_bound(&self) -> f64 {
        self.scip.best_bound()
    }

    /// Returns the number of nodes explored by the optimization model.
    fn n_nodes(&self) -> usize {
        self.scip.n_nodes()
    }

    /// Returns the total solving time of the optimization model.
    fn solving_time(&self) -> f64 {
        self.scip.solving_time()
    }

    /// Returns the number of LP iterations performed by the optimization model.
    fn n_lp_iterations(&self) -> usize {
        self.scip.n_lp_iterations()
    }
}

/// Creates a minimal `Model` instance and sets off a lot of SCIP plugins, useful for writing tests.
pub fn minimal_model() -> Model<ProblemCreated> {
    Model::default()
        .set_presolving(ParamSetting::Off)
        .set_heuristics(ParamSetting::Off)
        .set_separating(ParamSetting::Off)
}

impl<T> Model<T> {
    /// Returns a pointer to the SCIP instance. This is useful for passing to functions in the `ffi` module.
    pub fn scip_ptr(&self) -> *mut SCIP {
        self.scip.raw
    }

    /// Returns the status of the optimization model.
    pub fn status(&self) -> Status {
        self.scip.status()
    }

    /// Prints the version of SCIP used by the optimization model.
    pub fn print_version(&self) {
        self.scip.print_version()
    }

    /// Hides the output of the optimization model by setting the `display/verblevel` parameter to 0.
    #[allow(unused_mut)]
    pub fn hide_output(mut self) -> Self {
        self.scip
            .set_int_param("display/verblevel", 0)
            .expect("Failed to set display/verblevel to 0");
        self
    }

    /// Sets the time limit for the optimization model.
    ///
    /// # Arguments
    ///
    /// * `time_limit` - The time limit in seconds.
    #[allow(unused_mut)]
    pub fn set_time_limit(mut self, time_limit: usize) -> Self {
        self.scip
            .set_real_param("limits/time", time_limit as f64)
            .expect("Failed to set time limit");
        self
    }

    /// Sets the memory limit for the optimization model.
    ///
    /// # Arguments
    ///
    /// * `memory_limit` - The memory limit in MB.
    #[allow(unused_mut)]
    pub fn set_memory_limit(mut self, memory_limit: usize) -> Self {
        self.scip
            .set_real_param("limits/memory", memory_limit as f64)
            .expect("Failed to set memory limit");
        self
    }

    /// Includes all default plugins in the SCIP instance and returns a new `Model` instance with a `PluginsIncluded` state.
    #[allow(unused_mut)]
    pub fn include_default_plugins(mut self) -> Model<PluginsIncluded> {
        self.scip
            .include_default_plugins()
            .expect("Failed to include default plugins");
        Model {
            scip: self.scip,
            state: PluginsIncluded {},
        }
    }

    /// Sets a SCIP string parameter and returns a new `Model` instance with the parameter set.
    #[allow(unused_mut)]
    pub fn set_str_param(mut self, param: &str, value: &str) -> Result<Self, Retcode> {
        self.scip.set_str_param(param, value)?;
        Ok(self)
    }

    /// Sets a SCIP boolean parameter and returns a new `Model` instance with the parameter set.
    #[allow(unused_mut)]
    pub fn set_bool_param(mut self, param: &str, value: bool) -> Result<Self, Retcode> {
        self.scip.set_bool_param(param, value)?;
        Ok(self)
    }

    /// Sets a SCIP integer parameter and returns a new `Model` instance with the parameter set.
    #[allow(unused_mut)]
    pub fn set_int_param(mut self, param: &str, value: i32) -> Result<Self, Retcode> {
        self.scip.set_int_param(param, value)?;
        Ok(self)
    }

    /// Sets a SCIP long integer parameter and returns a new `Model` instance with the parameter set.
    #[allow(unused_mut)]
    pub fn set_longint_param(mut self, param: &str, value: i64) -> Result<Self, Retcode> {
        self.scip.set_longint_param(param, value)?;
        Ok(self)
    }

    /// Sets a SCIP real parameter and returns a new `Model` instance with the parameter set.
    #[allow(unused_mut)]
    pub fn set_real_param(mut self, param: &str, value: f64) -> Result<Self, Retcode> {
        self.scip.set_real_param(param, value)?;
        Ok(self)
    }

    /// Sets the presolving parameter of the SCIP instance and returns the same `Model` instance.
    #[allow(unused_mut)]
    pub fn set_presolving(mut self, presolving: ParamSetting) -> Self {
        self.scip
            .set_presolving(presolving)
            .expect("Failed to set presolving with valid value");
        self
    }

    /// Sets the separating parameter of the SCIP instance and returns the same `Model` instance.
    #[allow(unused_mut)]
    pub fn set_separating(mut self, separating: ParamSetting) -> Self {
        self.scip
            .set_separating(separating)
            .expect("Failed to set separating with valid value");
        self
    }

    /// Sets the heuristics parameter of the SCIP instance and returns the same `Model` instance.
    #[allow(unused_mut)]
    pub fn set_heuristics(mut self, heuristics: ParamSetting) -> Self {
        self.scip
            .set_heuristics(heuristics)
            .expect("Failed to set heuristics with valid value");
        self
    }
}

/// The default implementation for a `Model` instance in the `ProblemCreated` state.
impl Default for Model<ProblemCreated> {
    /// Creates a new `Model` instance with the default plugins included and a problem named "problem".
    fn default() -> Self {
        Model::new()
            .include_default_plugins()
            .create_prob("problem")
    }
}

/// An enum representing the possible settings for a SCIP parameter.
#[derive(Debug)]
pub enum ParamSetting {
    /// Use default values.
    Default,
    /// Set to aggressive settings.
    Aggressive,
    /// Set to fast settings.
    Fast,
    /// Turn off.
    Off,
}

impl From<ParamSetting> for ffi::SCIP_PARAMSETTING {
    /// Converts a `ParamSetting` enum variant into its corresponding `ffi::SCIP_PARAMSETTING` value.
    fn from(val: ParamSetting) -> Self {
        match val {
            ParamSetting::Default => ffi::SCIP_ParamSetting_SCIP_PARAMSETTING_DEFAULT,
            ParamSetting::Aggressive => ffi::SCIP_ParamSetting_SCIP_PARAMSETTING_AGGRESSIVE,
            ParamSetting::Fast => ffi::SCIP_ParamSetting_SCIP_PARAMSETTING_FAST,
            ParamSetting::Off => ffi::SCIP_ParamSetting_SCIP_PARAMSETTING_OFF,
        }
    }
}

/// An enum representing the objective sense of a SCIP optimization model.
#[derive(Debug)]
pub enum ObjSense {
    /// The problem is a minimization problem.
    Minimize,
    /// The problem is a maximization problem.
    Maximize,
}

impl From<ObjSense> for ffi::SCIP_OBJSENSE {
    /// Converts an `ObjSense` enum variant into its corresponding `ffi::SCIP_OBJSENSE` value.
    fn from(val: ObjSense) -> Self {
        match val {
            ObjSense::Maximize => ffi::SCIP_Objsense_SCIP_OBJSENSE_MAXIMIZE,
            ObjSense::Minimize => ffi::SCIP_Objsense_SCIP_OBJSENSE_MINIMIZE,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::status::Status;
    use rayon::prelude::*;
    use std::fs;
    use std::path::Path;

    use super::*;

    #[test]
    fn solve_from_lp_file() {
        let model = Model::new()
            .hide_output()
            .include_default_plugins()
            .read_prob("data/test/simple.lp")
            .unwrap()
            .solve();
        let status = model.status();
        assert_eq!(status, Status::Optimal);

        //test objective value
        let obj_val = model.obj_val();
        assert_eq!(obj_val, 200.);

        //test constraints
        let conss = model.conss();
        assert_eq!(conss.len(), 2);

        //test solution values
        let sol = model.best_sol().unwrap();
        let vars = model.vars();
        assert_eq!(vars.len(), 2);
        assert_eq!(sol.val(vars[0].clone()), 40.);
        assert_eq!(sol.val(vars[1].clone()), 20.);

        assert_eq!(sol.obj_val(), model.obj_val());
    }

    #[test]
    fn set_obj_integral() {
        let model = Model::new()
            .hide_output()
            .include_default_plugins()
            .read_prob("data/test/simple.lp")
            .unwrap()
            .set_obj_integral()
            .solve();
        let status = model.status();
        assert_eq!(status, Status::Optimal);

        //test objective value
        let obj_value = model.obj_val();
        assert_eq!(obj_value, 200.);
    }

    #[test]
    fn set_time_limit() {
        let model = Model::new()
            .hide_output()
            .set_time_limit(0)
            .include_default_plugins()
            .read_prob("data/test/simple.lp")
            .unwrap()
            .solve();
        let status = model.status();
        assert_eq!(status, Status::TimeLimit);
        assert!(model.solving_time() < 0.5);
        assert_eq!(model.n_nodes(), 0);
        assert_eq!(model.n_lp_iterations(), 0);
    }

    #[test]
    fn set_memory_limit() {
        let model = Model::new()
            .hide_output()
            .set_memory_limit(0)
            .include_default_plugins()
            .read_prob("data/test/simple.lp")
            .unwrap()
            .solve();
        let status = model.status();
        assert_eq!(status, Status::MemoryLimit);
        assert_eq!(model.n_nodes(), 0);
        assert_eq!(model.n_lp_iterations(), 0);
    }

    #[test]
    fn add_variable() {
        let mut model = Model::new()
            .hide_output()
            .include_default_plugins()
            .create_prob("test")
            .set_obj_sense(ObjSense::Maximize);
        let x1_id = model
            .add_var(0., f64::INFINITY, 3., "x1", VarType::Integer)
            .index();
        let x2_id = model
            .add_var(0., f64::INFINITY, 4., "x2", VarType::Continuous)
            .index();
        let x1 = model.var(x1_id).unwrap();
        let x2 = model.var(x2_id).unwrap();
        assert_eq!(model.n_vars(), 2);
        assert_eq!(model.vars().len(), 2);
        assert!(x1.raw != x2.raw);
        assert!(x1.var_type() == VarType::Integer);
        assert!(x2.var_type() == VarType::Continuous);
        assert!(x1.name() == "x1");
        assert!(x2.name() == "x2");
        assert!(x1.obj() == 3.);
        assert!(x2.obj() == 4.);
    }

    fn create_model() -> Model<ProblemCreated> {
        let mut model = Model::new()
            .hide_output()
            .include_default_plugins()
            .create_prob("test")
            .set_obj_sense(ObjSense::Maximize);

        let x1 = model.add_var(0., f64::INFINITY, 3., "x1", VarType::Integer);
        let x2 = model.add_var(0., f64::INFINITY, 4., "x2", VarType::Integer);
        model.add_cons(
            vec![x1.clone(), x2.clone()],
            &[2., 1.],
            -f64::INFINITY,
            100.,
            "c1",
        );
        model.add_cons(vec![x1, x2], &[1., 2.], -f64::INFINITY, 80., "c2");

        model
    }

    #[test]
    fn build_model_with_functions() {
        let model = create_model();
        assert_eq!(model.vars().len(), 2);
        assert_eq!(model.n_conss(), 2);

        let conss = model.conss();
        assert_eq!(conss.len(), 2);
        assert_eq!(conss[0].name(), "c1");
        assert_eq!(conss[1].name(), "c2");

        let solved_model = model.solve();

        let status = solved_model.status();
        assert_eq!(status, Status::Optimal);

        let obj_val = solved_model.obj_val();
        assert_eq!(obj_val, 200.);

        let sol = solved_model.best_sol().unwrap();
        let vars = solved_model.vars();
        assert_eq!(vars.len(), 2);
        assert_eq!(sol.val(vars[0].clone()), 40.);
        assert_eq!(sol.val(vars[1].clone()), 20.);
    }

    #[test]
    fn unbounded_model() {
        let mut model = Model::default()
            .set_obj_sense(ObjSense::Maximize)
            .hide_output();

        model.add_var(0., f64::INFINITY, 1., "x1", VarType::Integer);
        model.add_var(0., f64::INFINITY, 1., "x2", VarType::Integer);

        let solved_model = model.solve();

        let status = solved_model.status();
        assert_eq!(status, Status::Unbounded);

        let sol = solved_model.best_sol();
        assert!(sol.is_some());
    }

    #[test]
    fn infeasible_model() {
        let mut model = Model::default()
            .set_obj_sense(ObjSense::Maximize)
            .hide_output();

        let var = model.add_var(0., 1., 1., "x1", VarType::Integer);

        model.add_cons(vec![var], &[1.], -f64::INFINITY, -1., "c1");

        let solved_model = model.solve();

        let status = solved_model.status();
        assert_eq!(status, Status::Infeasible);

        assert_eq!(solved_model.n_sols(), 0);
        let sol = solved_model.best_sol();
        assert!(sol.is_none());
    }

    #[test]
    fn scip_ptr() {
        let mut model = Model::new()
            .hide_output()
            .include_default_plugins()
            .create_prob("test")
            .set_obj_sense(ObjSense::Maximize);

        let x1 = model.add_var(0., f64::INFINITY, 3., "x1", VarType::Integer);
        let x2 = model.add_var(0., f64::INFINITY, 4., "x2", VarType::Integer);
        model.add_cons(
            vec![x1.clone(), x2.clone()],
            &[2., 1.],
            -f64::INFINITY,
            100.,
            "c1",
        );
        model.add_cons(
            vec![x1.clone(), x2.clone()],
            &[1., 2.],
            -f64::INFINITY,
            80.,
            "c2",
        );

        let scip_ptr = model.scip.raw;
        assert!(!scip_ptr.is_null());
    }

    #[test]
    fn add_cons_coef() {
        let mut model = Model::new()
            .hide_output()
            .include_default_plugins()
            .create_prob("test")
            .set_obj_sense(ObjSense::Maximize);

        let x1 = model.add_var(0., f64::INFINITY, 3., "x1", VarType::Integer);
        let x2 = model.add_var(0., f64::INFINITY, 4., "x2", VarType::Integer);
        let cons = model.add_cons(vec![], &[], -f64::INFINITY, 10., "c1");

        model.add_cons_coef(cons.clone(), x1, 0.); // x1 is unconstrained
        model.add_cons_coef(cons, x2, 10.); // x2 can't be be used

        let solved_model = model.solve();
        let status = solved_model.status();
        assert_eq!(status, Status::Unbounded);
    }

    #[test]
    fn set_cover_partitioning_and_packing() {
        let mut model = Model::new()
            .hide_output()
            .include_default_plugins()
            .create_prob("test")
            .set_obj_sense(ObjSense::Minimize);

        let x1 = model.add_var(0., 1., 3., "x1", VarType::Binary);
        let x2 = model.add_var(0., 1., 4., "x2", VarType::Binary);
        let cons1 = model.add_cons_set_part(vec![], "c");
        model.add_cons_coef_setppc(cons1, x1);

        model.add_cons_set_cover(vec![x2.clone()], "c");
        model.add_cons_set_pack(vec![x2], "c");

        let solved_model = model.solve();
        let status = solved_model.status();
        assert_eq!(status, Status::Optimal);
        assert_eq!(solved_model.obj_val(), 7.);
    }

    #[test]
    fn cardinality_constraint() {
        let mut model = Model::new()
            .hide_output()
            .include_default_plugins()
            .create_prob("test")
            .set_obj_sense(ObjSense::Maximize);

        // set up three variables with different objective weights
        let x1 = model.add_var(0., 10., 4., "x1", VarType::Continuous);
        let x2 = model.add_var(0., 10., 2., "x2", VarType::Integer);
        let x3 = model.add_var(0., 10., 3., "x3", VarType::Integer);

        // cardinality constraint allows just two variables to be non-zero
        model.add_cons_cardinality(vec![x1.clone(), x2.clone(), x3.clone()], 2, "cardinality");

        let solved_model = model.solve();
        let status = solved_model.status();
        assert_eq!(status, Status::Optimal);
        assert_eq!(solved_model.obj_val(), 70.);

        let solution = solved_model.best_sol().unwrap();
        assert_eq!(solution.val(x1), 10.);
        assert_eq!(solution.val(x2), 0.);
        assert_eq!(solution.val(x3), 10.);
    }

    #[test]
    fn indicator_constraint() {
        let mut model = Model::new()
            .hide_output()
            .include_default_plugins()
            .create_prob("test")
            .set_obj_sense(ObjSense::Maximize);

        // set up two integers variables with weight 1 and a binary variable with weight 0
        let x1 = model.add_var(0., 10., 1., "x1", VarType::Integer);
        let x2 = model.add_var(0., 10., 1., "x2", VarType::Integer);
        let b = model.add_var(0., 1., 0., "b", VarType::Binary);

        // Indicator constraint: `b == 1` implies `x1 - x2 <= -1`
        model.add_cons_indicator(
            b.clone(),
            vec![x1.clone(), x2.clone()],
            &mut [1., -1.],
            -1.,
            "indicator",
        );

        // Force `b` to be exactly 1 and later make sure that the constraint `x1 - x2 <= -1` is
        // indeed active
        model.add_cons(vec![b.clone()], &[1.], 1., 1., "c1");

        let solved_model = model.solve();
        let status = solved_model.status();
        assert_eq!(status, Status::Optimal);
        assert_eq!(solved_model.obj_val(), 19.);

        let solution = solved_model.best_sol().unwrap();

        // Indeed `x1 - x2 <= -1` when `b == 1`
        assert_eq!(solution.val(x1), 9.);
        assert_eq!(solution.val(x2), 10.);
        assert_eq!(solution.val(b), 1.);
    }

    #[test]
    fn create_sol() {
        let mut model = Model::new()
            .hide_output()
            .include_default_plugins()
            .create_prob("test")
            .set_obj_sense(ObjSense::Minimize);

        let x1 = model.add_var(0., 1., 3., "x1", VarType::Binary);
        let x2 = model.add_var(0., 1., 4., "x2", VarType::Binary);
        let cons1 = model.add_cons_set_part(vec![], "c");
        model.add_cons_coef_setppc(cons1, x1.clone());

        model.add_cons_set_pack(vec![x2.clone()], "c");

        let sol = model.create_sol();
        assert_eq!(sol.obj_val(), 0.);

        sol.set_val(x1, 1.);
        sol.set_val(x2, 1.);
        assert_eq!(sol.obj_val(), 7.);

        assert!(model.add_sol(sol).is_ok());

        let model = model.solve();
        assert_eq!(model.n_sols(), 2);
    }

    #[test]
    fn quadratic_constraint() {
        // this model should find the maximum manhattan distance a point in a unit-circle can have.
        // This should be 2*sin(pi/4) = sqrt(2).
        let mut model = Model::new()
            .hide_output()
            .include_default_plugins()
            .create_prob("test")
            .set_obj_sense(ObjSense::Maximize);

        let x1 = model.add_var(0., 1., 1., "x1", VarType::Continuous);
        let x2 = model.add_var(0., 1., 1., "x2", VarType::Continuous);

        let _cons = model.add_cons_quadratic(
            vec![],
            &mut [],
            vec![x1.clone(), x2.clone()],
            vec![x1, x2],
            &mut [1., 1.],
            0.,
            1.,
            "circle",
        );

        let solved_model = model.solve();
        let status = solved_model.status();
        assert_eq!(status, Status::Optimal);

        assert!((2f64.sqrt() - solved_model.obj_val()).abs() < 1e-3);
    }

    #[test]
    fn set_str_param() {
        let output_path = "data/ignored/test.vbc";
        let mut model = Model::new()
            .hide_output()
            .set_str_param("visual/vbcfilename", output_path)
            .unwrap()
            .include_default_plugins()
            .create_prob("test")
            .set_obj_sense(ObjSense::Minimize);

        let x1 = model.add_var(0., 1., 3., "x1", VarType::Binary);
        let x2 = model.add_var(0., 1., 4., "x2", VarType::Binary);
        model.add_cons_set_part(vec![x1, x2], "c");

        let solved_model = model.solve();
        let status = solved_model.status();
        assert_eq!(status, Status::Optimal);
        assert_eq!(solved_model.obj_val(), 3.);

        assert!(Path::new(output_path).exists());

        // drop model so the file is closed and it can be removed
        drop(solved_model);
        fs::remove_file(output_path).unwrap();
    }

    #[test]
    fn set_heurs_presolving_separation() {
        let model = Model::new()
            .hide_output()
            .set_heuristics(ParamSetting::Aggressive)
            .set_presolving(ParamSetting::Fast)
            .set_separating(ParamSetting::Off)
            .include_default_plugins()
            .read_prob("data/test/simple.lp")
            .unwrap()
            .solve();

        assert_eq!(model.status(), Status::Optimal);
    }

    #[test]
    fn write_and_read_lp() {
        let model = create_model();

        model.write("test.lp", "lp").unwrap();

        let read_model = Model::new()
            .include_default_plugins()
            .read_prob("test.lp")
            .unwrap();

        let solved = model.solve();
        let read_solved = read_model.solve();

        assert_eq!(solved.status(), read_solved.status());
        assert_eq!(solved.obj_val(), read_solved.obj_val());

        fs::remove_file("test.lp").unwrap();
    }

    #[test]
    fn print_version() {
        Model::new().print_version();
    }

    #[test]
    fn set_bool_param() {
        Model::new()
            .hide_output()
            .set_bool_param("display/allviols", true)
            .unwrap();
    }

    #[test]
    fn set_int_param() {
        let res = Model::new()
            .hide_output()
            .set_int_param("display/verblevel", -1)
            .unwrap_err();

        assert_eq!(res, Retcode::ParameterWrongVal);
    }

    #[test]
    fn set_real_param() {
        let model = Model::new()
            .hide_output()
            .set_real_param("limits/time", 0.)
            .unwrap()
            .include_default_plugins()
            .read_prob("data/test/simple.lp")
            .unwrap()
            .solve();

        assert_eq!(model.status(), Status::TimeLimit);
    }

    #[test]
    fn test_thread_safety() {
        let statuses = (0..1000)
            .into_par_iter()
            .map(|_| {
                let model = create_model().hide_output().solve();
                model.status()
            })
            .collect::<Vec<_>>();

        assert!(statuses.iter().all(|&s| s == Status::Optimal));
    }

    #[test]
    fn set_param_all_states() {
        Model::new()
            .set_int_param("display/verblevel", 0)
            .unwrap()
            .include_default_plugins()
            .set_int_param("display/verblevel", 0)
            .unwrap()
            .read_prob("data/test/simple.lp")
            .unwrap()
            .set_int_param("display/verblevel", 0)
            .unwrap()
            .solve()
            .set_int_param("display/verblevel", 0)
            .unwrap();
    }

    #[test]
    fn free_transform() {
        let model = create_model();
        let solved_model = model.solve();
        let obj_val = solved_model.obj_val();

        let mut second_model = solved_model.free_transform();

        let x3 = second_model.add_var(0.0, f64::INFINITY, 1.0, "x3", VarType::Integer);

        let bound = 2.0;
        second_model.add_cons(vec![x3], &[1.0], 0.0, bound, "x3-cons");

        let second_solved = second_model.solve();
        let expected_obj = obj_val + bound;
        assert_eq!(second_solved.status(), Status::Optimal);
        assert!((second_solved.obj_val() - expected_obj).abs() <= 1e-6);
    }

    #[test]
    fn best_bound() {
        let model = create_model();
        let solved_model = model.solve();
        let best_bound = solved_model.best_bound();
        let obj_val = solved_model.obj_val();
        assert!((best_bound - obj_val) < 1e-6);
    }
}
