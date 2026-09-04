# Nonlinear constraints in russcip

`Expr` is an expression tree over the model's variables. Building one is pure Rust — it allocates
nothing in SCIP and cannot fail — and `cons().expression(..)` turns it into a constraint:

```rust
use russcip::prelude::*;

let mut model = Model::default().maximize().hide_output();
let x = model.add(var().name("x").obj(1.).cont(0.0..=10.0));

// x^2 <= 16  =>  x <= 4
model.add(cons().expression(Expr::pow(Expr::var(&x), 2.0)).le(16.0));

let solved = model.solve();
assert!((solved.obj_val() - 4.0).abs() < 1e-6);
```

## Building an expression

The leaves are `Expr::var(&v)` for a model variable and `Expr::constant(k)` for a number. From
there, the arithmetic operators and the named constructors build up the tree.

```rust
use russcip::prelude::*;

let mut model = Model::default().hide_output();
let x = model.add(var().name("x").cont(1.0..=10.0));
let y = model.add(var().name("y").cont(1.0..=10.0));

let a = Expr::pow(Expr::var(&x), 2.0) + 3.0 * Expr::var(&y) - Expr::exp(Expr::var(&x));
let b = Expr::pow(1.0 - Expr::var(&x), 2.0)
    + 100.0 * Expr::pow(Expr::var(&y) - Expr::pow(Expr::var(&x), 2.0), 2.0);
let c = Expr::log(Expr::var(&x)) + 1.0 / Expr::var(&x);
```

Nothing is moved — `Expr::var` borrows and clones the handle — so the same variable can appear in
as many expressions as you like.

**Operators**: `+ - * /`, unary `-`, with `f64` accepted on either side (`2.0 * Expr::var(&x)`
and `Expr::var(&x) * 2.0` both work). Division is lowered to `a * b^-1`, since SCIP has no
division expression.

`^` is deliberately **not** implemented. Rust's `^` is `BitXor`, which binds *looser* than both
`+` and `*`, so `a ^ 2 + b` would silently mean `a ^ (2 + b)`. Powers are written
`Expr::pow(a, 2.0)`.

**Functions**: `Expr::exp`, `Expr::log`, `Expr::sin`, `Expr::cos`, `Expr::abs`, `Expr::entropy`,
`Expr::pow(e, n)` and `Expr::signpower(e, n)`.

Note there is no `sqrt` — SCIP has no `sqrt` expression handler. Write `Expr::pow(x, 0.5)`.

The exponent of `pow` and `signpower` is an `f64`, not an expression. That is not a limitation of
this API — `SCIPcreateExprPow` takes a `SCIP_Real`, so a variable exponent was never
representable.

`signpower(x, n)` is `sign(x)·|x|ⁿ`, the odd extension of a power. For odd integer `n` it agrees
with `x^n`; it earns its place at fractional exponents (where `x^n` is undefined for negative
`x`) and even ones (where `x^n` cannot express direction). It is monotone and odd, hence concave
on `x <= 0` and convex on `x >= 0`, which is a far better structure for the solver than a
hand-rolled sign disjunction. Gas- and water-network models use it for direction-dependent flow.

## Turning an expression into a constraint

`cons().expression(e)` sets the body; the bound methods close it:

```rust
use russcip::prelude::*;

let mut model = Model::default().hide_output();
let x = model.add(var().name("x").cont(0.0..=10.0));
let y = model.add(var().name("y").cont(0.0..=10.0));

let x2 = || Expr::pow(Expr::var(&x), 2.0);
let y2 = || Expr::pow(Expr::var(&y), 2.0);

let a = cons().expression(x2() + y2()).le(16.0);                           // one-sided
let b = cons().expression(Expr::var(&x) + Expr::var(&y)).bounds(1.0, 5.0); // two-sided
let c = cons().expression(Expr::var(&x) * Expr::var(&y)).eq(1.0);          // equality
let d = cons().expression(x2() - Expr::var(&y)).le(0.0);                   // x^2 <= y
let e = cons().expression(Expr::var(&x) + Expr::var(&y)).le(10.0).name("cap").removable(true);

model.add(vec![a, b, c, d, e]);          // or add them one at a time
```

