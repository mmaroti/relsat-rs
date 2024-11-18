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

#![allow(dead_code)]

use super::solver::*;

pub fn main1() {
    let mut sol: Solver = Default::default();
    let set = sol.add_domain("set".into(), 7);
    let one = sol.add_variable("one".into(), vec![set.clone()]);
    let inv = sol.add_variable("inv".into(), vec![set.clone(), set.clone()]);
    let mul = sol.add_variable("mul".into(), vec![set.clone(), set.clone(), set.clone()]);
    let equ = sol.add_variable("equ".into(), vec![set.clone(), set.clone()]);

    // equivalence relation
    if true {
        sol.add_clause(vec![(true, equ.clone(), vec![0, 0])]);

        sol.add_clause(vec![
            (false, equ.clone(), vec![0, 1]),
            (true, equ.clone(), vec![1, 0]),
        ]);

        sol.add_clause(vec![
            (false, equ.clone(), vec![0, 1]),
            (false, equ.clone(), vec![1, 2]),
            (true, equ.clone(), vec![0, 2]),
        ]);
    }

    sol.add_clause(vec![
        (false, mul.clone(), vec![0, 1, 3]),
        (false, mul.clone(), vec![3, 2, 4]),
        (false, mul.clone(), vec![1, 2, 5]),
        (true, mul.clone(), vec![0, 5, 4]),
    ]);

    // trivial consequences but not unit propagated
    sol.add_clause(vec![
        (false, mul.clone(), vec![0, 1, 0]),
        (false, mul.clone(), vec![1, 1, 2]),
        (true, mul.clone(), vec![0, 2, 0]),
    ]);
    sol.add_clause(vec![
        (false, mul.clone(), vec![0, 0, 1]),
        (false, mul.clone(), vec![1, 0, 2]),
        (true, mul.clone(), vec![0, 1, 2]),
    ]);
    sol.add_clause(vec![
        (false, mul.clone(), vec![0, 1, 1]),
        (false, mul.clone(), vec![1, 2, 3]),
        (true, mul.clone(), vec![0, 3, 3]),
    ]);

    sol.add_clause(vec![
        (false, mul.clone(), vec![1, 2, 3]),
        (false, mul.clone(), vec![0, 3, 4]),
        (false, mul.clone(), vec![0, 1, 5]),
        (true, mul.clone(), vec![5, 2, 4]),
    ]);

    // trivial consequences but not unit propagated
    sol.add_clause(vec![
        (false, mul.clone(), vec![0, 1, 1]),
        (false, mul.clone(), vec![0, 0, 2]),
        (true, mul.clone(), vec![2, 1, 1]),
    ]);
    sol.add_clause(vec![
        (false, mul.clone(), vec![0, 0, 1]),
        (false, mul.clone(), vec![0, 1, 2]),
        (true, mul.clone(), vec![1, 0, 2]),
    ]);
    sol.add_clause(vec![
        (false, mul.clone(), vec![1, 2, 1]),
        (false, mul.clone(), vec![0, 1, 3]),
        (true, mul.clone(), vec![3, 2, 3]),
    ]);

    sol.add_clause(vec![
        (false, inv.clone(), vec![0, 1]),
        (false, mul.clone(), vec![1, 0, 2]),
        (true, one.clone(), vec![2]),
    ]);

    sol.add_clause(vec![
        (false, one.clone(), vec![0]),
        (true, mul.clone(), vec![0, 1, 1]),
    ]);

    // mul is an operation
    sol.add_clause(vec![
        (false, mul.clone(), vec![0, 1, 2]),
        (false, mul.clone(), vec![0, 1, 3]),
        (true, equ.clone(), vec![2, 3]),
    ]);
    if true {
        sol.add_clause(vec![
            (false, mul.clone(), vec![0, 1, 2]),
            (false, equ.clone(), vec![0, 3]),
            (true, mul.clone(), vec![3, 1, 2]),
        ]);

        sol.add_clause(vec![
            (false, mul.clone(), vec![0, 1, 2]),
            (false, equ.clone(), vec![1, 3]),
            (true, mul.clone(), vec![0, 3, 2]),
        ]);

        sol.add_clause(vec![
            (false, mul.clone(), vec![0, 1, 2]),
            (false, equ.clone(), vec![2, 3]),
            (true, mul.clone(), vec![0, 1, 3]),
        ]);
    }

    sol.add_exist(mul.clone());

    // inv is an operation
    sol.add_clause(vec![
        (false, inv.clone(), vec![0, 1]),
        (false, inv.clone(), vec![0, 2]),
        (true, equ.clone(), vec![1, 2]),
    ]);
    if true {
        sol.add_clause(vec![
            (false, inv.clone(), vec![0, 1]),
            (false, equ.clone(), vec![1, 2]),
            (true, inv.clone(), vec![0, 2]),
        ]);
        sol.add_clause(vec![
            (false, inv.clone(), vec![0, 1]),
            (false, equ.clone(), vec![0, 2]),
            (true, inv.clone(), vec![2, 1]),
        ]);
    }

    sol.add_exist(inv.clone());

    sol.add_clause(vec![
        (false, one.clone(), vec![0]),
        (false, one.clone(), vec![1]),
        (true, equ.clone(), vec![0, 1]),
    ]);
    if true {
        sol.add_clause(vec![
            (false, one.clone(), vec![0]),
            (false, equ.clone(), vec![0, 1]),
            (true, one.clone(), vec![1]),
        ]);
    }

    sol.add_exist(one.clone());

    // learned
    sol.add_clause(vec![
        (false, one.clone(), vec![0]),
        (false, inv.clone(), vec![1, 1]),
        (true, mul.clone(), vec![1, 0, 1]),
    ]);

    // learned
    sol.add_clause(vec![
        (false, one.clone(), vec![0]),
        (false, inv.clone(), vec![1, 2]),
        (false, mul.clone(), vec![1, 0, 2]),
        (true, mul.clone(), vec![1, 0, 1]),
    ]);

    // learned
    sol.add_clause(vec![
        (false, one.clone(), vec![0]),
        (false, mul.clone(), vec![1, 1, 2]),
        (false, mul.clone(), vec![1, 2, 0]),
        (true, mul.clone(), vec![1, 0, 1]),
    ]);

    // learned
    sol.add_clause(vec![
        (false, one.clone(), vec![0]),
        (false, inv.clone(), vec![1, 2]),
        (false, inv.clone(), vec![0, 0]),
        (false, mul.clone(), vec![1, 1, 2]),
        (true, mul.clone(), vec![1, 0, 1]),
    ]);

    // learned
    sol.add_clause(vec![
        (false, one.clone(), vec![0]),
        (false, mul.clone(), vec![1, 0, 2]),
        (false, inv.clone(), vec![2, 1]),
        (false, mul.clone(), vec![1, 1, 3]),
        (false, inv.clone(), vec![3, 1]),
        (true, mul.clone(), vec![1, 0, 1]),
    ]);

    // learned
    sol.add_clause(vec![
        (false, one.clone(), vec![0]),
        (false, inv.clone(), vec![1, 2]),
        (false, mul.clone(), vec![1, 1, 3]),
        (false, inv.clone(), vec![3, 2]),
        (true, mul.clone(), vec![1, 0, 1]),
    ]);

    // learned
    sol.add_clause(vec![
        (false, one.clone(), vec![0]),
        (false, mul.clone(), vec![1, 0, 2]),
        (false, inv.clone(), vec![2, 1]),
        (false, mul.clone(), vec![1, 1, 3]),
        (false, inv.clone(), vec![3, 2]),
        (true, mul.clone(), vec![1, 0, 1]),
    ]);

    sol.set_value(true, &one.clone(), &[0]);
    sol.set_value(false, &mul.clone(), &[1, 0, 1]);
    sol.propagate_clauses();
    // sol.set_value(true, inv.clone(), &[1, 2]);
    sol.propagate_clauses();
    sol.set_value(true, &mul.clone(), &[1, 0, 3]);
    // sol.set_value(true, inv.clone(), &[0, 0]);
    // sol.set_value(true, inv.clone(), &[2, 1]);
    // sol.set_value(true, inv.clone(), &[3, 1]);
    sol.propagate_clauses();
    sol.set_value(true, &mul.clone(), &[1, 1, 4]);
    sol.propagate_clauses();
    sol.set_value(true, &inv.clone(), &[4, 5]);
    sol.propagate_clauses();
    sol.set_value(true, &mul.clone(), &[1, 3, 6]);

    sol.search_all();
}

