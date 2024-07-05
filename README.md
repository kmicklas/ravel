# Ravel

Ravel is an experimental approach to UI in Rust with a focus on ergonomics, efficiency, and simplicity.

Semantically, it's in the same family as [Elm](https://elm-lang.org/) and [React](https://react.dev/).
Like [Xilem](https://github.com/linebender/xilem) and [SwiftUI](https://developer.apple.com/xcode/swiftui/), it uses strongly typed view objects with monomorphization to produce efficient code.
Operationally, unlike all of these, view objects are never diffed against each other.
Instead, views have an associated retained state type, which they update directly.

Similar to Xilem, Ravel tries very hard to play nicely with Rust:

- Macros are entirely optional
- Easily write views over borrowed data
- Event handlers can have direct mutable access to state

Currently we only target DOM with WebAssembly.
The design easily generalizes to any retained mode backend framework, and I'd like to support other platforms eventually (especially TUI and mobile).

## Examples

To run the examples, you'll need to install [Trunk](https://trunkrs.dev/).

```shell
cargo install --locked trunk
```

### [Tutorial](examples/tutorial/src/main.rs)

The tutorial provides a pedagogically ordered overview of basic features and application structure.

```shell
trunk serve examples/tutorial/index.html
```

### [TodoMVC](examples/todomvc/src/main.rs)

The implementation of [TodoMVC](https://todomvc.com/) is a slightly more realistic demo.

```shell
trunk serve examples/todomvc/index.html
```

## Roadmap

### Features

- [x] Local state
- [ ] Memoization
- [ ] `async` actions
- [ ] Support "message"/"reducer" architecture rather than direct model mutation
- [ ] More collection types
- [ ] DOM builder macro for convenience/performance
- [ ] Prerendering/hydration
- [ ] Integration with some web framework
- [ ] Modular CSS (export to CSS after dead code elimination with linker hacks)

### Optimization

- [ ] No string serialization for defined elements/attributes
- [ ] Dynamic views without marker DOM comment nodes
- [ ] Fully static string types (no pointer storage/comparison)

### Infrastructure

- [ ] Tests
- [ ] Benchmarks
  - [x] [js-framework-benchmark](https://github.com/krausest/js-framework-benchmark)
  - [ ] [Performance API](https://developer.mozilla.org/en-US/docs/Web/API/Performance_API) integration
