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
use std::{fmt, ptr};

struct SolverItem<'a, ITEM: ?Sized>(&'a Solver, &'a ITEM);

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

fn get_offset(domains: &[Rc<Domain>], coords: &[usize]) -> usize {
    debug_assert_eq!(domains.len(), coords.len());
    let mut offset = 0;
    for (size, &coord) in domains.iter().map(|dom| dom.size()).zip(coords.iter()) {
        debug_assert!(coord < size);
        offset *= size;
        offset += coord;
    }
    offset
}

#[derive(Debug, Default, Clone)]
struct BooleanVariable {}

#[derive(Debug)]
struct Predicate {
    name: String,
    domains: Box<[Rc<Domain>]>,
    first_variable: usize,
    num_variables: usize,
}

impl Predicate {
    fn new(solver: &Solver, name: String, domains: Vec<Rc<Domain>>) -> Self {
        let domains = domains.into_boxed_slice();
        let first_variable = solver.num_variables();
        let num_variables = domains.iter().map(|dom| dom.size).product();
        Self {
            name,
            domains,
            first_variable,
            num_variables,
        }
    }

    fn arity(&self) -> usize {
        self.domains.len()
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

#[derive(Debug)]
struct AtomicFormula {
    sign: bool,
    predicate: Rc<Predicate>,
    variables: Box<[usize]>,
}

impl AtomicFormula {
    fn new(sign: bool, predicate: Rc<Predicate>, variables: Vec<usize>) -> Self {
        let variables = variables.into_boxed_slice();
        assert_eq!(predicate.arity(), variables.len());

        Self {
            sign,
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
            if self.sign { '+' } else { '-' },
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
}

impl UniversalFormula {
    fn new(disjunction: Vec<(bool, Rc<Predicate>, Vec<usize>)>) -> Self {
        let mut domains: Vec<Option<Rc<Domain>>> = Default::default();
        let disjunction: Vec<AtomicFormula> = disjunction
            .into_iter()
            .map(|(sign, pred, vars)| {
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
                AtomicFormula::new(sign, pred, vars)
            })
            .collect();

        let domains: Vec<Rc<Domain>> = domains.into_iter().map(|d| d.unwrap()).collect();
        Self {
            domains: domains.into_boxed_slice(),
            disjunction: disjunction.into_boxed_slice(),
        }
    }

    fn arity(&self) -> usize {
        self.domains.len()
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

#[derive(Debug, Default)]
pub struct Solver {
    domains: Vec<Rc<Domain>>,
    variables: Vec<BooleanVariable>,
    predicates: Vec<Rc<Predicate>>,
    formulas: Vec<UniversalFormula>,
}

#[derive(Debug, Clone, Copy)]
pub struct DomainIdx(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PredicateIdx(usize);

impl Solver {
    pub fn add_domain(&mut self, name: String, size: usize) -> DomainIdx {
        let idx = DomainIdx(self.domains.len());
        self.domains.push(Rc::new(Domain::new(name, size)));
        idx
    }

    pub fn num_variables(&self) -> usize {
        self.variables.len()
    }

    pub fn add_predicate(&mut self, name: String, domains: Vec<DomainIdx>) -> PredicateIdx {
        let domains: Vec<Rc<Domain>> = domains
            .into_iter()
            .map(|idx| self.domains[idx.0].clone())
            .collect();
        let idx = PredicateIdx(self.predicates.len());
        let pred = Rc::new(Predicate::new(self, name, domains));
        self.variables.resize(
            self.variables.len() + pred.num_variables,
            Default::default(),
        );
        self.predicates.push(pred);
        idx
    }

    pub fn add_formula(&mut self, disjunction: Vec<(bool, PredicateIdx, Vec<usize>)>) {
        let disjunction: Vec<(bool, Rc<Predicate>, Vec<usize>)> = disjunction
            .into_iter()
            .map(|(sign, pred, vars)| (sign, self.predicates[pred.0].clone(), vars))
            .collect();
        let formula = UniversalFormula::new(disjunction);
        self.formulas.push(formula);
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
    }
}