`bounds(lhs, rhs)` is the two-sided form, mapping directly onto SCIP's `lhs <= expression <= rhs`.

A constraint has a single body, so a comparison with variables on both sides is written as one
expression: `A <= B` becomes `A - B <= 0`. Any constant left in the body is folded back out into
the bound, so `cons().expression(x_sq - 16.0).le(0.0)` and `cons().expression(x_sq).le(16.0)`
build the same constraint.

`expression` is not only for nonlinear bodies. A body that turns out to be linear is recognised
as such and builds an ordinary linear constraint, so `cons().expression(2.0 * Expr::var(&x) +
Expr::var(&y)).le(100.0)` costs nothing over the equivalent `cons().coef(..)` chain.

## Sums and products

Aggregates are the usual shape in a real model, and they come from iterators:

```rust
use russcip::prelude::*;

let mut model = Model::default().maximize().hide_output();
let xs: Vec<_> = (0..4)
    .map(|i| model.add(var().name(&format!("x{i}")).obj(1.).cont(0.0..=10.0)))
    .collect();

// sum of squares <= 4
let ball = Expr::sum(xs.iter().map(|x| Expr::pow(Expr::var(x), 2.0)));
model.add(cons().expression(ball).le(4.0).name("ball"));

// the usual `sum(c_i * x_i)` shape
let profit = [2.5, 1.0, 3.0, 0.5];
let weighted = Expr::sum_weighted(profit.iter().zip(&xs).map(|(c, x)| (*c, Expr::var(x))));

// over indices instead, when that reads better
let n = xs.len();
let by_index = Expr::sum_weighted((0..n).map(|i| (profit[i], Expr::var(&xs[i]))));

// a plain sum takes variables directly, since `&Variable: Into<Expr>`
let total = Expr::sum(xs.iter());

// products too
let p = Expr::product(xs.iter().map(Expr::var));
```

- `Expr::sum(iter)` — `Σ eᵢ`
- `Expr::sum_weighted(iter)` — `Σ cᵢ·eᵢ`, over `(f64, impl Into<Expr>)` pairs
- `Expr::product(iter)` — `Π eᵢ`

Empty aggregates give the identity elements, `0` and `1`.

An aggregate is an ordinary `Expr`, so it composes with the operators — extending a sum widens it
rather than nesting it:

```rust
use russcip::prelude::*;

let mut model = Model::default().hide_output();
let xs: Vec<_> = (0..3)
    .map(|i| model.add(var().name(&format!("x{i}")).cont(0.0..=10.0)))
    .collect();
let y = model.add(var().name("y").cont(0.0..=10.0));

let e = Expr::sum(xs.iter()) + Expr::pow(Expr::var(&y), 2.0);
assert_eq!(e.to_string(), "(<x0> + <x1> + <x2> + (<y>^2))");
```

Sums and products are **n-ary** and flatten as they are built, matching SCIP's own expression
model. A sum over a million terms is a single `SCIPcreateExprSum` call one level deep, not a
tower of binary nodes — this holds whether you use `Expr::sum` or fold an iterator with `+`.

`Expr::sum_weighted` puts its coefficients on the sum's edges, where SCIP keeps them, rather than
wrapping each term in a `Product` node. `Expr::sum(iter.map(|(c, x)| c * Expr::var(x)))` builds
the same mathematical expression with an extra node per term.

## Resolving values by type

`AsExpr` turns either a `Variable` or an `f64` into an `Expr`, which lets one closure walk a
coefficient array and a variable array without annotation. It is implemented through references
too, so it survives however many layers of borrowing an iterator binding introduces.

