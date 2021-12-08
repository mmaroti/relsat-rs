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

use super::bitops::*;
use super::buffer::Buffer2;
use super::shape::{Shape, ShapeIter};

#[derive(Debug, Default)]
struct State {
    assignment: Buffer2,
}

#[derive(Debug)]
pub struct Domain {
    name: String,
    size: usize,
}

impl Domain {
    fn new(name: &str, size: usize) -> Self {
        let name = name.to_string();
        Self { name, size }
    }

    fn eq(dom1: &Rc<Domain>, dom2: &Rc<Domain>) -> bool {
        std::ptr::eq(&**dom1, &**dom2)
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
    shape: Shape,
    xbuffer: RefCell<Buffer2>,
    state: Rc<State>,
}

impl Variable {
    fn new(name: &str, domains: Vec<Rc<Domain>>, state: Rc<State>) -> Self {
        let name = name.to_string();
        let shape = Shape::new(domains.iter().map(|d| d.size).collect(), 0);
        let buffer = RefCell::new(Buffer2::new(shape.volume(), BOOL_UNDEF));
        Self {
            name,
            domains,
            shape,
            xbuffer: buffer,
            state,
        }
    }

    pub fn set_equality(&self) {
        assert!(self.shape.dimension() == 2 && self.shape[0] == self.shape[1]);
        let size = self.shape[0];

        let mut buffer = self.xbuffer.borrow_mut();
        buffer.fill(BOOL_FALSE);
        for i in 0..size {
            buffer.set(i * (size + 1), BOOL_TRUE);
        }
    }

    pub fn set_value(&self, indices: &[usize], value: bool) {
        let pos = self.shape.position(indices);
        let mut buffer = self.xbuffer.borrow_mut();
        assert!(buffer.get(pos) == BOOL_UNDEF);
        buffer.set(pos, if value { BOOL_TRUE } else { BOOL_FALSE });
    }

    pub fn print_table(&self) {
        let buffer = self.xbuffer.borrow();
        let mut cor = vec![0; self.shape.dimension()];
        for pos in self.shape.positions() {
            self.shape.coordinates(pos, &mut cor);
            let val = BOOL_FORMAT[buffer.get(pos) as usize];
            println!("  {:?} = {}", cor, val);
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
        write!(f, ") = {}", self.xbuffer.borrow())
    }
}

#[derive(Debug)]
pub struct Literal {
    variable: Rc<Variable>,
    indices: Box<[usize]>,
    positions: RefCell<ShapeIter>,
    sign: bool,
}

impl Literal {
    pub fn new(shape: &Shape, sign: bool, var: &Rc<Variable>, indices: Vec<usize>) -> Self {
        let variable = var.clone();
        let indices = indices.into_boxed_slice();
        let positions = RefCell::new(
            variable
                .shape
                .view()
                .polymer(shape, &indices)
                .simplify()
                .positions(),
        );
        Literal {
            variable,
            indices,
            positions,
            sign,
        }
    }

    pub fn evaluate(&self, target: &mut Buffer2) {
        let source = self.variable.xbuffer.borrow();
        let mut positions = self.positions.borrow_mut();
        positions.reset();
        let op = if self.sign { FOLD_POS } else { FOLD_NEG };
        target.update(op, &*source, &mut *positions);
    }

    pub fn propagate(&self, coordinates: &[usize]) {
        let crd: Vec<usize> = self.indices.iter().map(|&idx| coordinates[idx]).collect();
        let pos = self.variable.shape.position(&crd);
        let mut buffer = self.variable.xbuffer.borrow_mut();
        let val = buffer.get(pos);
        if val == BOOL_UNDEF {
            buffer.set(pos, if self.sign { BOOL_TRUE } else { BOOL_FALSE });
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
    domains: Vec<Rc<Domain>>,
    literals: Vec<Literal>,
    shape: Shape,
    buffer: RefCell<Buffer2>,
}

impl Clause {
    pub fn new(shape: Shape, domains: Vec<Rc<Domain>>, literals: Vec<Literal>) -> Self {
        let buffer = RefCell::new(Buffer2::new(shape.volume(), EVAL_FALSE));
        Self {
            shape,
            domains,
            literals,
            buffer,
        }
    }

    pub fn evaluate(&self) {
        let mut buffer = self.buffer.borrow_mut();
        buffer.fill(EVAL_FALSE);
        for lit in self.literals.iter() {
            lit.evaluate(&mut *buffer);
        }
    }

    pub fn propagate(&self) -> u32 {
        let buffer = self.buffer.borrow();
        let mut result = EVAL_TRUE;
        for pos in 0..buffer.len() {
            let val = buffer.get(pos);
            if val == EVAL_UNIT {
                let mut coordinates = vec![0; self.shape.dimension()];
                self.shape.coordinates(pos, &mut coordinates);
                for lit in self.literals.iter() {
                    lit.propagate(&coordinates);
                }
            }
            result = operation_222(EVAL_AND, result, val);
        }
        result
    }

    pub fn status(&self) -> u32 {
        let buffer = self.buffer.borrow_mut();
        let mut result = EVAL_TRUE;
        for pos in 0..buffer.len() {
            let val = buffer.get(pos);
            result = operation_222(EVAL_AND, result, val);
        }
        result
    }

    pub fn print_table(&self) {
        let buffer = self.buffer.borrow();
        let mut cor = vec![0; self.shape.dimension()];
        for pos in self.shape.positions() {
            self.shape.coordinates(pos, &mut cor);
            let val = EVAL_FORMAT[buffer.get(pos) as usize];
            println!("  {:?} = {}", cor, val);
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

        const TABLE: [&str; 4] = ["false", "unit", "undef", "true"];
        write!(f, " = {}", TABLE[self.status() as usize])
    }
}

#[derive(Debug, Default)]
pub struct Solver {
    state: Rc<State>,
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
        let rel = Rc::new(Variable::new(name, domains, self.state.clone()));
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
                    assert!(Domain::eq(dom1, dom2));
                }
            }
        }
        let domains: Vec<Rc<Domain>> = domains.into_iter().map(|d| d.unwrap()).collect();

        let shape = Shape::new(domains.iter().map(|d| d.size).collect(), 0);
        let literals: Vec<Literal> = literals
            .into_iter()
            .map(|(sign, var, indices)| Literal::new(&shape, sign, var, indices))
            .collect();

        let cla = Clause::new(shape, domains, literals);
        self.clauses.push(cla);
    }

    pub fn evaluate(&self) {
        for cla in self.clauses.iter() {
            cla.evaluate();
        }
    }

    pub fn propagate(&mut self) {
        let mut num = 0;
        let mut idx = 0;
        while num < self.clauses.len() {
            if idx >= self.clauses.len() {
                idx = 0;
            }
            let cla = &self.clauses[idx];
            idx += 1;
            cla.evaluate();
            let val = cla.propagate();
            if val == EVAL_FALSE {
                break;
            } else if val == EVAL_UNIT {
                num = 0;
            } else {
                num += 1;
            }
        }
    }

    pub fn print(&self) {
        for dom in self.domains.iter() {
            println!("domain {}", dom);
        }
        for var in self.variables.iter() {
            println!("variable {}", var);
            var.print_table();
        }
        for cla in self.clauses.iter() {
            cla.evaluate();
            println!("clause {}", cla);
            // cla.print_table();
        }
    }
}
