#!/usr/bin/env bash
# Render plugin templates using project.config.toml
set -euo pipefail

PLUGIN_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONFIG="${1:-${PLUGIN_ROOT}/project.config.toml}"
TARGET_ROOT="${2:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"
MODE="${MODE:-}"
DRY_RUN="${DRY_RUN:-0}"

if [[ ! -f "$CONFIG" ]]; then
  echo "missing config: $CONFIG (copy project.config.example.toml)" >&2
  exit 1
fi

case "$MODE" in
  merge|force) ;;
  *) echo "missing or invalid MODE: use merge or force" >&2; exit 1 ;;
esac

export PLUGIN_ROOT CONFIG TARGET_ROOT MODE DRY_RUN

python3 <<'PY'
import os
import json
import re
import shutil
import subprocess
from pathlib import Path

try:
    import tomllib
except ImportError:
    import tomli as tomllib  # type: ignore

OVERLAY_BEGIN = "<!-- agent-governance:begin -->"
OVERLAY_END = "<!-- agent-governance:end -->"

plugin_root = Path(os.environ["PLUGIN_ROOT"])
config_path = Path(os.environ["CONFIG"])
target_root = Path(os.environ["TARGET_ROOT"]).resolve()
mode = os.environ["MODE"]
dry_run = os.environ.get("DRY_RUN", "0") == "1"
dry_run_format = os.environ.get("DRY_RUN_FORMAT", "text")
force = mode == "force"
cfg = tomllib.loads(config_path.read_text())
manifest = tomllib.loads((plugin_root / "MANIFEST.toml").read_text())

allowed_config = {
    "project": {
        "repo_name",
        "repo_root",
        "scratch_root",
        "constitution_path",
        "vision_path",
        "agents_path",
        "design_docs_path",
    },
    "task_registry": {
        "cli_command",
        "registry_id",
        "registry_path",
        "plans_path",
        "archive_dir",
        "verify_landing_command",
    },
    "mutation_gate": {"verify_hook_command", "hook_script_path"},
    "validation": {"focused", "full"},
    "environments": {
        "render_codex",
        "render_antigravity",
        "render_cursor",
        "render_claude_code",
        "install_global_antigravity_plugin",
        "minimum_agy_version",
    },
    "commit_governance": {
        "verify_command",
        "plan_id_footer_required",
        "implementation_path_globs",
    },
    "authority": {"order"},
}
unknown_sections = sorted(set(cfg) - set(allowed_config))
if unknown_sections:
    raise SystemExit(f"unknown project.config.toml section(s): {', '.join(unknown_sections)}")
for section, allowed_keys in allowed_config.items():
    unknown_keys = sorted(set(cfg.get(section, {})) - allowed_keys)
    if unknown_keys:
        raise SystemExit(
            f"unknown project.config.toml key(s) in [{section}]: {', '.join(unknown_keys)}"
        )

install_policy = manifest.get("install_policy", {})
allowed_actions = set(install_policy.get("action_vocabulary", []))
dry_run_prefix = install_policy.get("dry_run_prefix", "would-")
stale_absent = install_policy.get("stale_absent", [])
if not allowed_actions or not stale_absent:
    raise SystemExit("MANIFEST.toml install_policy requires action_vocabulary and stale_absent")
if dry_run_format not in {"text", "json"}:
    raise SystemExit("DRY_RUN_FORMAT must be text or json")

def validate_manifest_path(path: str, field: str) -> None:
    candidate = Path(path)
    if candidate.is_absolute() or ".." in candidate.parts:
        raise SystemExit(f"MANIFEST.toml {field} must be repo-relative: {path}")

for section in ("render", "copy", "generated", "symlinks", "plugin_only"):
    for entry in manifest.get(section, []):
        for key in ("template", "destination", "source", "link", "from"):
            if key in entry:
                validate_manifest_path(str(entry[key]), f"{section}.{key}")

def validate_hook_script_path(path: str) -> None:
    candidate = Path(path)
    if (
        not path
        or candidate.is_absolute()
        or ".." in candidate.parts
        or not path.startswith("tools/agent-governance/")
        or not path.endswith(".sh")
        or re.search(r"[^A-Za-z0-9._/-]", path)
    ):
        raise SystemExit(
            f"unsafe project.config.toml [mutation_gate].hook_script_path: {path!r}; expected repo-relative tools/agent-governance/*.sh"
        )

