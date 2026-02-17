use std::collections::HashMap;

use anyhow::Context;
use hotel::HotelMap;

use crate::{
    util::{self, IdPath},
    Config,
};

use super::{Module, Project};

pub type ProjectInfo = HotelMap<IdPath, Project>;

pub fn read_all_projects(config: &Config) -> anyhow::Result<ProjectInfo> {
    let mut projects = HotelMap::new();
    let p = Project::open(&config.project_root, util::target_id())
        .with_context(|| format!("opening project at {}", config.project_root))?;

    fn insert_all(
        p: Project,
        projects: &mut HotelMap<IdPath, Project>,
        config: &Config,
    ) -> anyhow::Result<()> {
        for dep in p.config.deps() {
            let path = dep.basepath();
            // skip project, if we have already read it.
            if projects.contains(&path) {
                continue;
            }

            let dir = dep.dir(&config);
            let p =
                Project::open(&dir, path).with_context(|| format!("opening project at {}", dir))?;
            insert_all(p, projects, config)?;
        }

        projects.insert(p.basepath.clone(), p);

        Ok(())
    }

    insert_all(p, &mut projects, config)?;

    Ok(projects)
}

/// Mapping from IdPaths/ModulePaths (use @std.0.1.0.types.string.String) to all modules.
/// ASTs can be found inside the modules.
pub type GlobalModules<'a> = HashMap<IdPath, Module<'a>>;

/// create global mapping of ModulePaths to Modules
/// i.e. across all dependencies and sub-dependencies
pub fn read_modules(projects: &ProjectInfo) -> anyhow::Result<GlobalModules<'_>> {
    let mut modules = HashMap::new();

    for (project_id, project) in projects.iter_values() {
        let symbol_table = project
            .read_all(project_id)
            .context(format!("reading project {}", project.fsroot))?;

        for (sym, path) in symbol_table.into_iter() {
            modules.insert(sym, path);
        }
    }

    Ok(modules)
}
