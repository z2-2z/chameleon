# chameleon

Re-implementation of [Gramatron](https://github.com/HexHive/Gramatron) with an emphasis on performance and ease-of-use.

Chameleon builds on the theoretical foundations of Gramatron. It converts a context-free grammar to
a push-down automaton and uses it to generate/mutate inputs. However, while other Gramatron-based tools
realize the PDA as data (e.g. in form of a matrix), this tool translates the production rules of a grammar
into code and produces a mutation and generation procedure in C. This achieves >= 3x more performance.

Chameleon offers
- Support for text AND binary content
- Grammar-based mutations that are hard-coded into generated C code
- LibAFL components to use the code in a fuzzer

## Grammars
Chameleon uses its own grammar syntax and requires the grammars to have the `.chm` extension.
See [json.chm](test-data/grammars/json.chm) for a real-world example of a grammar that generates
valid JSON.

## How to use
1. Write a grammar
2. Use `chameleon translate` to generate code
   ```
   Usage: chameleon translate [OPTIONS] --output <OUTPUT> <GRAMMARS>...

   Arguments:
     <GRAMMARS>...  Paths to .chm grammar files

   Options:
     -e, --entrypoint <ENTRYPOINT>  Sets the entrypoint for the grammar
     -v, --verbose                  Enable verbose logging
     -b, --baby                     Enable "baby" mode to output just a simple generator and not a full mutation procedure
     -p, --prefix <PREFIX>          Sets a prefix for the generated function names
     -o, --output <OUTPUT>          Name of resulting .c file
   ```
3. Either
    - Use the C code directly in your project via `include/chameleon.h`
    - Compile C code into a `.so` and use it with Chameleon's LibAFL components
      ```rs
      let chameleon = Chameleon::load("path/to/mutator.so");
      let generator = ChameleonGenerator::new(chameleon); // LibAFL Generator
      let mutator = ChameleonMutator::new(chameleon); // LibAFL Mutator
      ```