project = cfg.get("project", {})
task_registry = cfg.get("task_registry", {})
mutation = cfg.get("mutation_gate", {})
validation = cfg.get("validation", {})
commit = cfg.get("commit_governance", {})
authority = cfg.get("authority", {})
environments = cfg.get("environments", {})

canonical_task_registry = {
    "cli_command": ".codex/scripts/task-registry",
    "registry_path": "docs/task-registry.toml",
    "plans_path": "docs/plans",
    "archive_dir": "docs/task-registry/archive",
    "verify_landing_command": ".codex/scripts/task-registry verify-landing --plan-id <plan_id> --changed-files <paths>",
}
for key, expected in canonical_task_registry.items():
    actual = task_registry.get(key, expected)
    if actual != expected:
        raise SystemExit(
            f"noncanonical project.config.toml [task_registry].{key}: {actual!r}; expected {expected!r}"
        )

if mutation.get("verify_hook_command", ".codex/scripts/task-registry verify-mutation-hook") != ".codex/scripts/task-registry verify-mutation-hook":
    raise SystemExit(
        "noncanonical project.config.toml [mutation_gate].verify_hook_command; expected '.codex/scripts/task-registry verify-mutation-hook'"
    )

for key in ("render_codex", "render_antigravity", "render_cursor", "render_claude_code"):
    if environments.get(key, True) is not True:
        raise SystemExit(
            f"project.config.toml [environments].{key}=false is not supported by the v2 canonical runtime projection"
        )
if environments.get("install_global_antigravity_plugin", False) is not False:
    raise SystemExit(
        "project.config.toml [environments].install_global_antigravity_plugin=true is not supported by this local-first installer"
    )
if environments.get("minimum_agy_version", "1.0.3") != "1.0.3":
    raise SystemExit(
        "project.config.toml [environments].minimum_agy_version must remain 1.0.3 for the v2 templates"
    )

repo_root = project.get("repo_root", "{{AUTO_REPO_ROOT}}")
if repo_root == "{{AUTO_REPO_ROOT}}":
    repo_root = subprocess.check_output(
        ["git", "-C", str(target_root), "rev-parse", "--show-toplevel"],
        text=True,
    ).strip()

repo_name = project.get("repo_name", Path(repo_root).name)
repo_slug = re.sub(r"[^A-Za-z0-9]+", "-", repo_name.strip().lower()).strip("-") or "repo"
scratch_root = project.get("scratch_root", f"/tmp/{repo_name}-gap-closure")
constitution_path = project.get("constitution_path", "CONSTITUTION.md")
vision_path = project.get("vision_path", "VISION.md")
design_docs_path = project.get("design_docs_path", "docs/design")

cli_command = ".codex/scripts/task-registry"
registry_path = "docs/task-registry.toml"
plans_path = "docs/plans"
archive_dir = "docs/task-registry/archive"
registry_id = task_registry.get("registry_id", f"{repo_slug}-task-registry")

verify_hook = ".codex/scripts/task-registry verify-mutation-hook"
hook_script = mutation.get(
    "hook_script_path",
    "tools/agent-governance/pre-tool-use-gap-closure.sh",
)
validate_hook_script_path(hook_script)

authority_order = authority.get("order", [constitution_path, vision_path, f"{design_docs_path}/*"])
authority_order_md = " → ".join(f"`{item}`" for item in authority_order)
authority_order_toml = "[" + ", ".join(f'"{item}"' for item in authority_order) + "]"

focused = validation.get("focused", ["cargo test"])
full = validation.get("full", ["cargo test"])
focused_with_source = [".codex/scripts/task-registry source-limit check"] + [
    command for command in focused if command != ".codex/scripts/task-registry source-limit check"
]
full_with_source = [".codex/scripts/task-registry validate"] + [
    command for command in full if command != ".codex/scripts/task-registry validate"
]
focused_md = ", ".join(f"`{c}`" for c in focused) or "_(configure in project.config.toml)_"
full_md = ", ".join(f"`{c}`" for c in full) or "_(configure in project.config.toml)_"
focused_toml = "[" + ", ".join(f'"{c}"' for c in focused) + "]"
full_toml = "[" + ", ".join(f'"{c}"' for c in full) + "]"
focused_with_source_toml = "[" + ", ".join(f'"{c}"' for c in focused_with_source) + "]"
full_with_source_toml = "[" + ", ".join(f'"{c}"' for c in full_with_source) + "]"

