/*
* Copyright (C) 2019-2021, Miklos Maroti
*
* This program is free software: you can redistribute it and/or modify
* it under the terms of the GNU General Public License as published by
* the Free Software Foundation, either version 3 of the License, or
* (at your option) any later version.
*
* This program is distributed in the hope that it will be useful,
* but WITHOUT ANY WARRANTY; without even the implied warranty of
* MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
* GNU General Public License for more details.
*
* You should have received a copy of the GNU General Public License
* along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

use std::rc::Rc;
use std::{fmt, ops, ptr};

struct SolverItem<'a, ITEM: ?Sized>(&'a State, &'a ITEM);

#[derive(Debug)]
struct Domain {
    size: usize,
    name: String,
}

impl Domain {
    fn new(name: String, size: usize) -> Self {
        Self { name, size }
    }

    fn size(&self) -> usize {
        self.size
    }

    fn ptr_eq(&self, other: &Domain) -> bool {
        ptr::eq(self, other)
    }
}

fn get_coords(domains: &[Rc<Domain>], mut offset: usize, coords: &mut [usize]) {
    debug_assert_eq!(domains.len(), coords.len());
    for (size, coord) in domains
        .iter()
        .map(|dom| dom.size())
        .zip(coords.iter_mut())
        .rev()
    {
        *coord = offset % size;
        offset /= size;
    }
    debug_assert_eq!(offset, 0);
}

fn get_offset<I>(domains: &[Rc<Domain>], coords: I) -> usize
where
    I: ExactSizeIterator<Item = usize>,
{
    debug_assert_eq!(domains.len(), coords.len());
    let mut offset = 0;
    for (size, coord) in domains.iter().map(|dom| dom.size()).zip(coords) {
        debug_assert!(coord < size);
        offset *= size;
        offset += coord;
    }
    offset
}

#[derive(Debug)]
struct Predicate {
    name: String,
    domains: Box<[Rc<Domain>]>,
    var_start: usize,
    var_count: usize,
}

impl Predicate {
    fn new(name: String, domains: Vec<Rc<Domain>>, var_start: usize) -> Self {
        let domains = domains.into_boxed_slice();
        let var_count = domains.iter().map(|dom| dom.size).product();
        Self {
            name,
            domains,
            var_start,
            var_count,
        }
    }

    fn arity(&self) -> usize {
        self.domains.len()
    }

    fn get_coords(&self, offset: usize, coords: &mut [usize]) {
        get_coords(&self.domains, offset, coords);
    }

    fn get_offset<I>(&self, coords: I) -> usize
    where
        I: ExactSizeIterator<Item = usize>,
    {
        get_offset(&self.domains, coords)
    }
}

impl fmt::Display for Predicate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}(", self.name)?;
        let mut first = true;
        for dom in self.domains.iter() {
            if first {
                first = false;
            } else {
                write!(f, ",")?;
            }
            write!(f, "{}", dom.name)?;
        }
        write!(f, ")")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LiteralIdx(usize);

impl LiteralIdx {
    fn new(negated: bool, variable: usize) -> Self {
        debug_assert!(variable <= (usize::MAX >> 1));
        Self((variable << 1) + (negated as usize))
    }

    fn negated(self) -> bool {
        (self.0 & 1) != 0
    }

    fn variable(self) -> usize {
        self.0 >> 1
    }
}

impl ops::Not for LiteralIdx {
    type Output = Self;

    fn not(self) -> Self {
        LiteralIdx(self.0 ^ 1)
    }
}

impl ops::BitXor<bool> for LiteralIdx {
    type Output = Self;

    fn bitxor(self, rhs: bool) -> Self {
        LiteralIdx(self.0 ^ (rhs as usize))
    }
}

#[derive(Debug, Clone)]
struct Literal<'a> {
    negated: bool,
    predicate: &'a Rc<Predicate>,
    coords: Vec<usize>,
}

impl<'a> Literal<'a> {
    fn new(negated: bool, predicate: &'a Rc<Predicate>, coords: Vec<usize>) -> Self {
        debug_assert_eq!(coords.len(), predicate.arity());
        Self {
            negated,
            predicate,
            coords,
        }
    }

    fn idx(&self) -> LiteralIdx {
        let var = self.predicate.var_start + self.predicate.get_offset(self.coords.iter().cloned());
        LiteralIdx::new(self.negated, var)
    }

    fn destroy(self) -> Vec<usize> {
        self.coords
    }
}

impl<'a> fmt::Display for Literal<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}{}[",
            if self.negated { '-' } else { '+' },
            self.predicate.name
        )?;
        let mut first = true;
        for coord in self.coords.iter() {
            if first {
                first = false;
            } else {
                write!(f, ",")?;
            }
            write!(f, "{}", coord)?;
        }
        write!(f, "]")
    }
}

#[derive(Debug)]
struct AtomicFormula {
    negated: bool,
    predicate: Rc<Predicate>,
    variables: Box<[usize]>,
}

impl AtomicFormula {
    fn new(negated: bool, predicate: Rc<Predicate>, variables: Vec<usize>) -> Self {
        let variables = variables.into_boxed_slice();
        assert_eq!(predicate.arity(), variables.len());

        Self {
            negated,
            predicate,
            variables,
        }
    }
}

impl fmt::Display for AtomicFormula {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}{}(",
            if self.negated { '-' } else { '+' },
            self.predicate.name
        )?;
        let mut first = true;
        for &var in self.variables.iter() {
            if first {
                first = false;
            } else {
                write!(f, ",")?;
            }
            write!(f, "x{}", var)?;
        }
        write!(f, ")")
    }
}

#[derive(Debug)]
struct UniversalFormula {
    domains: Box<[Rc<Domain>]>,
    disjunction: Box<[AtomicFormula]>,
    cla_start: usize,
    cla_count: usize,
}

impl UniversalFormula {
    fn new<ITER>(disjunction: ITER, cla_start: usize) -> Self
    where
        ITER: ExactSizeIterator<Item = (bool, Rc<Predicate>, Vec<usize>)>,
    {
        let mut domains: Vec<Option<Rc<Domain>>> = Default::default();
        let disjunction: Vec<AtomicFormula> = disjunction
            .map(|(neg, pred, vars)| {
                for (pos, &var) in vars.iter().enumerate() {
                    if domains.len() <= var {
                        domains.resize(var + 1, None);
                    }
                    let dom1 = &pred.domains[pos];
                    let dom2 = &mut domains[var];
                    if let Some(dom2) = dom2 {
                        assert!(dom1.ptr_eq(dom2));
                    } else {
                        *dom2 = Some(dom1.clone());
                    }
                }
                AtomicFormula::new(neg, pred, vars)
            })
            .collect();

        let domains: Vec<Rc<Domain>> = domains.into_iter().map(|d| d.unwrap()).collect();
        let cla_count = domains.iter().map(|dom| dom.size()).product();

        Self {
            domains: domains.into_boxed_slice(),
            disjunction: disjunction.into_boxed_slice(),
            cla_start,
            cla_count,
        }
    }

    fn arity(&self) -> usize {
        self.domains.len()
    }

    fn get_coords(&self, offset: usize, coords: &mut [usize]) {
        get_coords(&self.domains, offset, coords);
    }

    fn get_offset<I>(&self, coords: I) -> usize
    where
        I: ExactSizeIterator<Item = usize>,
    {
        get_offset(&self.domains, coords)
    }
}

impl fmt::Display for UniversalFormula {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut first = true;
        for atom in self.disjunction.iter() {
            if first {
                first = false;
            } else {
                write!(f, " | ")?;
            }
            write!(f, "{}", atom)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ClauseIdx(usize);

#[derive(Debug, Clone)]
struct Clause<'a> {
    formula: &'a UniversalFormula,
    coords: Vec<usize>,
}

impl<'a> Clause<'a> {
    fn new(formula: &'a UniversalFormula, coords: Vec<usize>) -> Self {
        debug_assert_eq!(coords.len(), formula.arity());
        Self { formula, coords }
    }

    fn idx(&self) -> ClauseIdx {
        let cla_offset = self.formula.get_offset(self.coords.iter().cloned());
        ClauseIdx(self.formula.cla_start + cla_offset)
    }

    fn literals(&self) -> Vec<Literal> {
        self.formula
            .disjunction
            .iter()
            .map(|atom| {
                Literal::new(
                    atom.negated,
                    &atom.predicate,
                    atom.variables.iter().map(|&var| self.coords[var]).collect(),
                )
            })
            .collect()
    }

    fn destroy(self) -> Vec<usize> {
        self.coords
    }
}

impl<'a> fmt::Display for Clause<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut coords = vec![0; 0];
        let mut first = true;
        for atom in self.formula.disjunction.iter() {
            if first {
                first = false;
            } else {
                write!(f, " | ")?;
            }

            coords.clear();
            coords.extend(atom.variables.iter().map(|&var| self.coords[var]));
            let lit = Literal::new(atom.negated, &atom.predicate, coords);
            write!(f, "{}", lit)?;
            coords = lit.destroy();
        }
        Ok(())
    }
}

trait Watcher: fmt::Debug {
    /// Searchers all combination of coordinates and unit propagates assuming
    /// that all previous values were false.
    fn propagate(&self, state: &mut State, coords: &mut Vec<usize>);

    /// Returns true if for some combination of coordinates all subsequent
    /// predicate becomes false.
    fn has_false(&self, state: &State, coords: &mut Vec<usize>) -> bool;
}

struct WatcherLast;

impl fmt::Debug for WatcherLast {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("WatcherLast").finish()
    }
}

impl Watcher for WatcherLast {
    fn propagate(&self, _state: &mut State, _coords: &mut Vec<usize>) {}

    fn has_false(&self, _state: &State, _coords: &mut Vec<usize>) -> bool {
        true
    }
}

struct WatcherPred {
    negated: bool,
    predicate: Rc<Predicate>,
    variables: Box<[usize]>,
    next: Box<dyn Watcher>,
}

impl fmt::Debug for WatcherPred {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("WatcherPred")
            .field("megated", &self.negated)
            .field("predicate", &self.predicate)
            .field("variables", &self.variables)
            .field("next", &self.next)
            .finish()
    }
}

impl WatcherPred {
    fn get_literal(&self, coords: &[usize]) -> LiteralIdx {
        let offset = self
            .predicate
            .get_offset(self.variables.iter().map(|&i| coords[i]));
        LiteralIdx::new(self.negated, offset)
    }
}

impl Watcher for WatcherPred {
    fn propagate(&self, state: &mut State, coords: &mut Vec<usize>) {
        let lit = self.get_literal(coords);
        let val = state.get_value(lit);
        if val < 0 {
            self.next.propagate(state, coords);
        } else if val == 0 && self.next.has_false(state, coords) {
            state.enqueue(lit);
        }
    }

    fn has_false(&self, solver: &State, coords: &mut Vec<usize>) -> bool {
        let val = solver.get_value(self.get_literal(coords));
        if val < 0 {
            self.next.has_false(solver, coords)
        } else {
            false
        }
    }
}

#[derive(Debug, Default)]
pub struct State {
    values: Vec<i8>,
}

impl State {
    fn get_variables(&self) -> usize {
        self.values.len()
    }

    fn set_variables(&mut self, count: usize) {
        self.values.resize(count, 0);
    }

    fn get_value(&self, idx: LiteralIdx) -> i8 {
        let value = self.values[idx.variable()];
        if idx.negated() {
            -value
        } else {
            value
        }
    }

    fn enqueue(&mut self, lit: LiteralIdx) {
        assert!(self.get_value(lit) == 0);
    }
}

#[derive(Debug, Default)]
pub struct Solver {
    state: State,
    domains: Vec<Rc<Domain>>,
    predicates: Vec<Rc<Predicate>>,
    formulas: Vec<UniversalFormula>,
    cla_count: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct DomainIdx(usize);

#[derive(Debug, Clone, Copy)]
pub struct PredicateIdx(usize);

impl Solver {
    pub fn add_domain(&mut self, name: String, size: usize) -> DomainIdx {
        let idx = DomainIdx(self.domains.len());
        self.domains.push(Rc::new(Domain::new(name, size)));
        idx
    }

    pub fn add_predicate(&mut self, name: String, domains: Vec<DomainIdx>) -> PredicateIdx {
        let domains: Vec<Rc<Domain>> = domains
            .into_iter()
            .map(|idx| self.domains[idx.0].clone())
            .collect();
        let idx = PredicateIdx(self.predicates.len());
        let pred = Rc::new(Predicate::new(name, domains, self.state.get_variables()));
        self.state
            .set_variables(self.state.get_variables() + pred.var_count);
        self.predicates.push(pred);
        idx
    }

    pub fn add_formula(&mut self, disjunction: Vec<(bool, PredicateIdx, Vec<usize>)>) {
        let disjunction = disjunction
            .into_iter()
            .map(|(pos, pred, vars)| (!pos, self.predicates[pred.0].clone(), vars));
        let formula = UniversalFormula::new(disjunction, self.cla_count);
        self.cla_count += formula.cla_count;
        self.formulas.push(formula);
    }

    fn get_literal(&self, idx: LiteralIdx) -> Literal {
        let negated = idx.negated();
        let mut offset = idx.variable();
        for predicate in self.predicates.iter() {
            if offset < predicate.var_count {
                let mut coords = vec![0; predicate.arity()];
                predicate.get_coords(offset, &mut coords);
                let lit = Literal::new(negated, predicate, coords);
                debug_assert_eq!(lit.idx(), idx);
                return lit;
            }
            offset -= predicate.var_count;
        }
        panic!();
    }

    fn get_clause(&self, idx: ClauseIdx) -> Clause {
        let mut offset = idx.0;
        for formula in self.formulas.iter() {
            if offset < formula.cla_count {
                let mut coords = vec![0; formula.arity()];
                formula.get_coords(offset, &mut coords);
                let cla = Clause::new(formula, coords);
                debug_assert_eq!(cla.idx(), idx);
                return cla;
            }
            offset -= formula.cla_count;
        }
        panic!();
    }

    pub fn print(&self) {
        for dom in self.domains.iter() {
            println!("domain {} = {}", dom.name, dom.size);
        }
        for pred in self.predicates.iter() {
            println!("predicate {}", pred);
        }
        for form in self.formulas.iter() {
            println!("formula {}", form);
        }
        println!("variable count {}", self.state.get_variables());
        println!("clause count {}", self.cla_count);

        let watcher = WatcherPred {
            negated: false,
            predicate: self.predicates[0].clone(),
            variables: vec![0, 1].into_boxed_slice(),
            next: Box::new(WatcherLast),
        };
        println!("{:?}", watcher);
    }
}