```rust
use russcip::prelude::*;

let mut model = Model::default().hide_output();
let x = model.add(var().name("x").cont(0.0..=10.0));
let a = 2.5f64;

assert_eq!((a.as_expr() * x.as_expr()).to_string(), "(2.5 * <x>)");
```

## Inspecting an expression

`Display` renders an expression on one line, fully parenthesised. `as_tree` shows the node
structure instead, which is what reveals the flattening:

```rust
use russcip::prelude::*;

let mut model = Model::default().hide_output();
let x = model.add(var().name("x").cont(0.0..=10.0));
let y = model.add(var().name("y").cont(0.0..=10.0));

let e = Expr::pow(Expr::var(&x), 2.0) + 3.0 * Expr::var(&y);
println!("{e}");
// ((<x>^2) + (3 * <y>))
println!("{}", e.as_tree());
// Sum
// ├─ Pow(2)
// │  └─ Var(x)
// └─ Product ×3
//    └─ Var(y)
```

## Expressions from strings

`parse_expr` uses SCIP's own syntax, in which variables are named between angle brackets:

```rust
use russcip::prelude::*;

let mut model = Model::default().hide_output();
let x = model.add(var().name("x").cont(0.0..=10.0));

let e = model.parse_expr("<x>^2").unwrap();
```

It returns a `Result`, since parsing can fail on input the caller supplies.

### Which to use

Prefer `Expr` whenever the expression is known at compile time. It refers to variables **by
handle**, whereas `parse_expr` resolves them **by name**, and that difference is observable:

- **Duplicate names.** SCIP permits two variables both named `x`. `parse_expr("<x>")` succeeds
  and silently binds to whichever it finds first; `Expr::var(&x1)` names the one you meant.
- **Awkward names.** A variable named `a>b` cannot be written in `<name>` syntax at all — there
  is no escape — so it is unreachable from `parse_expr`.
- **Errors move to compile time.** Typos and unknown functions are caught by the compiler rather
  than surfacing as a runtime `Retcode::ReadError`.

`parse_expr` remains the right tool for expressions that only exist at runtime — read from a
file, or entered by a user.

## Lower-level building

`build_expr` turns an `Expr` into a `ScipExpr` without making a constraint of it, which is what
you want when the same expression feeds several constraints, or when you need
`add_cons_nonlinear` directly:

```rust
use russcip::prelude::*;

let mut model = Model::default().maximize().hide_output();
let x = model.add(var().name("x").obj(1.).cont(0.0..=10.0));

let e = model.build_expr(&Expr::pow(Expr::var(&x), 2.0)).unwrap();
model.add_cons_nonlinear(&e, -f64::INFINITY, 16.0, "c");
```

A `ScipExpr` is reference-counted by SCIP and releases its reference on `Drop`, so it may be
handed to `add_cons_nonlinear` more than once.

## A nonlinear objective

SCIP has no nonlinear objective function. The standard workaround is an auxiliary variable that
carries the objective, bounded by the nonlinear expression:

```rust
use russcip::prelude::*;

let mut model = Model::default().minimize().hide_output();
let x = model.add(var().name("x").cont(-2.0..=2.0));
let y = model.add(var().name("y").cont(-2.0..=2.0));
let t = model.add(var().name("t").obj(1.0).cont(0.0..=1e6));

// minimise t subject to  (1-x)^2 + 100*(y - x^2)^2 <= t
let body = Expr::pow(1.0 - Expr::var(&x), 2.0)
    + 100.0 * Expr::pow(Expr::var(&y) - Expr::pow(Expr::var(&x), 2.0), 2.0)
    - Expr::var(&t);
model.add(cons().expression(body).le(0.0).name("rosenbrock"));

let solved = model.solve();
assert!(solved.obj_val() < 1e-4); // optimum is 0 at (1, 1)
```

See [`examples/nonlinear.rs`](examples/nonlinear.rs) for this and two more worked models.
