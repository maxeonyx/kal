# kal

The kal language is my idea for a simple dynamically typed language.

```rust

// JS-like object literals
let obj = {
    dogs: 5,
    cats: 9,
};

// Short `fn` keyword for functions
let get_num_cats = fn() {
    // Implicit return at the end of a function.
    obj.cats
};

// Single-expression function literals
let get_num_dogs = fn() obj.dogs;

// Files are just zero-argument functions, and also have implicit returns.
// Exporting is done by returning a value.
{
    get_num_dogs,
    get_num_cats,
}
```

I am drawing inspiration mainly from modern JS and Rust, with some Python and Lua. eg.

- [x] Dynamic type system (JS/Python/Lua)
- [ ] Explicit mutability (Rust)
- [x] No garbage collection, only reference counting (Rust)
- [x] Object/List spread operators (JS)
- [ ] Implicit cast to big integers on overflow (Python)
- [x] No implicit `this` or `self` parameter (Lua)
- [ ] Symbols for private fields and language-defined behaviour (JS)

Unlike all of these languages, I have implemented an effects system - think of it like generalized exceptions, which can be resumed. I am planning to use this to implement a mix of built-in monad instances - namely Async, Try and Yield. (Including any mix of those, making the language suitable for reactive programming). The effects system will also allow fully encapsulating libraries, as they can only talk to the outside world (Files, networks etc.) through "Runtime Requests", a.k.a the IO monad. Effects will implicitly bubble up through the program to the runtime, or can be caught and handled in a custom way.

I would ideally like the language to be able to be syntax-agnostic. This would mean providing a stable API for the AST, and having a pluggable parser. A future project might be implementing a visual scripting language on top of that.

I would also like to provide a simple wrapper library that allows users to statically "compile" their kal code. What this would mean is producing a single executable which bundles the kal interpreter with all of the users code (via `include_str!()` or similar). This executable therefore has very few runtime dependencies like a normal rust binary, and can be distributed more easily.

So my wishlist is as follows:
- Dynamic language
- Choose your own syntax
- Built-in common monads
- Single "binary" output

And my todo list is as follows:
- [x] Functions
- [x] Let bindings
- [ ] Let patterns
- [x] Addition, multiplication, subtraction, division
- [x] Comparisons, boolean operators
- [x] If expressions
- [ ] Forever loops
- [ ] Foreach loops
- [x] Lists
- [x] Objects
- [ ] Strings
- [x] Symbols
- [x] Effects (`send` / `handle` / `resume`)
- [ ] Intrinsics (language-defined functions)
- [x] Mutable let bindings
- [ ] Mutable assignment operators
- [ ] Mutable function parameters
- [ ] Mutable object keys and list elements
- [ ] Import / export
- [ ] Print
- [x] List spread operator
- [ ] Object spread operator
- [ ] Function param spread (in / out)
- [ ] Proper error support for type errors.
- [ ] Proper error support for syntax errors.
- [x] Non-recursive, higher performance interpreter
- [x] Replace KalRef with Rc
- [ ] BigInts and coercion on over/underflow
- [ ] Floating point numbers
- [ ] Markdown-like comments with `#` symbol
- [ ] Doc comments and built-in `help` function
- [ ] CLI binary
- [ ] Embeddable Rust library
- [ ] JS/WASM runtime
- [ ] Native runtime 