verify_cmd = commit.get("verify_command", "")
plan_id_required = commit.get("plan_id_footer_required", False)
impl_globs = commit.get("implementation_path_globs", [])

commit_section = ""
commit_overlay = ""
commit_toml = "# [commit_governance] — optional; configure in project.config.toml and extend AGENTS.md locally"

if verify_cmd or plan_id_required:
    globs = ", ".join(f"`{g}`" for g in impl_globs) if impl_globs else "configured implementation paths"
    globs_toml = "[" + ", ".join(f'"{g}"' for g in impl_globs) + "]"
    commit_section = f"""## Commits (optional)

When enabled in project config, commits touching {globs} should include a registered plan ID (e.g. footer `Plan-ID: PLAN-YYYY-MM-DD-short-slug`) bound to an **active** plan in `docs/task-registry.toml`.

Verify command: `{verify_cmd or "(configure commit_governance.verify_command)"}`"""
    commit_overlay = f"""### Commits

Commits touching {globs} require a `Plan-ID:` footer bound to an active plan. Verify: `{verify_cmd or "(configure commit_governance.verify_command)"}`."""
    commit_toml = f"""[commit_governance]
verify_command = "{verify_cmd}"
plan_id_footer_required = {"true" if plan_id_required else "false"}
implementation_path_globs = {globs_toml}
design_rule = "Optional project policy — configure enforcement in CI separately from this plugin."
"""

subs = {
    "{{REPO_NAME}}": repo_name,
    "{{REPO_SLUG}}": repo_slug,
    "{{REPO_ROOT}}": repo_root,
    "{{PLUGIN_ROOT}}": str(plugin_root),
    "{{SCRATCH_ROOT}}": scratch_root,
    "{{CONSTITUTION_PATH}}": constitution_path,
    "{{VISION_PATH}}": vision_path,
    "{{DESIGN_DOCS_PATH}}": design_docs_path,
    "{{AUTHORITY_ORDER}}": authority_order_md,
    "{{AUTHORITY_ORDER_TOML}}": authority_order_toml,
    "{{TASK_REGISTRY_CLI}}": cli_command.rstrip(),
    "{{REGISTRY_PATH}}": registry_path,
    "{{PLANS_PATH}}": plans_path,
    "{{ARCHIVE_DIR}}": archive_dir,
    "{{REGISTRY_ID}}": registry_id,
    "{{VERIFY_HOOK_COMMAND}}": verify_hook,
    "{{MUTATION_HOOK_SCRIPT}}": hook_script,
    "{{VALIDATION_FOCUSED}}": focused_md,
    "{{VALIDATION_FULL}}": full_md,
    "{{VALIDATION_FOCUSED_TOML}}": focused_toml,
    "{{VALIDATION_FULL_TOML}}": full_toml,
    "{{VALIDATION_FOCUSED_WITH_SOURCE_TOML}}": focused_with_source_toml,
    "{{VALIDATION_FULL_WITH_SOURCE_TOML}}": full_with_source_toml,
    "{{COMMIT_GOVERNANCE_SECTION}}": commit_section,
    "{{COMMIT_GOVERNANCE_OVERLAY}}": commit_overlay,
    "{{COMMIT_GOVERNANCE_TOML}}": commit_toml,
}

def substitute(text: str) -> str:
    for key, value in subs.items():
        text = text.replace(key, value)
    return text

def render_template(template_path: Path) -> str:
    return substitute(template_path.read_text())

def rel(path: Path) -> str:
    try:
        return str(path.relative_to(target_root))
    except ValueError:
        return str(path)

def projected(action: str) -> str:
    if not dry_run or action.startswith("preserve") or action in {"aligned", "skip"}:
        return action
    return f"{dry_run_prefix}{action}"

def validate_action(action: str) -> None:
    normalized = action
    if normalized.startswith(dry_run_prefix):
        normalized = normalized[len(dry_run_prefix):]
    if normalized not in allowed_actions:
        raise SystemExit(f"unknown installer action emitted: {action}")

def remove_existing(path: Path) -> None:
    if path.is_symlink() or path.is_file():
        path.unlink()
    elif path.is_dir():
        shutil.rmtree(path)

