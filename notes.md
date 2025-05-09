# The problem with edges

Edges are a bit of a pain in the ass, because there doesn't seem to be a nice strucuted way to represent them. `normal + 2 dirs` is fine but it's a hassle to find the direction given an axis.

Basically, it would be nice to be able to, easily, given an axis either
- determine whether it's normal to the edge
- if it isn't, find the direction along the axis and the direction towards the other.

We can store xyz in the bits to solve solve the 'finding the direction along the axis', but the other one is just as important and hard in this case.

Wait, can we store xzy and a mask?

We can also store orientation and three 2-bit coordinates that can be -1, 0 or 1. In fact, the more I think about it the more this seems to me to be the correct option.

But it's also extremely useful for the position and the index to be equivalent.

So, is there any way to map the numbers 0-11 in such a way that we can get the coordinates as 3-value numbers?

With normal, a and b it's annoying. Basically, we need to do this:

```rust
impl Edge {
  pub fn axis_coordinate(self, axis: Axis) -> Coord {
    if axis == self.normal() {
      Coord::ZERO
    } else if axis == self.normal().next() {
      Coord::from_u8(self.a())
    } else {
      assert!(axis == self.normal().prev());
      Coord::from_u8(self.b())
    }
  }
}
```

It would be nice to be able to do that with just bit manipulations. Actually, can we just compute the difference??S

```rust
impl Edge {
  pub fn axis_coordinate(self, axis: Axis) -> Coord {
    Coord::from_u8(self.data >> ((3 - axis - self.normal().u8()) % 3) & 0b1)
  }
}
```

Of course that doesn't quite work because we don't differentiate between axis being the normal. But if it's not the normal, it works! Doesn't it? Was I just overcomplicating it? That is almost quite literally what `Axis::next` and `Axis::prev` do!!

Now, this doesn't really help for rotations. I guess we can transform the encoding, do the rotation, transform back. And at the end of the day we really need a compact representation because we use it to index into the array to get positions. But that seems a bit wasteful. We don't need to convert the entire encoding, we can just get the relevant positions, transform them and... I think we have to basically throw out the previous encoding and write it back. But honestly it seems it could be worse! I'm going to try to do that.
