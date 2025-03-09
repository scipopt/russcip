use russcip::prelude::*;
use russcip::*;

/// Consider the cutting stock problem: Given large paper rolls of width `W` and demand `b_i` for rolls of width `w_i` (`i` \in 1..m), how
/// many large rolls are needed to resolve the order.
///
/// An ILP formulation for this is
/// ```
///  min \sum_{j=1..k} y_j                                 % y_j \in {0,1}, use roll j, k sufficiently large, e.g. \sum_i b_i
///      s.t. a) \sum_{j=1..k} z_{i,j} >= b_i              % z_{i,j} \in \Z_{>=0}, number of rolls of width w_i cut from the large roll j
///           b) \sum_{i=1..m} w_i z_{i,j} <= W y_j
/// ```
///
/// The reformulation of this into a problem suitable for branch and price is conducted in two steps: 1) identify structure for
/// decomposition and determine the aggregated master problem, 2) determine a pricing problem that sets solutions of restricted problems in
/// relation to actual problems.
///
/// # Identify Structure and Master Problem
///
/// We use the constraints a) as "easy" constraints and b) as "linking" constraints. Consider a "block" wrt the variables
/// `(y_j, z_{1,j}, ... z_{m,j})` and define `D_j := { (y_j, z_{1,j}, ...) | \sum_{i=1..m} w_i z_{i,j} <= W y_j }`. The "direct" Dantzig-
/// Wolfe Reformulation asks to write each point of each `D_j` as a convex combination of its extreme points and to substitute it into the
/// original problem. Write `D_j = conv({p_{j,t}})` where `t` is in some finite index set depending on `j`, and write `y_{j,t}` as well as
/// `z_{i,j,t}` for the values of `y_j` and `z_{i,j}` at `p_{j,t}`, respectively. Then substituting back into the ILP yields
///
/// ```
///   \min \sum_j\sum_t λ_{j,t} y_{j,t}
///      s.t. \sum_j\sum_t λ_{j,t} z_{i,j,t} >= b_i
///           \sum_t λ_{j,t} = 1                          % for all j, λ_{j,t} \in {0,1}
/// ```
///
/// We simplify in two ways: First, since all `D_j` are identical there is no point in tracking the various `j`. Second, there's little
/// reason to care for the decision variable `y_j`. To get rid of both, consider the set of patterns
/// `{p = (p_1, ..., p_m) |  \sum_{i=1..m} w_i p_{i} <= W} = conv({p_t}_{t\in T})`.
/// We end up with the following (aggregated) master problem
///
/// ```
///   \min \sum_t λ_t * 1
///      s.t. \sum_t λ_t z_{i,t} >= b_i    [π_i]
///           λ_t >= 0
/// ```
///
/// where `λ_t` and `z_{i,t}` carry similar meanings as before. A more correct way to write the objective might be to write
/// `\sum_t λ_t y_t,` but since `y_t` is always `1` that would be redundant. Similarly, the constraint `\sum_t λ_{j,t} = 1` simplifies
/// to `\sum_t λ_{t} <= k` which is artificial anyway.
///
/// # Restricted Master Problem and Pricing Problem
///
/// To practically solve the master problem, we need to restrict the set of all patterns to a reasonably small set. The result is a
/// restricted master problem. A reasonable set of patterns to start with is the set of all patterns that contain exactly one item.
///
/// The optimal solution to a restricted master problem is a feasible solution to the original problem. Taking an associated restricted dual
/// solution, it may or may not be the case that the dual solution is feasible for the original problem. If it is, we would have proven
/// optimality to the linear problem. If it is not, there is at least one dual constraint that is violated. The dual of the master problem
/// is
///
/// ```
///   \max \sum_i b_i π_i
///     s.t. \sum_i π_i z_{i,t} <= 1    [λ_t]
///          π_i >= 0
/// ```
///
/// Note that the constraint corresponding to `λ_t` is violated iff its slack, known as its reduced cost
/// `\bar{c}(λ_t) = 1 - \sum_i π_i z_{i,t}`, is negative. Of course, instead of checking for all `t`, we introduce the pricing problem
///
/// ```
///   \min 1 - \sum_i π_i z_{i}
///     s.t. \sum_i w_i z_{i} <= W
///          z_{i} >= 0
/// ```
///
// Following https://scipbook.readthedocs.io/en/latest/bpp.html and https://www.gerad.ca/fr/papers/G-2024-36.pdf
fn main() {
    let stock_length = 9;
    let item_sizes = &[6.0, 5.0, 4.0, 2.0, 3.0, 7.0, 5.0, 8.0, 4.0, 5.0];
    let demand = &[2, 3, 4, 4, 2, 2, 2, 2, 2, 1];

    // Vector of cutting_patterns, initially populated with the trivial ones that contain exactly
    // one item. cutting_patterns[i][j] indicates how often item j is in pattern i.
    let initial_cutting_patterns: Vec<Vec<i32>> = (0..item_sizes.len())
        .map(|i| {
            (0..item_sizes.len())
                .map(|j| if i == j { 1 } else { 0 })
                .collect::<Vec<i32>>()
        })
        .collect();

    let mut main_problem = Model::default().minimize();

    let cutting_pattern_vars: Vec<Variable> = initial_cutting_patterns
        .iter()
        .enumerate()
        .map(|(i, _pattern)| {
            let pattern = (0..10)
                .map(|x| if x == i { "1" } else { "0" })
                .collect::<Vec<_>>()
                .join("-");
            main_problem.add(var().int(0..).obj(1.0).name(&format!("pattern_{pattern}")))
        })
        .collect();

    demand.iter().enumerate().for_each(|(i, &count)| {
        let demand_constraint = main_problem.add(
            cons()
                .name(&format!("demand_for_item_{i}"))
                .coef(&cutting_pattern_vars[i], 1.0)
                .ge(count as f64),
        );

        main_problem.set_cons_modifiable(&demand_constraint, true);
    });

    let csp_pricer = CSPPricer {
        stock_length,
        item_sizes,
    };

    main_problem.add(pricer(csp_pricer).name("CSPPricer"));
    let solved_model = main_problem.solve();

    println!("\nSolution");
    let solution = solved_model.best_sol().unwrap();
    for var in solved_model.vars().iter() {
        let name = var.name();
        let value = solution.val(var);
        if value != 0.0 {
            println!("  {name}={value}")
        }
    }
}

