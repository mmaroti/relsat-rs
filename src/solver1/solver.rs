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

use super::bitops::*;
use super::buffer::Buffer2;
use super::shape::{PositionIter, Shape};

#[derive(Debug)]
enum Reason {
    Initial,
    Decision,
    Clause(Vec<usize>),
    Exists,
}

#[derive(Debug)]
struct Step {
    bvar: usize,
    reason: Reason,
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
            domains.iter().map(|dom| dom.size).collect(),
            self.assignment.len(),
        );
        self.assignment.append(shape.volume(), BOOL_UNDEF1);
        shape
    }

    fn print_table(&self, shape: &Shape) {
        let mut cor = vec![0; shape.dimension()];
        for pos in shape.positions() {
            shape.coordinates(pos, &mut cor);
            let val = BOOL_FORMAT1[self.assignment.get(pos).idx()];
            println!("  {:?} = {}", cor, val);
        }
    }

    fn assign(&mut self, pos: usize, sign: bool, reason: Reason) {
        assert!(self.assignment.get(pos) == BOOL_UNDEF1);
        self.assignment
            .set(pos, if sign { BOOL_TRUE } else { BOOL_FALSE });
        self.steps.push(Step { bvar: pos, reason });
    }

    fn make_decision(&mut self) -> bool {
        let pos = (0..self.assignment.len()).find(|&i| self.assignment.get(i) == BOOL_UNDEF1);
        if let Some(pos) = pos {
            self.levels.push(self.steps.len());
            self.assignment.set(pos, BOOL_TRUE);
            self.steps.push(Step {
                bvar: pos,
                reason: Reason::Decision,
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
                assert!(self.assignment.get(step.bvar) != BOOL_UNDEF1);
                self.assignment.set(step.bvar, BOOL_UNDEF1);
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
    fn new(name: String, size: usize) -> Self {
        Self { name, size }
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

impl std::fmt::Display for Domain {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "domain {} = {}", self.name, self.size)
    }
}

#[derive(Debug)]
pub struct Predicate {
    shape: Shape,
    name: String,
    domains: Box<[Rc<Domain>]>,
}

impl Predicate {
    fn new(state: &mut State, name: String, domains: Vec<Rc<Domain>>) -> Self {
        let shape = state.create_table(&domains);
        let domains = domains.into_boxed_slice();
        Self {
            name,
            domains,
            shape,
        }
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
struct Literal {
    predicate: Rc<Predicate>,
    variables: Box<[usize]>,
    positions: PositionIter,
    sign: bool,
}

impl Literal {
    fn new(shape: &Shape, sign: bool, predicate: Rc<Predicate>, variables: Vec<usize>) -> Self {
        let variables = variables.into_boxed_slice();
        let positions = predicate
            .shape
            .view()
            .polymer(shape, &variables)
            .simplify()
            .positions();
        Literal {
            predicate,
            variables,
            positions,
            sign,
        }
    }

    fn evaluate(&mut self, state: &State, target: &mut Buffer2) {
        self.positions.reset();
        let op = if self.sign { BOOL_OR } else { BOOL_ORNOT };
        target.apply(op, &state.assignment, &mut self.positions);
    }

    fn position(&self, coordinates: &[usize]) -> usize {
        self.predicate
            .shape
            .position(self.variables.iter().map(|&var| &coordinates[var]))
    }
}

impl std::fmt::Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}{}(",
            if self.sign { '+' } else { '-' },
            self.predicate.name,
        )?;
        for (idx, var) in self.variables.iter().enumerate() {
            if idx != 0 {
                write!(f, ",")?;
            }
            write!(f, "x{}", var)?;
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
        let buffer = Buffer2::new(shape.volume(), BOOL_FALSE);
        Self {
            shape,
            domains,
            literals,
            buffer,
        }
    }

    fn evaluate(&mut self, state: &State) {
        self.buffer.fill(BOOL_FALSE);
        for lit in self.literals.iter_mut() {
            lit.evaluate(state, &mut self.buffer);
        }
    }

    fn get_status(&self) -> Bit2 {
        let mut res = BOOL_TRUE;
        for pos in 0..self.buffer.len() {
            let val = self.buffer.get(pos);
            res = BOOL_AND.of(res, val);
        }
        res
    }

    // Returns BOOL_FALSE if the clause has failed (maybe with propagations),
    // BOOL_UNDEF1 if some propagations were made and the status is unclear,
    // BOOL_TRUE if the clause is universally true, and BOOL_UNDEF2 otherwise.
    fn propagate(&self, state: &mut State) -> Bit2 {
        let mut coordinates = vec![0; self.shape.dimension()];
        let mut result = BOOL_TRUE;
        for pos in 0..self.buffer.len() {
            let val = self.buffer.get(pos);
            result = BOOL_AND.of(result, val);
            if val == BOOL_FALSE {
                break;
            } else if val == BOOL_UNDEF1 {
                self.shape.coordinates(pos, &mut coordinates);
                let mut unit = 0;
                let mut sign = None;
                let mut reason = vec![];
                for lit in self.literals.iter() {
                    let bvar = lit.position(&coordinates);
                    let bval = state.assignment.get(bvar);
                    if bval == BOOL_UNDEF1 {
                        assert!(sign.is_none());
                        sign = Some(lit.sign);
                        unit = bvar;
                    } else {
                        reason.push(bvar);
                    }
                }
                // maybe it was already assigned.
                if let Some(sign) = sign {
                    state.assign(unit, sign, Reason::Clause(reason));
                }
            }
        }

        let check = self.get_status();
        assert!(result == check || result == BOOL_UNDEF1);
        result
    }

    fn get_failure(&self) -> Option<Vec<usize>> {
        for pos in 0..self.buffer.len() {
            if self.buffer.get(pos) == BOOL_FALSE {
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
            let val = BOOL_FORMAT1[self.buffer.get(pos).idx()];
            println!("  {:?} = {}", cor, val);
        }
    }
}

impl std::fmt::Display for Clause {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "clause ")?;
        for (idx, lit) in self.literals.iter().enumerate() {
            if idx != 0 {
                write!(f, " ")?;
            }
            write!(f, "{}", lit)?;
        }

        write!(f, " = {}", BOOL_FORMAT2[self.get_status().idx()])
    }
}

#[derive(Debug)]
struct Exist {
    predicate: Rc<Predicate>,
}

impl Exist {
    fn new(predicate: Rc<Predicate>) -> Self {
        Exist { predicate }
    }

    fn get_status(&self, state: &State) -> Bit2 {
        let shape = &self.predicate.shape;
        let range = shape.positions();
        let block = shape.length(shape.dimension() - 1);

        let mut value1 = BOOL_TRUE;
        let mut pos = range.start;
        while pos < range.end {
            let mut value2 = BOOL_FALSE;
            for i in pos..(pos + block) {
                value2 = BOOL_OR.of(value2, state.assignment.get(i));
            }
            value1 = BOOL_AND.of(value1, value2);
            pos += block;
        }
        value1
    }

    // Returns BOOL_FALSE if the clause has failed (maybe with propagations),
    // BOOL_UNDEF1 if some propagations were made and the status is unclear,
    // BOOL_TRUE if the clause is universally true, and BOOL_UNDEF2 otherwise.
    fn propagate(&self, state: &mut State) -> Bit2 {
        let shape = &self.predicate.shape;
        let range = shape.positions();
        let block = shape.length(shape.dimension() - 1);

        let mut result = BOOL_TRUE;
        let mut pos = range.start;
        while pos < range.end {
            let mut value2 = BOOL_FALSE;
            let mut unit_pos = None;
            for i in pos..(pos + block) {
                let val = state.assignment.get(i);
                value2 = BOOL_OR.of(value2, val);
                if val == BOOL_UNDEF1 {
                    unit_pos = Some(i);
                }
            }
            result = BOOL_AND.of(result, value2);
            if value2 == BOOL_FALSE {
                break;
            } else if value2 == BOOL_UNDEF1 {
                debug_assert!(unit_pos.is_some());
                state.assign(unit_pos.unwrap(), true, Reason::Exists);
            }
            pos += block;
        }

        let check = self.get_status(state);
        assert!(result == check || result == BOOL_UNDEF1);
        result
    }

    fn get_failure(&self, state: &State) -> Option<usize> {
        let shape = &self.predicate.shape;
        let range = shape.positions();
        let block = shape.length(shape.dimension() - 1);

        let mut pos = range.start;
        while pos < range.end {
            let mut value2 = BOOL_FALSE;
            for i in pos..(pos + block) {
                value2 = BOOL_OR.of(value2, state.assignment.get(i));
            }
            if value2 == BOOL_FALSE {
                return Some(pos);
            }
            pos += block;
        }
        None
    }
}

impl std::fmt::Display for Exist {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "exist {}", self.predicate.name)
    }
}

