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

use super::bitops::*;
use super::buffer::Buffer2;
use super::shape::{PositionIter, Shape};

#[derive(Debug, Default)]
enum Reason {
    #[default]
    Initial,
    Decision,
    Clause(Vec<usize>),
    Exists,
}

#[derive(Debug, Default)]
struct Step {
    bvar: usize,
    reason: Reason,
}

#[derive(Debug, Default)]
struct State {
    assignment: Buffer2,
    steps: Vec<Step>,
    levels: Vec<usize>,
    domains: Vec<Domain>,
    variables: Vec<Variable>,
}

struct Member<'a, T>(&'a State, T);

impl State {
    fn get_domain(&self, dom: Dom) -> &Domain {
        &self.domains[dom.idx()]
    }

    fn get_variable(&self, var: Var) -> &Variable {
        &self.variables[var.idx()]
    }

    fn create_table(&mut self, doms: &[Dom]) -> Shape {
        let shape = Shape::new(
            doms.iter().map(|&dom| self.get_domain(dom).size).collect(),
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
    fn new(name: &str, size: usize) -> Self {
        let name = name.to_string();
        Self { name, size }
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

impl std::fmt::Display for Member<'_, &Domain> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "domain {} = {}", self.1.name, self.1.size)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Dom(u32);

impl Dom {
    fn new(idx: usize) -> Self {
        debug_assert!(idx < u32::MAX as usize);
        Dom(idx as u32)
    }

    fn idx(&self) -> usize {
        self.0 as usize
    }
}

impl std::fmt::Display for Member<'_, Dom> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0.get_domain(self.1).name)
    }
}

impl std::fmt::Display for Member<'_, &[Dom]> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "(")?;
        let mut first = true;
        for &dom in self.1.iter() {
            if first {
                first = false;
            } else {
                write!(f, ",")?;
            }
            write!(f, "{}", Member(self.0, dom))?;
        }
        write!(f, ")")
    }
}

#[derive(Debug)]
pub struct Variable {
    shape: Shape,
    name: String,
    doms: Vec<Dom>,
}

impl Variable {
    fn new(state: &mut State, name: &str, doms: Vec<Dom>) -> Self {
        let name = name.to_string();
        let shape = state.create_table(&doms);
        Self { name, doms, shape }
    }
}

impl std::fmt::Display for Member<'_, &Variable> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}({})",
            self.1.name,
            Member(self.0, self.1.doms.as_slice())
        )
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Var(u32);

impl Var {
    fn new(idx: usize) -> Self {
        debug_assert!(idx < u32::MAX as usize);
        Var(idx as u32)
    }

    fn idx(&self) -> usize {
        self.0 as usize
    }
}

impl std::fmt::Display for Member<'_, Var> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0.get_variable(self.1).name)
    }
}

#[derive(Debug)]
struct Literal {
    var: Var,
    axes: Box<[usize]>,
    positions: PositionIter,
    sign: bool,
}

impl Literal {
    fn new(state: &State, shape: &Shape, sign: bool, var: Var, axes: Vec<usize>) -> Self {
        let axes = axes.into_boxed_slice();
        let positions = state
            .get_variable(var)
            .shape
            .view()
            .polymer(shape, &axes)
            .simplify()
            .positions();
        Literal {
            var,
            axes,
            positions,
            sign,
        }
    }

    fn evaluate(&mut self, state: &State, target: &mut Buffer2) {
        self.positions.reset();
        let op = if self.sign { BOOL_OR } else { BOOL_ORNOT };
        target.apply(op, &state.assignment, &mut self.positions);
    }

    fn position(&self, state: &State, coordinates: &[usize]) -> usize {
        state
            .get_variable(self.var)
            .shape
            .position(self.axes.iter().map(|&axis| &coordinates[axis]))
    }
}

impl std::fmt::Display for Member<'_, &Literal> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}{}(",
            if self.1.sign { '+' } else { '-' },
            Member(self.0, self.1.var),
        )?;
        let mut first = true;
        for &idx in self.1.axes.iter() {
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
    domains: Vec<Dom>,
    literals: Vec<Literal>,
    shape: Shape,
    buffer: Buffer2,
}

