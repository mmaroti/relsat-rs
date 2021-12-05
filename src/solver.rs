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
use super::shape::{Shape, ShapeIter};

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

    pub fn eq(dom1: &Rc<Domain>, dom2: &Rc<Domain>) -> bool {
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
    buffer: RefCell<Buffer2>,
}

impl Variable {
    const UNDEF: u32 = 0;
    const FALSE: u32 = 1;
    const TRUE: u32 = 2;

    pub fn new(name: &str, domains: Vec<Rc<Domain>>) -> Self {
        let name = name.to_string();
        let shape = Shape::new(domains.iter().map(|d| d.size).collect());
        let buffer = RefCell::new(Buffer2::new(shape.size(), Variable::UNDEF));
        Self {
            name,
            domains,
            shape,
            buffer,
        }
    }

    pub fn set_equality(&self) {
        assert!(self.shape.rank() == 2 && self.shape[0] == self.shape[1]);
        let size = self.shape[0];

        let mut buffer = self.buffer.borrow_mut();
        buffer.fill(Variable::FALSE);
        for i in 0..size {
            buffer.set(i * (size + 1), Variable::TRUE);
        }
    }

    pub fn set_value(&self, indices: &[usize], value: bool) {
        let pos = self.shape.position(indices);
        let mut buffer = self.buffer.borrow_mut();
        buffer.set(
            pos,
            if value {
                Variable::TRUE
            } else {
                Variable::FALSE
            },
        );
    }

    pub fn print_table(&self) {
        
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

    const PATTERN_POS: u32 = Buffer2::pattern(&[
        (Clause::FALSE, Variable::UNDEF, Clause::UNIT1),
        (Clause::FALSE, Variable::FALSE, Clause::FALSE),
        (Clause::FALSE, Variable::TRUE, Clause::TRUE),
        (Clause::UNIT1, Variable::UNDEF, Clause::UNIT2),
        (Clause::UNIT1, Variable::FALSE, Clause::UNIT1),
        (Clause::UNIT1, Variable::TRUE, Clause::TRUE),
        (Clause::UNIT2, Variable::UNDEF, Clause::UNIT2),
        (Clause::UNIT2, Variable::FALSE, Clause::UNIT2),
        (Clause::UNIT2, Variable::TRUE, Clause::TRUE),
        (Clause::TRUE, Variable::UNDEF, Clause::TRUE),
        (Clause::TRUE, Variable::FALSE, Clause::TRUE),
        (Clause::TRUE, Variable::TRUE, Clause::TRUE),
    ]);

    const PATTERN_NEG: u32 = Buffer2::pattern(&[
        (Clause::FALSE, Variable::UNDEF, Clause::UNIT1),
        (Clause::FALSE, Variable::TRUE, Clause::FALSE),
        (Clause::FALSE, Variable::FALSE, Clause::TRUE),
        (Clause::UNIT1, Variable::UNDEF, Clause::UNIT2),
        (Clause::UNIT1, Variable::TRUE, Clause::UNIT1),
        (Clause::UNIT1, Variable::FALSE, Clause::TRUE),
        (Clause::UNIT2, Variable::UNDEF, Clause::UNIT2),
        (Clause::UNIT2, Variable::TRUE, Clause::UNIT2),
        (Clause::UNIT2, Variable::FALSE, Clause::TRUE),
        (Clause::TRUE, Variable::UNDEF, Clause::TRUE),
        (Clause::TRUE, Variable::TRUE, Clause::TRUE),
        (Clause::TRUE, Variable::FALSE, Clause::TRUE),
    ]);

    pub fn evaluate(&self, target: &mut Buffer2) {
        let source = self.variable.buffer.borrow();
        let mut positions = self.positions.borrow_mut();
        positions.reset();
        let pattern = if self.sign {
            Literal::PATTERN_POS
        } else {
            Literal::PATTERN_NEG
        };
        target.update(pattern, &*source, &mut *positions);
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
    const FALSE: u32 = 0; // all false
    const UNIT1: u32 = 1; // exactly one undef
    const UNIT2: u32 = 2; // two or more undef
    const TRUE: u32 = 3; // at least one true

    pub fn new(shape: Shape, domains: Vec<Rc<Domain>>, literals: Vec<Literal>) -> Self {
        let buffer = RefCell::new(Buffer2::new(shape.size(), Clause::FALSE));
        Self {
            shape,
            domains,
            literals,
            buffer,
        }
    }

    pub fn evaluate(&self) {
        let mut buffer = self.buffer.borrow_mut();
        buffer.fill(Clause::FALSE);
        for lit in self.literals.iter() {
            lit.evaluate(&mut *buffer);
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
        write!(f, " = {}", self.buffer.borrow())
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
                    assert!(Domain::eq(dom1, dom2));
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

    pub fn evaluate(&self) {
        for cla in self.clauses.iter() {
            cla.evaluate();
        }
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
