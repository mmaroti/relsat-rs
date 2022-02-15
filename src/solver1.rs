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

use std::fmt;
use std::rc::Rc;

use super::bitops::*;
use super::buffer::Buffer2;
use super::shape::{PositionIter, Shape};

#[derive(Debug, Default)]
struct Step {
    bvar: usize,
    reason: Vec<usize>,
}

#[derive(Debug, Default)]
struct State {
    assignment: Buffer2,
    steps: Vec<Step>,
    levels: Vec<usize>,
}

impl State {
    fn create_table(&mut self, domains: &[Rc<Domain>]) -> Shape {
        let shape = Shape::new(
            domains.iter().map(|d| d.size).collect(),
            self.assignment.len(),
        );
        self.assignment.append(shape.volume(), BOOL_UNDEF);
        shape
    }

    fn print_table(&self, shape: &Shape) {
        let mut cor = vec![0; shape.dimension()];
        for pos in shape.positions() {
            shape.coordinates(pos, &mut cor);
            let val = BOOL_FORMAT[self.assignment.get(pos).idx() as usize];
            println!("  {:?} = {}", cor, val);
        }
    }

    fn assign(&mut self, pos: usize, sign: bool, reason: Vec<usize>) {
        assert!(self.assignment.get(pos) == BOOL_UNDEF);
        self.assignment
            .set(pos, if sign { BOOL_TRUE } else { BOOL_FALSE });
        self.steps.push(Step { bvar: pos, reason });
    }

    fn make_decision(&mut self) -> bool {
        let pos = (0..self.assignment.len()).find(|&i| self.assignment.get(i) == BOOL_UNDEF);
        if let Some(pos) = pos {
            self.levels.push(self.steps.len());
            self.assignment.set(pos, BOOL_TRUE);
            self.steps.push(Step {
                bvar: pos,
                reason: vec![],
            });
            true
        } else {
            false
        }
    }

