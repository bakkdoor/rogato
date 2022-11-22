# rogātō ⌘ 🏷

## A functional, declarative query-oriented programming language with an integrated graph database.

🛠 (Still WIP) 🛠

[![CI](https://github.com/bakkdoor/rogato/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/bakkdoor/rogato/actions/workflows/ci.yml)

**rogātō** aims to provide a simple yet powerful functional programming interface to a built-in graph database, powered by [indradb](https://github.com/indradb/indradb).

The syntax is inspired mostly by languages like Elm, F#, Haskell and Elixir. The goal is to provide a basic standard library for common tasks. The language is still at a very early stage, so a lot is still missing.

More information will follow here as the language matures.
For some example source code and syntax ideas (still WIP), check out the code examples under [examples/](examples/) or the standard library under [lib/](lib/).

### Language design goals

* Simple, typed, memory-safe functional core language for data processing
* Direct native access to built-in graph database
* First-class query manipulation support against graph db
* Symbolic language capabilities for easy knowledge graph construction and querying
* Easy importing of data from various data formats, ideally provided by std lib

### Build instructions

#### LLVM v14 Dependency

*rogātō* depends on LLVM version 14 to build. LLVM is used by the [`compiler`](compiler/) crate to generate optimized native machine code.

LLVM website with documentation and installation instructions:
- https://llvm.org

Latest compatible LLVM release:
- https://github.com/llvm/llvm-project/releases/tag/llvmorg-14.0.6

#### Build without optimizations (debug mode)

Build with cargo, Rust's package and build manager:

    cargo b

#### Build with optimizations (release mode)

    cargo b -r

#### Build and run tests (debug/test mode)

    cargo t

#### Build with parser tracing enabled

    cargo b --features trace

This works also for release builds:

    cargo b -r --features trace

To get parser tracing viz via pegviz, run:

    ./target/release/rogato examples/query.roga | pegviz --output pegviz.html

Then open pegviz.html to view parser tracing visualization.

### Q&A

#### Why?

I needed a project to learn Rust with. I always have been interested in graph databases and their applications to data problems and have been fascinated by what declarative programming can do for simplicity in querying large data sets.  
**rogātō** aims to unify some ideas and properties from functional and logic programming as well as graph databases into something new and cohesive that is fun and productive to use and can aid in discovering interesting facts about large data sets.

#### What does the name mean?

**rogātō** is the ablative case of the Latin perfect passive participle of [***rogātus***](https://en.wiktionary.org/wiki/rogatus#Latin) (having been enquired / asked / requested / invited).

###### How is it pronounced?
###### <sup>The a and final o (marked with macrons) are pronounced twice as long as the first o.</sup>