struct CSPPricer<'a> {
    stock_length: usize,
    item_sizes: &'a [f64],
}

impl Pricer for CSPPricer<'_> {
    fn generate_columns(
        &mut self,
        mut model: Model<Solving>,
        _pricer: SCIPPricer,
        farkas: bool,
    ) -> PricerResult {
        // Pricing has no idea what branching decisions were made by scip, so we only want to run the pricer at the root node
        if model.focus_node().depth() > 0 {
            return PricerResult {
                state: PricerResultState::NoColumns,
                lower_bound: None,
            };
        }

        if farkas {
            unreachable!("Unexpected infeasibility, root node should be feasible by construction and
            the pricer is not expected to be called in deeper nodes.");
        }

        let mut pricing_model = Model::default().hide_output().maximize();

        let vars = (0..self.item_sizes.len())
            .map(|i| {
                let cons = model.find_cons(&format!("demand_for_item_{i}")).unwrap();
                let dual_val = cons.dual_sol();
                pricing_model.add(
                    var()
                        .int(0..)
                        .name(&format!("demand_for_item_{i}"))
                        .obj(dual_val),
                )
            })
            .collect::<Vec<Variable>>();

        pricing_model.add(
            cons()
                .name("is_valid_pattern_constraint")
                .expr(
                    vars.iter()
                        .enumerate()
                        .map(|(idx, var)| (var, self.item_sizes[idx])),
                )
                .le(self.stock_length as f64),
        );

        let solved_model = pricing_model.solve();

        let reduced_cost = solved_model.best_sol().map(|sol| 1.0 - sol.obj_val());
        if reduced_cost.is_some_and(|rc| rc < -1e-6) {
            let solution = solved_model.best_sol().unwrap();
            let pattern = vars
                .iter()
                .map(|var| (solution.val(var) as u32).to_string())
                .collect::<Vec<String>>();

            // add variable for new cutting pattern
            let new_variable_name = &format!("pattern_{}", pattern.join("-"));
            if !model
                .vars()
                .iter()
                .any(|var| &var.name() == new_variable_name)
            {
                println!("    Adding {new_variable_name}");
                let new_variable = model.add_priced_var(
                    0.0,
                    f64::INFINITY,
                    1.0,
                    new_variable_name,
                    VarType::Integer,
                );

                (0..self.item_sizes.len()).for_each(|i| {
                    let constraint = model.find_cons(&format!("demand_for_item_{i}")).unwrap();
                    model.add_cons_coef(&constraint, &new_variable, solution.val(&vars[i]));
                });

                PricerResult {
                    state: PricerResultState::FoundColumns,
                    lower_bound: None,
                }
            } else {
                // avoid adding the same pattern twice and claim that we didn't find any columns instead
                PricerResult {
                    state: PricerResultState::NoColumns,
                    lower_bound: None,
                }
            }
        } else {
            println!(
                "    Didn't find column (obj_value = {}, reduced_cost = {reduced_cost:?})",
                model.obj_val()
            );
            PricerResult {
                state: PricerResultState::NoColumns,
                lower_bound: None,
            }
        }
    }
}
