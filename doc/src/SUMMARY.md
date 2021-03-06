The Rhai Scripting Language
==========================

1. [What is Rhai](about/index.md)
   1. [Features](about/features.md)
   2. [Supported Targets and Builds](about/targets.md)
   3. [What Rhai Isn't](about/non-design.md)
   4. [Licensing](about/license.md)
   5. [Related Resources](about/related.md)
2. [Getting Started](start/index.md)
   1. [Online Playground](start/playground.md)
   2. [Install the Rhai Crate](start/install.md)
   3. [Optional Features](start/features.md)
   4. [Special Builds](start/builds/index.md)
      1. [Performance](start/builds/performance.md)
      2. [Minimal](start/builds/minimal.md)
      3. [no-std](start/builds/no-std.md)
      4. [WebAssembly (WASM)](start/builds/wasm.md)
   5. [Examples](start/examples/index.md)
      1. [Rust](start/examples/rust.md)
      2. [Scripts](start/examples/scripts.md)
3. [Using the `Engine`](engine/index.md)
   1. [Hello World in Rhai - Evaluate a Script](engine/hello-world.md)
   2. [Compile to AST for Repeated Evaluations](engine/compile.md)
   3. [Call a Rhai Function from Rust](engine/call-fn.md)
   4. [Create a Rust Closure from a Rhai Function](engine/func.md)
   5. [Evaluate Expressions Only](engine/expressions.md)
   6. [Raw Engine](engine/raw.md)
   7. [Scope - Initializing and Maintaining State](engine/scope.md)
   8. [Engine Configuration Options](engine/options.md)
4. [Extend Rhai with Rust](rust/index.md)
   1. [Traits](rust/traits.md)
   2. [Register a Rust Function](rust/functions.md)
      1. [String Parameters in Rust Functions](rust/strings.md)
   3. [Register a Generic Rust Function](rust/generic.md)
   4. [Register a Fallible Rust Function](rust/fallible.md)
   6. [Override a Built-in Function](rust/override.md)
   7. [Operator Overloading](rust/operators.md)
   8. [Register a Custom Type and its Methods](rust/custom.md)
      1. [Getters and Setters](rust/getters-setters.md)
      2. [Indexers](rust/indexers.md)
      3. [Disable Custom Types](rust/disable-custom.md)
      4. [Printing Custom Types](rust/print-custom.md)
   9. [Packages](rust/packages/index.md)
      1. [Built-in Packages](rust/packages/builtin.md)
      2. [Load a Plugin Module as a Package](rust/packages/plugin.md)
      3. [Manually Create a Custom Package](rust/packages/create.md)
   10. [Plugins](plugins/index.md)
       1. [Export a Rust Module](plugins/module.md)
       2. [Export a Rust Function](plugins/function.md)
5. [Rhai Language Reference](language/index.md)
   1. [Comments](language/comments.md)
   2. [Values and Types](language/values-and-types.md)
      1. [Dynamic Values](language/dynamic.md)
      2. [type_of()](language/type-of.md)
      3. [Numbers](language/numbers.md)
         1. [Operators](language/num-op.md)
         2. [Functions](language/num-fn.md)
         3. [Value Conversions](language/convert.md)
      4. [Strings and Characters](language/strings-chars.md)
         1. [Built-in Functions](language/string-fn.md)
      5. [Arrays](language/arrays.md)
      6. [Object Maps](language/object-maps.md)
          1. [Parse from JSON](language/json.md)
          2. [Special Support for OOP](language/object-maps-oop.md)
      7. [Time-Stamps](language/timestamps.md)
   3. [Keywords](language/keywords.md)
   4. [Statements](language/statements.md)
   5. [Variables](language/variables.md)
   6. [Constants](language/constants.md)
   7. [Logic Operators](language/logic.md)
   8. [Other Operators](language/other-op.md)
   9. [If Statement](language/if.md)
   10. [While Loop](language/while.md)
   11. [Loop Statement](language/loop.md)
   12. [For Loop](language/for.md)
   13. [Return Values](language/return.md)
   14. [Throw Exception on Error](language/throw.md)
   15. [Functions](language/functions.md)
       1. [Call Method as Function](language/method.md)
       2. [Overloading](language/overload.md)
       3. [Namespaces](language/fn-namespaces.md)
       4. [Function Pointers](language/fn-ptr.md)
       5. [Anonymous Functions](language/fn-anon.md)
       6. [Currying](language/fn-curry.md)
       7. [Closures](language/fn-closure.md)
   16. [Print and Debug](language/print-debug.md)
   17. [Modules](language/modules/index.md)
       1. [Export Variables, Functions and Sub-Modules](language/modules/export.md)
       2. [Import Modules](language/modules/import.md)
       3. [Create from Rust](rust/modules/create.md)
       4. [Create from AST](language/modules/ast.md)
       5. [Module Resolvers](rust/modules/resolvers.md)
          1. [Custom Implementation](rust/modules/imp-resolver.md)
   18. [Eval Statement](language/eval.md)
6. [Safety and Protection](safety/index.md)
   1. [Checked Arithmetic](safety/checked.md)
   2. [Sand-Boxing](safety/sandbox.md)
   3. [Maximum Length of Strings](safety/max-string-size.md)
   4. [Maximum Size of Arrays](safety/max-array-size.md)
   5. [Maximum Size of Object Maps](safety/max-map-size.md)
   6. [Maximum Number of Operations](safety/max-operations.md)
      1. [Tracking Progress and Force-Termination](safety/progress.md)
   7. [Maximum Number of Modules](safety/max-modules.md)
   8. [Maximum Call Stack Depth](safety/max-call-stack.md)
   9. [Maximum Statement Depth](safety/max-stmt-depth.md)
7. [Advanced Topics](advanced.md)
   1. [Advanced Patterns](patterns/index.md)
      1. [Object-Oriented Programming (OOP)](patterns/oop.md)
      2. [Loadable Configuration](patterns/config.md)
      3. [Control Layer](patterns/control.md)
      4. [Singleton Command](patterns/singleton.md)
      5. [One Engine Instance Per Call](patterns/parallel.md)
   2. [Capture Scope for Function Call](language/fn-capture.md)
   3. [Serialization/Deserialization of `Dynamic` with `serde`](rust/serde.md)
   4. [Script Optimization](engine/optimize/index.md)
      1. [Optimization Levels](engine/optimize/optimize-levels.md)
      2. [Re-Optimize an AST](engine/optimize/reoptimize.md)
      3. [Eager Function Evaluation](engine/optimize/eager.md)
      4. [Side-Effect Considerations](engine/optimize/side-effects.md)
      5. [Volatility Considerations](engine/optimize/volatility.md)
      6. [Subtle Semantic Changes](engine/optimize/semantics.md)
   5. [Low-Level API](rust/register-raw.md)
   6. [Use as DSL](engine/dsl.md)
      1. [Disable Keywords and/or Operators](engine/disable.md)
      2. [Custom Operators](engine/custom-op.md)
      3. [Extending with Custom Syntax](engine/custom-syntax.md)
   7. [Multiple Instantiation](patterns/multiple.md)
8. [Appendix](appendix/index.md)
   1. [Keywords](appendix/keywords.md)
   2. [Operators and Symbols](appendix/operators.md)
   3. [Literals](appendix/literals.md)