    fn next_decision(&mut self) -> bool {
        while let Some(level) = self.levels.pop() {
            let val = self.assignment.get(self.steps[level].bvar);
            if val == BOOL_FALSE {
                continue;
            }
            assert!(val == BOOL_TRUE);
            for step in self.steps[level + 1..].iter() {
                assert!(self.assignment.get(step.bvar) != BOOL_UNDEF);
                self.assignment.set(step.bvar, BOOL_UNDEF);
            }
            self.levels.push(level);
            self.assignment.set(self.steps[level].bvar, BOOL_FALSE);
            self.steps.truncate(level + 1);
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
    shape: Shape,
    name: String,
    domains: Vec<Rc<Domain>>,
}

impl Variable {
    fn new(state: &mut State, name: &str, domains: Vec<Rc<Domain>>) -> Self {
        let name = name.to_string();
        let shape = state.create_table(&domains);
        Self {
            name,
            domains,
            shape,
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
struct Literal {
    variable: Rc<Variable>,
    axes: Box<[usize]>,
    positions: PositionIter,
    sign: bool,
}

impl Literal {
    fn new(shape: &Shape, sign: bool, var: &Rc<Variable>, axes: Vec<usize>) -> Self {
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

    fn evaluate(&mut self, state: &State, target: &mut Buffer2) {
        self.positions.reset();
        let op = if self.sign { FOLD_POS } else { FOLD_NEG };
        target.apply(op, &state.assignment, &mut self.positions);
    }

    fn position(&self, coordinates: &[usize]) -> usize {
        self.variable
            .shape
            .position(self.axes.iter().map(|&axis| &coordinates[axis]))
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
struct Clause {
    domains: Vec<Rc<Domain>>,
    literals: Vec<Literal>,
    shape: Shape,
    buffer: Buffer2,
}

impl Clause {
    fn new(shape: Shape, domains: Vec<Rc<Domain>>, literals: Vec<Literal>) -> Self {
        let buffer = Buffer2::new(shape.volume(), EVAL_FALSE);
        Self {
            shape,
            domains,
            literals,
            buffer,
        }
    }

    fn evaluate(&mut self, state: &State) {
        self.buffer.fill(EVAL_FALSE);
        for lit in self.literals.iter_mut() {
            lit.evaluate(state, &mut self.buffer);
        }
    }

    fn get_status(&self) -> Bit2 {
        let mut res = EVAL_TRUE;
        for pos in 0..self.buffer.len() {
            let val = self.buffer.get(pos);
            res = EVAL_AND.of(res, val);
        }
        res
    }

    fn propagate(&mut self, state: &mut State) -> Bit2 {
        let mut coordinates = vec![0; self.shape.dimension()];
        let mut res = EVAL_TRUE;
        for pos in 0..self.buffer.len() {
            let val = self.buffer.get(pos);
            if val == EVAL_UNIT {
                self.shape.coordinates(pos, &mut coordinates);
                let mut unit = 0;
                let mut sign = None;
                let mut reason = vec![];
                for lit in self.literals.iter() {
                    let bvar = lit.position(&coordinates);
                    if state.assignment.get(bvar) == BOOL_UNDEF {
                        assert!(sign.is_none());
                        sign = Some(lit.sign);
                        unit = bvar;
                    } else {
                        reason.push(bvar);
                    }
                }
                if let Some(sign) = sign {
                    state.assign(unit, sign, reason);
                }
            }
            res = EVAL_AND.of(res, val);
        }
        res
    }

    fn get_failure(&self) -> Option<Vec<usize>> {
        for pos in 0..self.buffer.len() {
            if self.buffer.get(pos) == EVAL_FALSE {
                let mut coordinates = vec![0; self.shape.dimension()];
                self.shape.coordinates(pos, &mut coordinates);
                return Some(
                    self.literals
                        .iter()
                        .map(|lit| lit.position(&coordinates))
                        .collect(),
                );
            }
        }
        None
    }

    fn print_table(&self) {
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

#[derive(Debug)]
struct Exist {
    variable: Rc<Variable>,
}

impl Exist {
    fn new(variable: Rc<Variable>) -> Self {
        Exist { variable }
    }

    fn get_status(&self, state: &State) -> Bit2 {
        let shape = &self.variable.shape;
        let range = shape.positions();
        let block = shape.length(shape.dimension() - 1);

        let mut value1 = EVAL_TRUE;
        let mut pos = range.start;
        while pos < range.end {
            let mut value2 = EVAL_FALSE;
            for i in pos..(pos + block) {
                value2 = FOLD_POS.of(value2, state.assignment.get(i));
            }
            value1 = EVAL_AND.of(value1, value2);
            pos += block;
        }
        value1
    }

    fn get_failure(&self, state: &State) -> Option<usize> {
        let shape = &self.variable.shape;
        let range = shape.positions();
        let block = shape.length(shape.dimension() - 1);

        let mut pos = range.start;
        while pos < range.end {
            let mut value2 = EVAL_FALSE;
            for i in pos..(pos + block) {
                value2 = FOLD_POS.of(value2, state.assignment.get(i));
            }
            if value2 == EVAL_FALSE {
                return Some(pos);
            }
            pos += block;
        }
        None
    }
}

impl fmt::Display for Exist {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.variable.fmt(f)
    }
}

#[derive(Debug, Default)]
pub struct Solver {
    state: State,
    domains: Vec<Rc<Domain>>,
    variables: Vec<Rc<Variable>>,
    clauses: Vec<Clause>,
    exists: Vec<Exist>,
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
        let rel = Rc::new(Variable::new(&mut self.state, name, domains));
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

    pub fn add_exist(&mut self, variable: &Rc<Variable>) {
        self.exists.push(Exist::new(variable.clone()));
    }

    pub fn set_value(&mut self, sign: bool, var: &Rc<Variable>, coordinates: &[usize]) {
        let pos = var.shape.position(coordinates.iter());
        self.state.assign(pos, sign, vec![]);
    }

    pub fn set_equality(&mut self, var: &Rc<Variable>) {
        let shape = &var.shape;
        assert!(shape.dimension() == 2);
        for i in 0..shape.length(0) {
            for j in 0..shape.length(1) {
                let pos = shape.position([i, j].iter());
                self.state.assign(pos, i == j, vec![]);
            }
        }
    }

    pub fn get_clauses_status(&self) -> Bit2 {
        let mut res = EVAL_TRUE;
        for cla in self.clauses.iter() {
            res = EVAL_AND.of(res, cla.get_status());
        }
        res
    }

    pub fn get_exists_status(&self) -> Bit2 {
        let mut res = EVAL_TRUE;
        for ext in self.exists.iter() {
            res = EVAL_AND.of(res, ext.get_status(&self.state));
        }
        res
    }

    pub fn evaluate_all(&mut self) {
        for cla in self.clauses.iter_mut() {
            cla.evaluate(&self.state);
        }
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
            cla.evaluate(&self.state);
            let val = cla.propagate(&mut self.state);
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
        assert!(res == self.get_clauses_status());
        res
    }

    pub fn search_all(&mut self) {
        loop {
            let val1 = self.propagate();
            let val2 = self.get_exists_status();

            if val1 == EVAL_FALSE {
                println!("*** LEARNING ***");
                self.evaluate_all();
                self.print();
                self.print_steps();
                println!("*** END OF LEARNING ***");
                break;
            } else if val2 == EVAL_FALSE {
                println!("*** EXISTS ***");
                self.evaluate_all();
                self.print();
                self.print_steps();
                println!();
                let ret = self.state.next_decision();
                if !ret {
                    break;
                }
            } else if val1 == EVAL_TRUE && val2 == EVAL_TRUE {
                println!("*** SOLUTION ***");
                for var in self.variables.iter() {
                    println!("variable {}", var);
                    self.state.print_table(&var.shape);
                }
                println!();
                let ret = self.state.next_decision();
                if !ret {
                    break;
                }
            } else {
                let ret = self.state.make_decision();
                assert!(ret);
            }
        }
    }

    fn lookup_var(&self, bvar: usize) -> &Rc<Variable> {
        for rvar in self.variables.iter() {
            if rvar.shape.positions().contains(&bvar) {
                return rvar;
            }
        }
        panic!();
    }

    fn format_var(&self, bvar: usize) -> String {
        let bval = self.state.assignment.get(bvar);
        assert!(bval == BOOL_FALSE || bval == BOOL_TRUE);

        let rvar = self.lookup_var(bvar);
        let mut coordinates = vec![0; rvar.shape.dimension()];
        rvar.shape.coordinates(bvar, &mut coordinates);

        format!(
            "{}{}{:?}",
            if bval == BOOL_TRUE { '+' } else { '-' },
            rvar.name,
            coordinates,
        )
    }

    fn print_step(&self, step: &Step) {
        let reason: Vec<String> = step
            .reason
            .iter()
            .map(|&bvar| self.format_var(bvar))
            .collect();
        println!("step {} from {:?}", self.format_var(step.bvar), reason);
    }

    pub fn print_steps(&self) {
        for step in self.state.steps.iter() {
            self.print_step(step);
        }
    }

    pub fn print(&self) {
        for dom in self.domains.iter() {
            println!("domain {}", dom);
        }
        for var in self.variables.iter() {
            println!("variable {}", var);
            self.state.print_table(&var.shape);
        }
        for cla in self.clauses.iter() {
            println!("clause {}", cla);
            if let Some(failure) = cla.get_failure() {
                // duh, this is negated
                let failure: Vec<String> = failure
                    .into_iter()
                    .map(|bvar| self.format_var(bvar))
                    .collect();
                println!("failure {:?}", failure);
            }
        }
        for ext in self.exists.iter() {
            // println!("exist {}", ext);
            println!(
                "exists {} = {}",
                ext.variable,
                EVAL_FORMAT2[ext.get_status(&self.state).idx() as usize]
            );
            if let Some(failure) = ext.get_failure(&self.state) {
                println!("failure {:?}", self.format_var(failure));
            }
        }
        println!("steps = {:?}", self.state.steps);
        println!("levels = {:?}", self.state.levels);
        println!(
            "clauses status = {}",
            EVAL_FORMAT2[self.get_clauses_status().idx() as usize]
        );
        println!(
            "exists status = {}",
            EVAL_FORMAT2[self.get_exists_status().idx() as usize]
        );
    }
}
