use crate::util::IdPath;
use anyhow::Result;
use solar_parser::ast::import::Selection;
use solar_parser::Ast;
use std::collections::HashMap;
use thiserror::Error;

pub type SymbolResolver = HashMap<String, Vec<IdPath>>;

pub struct Module<'a> {
    // NOTE u32 might be better
    pub project_id: usize,
    /// Set of all file inside this module
    pub files: Vec<FileInfo<'a>>,
    // compiled_functions on module level, because
    //     1.) we need file distinction only for resolving imports
    //     2.) we have a flat hierarchy inside a module.
    // TODO compiled_functions: {name => (args, ret, body)}
}

impl<'a> Module<'a> {
    pub fn new(project_id: usize) -> Self {
        Self {
            project_id,
            files: Vec::new(),
        }
    }

    pub fn add_file(&mut self, file: FileInfo<'a>) {
        self.files.push(file);
    }
}

pub struct FileInfo<'a> {
    // NOTE this might be redundant
    pub filename: String,
    /// Maps individual symbols (e.g. `length`) to paths,
    /// where they should be found in (e.g. std/0.0.1/string/).
    /// It may be, that multiple locations apply.
    /// e.g.
    ///    use std.string.length
    ///    use std.array.length
    /// is valid, expected
    /// and will require resolving from multiple locations.
    pub imports: SymbolResolver,
    pub ast: Ast<'a>,
}

#[derive(Debug, Error)]
pub enum ResolveError<'a> {
    LibNotInDeps {
        // TODO include location...
        // but how?
        libname: String,
    },
    ParseErr(solar_parser::ast::NomErr<'a>),
}

impl std::fmt::Display for ResolveError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolveError::LibNotInDeps { libname } => write!(
                f,
                "imported libraries '{libname}' not found in dependencies"
            ),
            ResolveError::ParseErr(e) => e.fmt(f),
        }
    }
}

impl<'a> FileInfo<'a> {
    pub fn from_code(
        filename: String,
        depmap: &HashMap<String, IdPath>,
        basepath: &IdPath,
        content: &'a str,
    ) -> Result<Self, ResolveError<'a>> {
        let r = Ast::from_source_code(content);
        let Ok(ast) = r else {
            let e = r.err().unwrap();
            return Err(ResolveError::ParseErr(e));
        };
        let imports = resolve_imports(&ast, depmap, basepath)?;

        Ok(FileInfo {
            filename,
            imports,
            ast,
        })
    }
}

/// Resolve all imports from the ast to their global symbols for later lookup.
fn resolve_imports<'a>(
    ast: &Ast<'a>,
    depmap: &HashMap<String, Vec<String>>,
    basepath: &Vec<String>,
) -> Result<SymbolResolver, ResolveError<'a>> {
    let mut imports = HashMap::new();

    for import in ast.imports.iter() {
        // the ID path might be from a library, or from this project.
        // Here we switch based on that.
        let mut path = if import.is_lib {
            // now let's resolve this relative import (e.g. std.types.string) to an absolute path
            // that we can use as global identifier.

            // get the name of the library
            let lib = import.path[0].value;
            // resolve correct version from library
            let Some(lib_path) = depmap.get(lib) else {
        return Err(ResolveError::LibNotInDeps { libname: lib.to_string() });
    };

            // append rest of the import path to the absolute path we just created
            let mut path = lib_path.clone();
            path.extend(import.path[1..].iter().map(String::from));
            path
        } else {
            // e.g.
            // use models.customer

            // basepath is the currently active project
            // and we just concatenate the import to this base path
            basepath
                .iter()
                .cloned()
                .chain(import.path.iter().map(String::from))
                .collect()
        };

        match &import.items {
            Selection::All => {
                unimplemented!("{}\n{}\n{}",
                "found '..' selection.",
                "Needs lookup for all symbols in a library.",
                "Will need to happen eventually anyway, in order to check that every import is valid (and public)"
            );
            }
            Selection::This => {
                // the last symbol of the path was the concrete import item.
                // just pop it of the path, and we're golden.
                let symbol = path
                    .pop()
                    .expect("Concrete symbol to be at the end of import path");
                imports.entry(symbol).or_insert_with(Vec::new).push(path);
            }
            Selection::Items(s) => {
                // Importing multiple symbols from this library.
                // Add them all!
                for symbol in s.iter() {
                    let symbol = symbol.value.to_string();
                    imports
                        .entry(symbol)
                        .or_insert_with(Vec::new)
                        .push(path.clone());
                }
            }
        }
    }

    Ok(imports)
}
