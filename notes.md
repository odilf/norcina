# The problem with edges

Edges are a bit of a pain in the ass, because there doesn't seem to be a nice strucuted way to represent them. `normal + 2 dirs` is fine but it's a hassle to find the direction given an axis.

Basically, it would be nice to be able to, easily, given an axis either
- determine it's normal to the edge
- if it isn't, find the direction along the axis and the direction towards the other.

We can store xyz in the bits to solve solve the 'finding the direction along the axis', but the other one is just as important and hard in this case.

Wait, can we store xzy and a mask?