def write_file(dest_path: Path, content: str, *, merge_updates: bool = True, preserve_existing: bool = False) -> str:
    exists = dest_path.exists() or dest_path.is_symlink()
    if exists and (dest_path.is_file() or dest_path.is_symlink()):
        try:
            if dest_path.read_text() == content:
                return "aligned"
        except (OSError, UnicodeDecodeError):
            pass

    if exists and preserve_existing:
        return "preserve"
    if exists and mode == "merge" and not merge_updates:
        return "preserve-drift"
    if exists and mode == "merge" and (dest_path.is_symlink() or not dest_path.is_file()):
        return "preserve-drift"

    action = "create"
    if exists:
        action = "replace" if dest_path.is_symlink() or not dest_path.is_file() else "update"

    if dry_run:
        return projected(action)

    dest_path.parent.mkdir(parents=True, exist_ok=True)
    if action == "replace":
        remove_existing(dest_path)
    dest_path.write_text(content)
    return action

def validate_existing_registry(dest_path: Path) -> None:
    try:
        registry = tomllib.loads(dest_path.read_text())
    except Exception as exc:
        raise SystemExit(
            f"{dest_path.relative_to(target_root)} exists but is not valid TOML; hard cutover refuses to overwrite it: {exc}"
        )
    required_statuses = {"planned", "active", "blocked", "deferred", "completed", "cancelled"}
    statuses = set(registry.get("status_vocabulary", []))
    if (
        registry.get("schema_version") != 1
        or registry.get("registry_authority") != "docs/task-registry.toml"
        or registry.get("activation_skill") != "task-registry-flow"
        or not required_statuses.issubset(statuses)
    ):
        raise SystemExit(
            f"{dest_path.relative_to(target_root)} is incompatible with agent-governance task registry schema v1; repair it before hard cutover"
        )
    for task in registry.get("tasks", []):
        if not task.get("behavior_ids"):
            task_id = task.get("task_id", "<unknown>")
            raise SystemExit(
                f"{dest_path.relative_to(target_root)} task {task_id} is missing behavior_ids; repair it before hard cutover"
            )
        if not task.get("targets"):
            task_id = task.get("task_id", "<unknown>")
            raise SystemExit(
                f"{dest_path.relative_to(target_root)} task {task_id} is missing targets; repair it before hard cutover"
            )

def merge_overlay(dest_path: Path, overlay_body: str) -> str:
    block = f"{OVERLAY_BEGIN}\n{overlay_body.rstrip()}\n{OVERLAY_END}\n"
    if dest_path.exists():
        text = dest_path.read_text()
        pattern = re.compile(
            re.escape(OVERLAY_BEGIN) + r".*?" + re.escape(OVERLAY_END) + r"\n?",
            re.DOTALL,
        )
        if pattern.search(text):
            updated = pattern.sub(block, text)
            if updated == text:
                return "aligned"
            action = "merge-update"
        else:
            updated = text.rstrip() + "\n\n" + block
            action = "merge-append"
    else:
        updated = f"# {repo_name} - Agent Instructions\n\n{block}"
        action = "merge-create"

    if dry_run:
        return projected(action)

    dest_path.parent.mkdir(parents=True, exist_ok=True)
    dest_path.write_text(updated)
    return action

templates = plugin_root / "templates"
actions: list[str] = []

if mode == "merge":
    agents_tpl = templates / "AGENTS.overlay.md.template"
    gemini_tpl = templates / "GEMINI.overlay.md.template"
    claude_tpl = templates / "CLAUDE.overlay.md.template"
    actions.append(
        f"AGENTS.md: {merge_overlay(target_root / 'AGENTS.md', render_template(agents_tpl))}"
    )
    actions.append(
        f"GEMINI.md: {merge_overlay(target_root / 'GEMINI.md', render_template(gemini_tpl))}"
    )
    actions.append(
        f"CLAUDE.md: {merge_overlay(target_root / 'CLAUDE.md', render_template(claude_tpl))}"
    )
else:
    actions.append(
        f"AGENTS.md: {write_file(target_root / 'AGENTS.md', render_template(templates / 'AGENTS.md.template'))}"
    )
    actions.append(
        f"GEMINI.md: {write_file(target_root / 'GEMINI.md', render_template(templates / 'GEMINI.md.template'))}"
    )
    actions.append(
        f"CLAUDE.md: {write_file(target_root / 'CLAUDE.md', render_template(templates / 'CLAUDE.md.template'))}"
    )

