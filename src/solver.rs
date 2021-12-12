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
    trail: Vec<usize>,
    levels: Vec<usize>,
}

impl State {
    pub fn create_table(&mut self, domains: &[Rc<Domain>]) -> Shape {
        let shape = Shape::new(
            domains.iter().map(|d| d.size).collect(),
            self.assignment.len(),
        );
        self.assignment.append(shape.volume(), BOOL_UNDEF);
        shape
    }

    pub fn print_table(&self, shape: &Shape) {
        let mut cor = vec![0; shape.dimension()];
        for pos in shape.positions() {
            shape.coordinates(pos, &mut cor);
            let val = BOOL_FORMAT[self.assignment.get(pos).idx() as usize];
            println!("  {:?} = {}", cor, val);
        }
    }

    pub fn set_value(&mut self, pos: usize, sign: bool) {
        assert!(self.levels.is_empty());
        assert!(self.assignment.get(pos) == BOOL_UNDEF);
        self.assignment
            .set(pos, if sign { BOOL_TRUE } else { BOOL_FALSE });
        self.trail.push(pos);
    }

    pub fn propagate(&mut self, pos: usize, sign: bool) {
        let old = self.assignment.get(pos);
        if old == BOOL_UNDEF {
            self.assignment
                .set(pos, if sign { BOOL_TRUE } else { BOOL_FALSE });
            self.trail.push(pos);
        }
    }

    pub fn make_decision(&mut self) -> bool {
        let pos = (0..self.assignment.len()).find(|&i| self.assignment.get(i) == BOOL_UNDEF);
        if let Some(pos) = pos {
            self.levels.push(self.trail.len());
            self.assignment.set(pos, BOOL_TRUE);
            self.trail.push(pos);
            println!("make trail={:?} levels={:?}", self.trail, self.levels);
            true
        } else {
            false
        }
    }

    pub fn next_decision(&mut self) -> bool {
        while let Some(start) = self.levels.pop() {
            let val = self.assignment.get(self.trail[start]);
            if val == BOOL_FALSE {
                continue;
            }
            assert!(val == BOOL_TRUE);
            for &pos in self.trail[start + 1..].iter() {
                assert!(self.assignment.get(pos) != BOOL_UNDEF);
                self.assignment.set(pos, BOOL_UNDEF);
            }
            self.levels.push(start);
            self.assignment.set(self.trail[start], BOOL_FALSE);
            self.trail.truncate(start + 1);
            println!("next trail={:?} levels={:?}", self.trail, self.levels);
            return true;
        }
        false
    }
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
    state: Rc<RefCell<State>>,
    shape: Shape,
    name: String,
    domains: Vec<Rc<Domain>>,
}

impl Variable {
    fn new(name: &str, domains: Vec<Rc<Domain>>, state: Rc<RefCell<State>>) -> Self {
        let name = name.to_string();
        let shape = state.borrow_mut().create_table(&domains);
        Self {
            name,
            domains,
            shape,
            state,
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
        write!(f, ")")
    }
}

#[derive(Debug)]
pub struct Literal {
    variable: Rc<Variable>,
    axes: Box<[usize]>,
    positions: ShapeIter,
    sign: bool,
}

impl Literal {
    pub fn new(shape: &Shape, sign: bool, var: &Rc<Variable>, axes: Vec<usize>) -> Self {
        let variable = var.clone();
        let axes = axes.into_boxed_slice();
        let positions = variable
            .shape
            .view()
            .polymer(shape, &axes)
            .simplify()
            .positions();
        Literal {
            variable,
            axes,
            positions,
            sign,
        }
    }

    pub fn evaluate(&mut self, target: &mut Buffer2) {
        let source = &self.variable.state.borrow().assignment;
        self.positions.reset();
        let op = if self.sign { FOLD_POS } else { FOLD_NEG };
        target.update(op, source, &mut self.positions);
    }

