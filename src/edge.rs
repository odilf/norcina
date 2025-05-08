use std::{array, fmt};

use crate::{
    cube::Sticker,
    math::{Axis, Direction, Face},
    mov::{Amount, Move},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edge {
    /// Packed field `---onnba`
    // TODO: Try masked version: `-ommmzyx`
    data: u8,
}

impl Edge {
    #[inline]
    pub const fn a(self) -> Direction {
        Direction::from_bool(self.data & 0b01 != 0)
    }

    #[inline]
    pub const fn b(self) -> Direction {
        Direction::from_bool(self.data & 0b10 != 0)
    }

    pub const fn solved(index: u8) -> Edge {
        assert!(index < 12);
        Edge { data: index }
    }

    #[inline]
    pub const fn is_oriented(self) -> bool {
        //            ---onnba
        self.data & 0b00010000 == 0
    }

    #[inline]
    pub const fn normal(self) -> Axis {
        Axis::from_u8((self.data >> 2) & 0b11)
    }

    pub const fn faces(self) -> [Face; 2] {
        let normal = self.normal();
        let a = Direction::from_bool(self.data & 0b01 != 0);
        let b = Direction::from_bool(self.data & 0b10 != 0);
        [Face::new(normal.next(), a), Face::new(normal.prev(), b)]
    }

    /// Given the origin [`Edge`] and the current `edges`, find the current state
    /// of the given edge.
    pub const fn current(self, edges: &[Edge; 12]) -> Edge {
        let index = self.data & 0b01111;
        edges[index as usize]
    }

    // #[inline]
    // fn face_on_axis(self, axis: Axis) -> Face {
    //     Face::new(axis, self.direction_on_axis(axis))
    // }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EdgePosition {
    data: u8,
}

impl EdgePosition {
    #[inline]
    pub const fn a(self) -> Direction {
        Direction::from_bool(self.data & 0b01 != 0)
    }

    #[inline]
    pub const fn b(self) -> Direction {
        Direction::from_bool(self.data & 0b10 != 0)
    }

    pub fn from_faces([f1, f2]: [Face; 2]) -> Self {
        let normal = Axis::other(f1.axis(), f2.axis());
        let properly_ordered = f1.axis() == normal.next();
        let (a, b) = if properly_ordered {
            (f1.direction(), f2.direction())
        } else {
            (f2.direction(), f1.direction())
        };

        EdgePosition {
            data: (a.u8() << 0) + (b.u8() << 1) + (normal.u8() << 2),
        }
    }

    // TODO: Maybe this should take the array directly. It's just 12 bytes...
    pub fn pick(self, edges: &[Edge; 12]) -> Edge {
        edges[self.data as usize]
    }

    #[inline]
    pub const fn normal(self) -> Axis {
        Axis::from_u8((self.data >> 2) & 0b11)
    }

    pub fn from_index(index: u8) -> Self {
        assert!(index < 12);
        EdgePosition { data: index }
    }

    fn direction_on_axis(self, axis: Axis) -> Direction {
        assert_ne!(self.normal(), axis);
        if self.normal().next() == axis {
            self.a()
        } else {
            self.b()
        }
    }

    fn face_on_axis(self, axis: Axis) -> Face {
        Face::new(axis, self.direction_on_axis(axis))
    }

    pub fn orientation_axis(self) -> Axis {
        if self.normal() == Axis::Y {
            Axis::Z
        } else {
            Axis::Y
        }
    }

    pub fn non_orientation_axis(self) -> Axis {
        match self.normal() {
            Axis::X => Axis::Z,
            Axis::Y => Axis::X,
            Axis::Z => Axis::X,
        }
    }

    pub fn orientation_face(self) -> Face {
        self.face_on_axis(self.orientation_axis())
    }

    pub fn other_face(self) -> Face {
        self.face_on_axis(self.non_orientation_axis())
    }

    pub fn contains_face(self, face: Face) -> bool {
        face.axis() != self.normal() && face.direction() == self.direction_on_axis(face.axis())
    }
}

pub fn sticker(edge: Edge, position: EdgePosition, face: Face) -> Sticker {
    // TODO: Implement orientation.
    assert_ne!(face.axis(), position.normal());
    if face.axis() == position.normal().next() {
        Face::new(edge.normal().next(), edge.a())
    } else {
        Face::new(edge.normal().prev(), edge.b())
    }
}

pub fn move_pieces(edges: [Edge; 12], mov: Move) -> [Edge; 12] {
    array::from_fn(|i| {
        let position = EdgePosition::from_index(i as u8);
        let (dir_mov, other_axis_offset) = if mov.face().axis() == position.normal().next() {
            (position.a(), 1)
        } else if mov.face().axis() == position.normal().prev() {
            (position.b(), 0)
        } else {
            return edges[i];
        };

        if dir_mov != mov.face().direction() {
            return edges[i];
        }

        if !matches!(mov.amount(), Amount::Double) {
            todo!();
        }

        edges[(i ^ (0b1 << other_axis_offset)) as usize]
    })
}

impl fmt::Display for Edge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [a, b] = self.faces();
        write!(
            f,
            "{a:?}{b:?} ({})",
            if self.is_oriented() { '✓' } else { 'x' }
        )
    }
}

impl fmt::Display for EdgePosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: Bodge
        let [a, b] = Edge { data: self.data }.faces();
        write!(f, "{a:?}{b:?}")
    }
}

#[cfg(test)]
mod tests {
    use quickcheck::{Arbitrary, Gen};

    use super::*;

    impl Arbitrary for Edge {
        fn arbitrary(g: &mut Gen) -> Self {
            let normal = Axis::arbitrary(g);
            let a = Direction::arbitrary(g);
            let b = Direction::arbitrary(g);
            Edge {
                data: (a.u8() << 0) + (b.u8() << 1) + (normal.u8() << 2),
            }
        }
    }
}