pub fn main2() {
    let mut sol: Solver = Default::default();
    let set = sol.add_domain("set".into(), 4);

    let equ = sol.add_variable("equ".into(), vec![set.clone(), set.clone()]);
    sol.set_equality(&equ);

    let mul = sol.add_variable("mul".into(), vec![set.clone(), set.clone(), set.clone()]);

    sol.add_exist(mul.clone());
    sol.add_clause(vec![
        (false, mul.clone(), vec![0, 1, 2]),
        (false, mul.clone(), vec![0, 1, 3]),
        (true, equ.clone(), vec![2, 3]),
    ]);

    sol.add_clause(vec![
        (false, mul.clone(), vec![0, 1, 3]),
        (false, mul.clone(), vec![3, 2, 4]),
        (false, mul.clone(), vec![1, 2, 5]),
        (true, mul.clone(), vec![0, 5, 4]),
    ]);

    // learnt at size 3
    sol.add_clause(vec![
        (false, mul.clone(), vec![0, 1, 0]),
        (true, mul.clone(), vec![1, 0, 0]),
        (false, mul.clone(), vec![1, 1, 0]),
    ]);

    sol.add_clause(vec![
        (false, mul.clone(), vec![0, 0, 0]),
        (true, mul.clone(), vec![0, 1, 1]),
        (false, mul.clone(), vec![0, 2, 1]),
    ]);

    sol.add_clause(vec![
        (true, mul.clone(), vec![0, 1, 1]),
        (false, mul.clone(), vec![0, 2, 2]),
        (false, mul.clone(), vec![2, 1, 1]),
    ]);

    sol.add_clause(vec![
        (true, mul.clone(), vec![0, 1, 1]),
        (false, mul.clone(), vec![0, 2, 2]),
        (false, mul.clone(), vec![2, 0, 1]),
    ]);

    sol.add_clause(vec![
        (true, mul.clone(), vec![1, 0, 0]),
        (false, mul.clone(), vec![1, 2, 2]),
        (false, mul.clone(), vec![2, 2, 0]),
    ]);

    sol.add_clause(vec![
        (true, equ.clone(), vec![0, 1]),
        (false, mul.clone(), vec![0, 1, 1]),
        (false, mul.clone(), vec![2, 1, 2]),
        (false, mul.clone(), vec![2, 2, 0]),
    ]);

    sol.add_clause(vec![
        (true, equ.clone(), vec![0, 1]),
        (false, mul.clone(), vec![0, 1, 1]),
        (false, mul.clone(), vec![1, 1, 1]),
        (false, mul.clone(), vec![2, 1, 0]),
    ]);

    sol.add_clause(vec![
        (false, mul.clone(), vec![0, 1, 1]),
        (true, mul.clone(), vec![1, 0, 1]),
        (false, mul.clone(), vec![1, 1, 0]),
    ]);

    sol.add_clause(vec![
        (false, mul.clone(), vec![0, 1, 2]),
        (true, mul.clone(), vec![1, 0, 2]),
        (false, mul.clone(), vec![1, 1, 0]),
    ]);

    // learnt at size 4
    sol.add_clause(vec![
        (false, mul.clone(), vec![1, 1, 0]),
        (true, mul.clone(), vec![2, 0, 2]),
        (false, mul.clone(), vec![2, 1, 2]),
    ]);

    sol.add_clause(vec![
        (true, equ.clone(), vec![2, 3]),
        (false, mul.clone(), vec![1, 2, 3]),
        (false, mul.clone(), vec![2, 0, 3]),
        (false, mul.clone(), vec![2, 1, 2]),
        (false, mul.clone(), vec![2, 2, 2]),
    ]);

    sol.add_clause(vec![
        (true, mul.clone(), vec![2, 0, 0]),
        (false, mul.clone(), vec![2, 3, 3]),
        (false, mul.clone(), vec![3, 1, 0]),
    ]);

    sol.add_clause(vec![
        (true, equ.clone(), vec![2, 3]),
        (false, mul.clone(), vec![1, 0, 1]),
        (false, mul.clone(), vec![2, 0, 3]),
        (false, mul.clone(), vec![2, 1, 2]),
    ]);

    sol.add_clause(vec![
        (true, equ.clone(), vec![1, 2]),
        (false, mul.clone(), vec![0, 0, 0]),
        (false, mul.clone(), vec![1, 0, 2]),
        (false, mul.clone(), vec![3, 0, 1]),
    ]);

    sol.add_clause(vec![
        (true, equ.clone(), vec![2, 0]),
        (false, mul.clone(), vec![1, 1, 0]),
        (false, mul.clone(), vec![0, 2, 0]),
        (false, mul.clone(), vec![1, 2, 2]),
    ]);

    sol.add_clause(vec![
        (true, equ.clone(), vec![2, 3]),
        (false, mul.clone(), vec![0, 2, 3]),
        (false, mul.clone(), vec![1, 1, 0]),
        (false, mul.clone(), vec![1, 2, 2]),
    ]);

    sol.add_clause(vec![
        (true, equ.clone(), vec![0, 1]),
        (false, mul.clone(), vec![1, 0, 0]),
        (false, mul.clone(), vec![1, 1, 1]),
        (false, mul.clone(), vec![2, 0, 3]),
        (false, mul.clone(), vec![2, 1, 3]),
        (false, mul.clone(), vec![2, 2, 1]),
    ]);

    sol.add_clause(vec![
        (true, equ.clone(), vec![0, 1]),
        (false, mul.clone(), vec![0, 1, 1]),
        (false, mul.clone(), vec![3, 1, 3]),
        (false, mul.clone(), vec![2, 3, 0]),
    ]);

    sol.add_clause(vec![
        (true, equ.clone(), vec![1, 2]),
        (false, mul.clone(), vec![1, 0, 2]),
        (false, mul.clone(), vec![3, 0, 3]),
        (false, mul.clone(), vec![3, 3, 1]),
    ]);

    sol.add_clause(vec![
        (true, equ.clone(), vec![0, 1]),
        (false, mul.clone(), vec![0, 3, 1]),
        (false, mul.clone(), vec![2, 3, 2]),
        (false, mul.clone(), vec![3, 2, 0]),
    ]);

    // learnt at size 5
    sol.add_clause(vec![
        (true, mul.clone(), vec![2, 0, 0]),
        (false, mul.clone(), vec![2, 0, 1]),
        (false, mul.clone(), vec![3, 0, 0]),
        (false, mul.clone(), vec![3, 1, 0]),
        (false, mul.clone(), vec![2, 2, 3]),
    ]);

    sol.add_clause(vec![
        (false, mul.clone(), vec![0, 3, 0]),
        (false, mul.clone(), vec![2, 2, 1]),
        (true, mul.clone(), vec![2, 3, 0]),
        (false, mul.clone(), vec![1, 2, 0]),
        (false, mul.clone(), vec![2, 3, 3]),
    ]);

    sol.add_clause(vec![
        (true, equ.clone(), vec![0, 3]),
        (false, mul.clone(), vec![2, 1, 3]),
        (false, mul.clone(), vec![3, 1, 3]),
        (false, mul.clone(), vec![3, 2, 0]),
        (false, mul.clone(), vec![2, 2, 3]),
    ]);

    sol.add_clause(vec![
        (true, equ.clone(), vec![0, 2]),
        (false, mul.clone(), vec![1, 0, 0]),
        (false, mul.clone(), vec![1, 2, 2]),
        (false, mul.clone(), vec![3, 0, 4]),
        (false, mul.clone(), vec![3, 2, 4]),
        (false, mul.clone(), vec![3, 3, 1]),
    ]);

    sol.add_clause(vec![
        (true, equ.clone(), vec![0, 2]),
        (false, mul.clone(), vec![1, 1, 2]),
        (false, mul.clone(), vec![2, 1, 0]),
        (false, mul.clone(), vec![3, 3, 3]),
        (false, mul.clone(), vec![3, 1, 2]),
    ]);

    sol.add_clause(vec![
        (true, equ.clone(), vec![0, 3]),
        (false, mul.clone(), vec![1, 1, 3]),
        (false, mul.clone(), vec![1, 4, 3]),
        (false, mul.clone(), vec![2, 1, 0]),
        (false, mul.clone(), vec![2, 4, 3]),
        (false, mul.clone(), vec![4, 1, 2]),
    ]);

    sol.add_clause(vec![
        (true, equ.clone(), vec![0, 3]),
        (false, mul.clone(), vec![1, 1, 3]),
        (false, mul.clone(), vec![1, 4, 3]),
        (false, mul.clone(), vec![2, 1, 3]),
        (false, mul.clone(), vec![2, 4, 0]),
        (false, mul.clone(), vec![4, 1, 2]),
    ]);

    sol.add_clause(vec![
        (true, equ.clone(), vec![0, 4]),
        (false, mul.clone(), vec![2, 0, 4]),
        (false, mul.clone(), vec![2, 1, 4]),
        (false, mul.clone(), vec![3, 0, 0]),
        (false, mul.clone(), vec![3, 1, 4]),
        (false, mul.clone(), vec![3, 2, 3]),
    ]);

    sol.search_all();
}