    pub fn propagate(&self, coordinates: &[usize]) {
        let coordinates: Vec<usize> = self.axes.iter().map(|&axis| coordinates[axis]).collect();
        let pos = self.variable.shape.position(&coordinates);
        self.variable.state.borrow_mut().propagate(pos, self.sign);
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
        for &idx in self.axes.iter() {
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
    buffer: Buffer2,
}

impl Clause {
    pub fn new(shape: Shape, domains: Vec<Rc<Domain>>, literals: Vec<Literal>) -> Self {
        let buffer = Buffer2::new(shape.volume(), EVAL_FALSE);
        Self {
            shape,
            domains,
            literals,
            buffer,
        }
    }

    pub fn evaluate(&mut self) {
        self.buffer.fill(EVAL_FALSE);
        for lit in self.literals.iter_mut() {
            lit.evaluate(&mut self.buffer);
        }
    }

    pub fn get_status(&self) -> Bit2 {
        let mut res = EVAL_TRUE;
        for pos in 0..self.buffer.len() {
            let val = self.buffer.get(pos);
            res = EVAL_AND.of(res, val);
        }
        res
    }

    pub fn propagate(&mut self) -> Bit2 {
        let mut res = EVAL_TRUE;
        for pos in 0..self.buffer.len() {
            let val = self.buffer.get(pos);
            if val == EVAL_UNIT {
                let mut coordinates = vec![0; self.shape.dimension()];
                self.shape.coordinates(pos, &mut coordinates);
                for lit in self.literals.iter() {
                    lit.propagate(&coordinates);
                }
            }
            res = EVAL_AND.of(res, val);
        }
        res
    }

    pub fn print_table(&self) {
        let mut cor = vec![0; self.shape.dimension()];
        for pos in self.shape.positions() {
            self.shape.coordinates(pos, &mut cor);
            let val = EVAL_FORMAT1[self.buffer.get(pos).idx() as usize];
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

        write!(f, " = {}", EVAL_FORMAT2[self.get_status().idx() as usize])
    }
}

#[derive(Debug, Default)]
pub struct Solver {
    state: Rc<RefCell<State>>,
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

    pub fn set_value(&mut self, var: &Rc<Variable>, coordinates: &[usize], sign: bool) {
        let pos = var.shape.position(coordinates);
        self.state.borrow_mut().set_value(pos, sign);
    }

    pub fn set_equality(&mut self, var: &Rc<Variable>) {
        let mut state = self.state.borrow_mut();
        let shape = &var.shape;
        assert!(shape.dimension() == 2);
        for i in 0..shape.length(0) {
            for j in 0..shape.length(1) {
                let pos = shape.position(&[i, j]);
                state.set_value(pos, i == j);
            }
        }
    }

    pub fn get_status(&self) -> Bit2 {
        let mut res = EVAL_TRUE;
        for cla in self.clauses.iter() {
            res = EVAL_AND.of(res, cla.get_status());
        }
        res
    }

    pub fn propagate(&mut self) -> Bit2 {
        let mut num = 0;
        let mut res = EVAL_TRUE;
        let mut idx = 0;
        while num < self.clauses.len() {
            if idx >= self.clauses.len() {
                idx = 0;
            }
            let cla = &mut self.clauses[idx];
            idx += 1;
            cla.evaluate();
            let val = cla.propagate();
            if val == EVAL_FALSE {
                res = EVAL_FALSE;
                break;
            } else if val == EVAL_UNIT {
                res = EVAL_TRUE;
                num = 0;
            } else {
                res = EVAL_AND.of(res, val);
                num += 1;
            }
        }
        assert!(res != EVAL_UNIT);
        assert!(res == self.get_status());
        res
    }

    pub fn search_all(&mut self) {
        loop {
            let val = self.propagate();
            // self.print();
            if val == EVAL_FALSE {
                self.print();
                println!("*** CONTRADICTION ***");
                break;
            } else if val == EVAL_TRUE {
                let mut state = self.state.borrow_mut();
                println!("solution");
                for var in self.variables.iter() {
                    println!("variable {}", var);
                    state.print_table(&var.shape);
                }
                println!("");
                let ret = state.next_decision();
                if !ret {
                    break;
                }
            } else {
                let mut state = self.state.borrow_mut();
                let ret = state.make_decision();
                assert!(ret);
            }
        }
    }

    pub fn print(&self) {
        for dom in self.domains.iter() {
            println!("domain {}", dom);
        }
        let state = self.state.borrow();
        for var in self.variables.iter() {
            println!("variable {}", var);
            state.print_table(&var.shape);
        }
        for cla in self.clauses.iter() {
            println!("clause {}", cla);
            // cla.print_table();
        }
        println!("trail = {:?}", self.state.borrow().trail);
        println!("levels = {:?}", self.state.borrow().levels);
        println!(
            "status = {}",
            EVAL_FORMAT2[self.get_status().idx() as usize]
        )
    }
}
