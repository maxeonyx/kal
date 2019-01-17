# kal

the kal language is my idea for a simple dynamically typed language similar to Javascript or Lua

I am drawing syntax mainly from modern JS and Rust. But I would ideally like the language to be
able to be syntax-agnostic. This would mean providing a stable API for the AST. A future project might be implementing a visual scripting language on top of that. 

I am planning to implement a mix of built-in monad instances - namely Async, Try and Yield. (And ideally any mix of those, making the language suitable for reactive programming).

I would also like to provide a simple wrapper library that allows users to statically "compile" their kal code. What this would mean is producing a single executable which bundles the kal interpreter with all of the users code (via `include_str!()` or similar). This executable therefore has very few runtime dependencies like a normal rust binary, and can be distributed more easily.

I am very interested with an idea about using the rust compiler as a target for other languages such as this one, providing easy interop between statically compiled code and more specific or high-level languages. See this thread https://internals.rust-lang.org/t/pre-rfc-first-class-support-for-compile-to-rust-languages/7610

After I get the interpreter running, I plan to experiment with [HolyJIT](https://github.com/nbp/holyjit) (Although from what I can tell it is very much experimental).

Lastly I have an idea for native interop with a git merge plugin, in order to more easily merge kal code. This would define git merge semantics for lists, object literals, function bodies, if statements etc. allowing smarter resolution of git merges. Essentially a provider for [Semantic Merge](https://www.semanticmerge.com/), although I am not fond of the fact that it is proprietary.

So my wishlist is as follows:
- Dynamic language
- Choose your own syntax
- Built-in common monads
- Single "binary" output
- Compile-to-rust via rustc plugin
- Experiment with JIT builder
- Semantic merge provider
