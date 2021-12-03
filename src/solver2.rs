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

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use super::buffer::Buffer2;
use super::shape::{Shape, View};

#[derive(Debug)]
pub struct Domain {
    name: String,
    size: usize,
}

impl Domain {
    pub fn new(name: &str, size: usize) -> Self {
        let name = name.to_string();
        Self { name, size }
    }
}

impl fmt::Display for Domain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} = {}", self.name, self.size)
    }
}

#[derive(Debug)]
pub struct Variable {
    name: String,
    domains: Vec<Rc<Domain>>,
    pub shape: Shape,
    buffer: RefCell<Buffer2>,
}

impl Variable {
    pub fn new(name: &str, domains: Vec<Rc<Domain>>) -> Self {
        let name = name.to_string();
        let shape = Shape::new(domains.iter().map(|d| d.size).collect());
        let buffer = RefCell::new(Buffer2::new(shape.size(), 0));
        Self {
            name,
            domains,
            shape,
            buffer,
        }
    }
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}(", self.name)?;
        let mut first = true;
        for dom in &self.domains {
            if first {
                first = false;
            } else {
                write!(f, ",")?;
            }
            write!(f, "{}", dom.name)?;
        }
        write!(f, ") = {}", self.buffer.borrow())
    }
}

#[derive(Debug)]
pub struct Literal {
    variable: Rc<Variable>,
    indices: Box<[usize]>,
    view: View,
    sign: bool,
}

impl Literal {
    pub fn new(shape: &Shape, sign: bool, var: &Rc<Variable>, indices: Vec<usize>) -> Self {
        let variable = var.clone();
        let indices = indices.into_boxed_slice();
        let view = variable.shape.view().polymer(shape, &indices);
        Literal {
            variable,
            indices,
            view,
            sign,
        }
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}{}(",
            if self.sign { '+' } else { '-' },
            self.variable.name
        )?;
        let mut first = true;
        for &idx in self.indices.iter() {
            if first {
                first = false;
            } else {
                write!(f, ",")?;
            }
            write!(f, "x{}", idx)?;
        }
        write!(f, ")")
    }
}

#[derive(Debug)]
pub struct Clause {
    shape: Shape,
    domains: Vec<Rc<Domain>>,
    literals: Vec<Literal>,
}

impl Clause {
    pub fn new(shape: Shape, domains: Vec<Rc<Domain>>, literals: Vec<Literal>) -> Self {
        Self {
            shape,
            domains,
            literals,
        }
    }
}

impl fmt::Display for Clause {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut first = true;
        for lit in self.literals.iter() {
            if first {
                first = false;
            } else {
                write!(f, " ")?;
            }
            write!(f, "{}", lit)?;
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct Solver {
    domains: Vec<Rc<Domain>>,
    variables: Vec<Rc<Variable>>,
    clauses: Vec<Clause>,
}

impl Solver {
    pub fn add_domain(&mut self, name: &str, size: usize) -> Rc<Domain> {
        assert!(self.domains.iter().all(|dom| dom.name != name));
        let dom = Rc::new(Domain::new(name, size));
        self.domains.push(dom.clone());
        dom
    }

    pub fn add_variable(&mut self, name: &str, domains: Vec<&Rc<Domain>>) -> Rc<Variable> {
        assert!(self.variables.iter().all(|rel| rel.name != name));
        let domains = domains.into_iter().cloned().collect();
        let rel = Rc::new(Variable::new(name, domains));
        self.variables.push(rel.clone());
        rel
    }

    pub fn add_clause(&mut self, literals: Vec<(bool, &Rc<Variable>, Vec<usize>)>) {
        let mut domains: Vec<Option<Rc<Domain>>> = Default::default();
        for (_, var, indices) in literals.iter() {
            assert_eq!(var.domains.len(), indices.len());
            for (pos, &idx) in indices.iter().enumerate() {
                if domains.len() <= idx {
                    domains.resize(idx + 1, None);
                }
                let dom1 = &var.domains[pos];
                let dom2 = &mut domains[idx];
                if dom2.is_none() {
                    *dom2 = Some(dom1.clone());
                } else {
                    let dom2 = dom2.as_ref().unwrap();
                    assert!(std::ptr::eq(&**dom1, &**dom2));
                }
            }
        }
        let domains: Vec<Rc<Domain>> = domains.into_iter().map(|d| d.unwrap()).collect();

        let shape = Shape::new(domains.iter().map(|d| d.size).collect());
        let literals: Vec<Literal> = literals
            .into_iter()
            .map(|(sign, var, indices)| Literal::new(&shape, sign, var, indices))
            .collect();

        let cla = Clause::new(shape, domains, literals);
        self.clauses.push(cla);
    }

    pub fn print(&self) {
        for dom in self.domains.iter() {
            println!("domain {}", dom);
        }
        for rel in self.variables.iter() {
            println!("variable {}", rel);
        }
        for cla in self.clauses.iter() {
            println!("clause {}", cla);
        }
    }
}
