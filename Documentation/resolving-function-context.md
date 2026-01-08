
the ast::Function is not enough for evaluation.
the if we call 
i.e.

string.length x

we don't just need the ast of string.length.
We need to context, in which string.length can be resolved internally.

That means we need to also know the import map.

This can be found in the FileInfo.

We also need the Module. That is because the import may also come from within the module.

That means, in order to fully resolve *any given function*, we need:

-  the AST
-  the module info (to find 'implicitly imported' symbols in the same module)
-  the import map (e.g. mapping symbols to import paths)
  - access to the GlobalModules
    - i.e access to the CompilerContext





ctx <- global_context()

module <- get_module(["self"])
f_main <- find(module, "main")

run(ctx, f_main)


should be all you need.
Here, f_main needs to contain info about
- it's module
- it's imports
- it's functions AST

i.e.

```julia
type Function
- module: ModuleInfo
- imports: ImportMap
- ast: Ast::Function
````
