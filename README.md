# rogātō
## Functional, declarative query language with integrated graph database.

**rogātō** provides a simple yet powerful functional programming interface to a built-in graph database, powered by [indradb](https://github.com/indradb/indradb).

The syntax is inspired mostly by languages like Elm, F#, Haskell and Elixir. The goal is to provide a basic standard library for common tasks. A package manager should also follow, but the language is still at a very early stage, so all of that is still missing.

More information will follow here as the language matures.
For some example source code and syntax ideas (still WIP), check out the code examples under [examples/](examples/).

### Language design goals

* Simple, typed, functional core language for data processing
* Direct native access to built-in graph database
* First-class query manipulation support against graph db
* Symbolic language capabilities for easy knowledge graph construction and querying