infra_files = [
    (
        templates / ".codex/config.toml.template",
        target_root / ".codex/config.toml",
        True,
    ),
    (
        templates / ".codex/hooks.json.template",
        target_root / ".codex/hooks.json",
        True,
    ),
    (
        templates / ".codex/agent-governance.toml.template",
        target_root / ".codex/agent-governance.toml",
        True,
    ),
    (
        templates / ".codex/scripts/task-registry.template",
        target_root / ".codex/scripts/task-registry",
        True,
    ),
    (
        templates / ".codex/templates/task-registry-plan-template.md.template",
        target_root / ".codex/templates/task-registry-plan-template.md",
        True,
    ),
    (
        templates / ".github/workflows/agent-governance.yml.template",
        target_root / ".github/workflows/agent-governance.yml",
        True,
    ),
    (
        templates / ".agents/hooks.json.template",
        target_root / ".agents/hooks.json",
        True,
    ),
    (
        templates / ".cursor/hooks.json.template",
        target_root / ".cursor/hooks.json",
        True,
    ),
    (
        templates / ".cursor/hooks/gap-closure-gate.sh.template",
        target_root / ".cursor/hooks/gap-closure-gate.sh",
        True,
    ),
    (
        templates / ".cursor/rules/agent-governance.mdc.template",
        target_root / ".cursor/rules/agent-governance.mdc",
        True,
    ),
    (
        templates / ".claude/settings.json.template",
        target_root / ".claude/settings.json",
        True,
    ),
    (
        templates / "tools/agent-governance/pre-tool-use-gap-closure.sh.template",
        target_root / Path(hook_script),
        True,
    ),
]

for src, dest, always in infra_files:
    action = write_file(dest, render_template(src), merge_updates=always)
    actions.append(f"{rel(dest)}: {action}")

for stale in stale_absent:
    stale_path = target_root / stale
    if stale_path.exists() or stale_path.is_symlink():
        if not dry_run:
            remove_existing(stale_path)
        actions.append(f"{stale}: {projected('remove-stale')}")

registry_dest = target_root / registry_path
if registry_dest.exists():
    validate_existing_registry(registry_dest)
    actions.append(f"{rel(registry_dest)}: preserve-valid")
else:
    actions.append(
        f"{rel(registry_dest)}: {write_file(registry_dest, render_template(templates / 'docs/task-registry.toml.template'))}"
    )

events_dest = target_root / "docs/task-registry/events.jsonl"
if events_dest.exists():
    actions.append("docs/task-registry/events.jsonl: preserve")
else:
    actions.append(f"docs/task-registry/events.jsonl: {write_file(events_dest, '')}")

skills_src = plugin_root / "skills"

def copied_skill_files(src: Path) -> dict[Path, bytes]:
    files: dict[Path, bytes] = {}
    for src_file in sorted(src.rglob("*")):
        if not src_file.is_file() or src_file.name == "PROJECT.md.template":
            continue
        files[src_file.relative_to(src)] = src_file.read_bytes()
    return files

def skill_aligned(src: Path, dest: Path, *, strict: bool) -> bool:
    if not dest.is_dir():
        return False
    desired = copied_skill_files(src)
    for rel_path, content in desired.items():
        dest_file = dest / rel_path
        if not dest_file.is_file() or dest_file.read_bytes() != content:
            return False
    if strict:
        allowed = set(desired) | {Path("PROJECT.md")}
        for existing in dest.rglob("*"):
            if existing.is_file() and existing.relative_to(dest) not in allowed:
                return False
    return True

def copy_skill_full(src: Path, dest: Path, preserved_project: str | None) -> None:
    if dest.exists() or dest.is_symlink():
        remove_existing(dest)
    shutil.copytree(
        src,
        dest,
        ignore=shutil.ignore_patterns("PROJECT.md.template"),
    )
    project_md = dest / "PROJECT.md"
    if preserved_project is not None:
        project_md.write_text(preserved_project)
    else:
        template = src / "PROJECT.md.template"
        if template.exists() and not project_md.exists():
            shutil.copy2(template, project_md)

def merge_skill_tree(src: Path, dest: Path, preserved_project: str | None) -> None:
    if not dest.exists():
        copy_skill_full(src, dest, preserved_project)
        return

    dest.mkdir(parents=True, exist_ok=True)
    shutil.copy2(src / "SKILL.md", dest / "SKILL.md")
    agents_src = src / "agents"
    agents_dest = dest / "agents"
    if agents_dest.exists() or agents_dest.is_symlink():
        remove_existing(agents_dest)
    if agents_src.exists():
        shutil.copytree(agents_src, agents_dest)

    project_md = dest / "PROJECT.md"
    if preserved_project is not None:
        project_md.write_text(preserved_project)
    else:
        template = src / "PROJECT.md.template"
        if template.exists() and not project_md.exists():
            shutil.copy2(template, project_md)

