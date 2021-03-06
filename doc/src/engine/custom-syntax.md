Extend Rhai with Custom Syntax
=============================

{{#include ../links.md}}


For the ultimate advantageous, there is a built-in facility to _extend_ the Rhai language
with custom-defined _syntax_.

But before going off to define the next weird statement type, heed this warning:


Don't Do It™
------------

Stick with standard language syntax as much as possible.

Having to learn Rhai is bad enough, no sane user would ever want to learn _yet_ another
obscure language syntax just to do something.

Try to use [custom operators] first.  Defining a custom syntax should be considered a _last resort_.


Where This Might Be Useful
-------------------------

* Where an operation is used a _LOT_ and a custom syntax saves a lot of typing.

* Where a custom syntax _significantly_ simplifies the code and _significantly_ enhances understanding of the code's intent.

* Where certain logic cannot be easily encapsulated inside a function.  This is usually the case where _closures_ are required, because Rhai does not have closures.

* Where you just want to confuse your user and make their lives miserable, because you can.


Step One - Design The Syntax
---------------------------

A custom syntax is simply a list of symbols.

These symbol types can be used:

* Standard [keywords]({{rootUrl}}/appendix/keywords.md)

* Standard [operators]({{rootUrl}}/appendix/operators.md#operators).

* Reserved [symbols]({{rootUrl}}/appendix/operators.md#symbols).

* Identifiers following the [variable] naming rules.

* `$expr$` - any valid expression, statement or statement block.

* `$block$` - any valid statement block (i.e. must be enclosed by `'{'` .. `'}'`).

* `$ident$` - any [variable] name.

### The First Symbol Must be a Keyword

There is no specific limit on the combination and sequencing of each symbol type,
except the _first_ symbol which must be a custom keyword that follows the naming rules
of [variables].

The first symbol also cannot be a reserved [keyword], unless that keyword
has been [disabled][disable keywords and operators].

In other words, any valid identifier that is not an active [keyword] will work fine.

### The First Symbol Must be Unique

Rhai uses the _first_ symbol as a clue to parse custom syntax.

Therefore, at any one time, there can only be _one_ custom syntax starting with each unique symbol.

Any new custom syntax definition using the same first symbol simply _overwrites_ the previous one.

### Example

```rust
exec $ident$ <- $expr$ : $block$
```

The above syntax is made up of a stream of symbols:

| Position | Input |  Symbol   | Description                                                                                              |
| :------: | :---: | :-------: | -------------------------------------------------------------------------------------------------------- |
|    1     |       |  `exec`   | custom keyword                                                                                           |
|    2     |   1   | `$ident$` | a variable name                                                                                          |
|    3     |       |   `<-`    | the left-arrow symbol (which is a [reserved symbol]({{rootUrl}}/appendix/operators.md#symbols) in Rhai). |
|    4     |   2   | `$expr$`  | an expression, which may be enclosed with `{` .. `}`, or not.                                            |
|    5     |       |    `:`    | the colon symbol                                                                                         |
|    6     |   3   | `$block$` | a statement block, which must be enclosed with `{` .. `}`.                                               |

This syntax matches the following sample code and generates three inputs (one for each non-keyword):

```rust
// Assuming the 'exec' custom syntax implementation declares the variable 'hello':
let x = exec hello <- foo(1, 2) : {
            hello += bar(hello);
            baz(hello);
        };

print(x);       // variable 'x'  has a value returned by the custom syntax

print(hello);   // variable declared by a custom syntax persists!
```


Step Two - Implementation
-------------------------

Any custom syntax must include an _implementation_ of it.

### Function Signature

The function signature of an implementation is:

> `Fn(engine: &Engine, context: &mut EvalContext, scope: &mut Scope, inputs: &[Expression]) -> Result<Dynamic, Box<EvalAltResult>>`

where:

* `engine: &Engine` - reference to the current [`Engine`].
* `context: &mut EvalContext` - mutable reference to the current evaluation _context_; **do not touch**.
* `scope: &mut Scope` - mutable reference to the current [`Scope`]; variables can be added to it.
* `inputs: &[Expression]` - a list of input expression trees.

#### WARNING - Lark's Vomit

The `context` parameter contains the evaluation _context_ and should not be touched or Bad Things Happen™.
It should simply be passed straight-through the the [`Engine`].

### Access Arguments

The most important argument is `inputs` where the matched identifiers (`$ident$`), expressions/statements (`$expr$`)
and statement blocks (`$block$) are provided.

To access a particular argument, use the following patterns:

| Argument type | Pattern (`n` = slot in `inputs`)         | Result type  | Description        |
| :-----------: | ---------------------------------------- | :----------: | ------------------ |
|   `$ident$`   | `inputs[n].get_variable_name().unwrap()` |    `&str`    | name of a variable |
|   `$expr$`    | `inputs.get(n).unwrap()`                 | `Expression` | an expression tree |
|   `$block$`   | `inputs.get(n).unwrap()`                 | `Expression` | an expression tree |

### Evaluate an Expression Tree

Use the `engine::eval_expression_tree` method to evaluate an expression tree.

```rust
let expr = inputs.get(0).unwrap();
let result = engine.eval_expression_tree(context, scope, expr)?;
```

### Declare Variables

New variables maybe declared (usually with a variable name that is passed in via `$ident$).

It can simply be pushed into the [`scope`].

However, beware that all new variables must be declared _prior_ to evaluating any expression tree.
In other words, any `scope.push(...)` calls must come _before_ any `engine::eval_expression_tree(...)` calls.

```rust
let var_name = inputs[0].get_variable_name().unwrap().to_string();
let expr = inputs.get(1).unwrap();

scope.push(var_name, 0 as INT);     // do this BEFORE 'engine.eval_expression_tree'!

let result = engine.eval_expression_tree(context, scope, expr)?;
```


Step Three - Register the Custom Syntax
--------------------------------------

Use `Engine::register_custom_syntax` to register a custom syntax.

Again, beware that the _first_ symbol must be unique.  If there already exists a custom syntax starting
with that symbol, the previous syntax will be overwritten.

The syntax is passed simply as a slice of `&str`.

```rust
// Custom syntax implementation
fn implementation_func(
    engine: &Engine,
    context: &mut EvalContext,
    scope: &mut Scope,
    inputs: &[Expression]
) -> Result<Dynamic, Box<EvalAltResult>> {
    let var_name = inputs[0].get_variable_name().unwrap().to_string();
    let stmt = inputs.get(1).unwrap();
    let condition = inputs.get(2).unwrap();

    // Push one new variable into the 'scope' BEFORE 'eval_expression_tree'
    scope.push(var_name, 0 as INT);

    loop {
        // Evaluate the statement block
        engine.eval_expression_tree(context, scope, stmt)?;

        // Evaluate the condition expression
        let stop = !engine.eval_expression_tree(context, scope, condition)?
                          .as_bool()
                          .map_err(|_| EvalAltResult::ErrorBooleanArgMismatch(
                              "do-while".into(), expr.position()
                           ))?;

        if stop {
            break;
        }
    }

    Ok(().into())
}

// Register the custom syntax (sample): do |x| -> { x += 1 } while x < 0;
engine.register_custom_syntax(
    &[ "do", "|", "$ident$", "|", "->", "$block$", "while", "$expr$" ], // the custom syntax
    1,  // the number of new variables declared within this custom syntax
    implementation_func
)?;
```


Step Four - Disable Unneeded Statement Types
-------------------------------------------

When a DSL needs a custom syntax, most likely than not it is extremely specialized.
Therefore, many statement types actually may not make sense under the same usage scenario.

So, while at it, better [disable][disable keywords and operators] those built-in keywords
and operators that should not be used by the user.  The would leave only the bare minimum
language surface exposed, together with the custom syntax that is tailor-designed for
the scenario.

A keyword or operator that is disabled can still be used in a custom syntax.

In an extreme case, it is possible to disable _every_ keyword in the language, leaving only
custom syntax (plus possibly expressions).  But again, Don't Do It™ - unless you are certain
of what you're doing.


Step Five - Document
--------------------

For custom syntax, documentation is crucial.

Make sure there are _lots_ of examples for users to follow.


Step Six - Profit!
------------------
