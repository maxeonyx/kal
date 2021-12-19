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

- [x] Functions e.g. `let add = fn (a, b) a + b; add(1, 2)`
- [x] Let bindings
- [x] Function name declaration syntax e.g. `fn add (a, b) { a + b };`
- [x] Addition, multiplication, subtraction, division
- [x] Comparisons, boolean operators e.g. `let is_true = (true and true) xor false;`
- [x] If expressions e.g. `if bool_expr() { } else { };`
- [x] Non-recursive (stack-based), higher performance interpreter.
- [x] Symbols, as in JS. ("Reference capabilities" in the literature) e.g. `let [unique1, unique2] = [symbol(), symbol()]; unique1 != unique2`
- [x] Effects (`send` / `handle` / `break [with <value>]` / `continue [with <value>]`)
- [ ] Explicit effect propagation? e.g. `do` / `do [symbol]`
- [x] Forever loops (`loop` / `break` / `break [with <value>]` / `continue`)
- [ ] Foreach loops (`for <ident> in <generator>` / `break [with <value>]` / `continue [with <value>]`)
- [x] Lists e.g. `[1, 2, 3]`
- [x] Objects e.g. `{ a: 1, b: 2, c: 3 }`
- [ ] Strings
- [x] Intrinsics (language-defined functions)
- [x] Mutable let bindings
- [ ] Mutable assignment operators
- [x] Mutable object values and list elements
- [ ] Non-string object keys
- [ ] Import / export
- [ ] Print
- [x] Patterns
    - [x] List spread operator e.g. `let abcde = ['a', ...bcd, 'e'];`
    - [x] List destructuring in `let` e.g. `let [a, [b, c, d], e] = abcde;`
    - [x] Destructure when receiving function parameters e.g. `fn (a, [b, c, d], e) { };`
    - [x] Spread lists into function calls e.g. `do_alphabet('a', ...[b, c, d], 'e')`
    - [x] Object spread operator e.g. `let thing = { name: 'Thing', ...other_attributes };`
    - [x] Object destructuring in `let` e.g. `let { a, b } = { a: 1, b: 2 };`
    - [x] Nested destructuring in `let` and function params e.g. `let [a, { b, c }, e] = [1, { b: 2, c: 3, d: 4}, 5];`
    - [x] `import` pattern e.g. `let * = { a: 1, b: 2 }; a + 1 == b` 
    - [ ] Renaming in object destructuring e.g. `let { a = x, b = y } = { x: 1, y: 2 };`
- [ ] Proper error support for type errors.
- [ ] Proper error support for syntax errors.
- [x] Replace KalRef with Rc
- [ ] Integer division operator
- [ ] Remainder operator `%`
- [ ] Integer Modulo operator `%%`
- [ ] Exponent operator `**`
- [ ] BigInts, and coercion on over/underflow
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