#[derive(Debug, Default)]
pub struct Solver {
    state: State,
    domains: Vec<Rc<Domain>>,
    predicates: Vec<Rc<Predicate>>,
    clauses: Vec<Clause>,
    exists: Vec<Exist>,
}

impl Solver {
    pub fn add_domain(&mut self, name: String, size: usize) -> Rc<Domain> {
        assert!(self.domains.iter().all(|dom| dom.name != name));
        let dom = Rc::new(Domain::new(name, size));
        self.domains.push(dom.clone());
        dom
    }

    pub fn add_variable(&mut self, name: String, domains: Vec<Rc<Domain>>) -> Rc<Predicate> {
        assert!(self.predicates.iter().all(|pred| pred.name != name));
        let pred = Rc::new(Predicate::new(&mut self.state, name, domains));
        self.predicates.push(pred.clone());
        pred
    }

    pub fn add_clause(&mut self, literals: Vec<(bool, Rc<Predicate>, Vec<usize>)>) {
        let mut domains: Vec<Option<Rc<Domain>>> = Default::default();
        for (_, pred, indices) in literals.iter() {
            assert_eq!(pred.domains.len(), indices.len());
            for (pos, &idx) in indices.iter().enumerate() {
                if domains.len() <= idx {
                    domains.resize(idx + 1, None);
                }
                let dom1 = &pred.domains[pos];
                let dom2 = &mut domains[idx];
                if dom2.is_none() {
                    *dom2 = Some(dom1.clone());
                } else {
                    assert!(Rc::ptr_eq(dom1, dom2.as_ref().unwrap()));
                }
            }
        }
        let domains: Vec<Rc<Domain>> = domains.into_iter().map(|dom| dom.unwrap()).collect();

        let shape = Shape::new(domains.iter().map(|dom| dom.size).collect(), 0);
        let literals: Vec<Literal> = literals
            .into_iter()
            .map(|(sign, pred, indices)| Literal::new(&shape, sign, pred, indices))
            .collect();

        let cla = Clause::new(shape, domains, literals);
        self.clauses.push(cla);
    }

