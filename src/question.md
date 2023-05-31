I am building a falling sand simulation where I want to have different types of sand that each have different properties.

Every instance of a sand particle should have its own velocity, but it's color should be constant/allocated once for a specific type of sand.

I wrote an MVP below where I solved this problem using functions but unsure if it's idiomatic or performant.

I have a feeling that to solve this properly I should be using different structs for each of the `BlockKind` types, then I could benefit from the Stack Overflow answer [here](https://stackoverflow.com/q/26549480), but I'd appreciate direction/review.

Thanks for your help. :)

```
enum BlockKind {
    Empty,
    Concrete(Block),
    Water(Block),
}

struct Block {
    velocity_x: i32,
    velocity_y: i32,
}

impl Block {
    fn new() -> Self {
        Block {
            velocity_x: 0,
            velocity_y: 0,
        }
    }
}

impl BlockKind {
    fn color(&self) -> Rgb<u8> {
        match self {
            BlockKind::Concrete(_) => Rgb([90, 90, 90]),
            BlockKind::Water(_) => Rgb([0, 0, 255]),
            _ => Rgb([0, 0, 0]),
        }
    }
}
```