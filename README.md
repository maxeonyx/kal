# kal

Kal is my idea for a simple dynamically typed language similar to Javascript or Lua

I am drawing syntax mainly from modern JS and Rust. But I would ideally like the language to be
able to be syntax-agnostic. This would mean providing a stable API for the AST. A future project might be implementing a visual scripting language on top of that. 

I am planning to implement a mix of built-in monad instances - namely Async, Try and Yield. (And ideally any mix of those, making the language suitable for reactive programming).

After I get the interpreter running, I plan to experiment with [HolyJIT](https://github.com/nbp/holyjit) (Although from what I can tell it is very much experimental).

Lastly I am very interested with an idea about using the rust compiler as a target for other languages such as this one, providing easy interop between statically compiled code and more specific or high-level languages. See this thread https://internals.rust-lang.org/t/pre-rfc-first-class-support-for-compile-to-rust-languages/7610