    pub fn add_exist(&mut self, predicate: Rc<Predicate>) {
        self.exists.push(Exist::new(predicate));
    }

    pub fn set_value(&mut self, sign: bool, predicate: &Predicate, coordinates: &[usize]) {
        let pos = predicate.shape.position(coordinates.iter());
        self.state.assign(pos, sign, Reason::Initial);
    }

    pub fn set_equality(&mut self, predicate: &Predicate) {
        for i in 0..predicate.shape.length(0) {
            for j in 0..predicate.shape.length(1) {
                let pos = predicate.shape.position([i, j].iter());
                self.state.assign(pos, i == j, Reason::Initial);
            }
        }
    }

    pub fn get_clauses_status(&self) -> Bit2 {
        let mut res = BOOL_TRUE;
        for cla in self.clauses.iter() {
            res = BOOL_AND.of(res, cla.get_status());
        }
        res
    }

    pub fn get_exists_status(&self) -> Bit2 {
        let mut res = BOOL_TRUE;
        for ext in self.exists.iter() {
            res = BOOL_AND.of(res, ext.get_status(&self.state));
        }
        res
    }

    pub fn get_status(&self) -> Bit2 {
        BOOL_AND.of(self.get_clauses_status(), self.get_exists_status())
    }

    pub fn evaluate_all(&mut self) {
        for cla in self.clauses.iter_mut() {
            cla.evaluate(&self.state);
        }
    }

    // Returns BOOL_FALSE if the clause has failed (maybe with propagations),
    // BOOL_UNDEF1 if some propagations were made and the status is unclear,
    // BOOL_TRUE if the clause is universally true, and BOOL_UNDEF2 otherwise.
    pub fn propagate_clauses(&mut self) -> Bit2 {
        let mut result = BOOL_TRUE;
        for cla in self.clauses.iter_mut() {
            cla.evaluate(&self.state);
            let val = cla.propagate(&mut self.state);
            result = BOOL_AND.of(result, val);
        }

        let check = self.get_clauses_status();
        assert!(result == check || result == BOOL_UNDEF1);
        result
    }

