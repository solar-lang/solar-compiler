use crate::util;
use crate::util::FindError;
use crate::Value;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Mutex;
use thiserror::Error;

use solar_parser::{ast, ast::expr::FullExpression, Ast};

pub struct InterpreterContext {
    pub stdout: Mutex<Box<dyn std::io::Write>>,
    pub stdin: Mutex<Box<dyn std::io::Read>>,
}

pub struct Context<'a> {
    pub sources: HashMap<Vec<String>, Ast<'a>>,
    pub interpreter_ctx: InterpreterContext,
}

pub struct FileContext<'a> {
    // Base identifier for this file.
    pub this: Vec<String>,
    pub ctx: Context<'a>,
    // imports
    // pub imports: HashMap<String, Import>, // Symbols inside the file
    // global_scope: HashMap<String, Value>,
}

// pub enum Import {
/// Helps resolving imports to other modules
/// e.g. std.collections
// Module(Vec<String>),

/// Concrete Symbol.
/// A Function or a global.
/// Stored is the ID to the entire thing.
// Symbol(Vec<String>),
// }

impl<'a> Deref for FileContext<'a> {
    type Target = Context<'a>;
    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}

impl<'a> FileContext<'a> {
    /// Returns the AST of this file
    fn ast(&self) -> &Ast<'a> {
        self.resolve_ast(&self.this)
            .expect("current AST to always be valid")
    }

    fn resolve_ast(&self, path: &[String]) -> Option<&Ast<'a>> {
        self.sources.get(path)
    }

    /// Checks, whether supplied function call is a buildin function
    /// buildin functions behave quite different from values in some respect,
    /// which is fine. They will be hidden away in the stdlib.
    /// Returns None, if the supplied function call does not address a buildin function.
    fn check_buildin_func(
        &'a self,
        func: &ast::expr::FunctionCall,
        args: &[Value<'a>],
    ) -> Option<Result<Value<'a>, EvalError>> {
        if func.function_name.value.len() != 1 {
            return None;
        }

        let fname = func.function_name.value[0].value;

        if !fname.starts_with("buildin_") && !fname.starts_with("Buildin_") {
            return None;
        }

        // cut off "buildin_" or "Buildin_"
        let shortened = &fname["buildin_".len()..];

        let res = match shortened {
            "str_concat" => self.buildin_str_concat(args),
            "identity" => self.buildin_identity(args),
            "readline" => self.buildin_readline(args),
            "print" => self.buildin_print(args),

            _ => Err(EvalError::WrongBuildin {
                found: fname.to_string(),
            }),
        };

        Some(res)
    }

    fn buildin_str_concat(&self, args: &[Value]) -> Result<Value, EvalError> {
        let mut s = String::new();

        for arg in args {
            match arg {
                Value::String(arg) => s.push_str(arg),
                _ => {
                    return Err(EvalError::TypeError {
                        got: arg.type_as_str().to_string(),
                        wanted: "String".to_string(),
                    })
                }
            }
        }

        Ok(s.into())
    }

    fn buildin_print(&self, args: &[Value]) -> Result<Value, EvalError> {
        // allowed overloadings:
        // [String]
        // []
        for arg in args {
            let mut out = self.interpreter_ctx.stdout.lock().expect("lock stdout");

            write!(out, "{arg}").expect("write to stdout");
            out.flush().expect("write to stdout");
        }

        Ok(Value::Void)
    }

    fn buildin_identity(&'a self, args: &[Value<'a>]) -> Result<Value<'a>, EvalError> {
        // only the identiy overloading is implemented for now.
        if args.len() != 1 {
            panic!("& is only implemented with 1 argument");
        }

        Ok(args[0].clone())
    }

    fn buildin_readline(&self, args: &[Value]) -> Result<Value, EvalError> {
        // allowed overloadings:
        // [String]
        // []
        if !args.is_empty() {
            if args.len() > 1 {
                panic!("Expected 1 argument of type string to buildin_readline");
            }

            let s = if let Value::String(s) = &args[0] {
                s
            } else {
                panic!("Expected argument to buildin_readline to be of type string");
            };

            let mut out = self.interpreter_ctx.stdout.lock().expect("lock stdout");

            write!(out, "{s}").expect("write to stdout");
            out.flush().expect("flush stdout");
        }

        let mut r = self
            .interpreter_ctx
            .stdin
            .lock()
            .expect("lock standart input");
        let mut s = Vec::new();

        loop {
            // read exactly one character
            let mut buf = [0];
            r.read_exact(&mut buf).expect("read from input");

            // grab buffer as character
            let b = buf[0];

            if b == b'\n' {
                break;
            }

            s.push(b)
        }

        let s = String::from_utf8(s).expect("parse stdin as a string");
        Ok(s.into())
    }

    pub fn find_main(&self) -> Result<&ast::Function, util::FindError> {
        // TODO this might be a value
        let path = Vec::new();
        let ast = self.sources.get(&path).unwrap();

        util::find_in_ast(ast, "main")
    }

    pub fn eval_function(
        &'a self,
        func: &ast::Function,
        args: &[Value<'a>],
    ) -> Result<Value<'a>, EvalError> {
        let mut scope = Scope::new();

        // TODO what to do with the type here?
        for ((ident, _ty), val) in func.args.iter().zip(args) {
            scope.push(ident.value, val.clone());
        }

        self.eval_full_expression(&func.body, &mut scope)
    }

    pub fn eval_full_expression(
        &'a self,
        expr: &FullExpression,
        scope: &mut Scope<'a>,
    ) -> Result<Value, EvalError> {
        match expr {
            FullExpression::Let(expr) => {
                // Insert all let bindings into scope
                // and evaluate their expressions
                for (ident, value) in &expr.definitions {
                    let value = self.eval_full_expression(value, scope)?;
                    scope.push(ident.value, value)
                }

                // We now have readied the scope and are able to evaluate the body

                let v = self.eval_full_expression(&expr.body, scope);

                // Now we remove the let bindings from the scope
                for _ in &expr.definitions {
                    scope.pop();
                }

                v
            }

            FullExpression::Expression(ref expr) => self.eval_minor_expr(expr, scope),
            FullExpression::Concat(expr) => {
                let e = expr.to_expr();
                self.eval_minor_expr(&e, scope)
            }
            expr => panic!("Unexpected type of expression: {expr:#?}"),
        }
    }

    fn eval_minor_expr(
        &'a self,
        expr: &ast::expr::Expression,
        scope: &mut Scope<'a>,
    ) -> Result<Value<'a>, EvalError> {
        match expr {
            ast::expr::Expression::FunctionCall(fc) => {
                // First, evaluate all arguments
                let mut args: Vec<Value> = Vec::with_capacity(fc.args.len());
                for arg in fc.args.iter() {
                    let v = self.eval_sub_expr(&arg.value, scope)?;
                    args.push(v);
                }

                // See, if we're calling a special buildin function
                if let Some(result) = self.check_buildin_func(fc, &args) {
                    return result;
                }

                // Find function name in scope
                let path = util::normalize_path(&fc.function_name);

                // TODO this does not yet check, if the module where the type of the first
                // argument is declared, contains the symbol. This has precedence over imports and declarations.
                let mut symbol_candidates = self.resolve_symbol(&path, scope)?;

                // TODO check all candidates first!
                if symbol_candidates.len() > 1 {
                    panic!("found multiple candidates for {path:?}:\n{symbol_candidates:#?}");
                }

                let symbol: Value = symbol_candidates.pop().unwrap();

                match symbol {
                    // Only evaluate functions directly
                    // otherwise return value
                    Value::AstFunction(func) => self.eval_function(func, &args),
                    // if there are argument supplied to values,
                    // this is definitly and error.
                    v if !args.is_empty() => Err(EvalError::TypeError {
                        got: format!("{v}"),
                        wanted: "fun(...) -> ...".to_string(),
                    }),
                    value => Ok(value),
                }
            }
            ast::expr::Expression::Value(value) => self.eval_sub_expr(value, scope),
        }
    }

    fn eval_sub_expr(
        &'a self,
        expr: &ast::expr::Value,
        scope: &mut Scope<'a>,
    ) -> Result<Value, EvalError> {
        use ast::expr::Literal;
        use ast::expr::Value as V;
        match expr {
            V::Literal(lit) => match lit {
                Literal::StringLiteral(s) => Ok(s.value.to_string().into()),
                Literal::Bool { value, .. } => Ok(Value::Bool(*value)),
                Literal::Int(int) => {
                    let i = util::eval_int(int);
                    if let Err(e) = i {
                        return Err(e.into());
                    }

                    Ok(Value::Int(i.unwrap()))
                }
                Literal::Float(f) => {
                    let f = f.parse::<f64>().expect("float to be in valid f64 form");
                    Ok(Value::Float(f))
                }
            },
            V::FullIdentifier(path) => {
                // Actually, I don't think I want to allow Paths here.
                // just field access.
                // this is likely to be deleted.

                let path = util::normalize_path(path);

                if path.len() != 1 {
                    panic!("no field access like this");
                }

                let mut result = self.resolve_symbol(&path, scope)?;
                if result.len() != 1 {
                    panic!("found multiple results for {path:?}:\n {result:#?}")
                }

                Ok(result.pop().unwrap())
            }
            V::Tuple(expr) => {
                if expr.values.len() > 1 {
                    panic!("tuple values are not ready");
                }
                let expr = &expr.values[0];

                self.eval_full_expression(expr, scope)
            }
            _ => panic!("evaluation not ready for \n{expr:#?}"),
        }
    }

    ///
    /// Returns a set of candidates for the symbol.
    /// Resolving the candidates requires further knowledge.
    ///
    /// how do we find symbols?
    /// 0.) Maybe it's just a symbol in scope
    /// [name] = path => might be symbolic lookup
    ///      if `name` is in scope:
    ///      return `scope[name]`
    ///
    /// candidates := []
    ///
    /// 1.) if the path has only one element,
    ///     we might be doing symbolic lookup in current module.
    ///     No Need to check imports for this.
    ///     But remember, there's a catch.
    /// candidates.append_all(find_inn_module(this_module))
    ///
    /// 2.) see, if the element is from an import
    ///
    /// basepath := imports.contains(path[0])
    /// full_path := basepath ++ path[1..]
    /// now, find the symbol full_path.last() in module fullpath[..(-1)]
    /// module: collection of files (ASTs) in directory and lib
    /// e.g. seek through all ASTs in module
    /// candidates.append_all(find_in_module(full_path))
    ///
    /// return candidates
    fn resolve_symbol(
        &'a self,
        path: &[String],
        // TODO type of first argument is also relevant! Add as argument
        scope: &Scope<'a>,
    ) -> Result<Vec<Value<'a>>, EvalError> {
        // TODO check if it was found before, and return compiled version

        // if the length of the path is > 1, it's guaranteed looking up an import.

        // if there is no path, this might
        // be just a symbol declared earlier
        // via let ... in, or passed as an argument
        if let [name] = path {
            // 0.) See, if it's a symbol in scope.
            // Local scope overrides everything.
            // The scope only holds arguments and let declarations.
            // Only one item will be returned by this.
            if let Some(item) = scope.get(name) {
                // TODO this is the place where we can return references
                // e.g. in order to assign to stuff.
                return Ok(vec![item.clone()]);
            }
        }

        let mut candidates: Vec<Value<'a>> = Vec::new();
        if let [name] = path {
            // if the path is only one element long,
            // we must also look up the local module.
            // that is ALL Asts within this module.
            // TODO requires imports/modules

            // TODO ALSO CHECK ALL OTHER ASTS IN CURRENT MODULE!
            // e.g. self.asts_in_module()
            let ast = self.ast();

            let res = util::find_in_ast(&ast, name)?;
            // if let Ok(found) = util::find_in_ast(&ast, name) {
            //     candidates.push(found);
            // }
            candidates.push(Value::AstFunction(res));
        }

        Ok(candidates)

        // 2.) see, if the element is from an import
        // Note, this might result in a number of candidates to check!
        // E.g.  add(Int, Float) -> Float     declared in local scope
        //       add(Int, Int) -> Int         imported
        //
        // basepath := imports.contains(path[0])
        // full_path := basepath ++ path[1..]
        // now, find the symbol full_path.last() in module fullpath[..(-1)]
        // module: collection of files (ASTs) in directory and lib
        // e.g. seek through all ASTs in module
        // candidates.append_all(find_in_module(full_path))
        //
        // return candidates

        // TODO how to represent the symbols available from a file?
        // TODO make value represent Functions.
        // unimplemented!("resolve imports and scope. Not found {path:?}")
    }
}

