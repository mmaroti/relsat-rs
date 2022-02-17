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

    fn get_offset(&self, coords: &[usize]) -> usize {
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

#[derive(Debug, Clone, Copy)]
struct LiteralIdx(usize);

#[derive(Debug, Clone, Copy)]
struct Literal<'a> {
    negated: bool,
    predicate: &'a Rc<Predicate>,
    var_offset: usize,
}

impl<'a> Literal<'a> {
    fn new(negated: bool, predicate: &'a Rc<Predicate>, var_offset: usize) -> Self {
        debug_assert!(var_offset < predicate.var_count);
        Self {
            negated,
            predicate,
            var_offset,
        }
    }

    fn idx(&self) -> LiteralIdx {
        debug_assert!(self.var_offset < self.predicate.var_count);
        let idx = self.predicate.var_start + self.var_offset;
        LiteralIdx((idx << 1) + self.negated as usize)
    }
}

impl<'a> fmt::Display for Literal<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut coords = vec![0; self.predicate.arity()];
        self.predicate.get_coords(self.var_offset, &mut coords);
        write!(
            f,
            "{}{}[",
            if self.negated { '-' } else { '+' },
            self.predicate.name
        )?;
        let mut first = true;
        for coord in coords.iter() {
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
    cla_count: usize,
}

impl UniversalFormula {
    fn new<ITER>(disjunction: ITER) -> Self
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
            cla_count,
        }
    }

    fn arity(&self) -> usize {
        self.domains.len()
    }

    fn get_coords(&self, offset: usize, coords: &mut [usize]) {
        get_coords(&self.domains, offset, coords);
    }

    fn get_offset(&self, coords: &[usize]) -> usize {
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

#[derive(Debug, Default)]
pub struct Solver {
    domains: Vec<Rc<Domain>>,
    values: Vec<i8>,
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
        let pred = Rc::new(Predicate::new(name, domains, self.values.len()));
        self.values.resize(self.values.len() + pred.var_count, 0);
        self.predicates.push(pred);
        idx
    }

    pub fn add_formula(&mut self, disjunction: Vec<(bool, PredicateIdx, Vec<usize>)>) {
        let disjunction = disjunction
            .into_iter()
            .map(|(pos, pred, vars)| (!pos, self.predicates[pred.0].clone(), vars));
        let formula = UniversalFormula::new(disjunction);
        self.cla_count += formula.cla_count;
        self.formulas.push(formula);
    }

    fn get_literal(&self, idx: LiteralIdx) -> Literal {
        let negated = idx.0 & 1 != 0;
        let mut var_offset = idx.0 >> 1;
        for predicate in self.predicates.iter() {
            if var_offset < predicate.var_count {
                let lit = Literal::new(negated, predicate, var_offset);
                debug_assert!(lit.idx().0 == idx.0);
                return lit;
            }
            var_offset -= predicate.var_count;
        }
        panic!();
    }

    fn get_value(&self, idx: LiteralIdx) -> i8 {
        let value = self.values[idx.0 >> 1];
        if idx.0 & 1 != 0 {
            -value
        } else {
            value
        }
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
        println!("variable count {}", self.values.len());
        println!("clause count {}", self.cla_count);
        for idx in 0..50 {
            println!("literal {}", self.get_literal(LiteralIdx(idx)));
        }
    }
}
