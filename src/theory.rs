/*
* Copyright (C) 2019-2022, Miklos Maroti
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

#[derive(Debug)]
pub struct Domain {
    name: String,
}

impl Domain {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        ptr::eq(self, other)
    }
}

impl fmt::Display for Domain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug)]
pub struct Predicate {
    name: String,
    domains: Box<[Rc<Domain>]>,
}

impl Predicate {
    pub fn new(name: String, domains: Vec<Rc<Domain>>) -> Self {
        let domains = domains.into_boxed_slice();
        Self { name, domains }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn arity(&self) -> usize {
        self.domains.len()
    }

    pub fn domains(&self) -> &[Rc<Domain>] {
        &self.domains
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        ptr::eq(self, other)
    }
}

impl fmt::Display for Predicate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}(", self.name)?;
        for (idx, dom) in self.domains.iter().enumerate() {
            if idx != 0 {
                write!(f, ",")?;
            }
            write!(f, "{}", dom.name)?;
        }
        write!(f, ")")
    }
}

#[derive(Debug)]
pub struct AtomicFormula {
    sign: bool,
    predicate: Rc<Predicate>,
    variables: Box<[usize]>,
}

impl AtomicFormula {
    pub fn new(sign: bool, predicate: Rc<Predicate>, variables: Vec<usize>) -> Self {
        assert_eq!(predicate.arity(), variables.len());
        let variables = variables.into_boxed_slice();
        Self {
            sign,
            predicate,
            variables,
        }
    }

    pub fn sign(&self) -> bool {
        self.sign
    }

    pub fn predicate(&self) -> &Rc<Predicate> {
        &self.predicate
    }

    pub fn arity(&self) -> usize {
        self.variables.len()
    }

    pub fn domains(&self) -> &[Rc<Domain>] {
        self.predicate.domains()
    }

    pub fn variables(&self) -> &[usize] {
        &self.variables
    }
}

impl fmt::Display for AtomicFormula {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", if self.sign { '+' } else { '-' })?;
        write!(f, "{}(", self.predicate.name)?;
        for (idx, &var) in self.variables.iter().enumerate() {
            if idx != 0 {
                write!(f, ",")?;
            }
            write!(f, "x{}", var)?;
        }
        write!(f, ")")
    }
}

#[derive(Debug)]
pub struct UniversalFormula {
    variables: Box<[Rc<Domain>]>,
    disjunction: Box<[AtomicFormula]>,
}

impl UniversalFormula {
    pub fn new(disjunction: Vec<AtomicFormula>) -> Self {
        let disjunction = disjunction.into_boxed_slice();
        let mut variables: Vec<Option<Rc<Domain>>> = Default::default();

        for fml in disjunction.iter() {
            for (dom1, &var) in fml.domains().iter().zip(fml.variables().iter()) {
                if variables.len() <= var {
                    variables.resize(var + 1, None);
                }
                let dom2 = &mut variables[var];
                if let Some(dom2) = dom2 {
                    assert!(dom1.ptr_eq(dom2));
                } else {
                    *dom2 = Some(dom1.clone());
                }
            }
        }

        let variables: Vec<Rc<Domain>> = variables.into_iter().map(|v| v.unwrap()).collect();
        let variables = variables.into_boxed_slice();

        Self {
            variables,
            disjunction,
        }
    }

    pub fn variables(&self) -> &[Rc<Domain>] {
        &self.variables
    }

    pub fn disjunction(&self) -> &[AtomicFormula] {
        &self.disjunction
    }
}

impl fmt::Display for UniversalFormula {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (idx, frm) in self.disjunction.iter().enumerate() {
            if idx != 0 {
                write!(f, " | ")?;
            }
            write!(f, "{}", frm)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Theory {
    domains: Vec<Rc<Domain>>,
    predicates: Vec<Rc<Predicate>>,
    formulas: Vec<Rc<UniversalFormula>>,
}

impl Theory {
    pub fn new() -> Self {
        Self {
            domains: Default::default(),
            predicates: Default::default(),
            formulas: Default::default(),
        }
    }

    pub fn add_domain(&mut self, domain: Rc<Domain>) {
        assert!(self.domains.iter().all(|dom| !dom.ptr_eq(&domain)));
        self.domains.push(domain);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let set = Rc::new(Domain::new("set".into()));
        let equ = Rc::new(Predicate::new("equ".into(), vec![set.clone(), set.clone()]));

        let equ_reflexive = Rc::new(UniversalFormula::new(vec![AtomicFormula::new(
            true,
            equ.clone(),
            vec![0, 0],
        )]));

        let equ_symmetric = Rc::new(UniversalFormula::new(vec![
            AtomicFormula::new(false, equ.clone(), vec![0, 1]),
            AtomicFormula::new(true, equ.clone(), vec![1, 0]),
        ]));

        let equ_transitive = Rc::new(UniversalFormula::new(vec![
            AtomicFormula::new(false, equ.clone(), vec![0, 1]),
            AtomicFormula::new(false, equ.clone(), vec![1, 2]),
            AtomicFormula::new(true, equ.clone(), vec![0, 2]),
        ]));

        println!("{}", set);
        println!("{}", equ);
        println!("{}", equ_reflexive);
        println!("{}", equ_symmetric);
        println!("{}", equ_transitive);
    }
}