    pub fn propagate_exists(&mut self) -> Bit2 {
        let mut result = BOOL_TRUE;
        for xst in self.exists.iter() {
            let val = xst.propagate(&mut self.state);
            result = BOOL_AND.of(result, val);
        }

        let check = self.get_exists_status();
        assert!(result == check || result == BOOL_UNDEF1);
        result
    }

    pub fn search_all(&mut self) {
        let mut num_solutions: usize = 0;
        let mut num_learnings: usize = 0;
        let mut num_deadends: usize = 0;

        loop {
            let mut used_exists = false;
            let mut value;
            loop {
                value = self.propagate_clauses();
                if value == BOOL_UNDEF1 {
                    continue;
                } else if value == BOOL_FALSE {
                    break;
                }

                used_exists = true;
                value = BOOL_AND.of(value, self.propagate_exists());
                if value == BOOL_UNDEF1 {
                    continue;
                } else {
                    break;
                }
            }

            assert!(value != BOOL_UNDEF1 && value == self.get_status());
            if value == BOOL_FALSE && !used_exists {
                num_learnings += 1;
                if false {
                    println!("*** LEARNING ***");
                    self.evaluate_all();
                    self.print();
                    println!("*** END OF LEARNING ***");
                }
                if !self.state.next_decision() {
                    break;
                }
            } else if value == BOOL_FALSE && used_exists {
                num_deadends += 1;
                if false {
                    println!("*** EXISTS ***");
                    self.evaluate_all();
                    self.print();
                    println!("*** END OF EXISTS ***");
                }
                if !self.state.next_decision() {
                    break;
                }
            } else if value == BOOL_TRUE {
                num_solutions += 1;
                if false {
                    println!("*** SOLUTION ***");
                    for pred in self.predicates.iter() {
                        println!("{}", pred);
                        self.state.print_table(&pred.shape);
                    }
                    println!("*** END OF SOLUTION ***");
                }
                if !self.state.next_decision() {
                    break;
                }
            } else {
                assert_eq!(value, BOOL_UNDEF2);
                let ret = self.state.make_decision();
                assert!(ret);
            }
        }

        println!("Total solutions: {}", num_solutions);
        println!("Total learnings: {}", num_learnings);
        println!("Total deadends: {}", num_deadends);
    }

    fn lookup_var(&self, bvar: usize) -> &Predicate {
        for rvar in self.predicates.iter() {
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

    fn format_reason(&self, reason: &Reason) -> String {
        match reason {
            Reason::Initial => "initial".into(),
            Reason::Decision => "decision".into(),
            Reason::Clause(vars) => vars
                .iter()
                .map(|&bvar| self.format_var(bvar))
                .collect::<Vec<String>>()
                .join(" "),
            Reason::Exists => "exists".into(),
        }
    }

    pub fn print(&self) {
        for dom in self.domains.iter() {
            println!("{}", dom);
        }
        for pred in self.predicates.iter() {
            println!("{}", pred);
            self.state.print_table(&pred.shape);
        }
        for step in self.state.steps.iter() {
            println!(
                "step {} from {}",
                self.format_var(step.bvar),
                self.format_reason(&step.reason)
            );
        }
        for cla in self.clauses.iter() {
            println!("{}", cla);
            if let Some(failure) = cla.get_failure() {
                // duh, this is negated
                let failure: Vec<String> = failure
                    .into_iter()
                    .map(|bvar| self.format_var(bvar))
                    .collect();
                println!("failure {}", failure.join(" "));
            }
        }
        for ext in self.exists.iter() {
            // println!("exist {}", ext);
            println!(
                "{} = {}",
                ext,
                BOOL_FORMAT2[ext.get_status(&self.state).idx()]
            );
            if let Some(failure) = ext.get_failure(&self.state) {
                println!("failure {}", self.format_var(failure));
            }
        }
        if false {
            println!("steps = {:?}", self.state.steps);
            println!("levels = {:?}", self.state.levels);
        }
        println!(
            "clauses status = {}",
            BOOL_FORMAT2[self.get_clauses_status().idx()]
        );
        println!(
            "exists status = {}",
            BOOL_FORMAT2[self.get_exists_status().idx()]
        );
    }
}