impl Clause {
    fn new(shape: Shape, domains: Vec<Dom>, literals: Vec<Literal>) -> Self {
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
                    let bvar = lit.position(state, &coordinates);
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

    fn get_failure(&self, state: &State) -> Option<Vec<usize>> {
        for pos in 0..self.buffer.len() {
            if self.buffer.get(pos) == BOOL_FALSE {
                let mut coordinates = vec![0; self.shape.dimension()];
                self.shape.coordinates(pos, &mut coordinates);
                return Some(
                    self.literals
                        .iter()
                        .map(|lit| lit.position(state, &coordinates))
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

impl std::fmt::Display for Member<'_, &Clause> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut first = true;
        for lit in self.1.literals.iter() {
            if first {
                first = false;
            } else {
                write!(f, " ")?;
            }
            write!(f, "{}", Member(self.0, lit))?;
        }

        write!(f, " = {}", BOOL_FORMAT2[self.1.get_status().idx()])
    }
}

#[derive(Debug)]
struct Exist {
    var: Var,
}

impl Exist {
    fn new(var: Var) -> Self {
        Exist { var }
    }

    fn get_status(&self, state: &State) -> Bit2 {
        let variable = state.get_variable(self.var);
        let shape = &variable.shape;
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
        let shape = &state.get_variable(self.var).shape;
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
        let shape = &state.get_variable(self.var).shape;
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

impl std::fmt::Display for Member<'_, &Exist> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Member(self.0, self.1.var).fmt(f)
    }
}

#[derive(Debug, Default)]
pub struct Solver {
    state: State,
    clauses: Vec<Clause>,
    exists: Vec<Exist>,
}

impl Solver {
    pub fn add_domain(&mut self, name: &str, size: usize) -> Dom {
        let domains = &mut self.state.domains;
        assert!(domains.iter().all(|domain| domain.name != name));
        let dom = Dom::new(domains.len());
        domains.push(Domain::new(name, size));
        dom
    }

    pub fn add_variable(&mut self, name: &str, doms: Vec<Dom>) -> Var {
        assert!(self.state.variables.iter().all(|rel| rel.name != name));
        let var = Var::new(self.state.variables.len());
        let variable = Variable::new(&mut self.state, name, doms);
        self.state.variables.push(variable);
        var
    }

    pub fn add_clause(&mut self, literals: Vec<(bool, Var, Vec<usize>)>) {
        let mut doms: Vec<Option<Dom>> = Default::default();
        for (_, var, indices) in literals.iter() {
            let variable = self.state.get_variable(*var);
            assert_eq!(variable.doms.len(), indices.len());
            for (pos, &idx) in indices.iter().enumerate() {
                if doms.len() <= idx {
                    doms.resize(idx + 1, None);
                }
                let dom1 = variable.doms[pos];
                let dom2 = &mut doms[idx];
                if dom2.is_none() {
                    *dom2 = Some(dom1);
                } else {
                    let dom2 = dom2.unwrap();
                    assert_eq!(dom1, dom2);
                }
            }
        }
        let doms: Vec<Dom> = doms.into_iter().map(|dom| dom.unwrap()).collect();

        let shape = Shape::new(
            doms.iter()
                .map(|&dom| self.state.get_domain(dom).size)
                .collect(),
            0,
        );
        let literals: Vec<Literal> = literals
            .into_iter()
            .map(|(sign, var, indices)| Literal::new(&self.state, &shape, sign, var, indices))
            .collect();

        let cla = Clause::new(shape, doms, literals);
        self.clauses.push(cla);
    }

    pub fn add_exist(&mut self, var: Var) {
        self.exists.push(Exist::new(var));
    }

    pub fn set_value(&mut self, sign: bool, var: Var, coordinates: &[usize]) {
        let pos = self
            .state
            .get_variable(var)
            .shape
            .position(coordinates.iter());
        self.state.assign(pos, sign, Reason::Initial);
    }

    pub fn set_equality(&mut self, var: Var) {
        // TODO: this is not efficient
        let (len0, len1) = {
            let shape = &self.state.get_variable(var).shape;
            assert!(shape.dimension() == 2);
            (shape.length(0), shape.length(1))
        };
        for i in 0..len0 {
            for j in 0..len1 {
                self.set_value(i == j, var, &[i, j]);
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
                println!("*** LEARNING ***");
                self.evaluate_all();
                self.print();
                println!("*** END OF LEARNING ***");
                break;
            } else if value == BOOL_FALSE && used_exists {
                if true {
                    println!("*** EXISTS ***");
                    self.evaluate_all();
                    self.print();
                    println!("*** END OF EXISTS ***");
                }
                if !self.state.next_decision() {
                    break;
                }
            } else if value == BOOL_TRUE {
                if false {
                    println!("*** SOLUTION ***");
                    for var in self.state.variables.iter() {
                        println!("variable {}", Member(&self.state, var));
                        self.state.print_table(&var.shape);
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
    }

    fn lookup_var(&self, bvar: usize) -> &Variable {
        for rvar in self.state.variables.iter() {
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
        for domain in self.state.domains.iter() {
            println!("{}", Member(&self.state, domain));
        }
        for var in self.state.variables.iter() {
            println!("variable {}", Member(&self.state, var));
            self.state.print_table(&var.shape);
        }
        for step in self.state.steps.iter() {
            println!(
                "step {} from {}",
                self.format_var(step.bvar),
                self.format_reason(&step.reason)
            );
        }
        for cla in self.clauses.iter() {
            println!("clause {}", Member(&self.state, cla));
            if let Some(failure) = cla.get_failure(&self.state) {
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
                Member(&self.state, ext.var),
                BOOL_FORMAT2[ext.get_status(&self.state).idx()]
            );
            if let Some(failure) = ext.get_failure(&self.state) {
                println!("failure {:?}", self.format_var(failure));
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
