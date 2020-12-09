Flagrant
========

A program for drawing flags, written for a talk at the Func Prog 5 meetup in
Stockholm.

Flag of Madagascar:

```
cargo run -- "(h 33 (s w) (v 50 (s r) (s g)))"
```

Flag of France:

```
cargo run -- "(h 33 (s b) (h 50 (s w) (s r)))"
```

Flag of Sweden:

```
cargo run -- "
(v 40
    (t top_strip
        (h 40
            (h 65 (s b) (s y))
            (s b)
        )
    )
    (v 30
        (s y)
        (r top_strip)
    )
)"
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

