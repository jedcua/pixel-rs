## Design Principles (from *A Philosophy of Software Design*, John Ousterhout)

The goal of every change is to reduce complexity. Complexity is anything in the structure of the code that makes it hard to understand or modify. It shows up in three ways:
- **Change amplification:** a simple change touches many places.
- **Cognitive load:** you must know a lot to make a change safely.
- **Unknown unknowns:** it isn't obvious what needs to change.

Complexity is caused by dependencies and obscurity. It accumulates in small increments, so no change is too small to matter.

### Work strategically, not tactically
- Working code is not enough. Each change should leave the design at least as clean as you found it.
- Don't add a special case, flag, or duplicated block just to get something working fast. If the clean fix needs a small refactor, do it and say so.
- When modifying code, ask: "If the system had been designed with this change in mind from the start, what would it look like?" Move toward that.

### Make modules deep
- A good module (function, class, file, service) has a simple interface that hides a substantial implementation. Judge it by how much it hides compared with how much its interface costs to learn.
- Avoid shallow modules: thin wrappers, helpers that only rename a call, and classes that just forward to another class.
- Don't split functions or classes merely to make them shorter. Split only when each piece is a clean abstraction that can be understood on its own. Depth matters, not length.
- Prefer fewer, more capable methods over many narrow, special-purpose ones.

### Hide information
- Each module should encapsulate its design decisions (data formats, algorithms, protocols) so other modules don't depend on them.
- **Information leakage** is when the same knowledge, such as a file format, is embedded in two places. Treat it as a red flag and consolidate that knowledge in one module.
- Don't structure code by the order operations happen in time (read, then parse, then write) if that spreads one piece of knowledge across several modules. Structure code by knowledge instead.
- Expose the minimum. Common cases should work without knowing about rare options, so provide sensible defaults.

### Prefer somewhat general-purpose interfaces
- Make the interface general enough to serve several uses, even if the implementation covers only today's needs.
- Ask: "What is the simplest interface that covers all current needs?" and "In how many situations will this be used?"
- Keep special-case logic out of general-purpose code. Push it up to callers or into a separate module.

### Give each layer a different abstraction
- Adjacent layers should offer different abstractions.
- Pass-through methods are red flags. These have the same signature as the method they call and add nothing.
- Pass-through variables are also red flags. These are threaded through many layers that don't use them.

### Pull complexity downward
- If complexity is unavoidable, handle it inside the module rather than pushing it to callers. The implementer deals with it once, while every caller would deal with it repeatedly.
- Don't add configuration parameters to avoid making a decision. Expose an option only when callers genuinely have information the module lacks.

### Define errors out of existence
- Error paths are a major source of complexity. Where possible, design APIs so the error can't happen. For example, deleting something that doesn't exist can simply be a no-op.
- Handle low-level errors inside a module when it can, and handle errors in one place rather than at every call site.
- For rare, unrecoverable conditions, failing fast with a clear message is often better than elaborate recovery code.
- Don't throw an exception just because you're unsure what to do. That pushes the decision onto every caller.

### Design it twice
- For any non-trivial interface, consider at least two substantially different approaches before implementing.
- Briefly note the alternative you rejected and why.

### Comments
- Write a comment only if it is necessary. First make the code clear through names, types, and signatures.
- A comment is necessary when the code cannot show the intent, the reason for a decision, an invariant, a unit, or a boundary condition.
- Do not write an interface comment on each public function or class. Write one only if the signature cannot show a precondition, a side effect, or an error that the caller must know. It states what the code does, not how.
- Don't write comments that repeat the code. A comment that reuses the words of the name it describes adds nothing.
- Implementation comments explain *what* and *why* at a higher level than the code. They do not walk through it line by line.
- If a short, clear comment is hard to write, the design is probably wrong.
- Keep comments next to the code they describe and update them in the same change.

### Names
- Use precise names that create a clear picture of what the thing is.
- Avoid vague names like `data`, `info`, `result`, `tmp`, and `manager`, except in tiny scopes.
- Use one name per concept everywhere, and never reuse a name for a different concept.
- If it's hard to find a precise name, take that as a sign the design is muddled.

### Consistency and obviousness
- Follow the codebase's existing conventions for naming, structure, error handling, and patterns, even if you'd prefer another. Don't introduce a new style for one change.
- Code should be obvious, meaning a reader's first guesses about it are correct.
- Avoid things that hide meaning:
    - tuples or generic `Pair` types for structured data
    - undocumented event-driven control flow
    - code that behaves differently than a reader would expect

### Use popular practices carefully
- Prefer composition over implementation inheritance, which couples parent and child classes.
- Develop incrementally, but make the increments abstractions, not just features. When you need an abstraction, build it fully rather than in pieces.
- Tests are essential and make refactoring safe, but passing tests is not evidence of good design.
- Avoid reflexive getters and setters. They make a class shallow and expose its internals.

### Performance
- Choose naturally efficient designs from the start, such as the right data structures and no needless copying.
- Measure before optimizing. When you do optimize, design around the critical path for the common case and keep it simple.

### Red flags to check before finishing
- **Shallow module:** the interface is about as complex as the implementation.
- **Information leakage:** the same knowledge lives in two modules.
- **Temporal decomposition:** code is structured by execution order instead of by knowledge.
- **Overexposure:** common uses require knowing about rare features.
- **Pass-through method:** a method only forwards to another with the same signature.
- **Repetition:** the same code or logic appears in multiple places.
- **Special-general mixture:** special-case logic is tangled into general-purpose code.
- **Conjoined methods:** you can't understand one method without reading another.
- **Comment repeats code:** the comment adds nothing beyond the code itself.
- **Implementation details in interface docs:** callers are told how, not just what.
- **Vague name:** the name doesn't say precisely what the thing is.
- **Hard-to-pick name:** no precise name fits, which suggests a muddled design.
- **Hard to describe:** the documentation needs to be long to be complete.
- **Nonobvious code:** a reader can't understand it quickly.