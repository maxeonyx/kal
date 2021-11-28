# Kal
#### (Kal lang)
The Kal language is my idea for a simple dynamically typed language. I/O, errors and generators are managed with effects, there 
is no garbage collector (only reference counting is required) and it has my favourite selection of syntax.

I plan to target webassembly, to take advantage of the effect system and no-permission-by-default model.

---

## Basics


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

---

## Effects

Unlike all of these languages, I have implemented an effects system - think of it like generalized exceptions, which can be resumed. 
I am planning to use this to implement a mix of built-in "monad" instances - namely Async, Try and Yield. (Including any mix of those,
 making the language suitable for reactive programming). The effects system will also allow fully encapsulating libraries, as they can 
 only talk to the outside world (Files, networks etc.) through "Runtime Requests", a.k.a the IO monad. Effects will implicitly bubble up 
 through the program to the runtime, or can be caught and handled in a custom way.

```rust

// no special function syntax when defining a generator
fn even_numbers() {
    let mut count = 0;
    // easy infinite loop (from Rust)
    loop {
        // "yield" here is a parameter, and can be any symbol.
        // yield is the built-in symbol used by for loops.
        send yield with count;
        count += 2;
    }
};

for num in even_numbers() {
    log(num);
};

// for loops are just syntactic sugar over a "handle" expression
// this is equivalent to the above "for" loop
handle even_numbers() {
    yield num {
        log(num);
        continue;
    }
};

// bare exec of a function (without a handle {}) will send any effects further
// up the call stack, similar to exceptions in other languages.
even_numbers();

```
---

## TODO List

- [x] Functions
- [x] Let bindings
- [ ] Function name declaration syntax
- [x] Addition, multiplication, subtraction, division
- [x] Comparisons, boolean operators
- [x] If expressions
- [x] Forever loops (`loop` / `break` / `continue`)
- [ ] Foreach loops
- [x] Lists
- [x] Objects
- [ ] Strings
- [x] Symbols
- [x] Effects (`send` / `handle` / `break` / `continue`)
- [x] Intrinsics (language-defined functions)
- [x] Mutable let bindings
- [ ] Mutable assignment operators
- [x] Mutable object values and list elements
- [ ] Non-string object keys
- [ ] Import / export
- [ ] Print
- [x] List spread operator
- [x] Object spread operator
- [x] Function param spread (in / out)
- [x] Let patterns (list destructuring)
- [ ] Let patterns (object destructuring)
- [x] Fn arg patterns (collection args)
- [ ] Proper error support for type errors.
- [ ] Proper error support for syntax errors.
- [x] Non-recursive, higher performance interpreter
- [x] Replace KalRef with Rc
- [ ] Integer division operator
- [ ] Modulo operator
- [ ] Exponent operator
- [ ] BigInts and coercion on over/underflow
- [ ] Numpy-style tensors using the `ndarray` crate
- [ ] Floating point numbers
- [ ] Markdown-like comments with `#` symbol
- [ ] Doc comments and built-in `help` function
- [ ] CLI binary
- [ ] Embeddable Rust library
- [ ] JS/WASM runtime
- [ ] Native runtime

## Wishlist

While my language is dynamic, I would love to take advantage of type information to improve runtime performance. This means
implementing a Gradual Typing system. One example of where this would be particularly useful in my vision is for numpy-style
n-dimensional arrays, which to be efficient should usually contain known types rather than the Value enum. Gradual typing should
also be useful for interop with Rust code, which is an possible future direction.

I would ideally like the language to be able to be syntax-agnostic. This would mean providing a stable API for the AST, and having 
a pluggable parser. A future project might be implementing a visual scripting language on top of that.

I would also like to provide a simple wrapper library that allows users to statically "compile" their kal code. What this would mean 
is producing a single executable which bundles the kal interpreter with all of the users code (via `include_str!()` or similar). This 
executable therefore has very few runtime dependencies like a normal rust binary, and can be distributed more easily.

As a programing language that requires the use of effects for interacting with the outside world, Kal is well-suited for targeting
Webassembly. Webassembly runtimes provide no capabilities by default, but programs written in many languages assume use
of a filesystem, clock or other resources. Kal programs can be run in these environments, and even if they expect side-effects to
suceed, these can be mocked by the runtim or calling Kal code.

So my wishlist is as follows:
- Gradual typing
- Choose your own syntax
- Single "binary" output
- Webassembly