#[derive(Debug, Clone, Default)]
/// Logical Scope, optimized for small number of entries.
/// Made so pushing and popping works fine.
pub struct Scope<'a> {
    values: Vec<(String, Value<'a>)>,
}

impl<'a> Scope<'a> {
    pub fn new() -> Self {
        Scope::default()
    }

    pub fn get(&self, name: &str) -> Option<&Value<'a>> {
        self.values.iter().rfind(|(n, _)| n == name).map(|(_, v)| v)
    }

    pub fn push(&mut self, name: &str, value: Value<'a>) {
        self.values.push((name.to_string(), value));
    }

    /// Pops the most recent value out of the scope.
    /// Popping of an empty scope is considered a programming error
    /// and results in a panic.
    pub fn pop(&mut self) -> Value<'a> {
        self.values.pop().expect("find value in local scope").1
    }
}

#[derive(Debug, Error)]
pub enum EvalError {
    IntConversion(#[from] std::num::ParseIntError),
    FindError(#[from] FindError),
    WrongBuildin { found: String },
    TypeError { got: String, wanted: String },
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IntConversion(e) => e.fmt(f),
            Self::FindError(e) => e.fmt(f),
            Self::WrongBuildin { found } => {
                write!(f, "only buildin methods are allowed to start with buildin_ or Buildin_.\n Found {found}.")
            }

            Self::TypeError { got, wanted } => {
                write!(f, "Wrong type supplied. Expected {wanted}, got {got}")
            }
        }
    }
}
