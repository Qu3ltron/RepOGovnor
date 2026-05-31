use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use fs2::FileExt;

use crate::model::{
    ARCHIVE_COMPLETED_PLAN_CHUNK_SIZE, REGISTRY_ARCHIVE_DIR, REGISTRY_PATH, RegistryPlan,
    RegistryTask, Result, TaskRegistry, TaskRegistryArchive,
};
use crate::schema::TaskStatus;
use crate::validation::{validate_archive_path, validate_archive_shape, validate_registry_root};

pub(crate) fn load_registry(root: &Path) -> Result<TaskRegistry> {
    let mut registry = load_current_registry(root)?;
    let archives = load_registry_archives(root, &registry.archive_paths)?;
    for archive in archives {
        registry.plans.extend(archive.plans);
        registry.tasks.extend(archive.tasks);
    }
    validate_registry_root(&registry)?;
    Ok(registry)
}

fn load_current_registry(root: &Path) -> Result<TaskRegistry> {
    let path = root.join(REGISTRY_PATH);
    let body =
        fs::read_to_string(&path).map_err(|error| format!("read {}: {error}", path.display()))?;
    toml::from_str(&body).map_err(|error| format!("parse {}: {error}", path.display()))
}

fn load_registry_archives(
    root: &Path,
    archive_paths: &[String],
) -> Result<Vec<TaskRegistryArchive>> {
    let mut archives = Vec::new();
    for archive_path in archive_paths {
        validate_archive_path(archive_path)?;
        let body = fs::read_to_string(root.join(archive_path))
            .map_err(|error| format!("read {archive_path}: {error}"))?;
        let archive = toml::from_str::<TaskRegistryArchive>(&body)
            .map_err(|error| format!("parse {archive_path}: {error}"))?;
        validate_archive_shape(archive_path, &archive)?;
        archives.push(archive);
    }
    Ok(archives)
}

pub(crate) fn save_registry(root: &Path, registry: &TaskRegistry) -> Result<()> {
    validate_registry_root(registry)?;
    let (main_registry, archives) = split_registry_for_archives(registry);
    write_registry_archives(root, &archives)?;
    let path = root.join(REGISTRY_PATH);
    let mut body = toml::to_string_pretty(&main_registry)
        .map_err(|error| format!("serialize {}: {error}", path.display()))?;
    if !body.ends_with('\n') {
        body.push('\n');
    }
    // Acquire exclusive lock, write to temp file, atomic rename
    let file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(&path)
        .map_err(|error| format!("lock {}: {error}", path.display()))?;
    file.try_lock_exclusive()
        .map_err(|_| "registry is locked by another process; retry in a moment".to_string())?;
    let tmp_path = path.with_extension("tmp");
    fs::write(&tmp_path, &body)
        .map_err(|error| format!("write {}: {error}", tmp_path.display()))?;
    // fsync temp file before atomic rename — prevents zero-length or partial
    // registry after an OS crash or power loss.
    std::fs::OpenOptions::new()
        .read(true)
        .open(&tmp_path)
        .and_then(|f| f.sync_all())
        .map_err(|error| format!("sync {}: {error}", tmp_path.display()))?;
    fs::rename(&tmp_path, &path).map_err(|error| {
        format!(
            "rename {} -> {}: {error}",
            tmp_path.display(),
            path.display()
        )
    })?;
    Ok(())
}

fn split_registry_for_archives(
    registry: &TaskRegistry,
) -> (TaskRegistry, Vec<(String, TaskRegistryArchive)>) {
    let completed_plan_ids = registry
        .plans
        .iter()
        .filter(|plan| plan_is_archive_eligible(plan, &registry.tasks))
        .map(|plan| plan.plan_id.clone())
        .collect::<BTreeSet<_>>();
    let mut main_registry = registry.clone();
    main_registry.plans = registry
        .plans
        .iter()
        .filter(|plan| !completed_plan_ids.contains(&plan.plan_id))
        .cloned()
        .collect();
    main_registry.tasks = registry
        .tasks
        .iter()
        .filter(|task| !completed_plan_ids.contains(&task.plan_id))
        .cloned()
        .collect();

    let completed_plans = registry
        .plans
        .iter()
        .filter(|plan| completed_plan_ids.contains(&plan.plan_id))
        .map(archived_completed_plan)
        .collect::<Vec<_>>();
    let mut archives = Vec::new();
    for (index, chunk) in completed_plans
        .chunks(ARCHIVE_COMPLETED_PLAN_CHUNK_SIZE)
        .enumerate()
    {
        let archive_id = format!("completed-{:03}", index + 1);
        let archive_path = format!("{REGISTRY_ARCHIVE_DIR}/{archive_id}.toml");
        let chunk_plan_ids = chunk
            .iter()
            .map(|plan| plan.plan_id.as_str())
            .collect::<BTreeSet<_>>();
        let tasks = registry
            .tasks
            .iter()
            .filter(|task| chunk_plan_ids.contains(task.plan_id.as_str()))
            .cloned()
            .collect::<Vec<_>>();
        archives.push((
            archive_path.clone(),
            TaskRegistryArchive {
                schema_version: registry.schema_version,
                registry_id: registry.registry_id.clone(),
                archive_id,
                archive_authority: archive_path,
                plans: chunk.to_vec(),
                tasks,
            },
        ));
    }
    main_registry.archive_paths = archives
        .iter()
        .map(|(archive_path, _)| archive_path.clone())
        .collect();
    (main_registry, archives)
}

fn write_registry_archives(root: &Path, archives: &[(String, TaskRegistryArchive)]) -> Result<()> {
    let archive_dir = root.join(REGISTRY_ARCHIVE_DIR);
    fs::create_dir_all(&archive_dir)
        .map_err(|error| format!("create {REGISTRY_ARCHIVE_DIR}: {error}"))?;
    for entry in fs::read_dir(&archive_dir)
        .map_err(|error| format!("read {REGISTRY_ARCHIVE_DIR}: {error}"))?
        .flatten()
    {
        let path = entry.path();
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if file_name.starts_with("completed-") && file_name.ends_with(".toml") {
            fs::remove_file(&path)
                .map_err(|error| format!("remove {}: {error}", path.display()))?;
        }
    }
    for (archive_path, archive) in archives {
        let mut body = toml::to_string_pretty(archive)
            .map_err(|error| format!("serialize {archive_path}: {error}"))?;
        if !body.ends_with('\n') {
            body.push('\n');
        }
        fs::write(root.join(archive_path), body)
            .map_err(|error| format!("write {archive_path}: {error}"))?;
    }
    Ok(())
}

fn plan_is_archive_eligible(plan: &RegistryPlan, tasks: &[RegistryTask]) -> bool {
    if plan.status == TaskStatus::Completed || plan.status == TaskStatus::Cancelled {
        return true;
    }
    let plan_tasks = tasks
        .iter()
        .filter(|task| task.plan_id == plan.plan_id)
        .collect::<Vec<_>>();
    !plan_tasks.is_empty()
        && plan_tasks.iter().all(|task| {
            task.status == TaskStatus::Completed || task.status == TaskStatus::Cancelled
        })
}

fn archived_completed_plan(plan: &RegistryPlan) -> RegistryPlan {
    let mut archived = plan.clone();
    if archived.status != TaskStatus::Cancelled {
        archived.status = TaskStatus::Completed;
    }
    archived
}
