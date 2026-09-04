//! Procedural macros for [`russcip`](https://docs.rs/russcip).
//!
//! [`expr!`] builds an expression, [`cons!`] builds a whole constraint from a
//! comparison. Both share one recursive-descent parser over the raw token
//! stream — `syn::ScipExpr` is unusable here because Rust parses `^` as `BitXor`,
//! which binds looser than `+` and `*`.

use proc_macro::TokenStream;
use proc_macro2::{Delimiter, Group, Spacing, Span, TokenStream as TS2, TokenTree};
use proc_macro_crate::{FoundCrate, crate_name};
use quote::{quote, quote_spanned};

/// The path to the `russcip` crate as seen from the crate invoking the macro.
///
/// `proc_macro_crate` resolves the real name, so a downstream crate that renames
/// the dependency (`scip = { package = "russcip", .. }`) still expands to a
/// resolvable path. When the macro is used inside the `russcip` package it stays
/// `::russcip` (see the match below), which resolves through
/// `extern crate self as russcip;` in the library and through the dependency in
/// doctests and examples.
fn russcip_path() -> TS2 {
    match crate_name("russcip") {
        // `Itself` is returned for every compilation unit that belongs to the
        // `russcip` package — the library, its unit tests, its doctests and its
        // examples. `crate` is only correct for the first two, so always use
        // `::russcip`, which resolves inside the library via
        // `extern crate self as russcip;` and in doctests/examples through the
        // dependency.
        Ok(FoundCrate::Itself) | Err(_) => quote!(::russcip),
        // A downstream crate may rename the dependency (`scip = { package =
        // "russcip", .. }`); resolve the real name so the path still works.
        Ok(FoundCrate::Name(name)) => {
            let ident = proc_macro2::Ident::new(&name, Span::call_site());
            quote!(::#ident)
        }
    }
}

/// Unary functions SCIP has an expression handler for.
///
/// `sqrt` is deliberately absent: SCIP has no `sqrt` handler, and `x^0.5` is
/// the supported spelling. The error message says so.
const FUNCS: &[&str] = &["exp", "log", "sin", "cos", "abs", "entropy"];

type PErr = (Span, String);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Cmp {
    Le,
    Ge,
    Eq,
}

/// Parses a Rust numeric literal as an `f64`.
///
/// `Literal::to_string()` keeps the source spelling, so it can carry separators
/// and a type suffix: `1_000`, `2.0_f64`, `1.5e3f32`. `parse::<f64>()` rejects
/// all of those, so strip the underscores and any trailing type suffix (the
/// trailing run of alphabetic characters — exponent digits stop the scan, so
/// `1.5e3f64` leaves `1.5e3`) before parsing.
fn parse_number(s: &str) -> Option<f64> {
    let s = s.replace('_', "");
    if let Ok(v) = s.parse::<f64>() {
        return Some(v);
    }
    let end = s
        .bytes()
        .rposition(|b| !b.is_ascii_alphabetic())
        .map(|i| i + 1)
        .unwrap_or(0);
    s[..end].parse::<f64>().ok()
}

struct Parser {
    toks: Vec<TokenTree>,
    pos: usize,
    crate_path: TS2,
}

impl Parser {
    fn new(ts: TS2) -> Self {
        Parser {
            toks: ts.into_iter().collect(),
            pos: 0,
            crate_path: russcip_path(),
        }
    }

    fn peek(&self) -> Option<&TokenTree> {
        self.toks.get(self.pos)
    }

    fn peek_at(&self, off: usize) -> Option<&TokenTree> {
        self.toks.get(self.pos + off)
    }

    fn span_here(&self) -> Span {
        self.peek()
            .map(|t| t.span())
            .unwrap_or_else(Span::call_site)
    }

    /// Consumes a single-character operator. The expression grammar has no
    /// multi-character operators, and every comparison operator starts with a
    /// character this never matches, so a plain character test is enough.
    fn eat(&mut self, c: char) -> bool {
        if let Some(TokenTree::Punct(p)) = self.peek()
            && p.as_char() == c
        {
            self.pos += 1;
            return true;
        }
        false
    }

    /// Consumes a comparison operator: `<=`, `>=`, `==` or `=`.
    ///
    /// `<=` arrives as `Punct('<', Joint)` followed by `Punct('=')`, whereas a
    /// bare `<` is `Punct('<', Alone)`, so the two are distinguishable.
    fn eat_cmp(&mut self) -> Option<(Cmp, Span)> {
        let (c, spacing, span) = match self.peek() {
            Some(TokenTree::Punct(p)) => (p.as_char(), p.spacing(), p.span()),
            _ => return None,
        };
        let then_eq = matches!(self.peek_at(1), Some(TokenTree::Punct(p)) if p.as_char() == '=');

        match c {
            '<' | '>' if spacing == Spacing::Joint && then_eq => {
                self.pos += 2;
                Some((if c == '<' { Cmp::Le } else { Cmp::Ge }, span))
            }
            '=' if spacing == Spacing::Joint && then_eq => {
                self.pos += 2;
                Some((Cmp::Eq, span))
            }
            '=' if spacing != Spacing::Joint => {
                self.pos += 1;
                Some((Cmp::Eq, span))
            }
            _ => None,
        }
    }

    /// The error to report where a comparison was expected but not found.
    fn cmp_error(&self) -> PErr {
        match self.peek() {
            Some(TokenTree::Punct(p)) if p.as_char() == '<' || p.as_char() == '>' => (
                p.span(),
                format!(
                    "`{0}` is not representable — a constraint has closed bounds; use `{0}=`",
                    p.as_char()
                ),
            ),
            Some(t) => (
                t.span(),
                format!("expected a comparison (`<=`, `>=`, `=`), found `{t}`"),
            ),
            None => (
                Span::call_site(),
                "expected a comparison (`<=`, `>=`, `=`)".to_string(),
            ),
        }
    }

    /// `expr := term (('+' | '-') term)*`
    fn expr(&mut self) -> Result<TS2, PErr> {
        let mut lhs = self.term()?;
        loop {
            if self.eat('+') {
                let r = self.term()?;
                lhs = quote! { ::core::ops::Add::add(#lhs, #r) };
            } else if self.eat('-') {
                let r = self.term()?;
                lhs = quote! { ::core::ops::Sub::sub(#lhs, #r) };
            } else {
                return Ok(lhs);
            }
        }
    }

    /// `term := unary (('*' | '/') unary)*`
    fn term(&mut self) -> Result<TS2, PErr> {
        let mut lhs = self.unary()?;
        loop {
            if self.eat('*') {
                let r = self.unary()?;
                lhs = quote! { ::core::ops::Mul::mul(#lhs, #r) };
            } else if self.eat('/') {
                let r = self.unary()?;
                lhs = quote! { ::core::ops::Div::div(#lhs, #r) };
            } else {
                return Ok(lhs);
            }
        }
    }

    /// `unary := ('-' | '+')* power`
    fn unary(&mut self) -> Result<TS2, PErr> {
        if self.eat('-') {
            let e = self.unary()?;
            return Ok(quote! { ::core::ops::Neg::neg(#e) });
        }
        if self.eat('+') {
            return self.unary();
        }
        self.power()
    }

    /// `power := atom ('^' signed-number)?`
    ///
    /// The exponent must be a numeric literal. That is not a shortcut: SCIP's
    /// own grammar is `Factor -> Base [ "^" number ]`, and `SCIPcreateExprPow`
    /// takes a `SCIP_Real` exponent, so a variable exponent was never
    /// representable. Rejecting it here turns it into a compile error rather
    /// than a confusing runtime failure.
    fn power(&mut self) -> Result<TS2, PErr> {
        let base = self.atom()?;
        if self.eat('^') {
            let (v, span) = self.signed_number()?;
            let cp = &self.crate_path;
            return Ok(quote_spanned! { span => #cp::Expr::pow(#base, #v) });
        }
        Ok(base)
    }

    /// An optionally signed numeric literal. `-` and the literal are separate
    /// tokens in a `TokenStream`, so the sign has to be taken here.
    fn signed_number(&mut self) -> Result<(f64, Span), PErr> {
        let neg = if self.eat('-') {
            true
        } else {
            self.eat('+');
            false
        };
        let (v, span) = self.number()?;
        Ok((if neg { -v } else { v }, span))
    }

    /// Like [`Parser::signed_number`] but speculative: on anything that is not
    /// a signed literal it rewinds and yields `None`.
    fn try_signed_number(&mut self) -> Option<(f64, Span)> {
        let save = self.pos;
        match self.signed_number() {
            Ok(v) => Some(v),
            Err(_) => {
                self.pos = save;
                None
            }
        }
    }

    fn number(&mut self) -> Result<(f64, Span), PErr> {
        match self.peek().cloned() {
            Some(TokenTree::Literal(lit)) => {
                self.pos += 1;
                let s = lit.to_string();
                match parse_number(&s) {
                    Some(v) => Ok((v, lit.span())),
                    None => Err((lit.span(), format!("`{s}` is not a numeric literal"))),
                }
            }
            Some(other) => Err((
                other.span(),
                format!("expected a numeric literal, found `{other}` — SCIP takes a constant here"),
            )),
            None => Err((Span::call_site(), "expected a numeric literal".to_string())),
        }
    }

    /// `atom := number | place | call | comprehension | '(' expr ')' | '{' rust '}'`
    fn atom(&mut self) -> Result<TS2, PErr> {
        let tt = match self.peek().cloned() {
            Some(t) => t,
            None => {
                return Err((
                    Span::call_site(),
                    "unexpected end of expression".to_string(),
                ));
            }
        };

        match tt {
            // `{ rust_expr }` splices in any value that is `Into<Expr>` — the way
            // to use a value built in Rust inside an otherwise literal
            // expression.
            TokenTree::Group(g) if g.delimiter() == Delimiter::Brace => {
                self.pos += 1;
                let inner = g.stream();
                let cp = &self.crate_path;
                Ok(quote_spanned! { g.span() => #cp::Expr::of(#inner) })
            }

            TokenTree::Group(g) if g.delimiter() == Delimiter::Parenthesis => {
                self.pos += 1;
                let mut inner = Parser::new(g.stream());
                let e = inner.expr()?;
                inner.finish()?;
                Ok(e)
            }

            TokenTree::Ident(id) => {
                self.pos += 1;
                let name = id.to_string();

                let call_group = match self.peek() {
                    Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Parenthesis => {
                        Some(g.clone())
                    }
                    _ => None,
                };

                if let Some(g) = call_group {
                    self.pos += 1;

                    if name == "sum" || name == "prod" {
                        return self.comprehension(&name, &g, id.span());
                    }

                    // `signpower(x, n)` = sign(x)|x|^n. Unlike the others it
                    // takes an exponent as well as a child, mirroring
                    // `SCIPcreateExprSignpower`.
                    if name == "signpower" {
                        let mut inner = Parser::new(g.stream());
                        let arg = inner.expr()?;
                        if !inner.eat(',') {
                            return Err((
                                id.span(),
                                "signpower takes two arguments: `signpower(expr, exponent)`"
                                    .to_string(),
                            ));
                        }
                        let (v, _) = inner.signed_number()?;
                        inner.finish()?;
                        let cp = &self.crate_path;
                        return Ok(
                            quote_spanned! { id.span() => #cp::Expr::signpower(#arg, #v) },
                        );
                    }

                    if !FUNCS.contains(&name.as_str()) {
                        let hint = if name == "sqrt" {
                            " — SCIP has no `sqrt` handler; write `x^0.5` instead".to_string()
                        } else {
                            format!(
                                " — supported: {}, signpower(expr, n), sum(..), prod(..)",
                                FUNCS.join(", ")
                            )
                        };
                        return Err((id.span(), format!("unknown function `{name}`{hint}")));
                    }
                    let mut inner = Parser::new(g.stream());
                    let arg = inner.expr()?;
                    inner.finish()?;
                    let f = proc_macro2::Ident::new(&name, id.span());
                    let cp = &self.crate_path;
                    return Ok(quote_spanned! { id.span() => #cp::Expr::#f(#arg) });
                }

                // A bare identifier, possibly indexed (`x`, `x[i]`, `g[i][j]`),
                // names a value in scope. `AsExpr` resolves it by type, so a
                // `Variable` becomes a variable term and an `f64` a constant —
                // which is what lets `c[i] * x[i]` work without annotation.
                let place = self.place(&id);
                let cp = &self.crate_path;
                Ok(quote_spanned! { id.span() => #cp::AsExpr::as_expr(&#place) })
            }

            TokenTree::Literal(lit) => {
                self.pos += 1;
                let s = lit.to_string();
                let cp = &self.crate_path;
                match parse_number(&s) {
                    Some(v) => Ok(quote_spanned! { lit.span() => #cp::Expr::constant(#v) }),
                    None => Err((lit.span(), format!("`{s}` is not a numeric literal"))),
                }
            }

            other => Err((other.span(), format!("unexpected token `{other}`"))),
        }
    }

    /// An identifier followed by any number of `[...]` index groups, re-emitted
    /// as the Rust place expression it is.
    fn place(&mut self, id: &proc_macro2::Ident) -> TS2 {
        let mut out = quote! { #id };
        while let Some(TokenTree::Group(g)) = self.peek() {
            if g.delimiter() != Delimiter::Bracket {
                break;
            }
            let g = g.clone();
            self.pos += 1;
            out = quote! { #out #g };
        }
        out
    }

    /// `sum(pattern in iterable, expression)`, and `prod` likewise.
    ///
    /// Expands to `Expr::sum(IntoIterator::into_iter(iterable).map(|pattern| expression))`.
    fn comprehension(&mut self, kind: &str, g: &Group, span: Span) -> Result<TS2, PErr> {
        let toks: Vec<TokenTree> = g.stream().into_iter().collect();
        let usage =
            format!("`{kind}` takes a comprehension: `{kind}(pattern in iterable, expression)`");

        // Nested groups are single token trees, so a match at this level is
        // genuinely top-level: `zip(a, b)` hides its comma, and a tuple pattern
        // `(i, x)` hides its own.
        let in_pos = toks
            .iter()
            .position(|t| matches!(t, TokenTree::Ident(i) if *i == "in"));
        let in_pos = match in_pos {
            Some(p) if p > 0 => p,
            _ => return Err((span, usage)),
        };

        let comma = match toks[in_pos + 1..]
            .iter()
            .position(|t| matches!(t, TokenTree::Punct(p) if p.as_char() == ','))
        {
            Some(c) => in_pos + 1 + c,
            None => return Err((span, usage)),
        };

        let pattern: TS2 = toks[..in_pos].iter().cloned().collect();
        let iterable: TS2 = toks[in_pos + 1..comma].iter().cloned().collect();
        let body_toks: TS2 = toks[comma + 1..].iter().cloned().collect();

        if iterable.is_empty() || body_toks.is_empty() {
            return Err((span, usage));
        }

        let mut inner = Parser::new(body_toks);
        let body = inner.expr()?;
        inner.finish()?;

        let ctor = proc_macro2::Ident::new(if kind == "sum" { "sum" } else { "product" }, span);
        let cp = &self.crate_path;

        Ok(quote_spanned! { span =>
            #cp::Expr::#ctor(
                ::core::iter::IntoIterator::into_iter(#iterable).map(|#pattern| #body)
            )
        })
    }

    fn finish(&mut self) -> Result<(), PErr> {
        match self.peek() {
            None => Ok(()),
            Some(t) => Err((t.span(), format!("unexpected trailing token `{t}`"))),
        }
    }
}

/// Parses a whole constraint: either `expr CMP expr` or `lit CMP expr CMP lit`.
fn parse_constraint(ts: TS2) -> Result<TS2, PErr> {
    let mut p = Parser::new(ts);

    if let Some(chained) = try_chained(&mut p)? {
        return Ok(chained);
    }

    let lhs = p.expr()?;
    let cmp = match p.eat_cmp() {
        Some((c, _)) => c,
        None => return Err(p.cmp_error()),
    };
    let rhs = p.expr()?;
    p.finish()?;

    // SCIP constraints are `lhs <= expression <= rhs` with constant bounds, so
    // an expression on both sides moves to one: `A <= B` becomes `A - B <= 0`.
    // Any constant inside is folded back out into the bound when the constraint
    // is added.
    let diff = quote! { ::core::ops::Sub::sub(#lhs, #rhs) };
    let bounded = match cmp {
        Cmp::Le => quote! { .le(0.0) },
        Cmp::Ge => quote! { .ge(0.0) },
        Cmp::Eq => quote! { .eq(0.0) },
    };
    let cp = &p.crate_path;
    Ok(quote! { #cp::builder::cons::cons().expression(#diff) #bounded })
}

/// Speculatively parses `lit <= expr <= lit` (or `>=`), rewinding if the input
/// is not of that shape. This is the two-sided form, which plain Rust cannot
/// express because it forbids chained comparison.
fn try_chained(p: &mut Parser) -> Result<Option<TS2>, PErr> {
    let save = p.pos;

    let lo = match p.try_signed_number() {
        Some((v, _)) => v,
        None => return Ok(None),
    };
    let first = match p.eat_cmp() {
        Some((c, _)) if c != Cmp::Eq => c,
        _ => {
            p.pos = save;
            return Ok(None);
        }
    };
    let ex = match p.expr() {
        Ok(e) => e,
        Err(_) => {
            p.pos = save;
            return Ok(None);
        }
    };
    let second = match p.eat_cmp() {
        Some((c, span)) => {
            if c != first {
                return Err((
                    span,
                    "the two comparisons in a two-sided constraint must point the same way"
                        .to_string(),
                ));
            }
            c
        }
        None => {
            // Just `lit <= expr`; let the ordinary path handle it.
            p.pos = save;
            return Ok(None);
        }
    };
    let hi = match p.try_signed_number() {
        Some((v, _)) => v,
        None => {
            return Err((
                p.span_here(),
                "the outer bounds of a two-sided constraint must be numeric literals".to_string(),
            ));
        }
    };
    p.finish()?;

    let (lhs, rhs) = if second == Cmp::Le {
        (lo, hi)
    } else {
        (hi, lo)
    };
    let cp = &p.crate_path;
    Ok(Some(
        quote! { #cp::builder::cons::cons().expression(#ex).bounds(#lhs, #rhs) },
    ))
}

/// Builds an [`Expr`](../russcip/enum.Expr.html) expression tree from mathematical syntax.
///
/// Identifiers refer to values in scope and are auto-referenced, so callers
/// write `x`, not `&x`, and nothing is moved. Indexing works, and the type
/// decides the meaning: a `Variable` becomes a variable term, an `f64` a
/// constant, so `c[i] * x[i]` needs no annotation.
///
/// Operator precedence is mathematical, **not** Rust's: `^` binds tighter than
/// `*`, so `x^2 + 3*y` means `(x^2) + (3*y)`. This is why the macro parses the
/// raw token stream instead of a `syn::ScipExpr` — Rust itself parses `^` as
/// `BitXor`, which binds looser than both `+` and `*`.
///
/// The exponent must be a numeric literal and `^` is **not** chainable: SCIP's
/// `SCIPcreateExprPow` takes a `SCIP_Real` exponent, so `x ^ 2 ^ 3` has no
/// meaning. Write `x ^ 8` instead.
///
/// ```ignore
/// expr!(x^2 + 3*y - exp(x))
/// expr!(sum(i in 0..n, c[i] * x[i]))
/// expr!(prod(x in &xs, x))
/// expr!({ precomputed } + y^2)
/// ```
#[proc_macro]
pub fn expr(input: TokenStream) -> TokenStream {
    let ts: TS2 = input.into();
    if ts.is_empty() {
        return quote! { compile_error!("expr! requires a non-empty expression") }.into();
    }

    let mut p = Parser::new(ts);
    let built = p.expr().and_then(|e| p.finish().map(|_| e));

    match built {
        Ok(e) => e.into(),
        Err((span, msg)) => quote_spanned! { span => compile_error!(#msg) }.into(),
    }
}

/// Builds a constraint from a comparison, ready to pass to `Model::add`.
///
/// Accepts `<=`, `>=`, `=` and `==`; strict `<` and `>` are rejected, since a
/// constraint has closed bounds. Either side may be an expression. The
/// two-sided form maps directly onto SCIP's `lhs <= expression <= rhs` and is
/// not expressible in plain Rust, which forbids chained comparison.
///
/// ```ignore
/// model.add(cons!(x^2 + y^2 <= 4));
/// model.add(cons!(1 <= x + y <= 5).name("band"));
/// model.add(cons!(x * y = 1));
/// model.add(cons!(sum(i in 0..n, c[i] * x[i]) <= 10));
/// ```
#[proc_macro]
pub fn cons(input: TokenStream) -> TokenStream {
    let ts: TS2 = input.into();
    if ts.is_empty() {
        return quote! { compile_error!("cons! requires a comparison, e.g. `cons!(x^2 <= 16)`") }
            .into();
    }

    match parse_constraint(ts) {
        Ok(e) => e.into(),
        Err((span, msg)) => quote_spanned! { span => compile_error!(#msg) }.into(),
    }
}