pub fn main3() {
    let mut sol: Solver = Default::default();
    let set = sol.add_domain("set".into(), 3);

    let equ = sol.add_variable("equ".into(), vec![set.clone(), set.clone()]);
    sol.set_equality(&equ);

    /*
    let ord = sol.add_variable("ord", vec![&set, &set]);
    sol.add_clause(vec![(true, &ord, vec![0, 0])]);
    sol.add_clause(vec![
        (false, &ord, vec![0, 1]),
        (false, &ord, vec![1, 0]),
        (true, equ.clone(), vec![0, 1]),
    ]);
    sol.add_clause(vec![
        (false, &ord, vec![0, 1]),
        (false, &ord, vec![1, 2]),
        (true, &ord, vec![0, 2]),
    ]);
    */

    let mul = sol.add_variable("mul".into(), vec![set.clone(), set.clone(), set.clone()]);
    sol.add_exist(mul.clone());
    sol.add_clause(vec![
        (false, mul.clone(), vec![0, 1, 2]),
        (false, mul.clone(), vec![0, 1, 3]),
        (true, equ.clone(), vec![2, 3]),
    ]);
    sol.add_clause(vec![
        (false, mul.clone(), vec![0, 1, 3]),
        (false, mul.clone(), vec![3, 2, 4]),
        (false, mul.clone(), vec![1, 2, 5]),
        (false, mul.clone(), vec![0, 5, 6]),
        (true, equ.clone(), vec![4, 6]),
    ]);

    sol.add_clause(vec![
        (false, mul.clone(), vec![1, 1, 0]),
        (false, mul.clone(), vec![2, 0, 0]),
        (false, mul.clone(), vec![2, 1, 2]),
        (true, equ.clone(), vec![0, 2]),
    ]);

    /*
    sol.add_clause(vec![
        (false, &ord, vec![0, 1]),
        (false, &ord, vec![2, 3]),
        (false, mul.clone(), vec![0, 2, 4]),
        (false, mul.clone(), vec![1, 3, 5]),
        (true, &ord, vec![4, 5]),
    ]);
    */

    sol.search_all();
}
