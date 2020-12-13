Flagrant
========

A program for drawing flags, written for a talk at the Func Prog 5 meetup in
Stockholm.

Flag of Madagascar:

```
cargo run -- "(h 1 (s w) 2 (v 1 (s r) 1 (s g)))"
```

Flag of France:

```
cargo run -- "(h 1 (s b) 1 (s w) 1 (s r))"
```

Flag of Sweden:

```
cargo run -- "
(v
    2 (t top
        (h
            2 (s b)
            1 (s y)
            3 (s b)))
    1 (s y)
    2 (r top))
"
```

The Flag Definition Language (tm) is based on S-expressions on the following
format:

 * `(s b)` - a solid, where the second letter denotes the color.
 * `(v x y p)` - a vertical split, where `x` and `y` are nested expressions and
   `p` is the percentage of the available space to allocate to `x`.
 * `(h x y p)` - a horizontal split that works like the vertical split above
 * `(t tag subexpr)` - tag a subexpr with the name `tag`
 * `(r tag)` - reference a subexpr named `tag`

The following colors are supported:

 * `b` - blue
 * `g` - green
 * `r` - red
 * `w` - white
 * `y` - yellow
 * `s` - black

