use super::Module;
/// This file contains code
/// for reading in dependencies and libraries.
/// and resolving their imports.
use crate::project::{FileInfo, SolarConfig};
use crate::util::IdPath;
use anyhow::Context;
use std::collections::HashMap;
use walkdir::WalkDir;

/// Contains information on a project,
/// and how to resolve imports inside a project.
#[derive(Debug)]
pub struct Project {
    /// Unique Id of this project
    pub basepath: IdPath,
    /// Root in filesystem
    pub fsroot: String,

    /// mapping needed to resolve imports in a project to actual
    /// dependencies
    pub dep_map: HashMap<String, IdPath>,

    /// Solarconfig of this project
    pub config: SolarConfig,
}

impl Project {
    /// Reads in the .solar file of a project with root `fsroot`.
    pub fn open(
        // The root is the root of the project
        // in the filetree
        fsroot: &str,
        // the basepath is the unique identifier for this project.
        // For libraries it is supposed to match the root.
        basepath: IdPath,
    ) -> anyhow::Result<Project> {
        let solarfile = format!("{fsroot}/solar.yaml");

        // solar config file of the project
        let config = SolarConfig::read(&solarfile)?;
        let fsroot = fsroot.to_string();

        let dep_map = config
            .deps()
            .into_iter()
            .map(|d| {
                let value = d.basepath();
                let key = d.name;
                (key, value)
            })
            .collect();

        Ok(Project {
            basepath,
            fsroot,
            dep_map,
            config,
        })
    }

    /// Reads and parses
    /// all solarfiles, organized into modules,
    /// into memory.
    /// the project id is supposed to be a reference to this exact [Project].
    ///
    /// Returns a Mapping of  {ModulePath => Module}
    pub fn read_all(&self, project_id: usize) -> anyhow::Result<HashMap<IdPath, Module<'_>>> {
        let mut map = HashMap::new();

        for entry in WalkDir::new(&self.fsroot) {
            let Ok(entry) = entry else {
                eprintln!("error walking directory: {entry:?}");
                continue;
            };

            if !entry.file_type().is_file() {
                continue;
            }

            let filename = entry.file_name().to_str().expect("read filename");
            if !filename.ends_with(".sol") {
                continue;
            }

            let path = entry.path();
            // We need to strip the path,
            // because we don't care about the root file system
            let filepath = path
                .strip_prefix(&self.fsroot)
                .expect("to strip common prefix of filepath");

            // absolute id path.
            let mut idmodule = self
                .basepath
                .iter()
                .cloned()
                .chain(
                    filepath
                        .iter()
                        .map(|f| f.to_str().expect("receive str from OsString").to_string()),
                )
                .collect::<Vec<_>>();
            // remove filename from IDmodule.
            idmodule.pop().unwrap();

            // read in source code of file.
            // and leak the memory.
            // NOTE: for now we just keep all the sourcefiles in memory.
            // maybe later we will do the opposite and never keep them in memory, but instead read them when needed only.
            // This requires changes to the AST Nodes. All have a span: &str.
            // We'd prefer to have a token_start: u32 on over node. (We don't REALLY need token_end)
            // and remember for each Ast the file name.
            let mut source_code = std::fs::read_to_string(path).expect("read solar file");
            source_code.shrink_to_fit();
            let content = leak_string(source_code);

            let path = path.to_str().expect("read filename").to_string();
            let context = format!("reading file {path}");
            let fileinfo = FileInfo::from_code(path, &self.dep_map, &self.basepath, content)
                .context(context)?;

            map.entry(idmodule)
                .or_insert(Module::new(project_id))
                .add_file(fileinfo);
        }

        Ok(map)
    }
}

fn leak_string(s: String) -> &'static str {
    let b = s.into_boxed_str();
    let b = Box::leak(b);

    unsafe { std::mem::transmute(b) }
}
