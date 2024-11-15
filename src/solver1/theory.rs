/*
* Copyright (C) 2019-2024, Miklos Maroti
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
}

impl std::fmt::Display for Domain {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "domain {}", self.name)
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
}

impl std::fmt::Display for Predicate {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "predicate {}(", self.name)?;
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
pub struct Literal {
    sign: bool,
    predicate: Rc<Predicate>,
    variables: Box<[usize]>,
}

impl Literal {
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

impl std::fmt::Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
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
pub struct Clause {
    domains: Box<[Rc<Domain>]>,
    literals: Box<[Literal]>,
}

impl Clause {
    pub fn new(literals: Vec<Literal>) -> Self {
        let literals = literals.into_boxed_slice();
        let mut domains: Vec<Option<Rc<Domain>>> = Default::default();

        for lit in literals.iter() {
            for (dom1, &var) in lit.domains().iter().zip(lit.variables().iter()) {
                if domains.len() <= var {
                    domains.resize(var + 1, None);
                }
                let dom2 = &mut domains[var];
                if let Some(dom2) = dom2 {
                    assert!(Rc::ptr_eq(dom1, dom2));
                } else {
                    *dom2 = Some(dom1.clone());
                }
            }
        }

        let domains: Vec<Rc<Domain>> = domains.into_iter().map(|v| v.unwrap()).collect();
        let domains = domains.into_boxed_slice();

        Self { domains, literals }
    }

    pub fn domains(&self) -> &[Rc<Domain>] {
        &self.domains
    }

    pub fn literals(&self) -> &[Literal] {
        &self.literals
    }
}

impl std::fmt::Display for Clause {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "clause ")?;
        for (idx, frm) in self.literals.iter().enumerate() {
            if idx != 0 {
                write!(f, " ")?;
            }
            write!(f, "{}", frm)?;
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct Theory {
    domains: Vec<Rc<Domain>>,
    predicates: Vec<Rc<Predicate>>,
    clauses: Vec<Rc<Clause>>,
}

impl Theory {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_domain(&mut self, domain: Rc<Domain>) {
        assert!(self.domains.iter().all(|dom| dom.name != domain.name));
        self.domains.push(domain);
    }

    fn has_domain(&self, domain: &Rc<Domain>) -> bool {
        self.domains.iter().any(|dom| Rc::ptr_eq(dom, domain))
    }

    pub fn add_predicate(&mut self, predicate: Rc<Predicate>) {
        assert!(self.predicates.iter().all(|prd| prd.name != predicate.name));
        assert!(predicate.domains.iter().all(|dom| self.has_domain(dom)));
        self.predicates.push(predicate);
    }

    fn has_predicate(&self, predicate: &Rc<Predicate>) -> bool {
        self.predicates.iter().any(|prd| Rc::ptr_eq(prd, predicate))
    }

    pub fn add_clause(&mut self, clause: Rc<Clause>) {
        assert!(clause.domains.iter().all(|dom| self.has_domain(dom)));
        assert!(clause
            .literals
            .iter()
            .all(|lit| self.has_predicate(lit.predicate())));
        self.clauses.push(clause);
    }
}

impl std::fmt::Display for Theory {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for dom in self.domains.iter() {
            writeln!(f, "{}", dom)?;
        }
        for prd in self.predicates.iter() {
            writeln!(f, "{}", prd)?;
        }
        for cla in self.clauses.iter() {
            writeln!(f, "{}", cla)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut thy = Theory::new();

        let set = Rc::new(Domain::new("set".into()));
        thy.add_domain(set.clone());

        let equ = Rc::new(Predicate::new("equ".into(), vec![set.clone(), set.clone()]));
        thy.add_predicate(equ.clone());

        thy.add_clause(Rc::new(Clause::new(vec![Literal::new(
            true,
            equ.clone(),
            vec![0, 0],
        )])));

        thy.add_clause(Rc::new(Clause::new(vec![
            Literal::new(false, equ.clone(), vec![0, 1]),
            Literal::new(true, equ.clone(), vec![1, 0]),
        ])));

        thy.add_clause(Rc::new(Clause::new(vec![
            Literal::new(false, equ.clone(), vec![0, 1]),
            Literal::new(false, equ.clone(), vec![1, 2]),
            Literal::new(true, equ.clone(), vec![0, 2]),
        ])));

        println!("{}", thy);
    }
}