def sync_skill(skill: str, base: Path) -> str:
    src = skills_src / skill
    dest = base / skill
    preserved = None
    if dest.is_symlink():
        action = "replace-symlink"
    elif dest.is_dir():
        project_md = dest / "PROJECT.md"
        preserved = project_md.read_text() if project_md.exists() else None
        if skill_aligned(src, dest, strict=force):
            action = "aligned"
        else:
            action = "update" if force else "sync-skill"
    elif dest.exists():
        action = "replace" if force else "preserve-drift"
    else:
        action = "create"

    if dry_run or action in {"aligned", "preserve-drift"}:
        return f"{rel(dest)}: {projected(action)}"

    if force or action == "replace-symlink":
        copy_skill_full(src, dest, preserved)
    else:
        merge_skill_tree(src, dest, preserved)
    return f"{rel(dest)}: {action}"

def render_agy_skill(skill: str) -> str:
    src = skills_src / skill / "SKILL.md"
    dest = target_root / ".agents/skills" / f"{skill}.md"
    return f"{rel(dest)}: {write_file(dest, src.read_text())}"

for skill in ("gap-closure-contract", "task-registry-flow"):
    actions.append(sync_skill(skill, target_root / ".cursor/skills"))
    actions.append(sync_skill(skill, target_root / ".agents/skills"))
    actions.append(sync_skill(skill, target_root / ".claude/skills"))
    actions.append(render_agy_skill(skill))

agents_skills = target_root / ".agents/skills"
if agents_skills.is_dir():
    actions.append(".agents/skills native projections: aligned")
elif agents_skills.exists() or agents_skills.is_symlink():
    action = "replace-dir" if force else "preserve-drift"
    if not dry_run and force:
        remove_existing(agents_skills)
        agents_skills.mkdir(parents=True, exist_ok=True)
    actions.append(f".agents/skills native projections: {projected(action)}")
else:
    if not dry_run:
        agents_skills.mkdir(parents=True, exist_ok=True)
    actions.append(f".agents/skills native projections: {projected('create-dir')}")

governance_env = target_root / ".codex/governance-cli.env"
governance_env_content = f'GOVERNANCE_VERIFY_HOOK_CMD="{verify_hook}"\n'
actions.append(
    f".codex/governance-cli.env: {write_file(governance_env, governance_env_content)}"
)

plugin_link = target_root / ".agents/plugins/agent-governance"
plugin_target = Path("../../plugins/agent-governance")
if plugin_link.is_symlink() and os.readlink(plugin_link) == str(plugin_target):
    actions.append(".agents/plugins/agent-governance: aligned")
elif plugin_link.exists() or plugin_link.is_symlink():
    action = "replace-symlink" if force else "preserve-drift"
    if not dry_run and force:
        remove_existing(plugin_link)
        plugin_link.parent.mkdir(parents=True, exist_ok=True)
        plugin_link.symlink_to(plugin_target)
    actions.append(f".agents/plugins/agent-governance: {projected(action)}")
else:
    if not dry_run:
        plugin_link.parent.mkdir(parents=True, exist_ok=True)
        plugin_link.symlink_to(plugin_target)
    actions.append(f".agents/plugins/agent-governance: {projected('create-symlink')}")

def ensure_executable(path: Path) -> str:
    if not path.exists():
        return projected("chmod")
    if path.stat().st_mode & 0o777 == 0o755:
        return "aligned"
    if dry_run:
        return projected("chmod")
    path.chmod(0o755)
    return "chmod"

for executable in (
    target_root / hook_script,
    target_root / ".cursor/hooks/gap-closure-gate.sh",
    target_root / ".codex/scripts/task-registry",
):
    actions.append(f"{rel(executable)}: {ensure_executable(executable)}")

verb = "Projected" if dry_run else "Rendered"
records = []
for line in actions:
    path, action = line.rsplit(": ", 1)
    validate_action(action)
    records.append({"path": path, "action": action})

if dry_run and dry_run_format == "json":
    print(json.dumps({
        "schema_version": 1,
        "surface": "installer-dry-run",
        "mode": mode,
        "target_root": str(target_root),
        "actions": records,
    }, indent=2))
else:
    print(f"{verb} agent-governance ({mode}) into {target_root}")
    for line in actions:
        print(f"  {line}")
PY
