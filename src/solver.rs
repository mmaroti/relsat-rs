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
use super::theory::{Clause, Literal, Theory, Variable};

#[derive(Debug)]
pub struct Relation {
    variable: Rc<Variable>,
    buffer: RefCell<Buffer2>,
    shape: Shape,
}

impl Relation {
    fn new(variable: Rc<Variable>, size: usize) -> Self {
        let shape = Shape::new(vec![size; variable.arity]);
        let buffer = RefCell::new(Buffer2::new(shape.size(), 2));
        Self {
            variable,
            buffer,
            shape,
        }
    }

    fn size(&self) -> usize {
        if self.shape.rank() > 0 {
            self.shape[0]
        } else {
            1
        }
    }

    pub fn set_equ(&self) {
        assert!(self.shape.rank() == 2);
        let size = self.shape[0];
        let mut buffer = self.buffer.borrow_mut();
        for i in 0..size {
            for j in 0..size {
                let pos = self.shape.position(&[i, j]);
                buffer.set(pos, if i == j { 1 } else { 0 })
            }
        }
    }
}

impl fmt::Display for Relation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} \"", self.variable)?;
        let size = self.size();
        let buffer = self.buffer.borrow();
        const FORMAT: [char; 4] = ['0', '1', '?', 'X'];
        for idx in 0..buffer.len() {
            if idx > 0 && idx % size == 0 {
                write!(f, " ")?;
            }
            let val = buffer.get(idx);
            write!(f, "{}", FORMAT[val as usize])?;
        }
        write!(f, "\"")
    }
}

#[derive(Debug)]
pub struct Polymer {
    relation: Rc<Relation>,
    literal: Rc<Literal>,
    view: View,
}

#[derive(Debug)]
pub struct Constraint {
    polymers: Vec<Rc<Polymer>>,
}

impl Constraint {
    pub fn new(_clause: &Clause) -> Self {
        Self {
            polymers: Default::default(),
        }
    }
}

#[derive(Debug)]
pub struct Solver {
    theory: Theory,
    size: usize,
    relations: Vec<Rc<Relation>>,
    constraints: Vec<Constraint>,
}

impl Solver {
    pub fn new(theory: Theory, size: usize) -> Self {
        let relations = theory
            .variables
            .iter()
            .map(|var| Rc::new(Relation::new(var.clone(), size)))
            .collect();
        let constraints = theory
            .clauses
            .iter()
            .map(|cla| Constraint::new(cla))
            .collect();
        Self {
            theory,
            size,
            relations,
            constraints,
        }
    }

    pub fn get_relation(&self, var: &Variable) -> Option<Rc<Relation>> {
        for rel in self.relations.iter() {
            if std::ptr::eq(&*rel.variable, var) {
                return Some(rel.clone());
            }
        }
        None
    }

    pub fn print(&self) {
        for rel in self.relations.iter() {
            println!("relation: {}", rel);
        }
    }
}
