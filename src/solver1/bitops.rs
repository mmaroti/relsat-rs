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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Bit1(u32);

impl Bit1 {
    #[inline(always)]
    pub fn new(val: u32) -> Bit1 {
        debug_assert!(val <= 1);
        Bit1(val)
    }

    #[inline(always)]
    pub fn idx(self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Bit2(u32);

impl Bit2 {
    #[inline(always)]
    pub const fn new(val: u32) -> Bit2 {
        debug_assert!(val <= 3);
        Bit2(val)
    }

    #[inline(always)]
    pub const fn idx(self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Op22(u32);

impl Op22 {
    pub const fn new(cases: &[(Bit2, Bit2)]) -> Self {
        assert!(cases.len() == 4);
        let mut set: u32 = 0;
        let mut val: u32 = 0;
        let mut idx = 0;
        while idx < cases.len() {
            let (a, b) = cases[idx];
            assert!(a.0 <= 3 && b.0 <= 3);
            let pos = a.0 << 1;
            val |= b.0 << pos;
            set |= 3 << pos;
            idx += 1;
        }
        assert!(set == 0xff);
        Op22(val)
    }

    #[inline(always)]
    pub const fn of(self, a: Bit2) -> Bit2 {
        Bit2((self.0 >> (a.0 << 1)) & 3)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Op222(u32);

impl Op222 {
    pub const fn new(cases: &[(Bit2, Bit2, Bit2)]) -> Self {
        assert!(cases.len() == 16);
        let mut set: u32 = 0;
        let mut val: u32 = 0;
        let mut idx = 0;
        while idx < cases.len() {
            let (a, b, c) = cases[idx];
            assert!(a.0 <= 3 && b.0 <= 3 && c.0 <= 3);
            let pos = (a.0 << 3) | (b.0 << 1);
            val |= c.0 << pos;
            set |= 3 << pos;
            idx += 1;
        }
        assert!(set == 0xffffffff);
        Op222(val)
    }

    #[inline(always)]
    pub const fn of(self, a: Bit2, b: Bit2) -> Bit2 {
        Bit2((self.0 >> ((a.0 << 3) | (b.0 << 1))) & 3)
    }
}

pub const BOOL_FALSE: Bit2 = Bit2(0);
pub const BOOL_UNDEF1: Bit2 = Bit2(1);
pub const BOOL_UNDEF2: Bit2 = Bit2(2);
pub const BOOL_TRUE: Bit2 = Bit2(3);

pub const BOOL_FORMAT1: [char; 4] = ['0', '?', 'x', '1'];
pub const BOOL_FORMAT2: [&str; 4] = ["false", "undef1", "undef2", "true"];

pub const BOOL_NOT: Op22 = Op22::new(&[
    (BOOL_FALSE, BOOL_TRUE),
    (BOOL_UNDEF1, BOOL_UNDEF1),
    (BOOL_UNDEF2, BOOL_UNDEF2),
    (BOOL_TRUE, BOOL_FALSE),
]);

pub const BOOL_OR: Op222 = Op222::new(&[
    (BOOL_FALSE, BOOL_FALSE, BOOL_FALSE),
    (BOOL_FALSE, BOOL_UNDEF1, BOOL_UNDEF1),
    (BOOL_FALSE, BOOL_UNDEF2, BOOL_UNDEF2),
    (BOOL_FALSE, BOOL_TRUE, BOOL_TRUE),
    (BOOL_UNDEF1, BOOL_FALSE, BOOL_UNDEF1),
    (BOOL_UNDEF1, BOOL_UNDEF1, BOOL_UNDEF2),
    (BOOL_UNDEF1, BOOL_UNDEF2, BOOL_UNDEF2),
    (BOOL_UNDEF1, BOOL_TRUE, BOOL_TRUE),
    (BOOL_UNDEF2, BOOL_FALSE, BOOL_UNDEF2),
    (BOOL_UNDEF2, BOOL_UNDEF1, BOOL_UNDEF2),
    (BOOL_UNDEF2, BOOL_UNDEF2, BOOL_UNDEF2),
    (BOOL_UNDEF2, BOOL_TRUE, BOOL_TRUE),
    (BOOL_TRUE, BOOL_FALSE, BOOL_TRUE),
    (BOOL_TRUE, BOOL_UNDEF1, BOOL_TRUE),
    (BOOL_TRUE, BOOL_UNDEF2, BOOL_TRUE),
    (BOOL_TRUE, BOOL_TRUE, BOOL_TRUE),
]);

pub const BOOL_ORNOT: Op222 = Op222::new(&[
    (BOOL_FALSE, BOOL_TRUE, BOOL_FALSE),
    (BOOL_FALSE, BOOL_UNDEF1, BOOL_UNDEF1),
    (BOOL_FALSE, BOOL_UNDEF2, BOOL_UNDEF2),
    (BOOL_FALSE, BOOL_FALSE, BOOL_TRUE),
    (BOOL_UNDEF1, BOOL_TRUE, BOOL_UNDEF1),
    (BOOL_UNDEF1, BOOL_UNDEF1, BOOL_UNDEF2),
    (BOOL_UNDEF1, BOOL_UNDEF2, BOOL_UNDEF2),
    (BOOL_UNDEF1, BOOL_FALSE, BOOL_TRUE),
    (BOOL_UNDEF2, BOOL_TRUE, BOOL_UNDEF2),
    (BOOL_UNDEF2, BOOL_UNDEF1, BOOL_UNDEF2),
    (BOOL_UNDEF2, BOOL_UNDEF2, BOOL_UNDEF2),
    (BOOL_UNDEF2, BOOL_FALSE, BOOL_TRUE),
    (BOOL_TRUE, BOOL_TRUE, BOOL_TRUE),
    (BOOL_TRUE, BOOL_UNDEF1, BOOL_TRUE),
    (BOOL_TRUE, BOOL_UNDEF2, BOOL_TRUE),
    (BOOL_TRUE, BOOL_FALSE, BOOL_TRUE),
]);

pub const BOOL_AND: Op222 = Op222::new(&[
    (BOOL_FALSE, BOOL_FALSE, BOOL_FALSE),
    (BOOL_FALSE, BOOL_UNDEF1, BOOL_FALSE),
    (BOOL_FALSE, BOOL_UNDEF2, BOOL_FALSE),
    (BOOL_FALSE, BOOL_TRUE, BOOL_FALSE),
    (BOOL_UNDEF1, BOOL_FALSE, BOOL_FALSE),
    (BOOL_UNDEF1, BOOL_UNDEF1, BOOL_UNDEF1),
    (BOOL_UNDEF1, BOOL_UNDEF2, BOOL_UNDEF1),
    (BOOL_UNDEF1, BOOL_TRUE, BOOL_UNDEF1),
    (BOOL_UNDEF2, BOOL_FALSE, BOOL_FALSE),
    (BOOL_UNDEF2, BOOL_UNDEF1, BOOL_UNDEF1),
    (BOOL_UNDEF2, BOOL_UNDEF2, BOOL_UNDEF2),
    (BOOL_UNDEF2, BOOL_TRUE, BOOL_UNDEF2),
    (BOOL_TRUE, BOOL_FALSE, BOOL_FALSE),
    (BOOL_TRUE, BOOL_UNDEF1, BOOL_UNDEF1),
    (BOOL_TRUE, BOOL_UNDEF2, BOOL_UNDEF2),
    (BOOL_TRUE, BOOL_TRUE, BOOL_TRUE),
]);

#[cfg(test)]
mod tests {
    use super::*;

    fn idempotent(op: Op222) -> bool {
        for a in 0..3 {
            let a = Bit2(a);
            if op.of(a, a) != a {
                return false;
            }
        }
        true
    }

    fn commutative(op: Op222) -> bool {
        for a in 0..3 {
            let a = Bit2(a);
            for b in 0..3 {
                let b = Bit2(b);
                if op.of(a, b) != op.of(b, a) {
                    return false;
                }
            }
        }
        true
    }

    fn associative(op: Op222) -> bool {
        for a in 0..3 {
            let a = Bit2(a);
            for b in 0..3 {
                let b = Bit2(b);
                for c in 0..3 {
                    let c = Bit2(c);
                    if op.of(op.of(a, b), c) != op.of(a, op.of(b, c)) {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn distributive(op1: Op222, op2: Op222) -> bool {
        for a in 0..3 {
            let a = Bit2(a);
            for b in 0..3 {
                let b = Bit2(b);
                for c in 0..3 {
                    let c = Bit2(c);
                    if op1.of(a, op2.of(b, c)) != op2.of(op1.of(a, b), op1.of(a, c)) {
                        return false;
                    }
                }
            }
        }
        true
    }

    #[test]
    fn laws() {
        assert!(idempotent(BOOL_AND));
        assert!(commutative(BOOL_AND));
        assert!(associative(BOOL_AND));

        assert!(!idempotent(BOOL_OR));
        assert!(commutative(BOOL_OR));
        assert!(associative(BOOL_OR));

        assert!(distributive(BOOL_OR, BOOL_AND));
        assert!(!distributive(BOOL_AND, BOOL_OR));

        for a in 0..3 {
            let a = Bit2(a);
            for b in 0..3 {
                let b = Bit2(b);
                assert_eq!(BOOL_ORNOT.of(a, b), BOOL_OR.of(a, BOOL_NOT.of(b)));
            }
        }
    }
}
