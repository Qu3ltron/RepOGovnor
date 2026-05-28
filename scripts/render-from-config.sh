#!/usr/bin/env bash
# Render plugin templates using project.config.toml
set -euo pipefail

PLUGIN_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONFIG="${1:-${PLUGIN_ROOT}/project.config.toml}"
TARGET_ROOT="${2:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"
OVERLAY="${OVERLAY:-0}"
FORCE="${FORCE:-0}"

if [[ ! -f "$CONFIG" ]]; then
  echo "missing config: $CONFIG (copy project.config.example.toml)" >&2
  exit 1
fi

export PLUGIN_ROOT CONFIG TARGET_ROOT OVERLAY FORCE

python3 <<'PY'
import os
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
overlay = os.environ.get("OVERLAY", "0") == "1"
force = os.environ.get("FORCE", "0") == "1"
cfg = tomllib.loads(config_path.read_text())

project = cfg.get("project", {})
task_registry = cfg.get("task_registry", {})
mutation = cfg.get("mutation_gate", {})
validation = cfg.get("validation", {})
commit = cfg.get("commit_governance", {})
authority = cfg.get("authority", {})

repo_root = project.get("repo_root", "{{AUTO_REPO_ROOT}}")
if repo_root == "{{AUTO_REPO_ROOT}}":
    repo_root = subprocess.check_output(
        ["git", "-C", str(target_root), "rev-parse", "--show-toplevel"],
        text=True,
    ).strip()

repo_name = project.get("repo_name", Path(repo_root).name)
scratch_root = project.get("scratch_root", f"/tmp/{repo_name}-gap-closure")
constitution_path = project.get("constitution_path", "CONSTITUTION.md")
vision_path = project.get("vision_path", "VISION.md")
design_docs_path = project.get("design_docs_path", "docs/design")

cli_command = task_registry.get("cli_command", "cargo run --bin task_registry --")
registry_path = task_registry.get("registry_path", "docs/task-registry.toml")
plans_path = task_registry.get("plans_path", "docs/plans")
archive_dir = task_registry.get("archive_dir", "docs/task-registry/archive")

verify_hook = mutation.get(
    "verify_hook_command",
    f"{cli_command.rstrip()} verify-mutation-hook",
)
hook_script = mutation.get(
    "hook_script_path",
    "tools/antigravity/pre-tool-use-gap-closure.sh",
)

authority_order = authority.get("order", [constitution_path, vision_path, f"{design_docs_path}/*"])
authority_order_md = " → ".join(f"`{item}`" for item in authority_order)
authority_order_toml = "[" + ", ".join(f'"{item}"' for item in authority_order) + "]"

focused = validation.get("focused", ["cargo test"])
full = validation.get("full", ["cargo test"])
focused_md = ", ".join(f"`{c}`" for c in focused) or "_(configure in project.config.toml)_"
full_md = ", ".join(f"`{c}`" for c in full) or "_(configure in project.config.toml)_"
focused_toml = "[" + ", ".join(f'"{c}"' for c in focused) + "]"
full_toml = "[" + ", ".join(f'"{c}"' for c in full) + "]"

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
    "{{REPO_ROOT}}": repo_root,
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
    "{{VERIFY_HOOK_COMMAND}}": verify_hook,
    "{{MUTATION_HOOK_SCRIPT}}": hook_script,
    "{{VALIDATION_FOCUSED}}": focused_md,
    "{{VALIDATION_FULL}}": full_md,
    "{{VALIDATION_FOCUSED_TOML}}": focused_toml,
    "{{VALIDATION_FULL_TOML}}": full_toml,
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

def write_file(dest_path: Path, content: str, *, always: bool = False) -> str:
    dest_path.parent.mkdir(parents=True, exist_ok=True)
    if dest_path.exists() and not force and not always:
        return "skip"
    dest_path.write_text(content)
    return "write"

def merge_overlay(dest_path: Path, overlay_body: str) -> str:
    block = f"{OVERLAY_BEGIN}\n{overlay_body.rstrip()}\n{OVERLAY_END}\n"
    if dest_path.exists():
        text = dest_path.read_text()
        pattern = re.compile(
            re.escape(OVERLAY_BEGIN) + r".*?" + re.escape(OVERLAY_END) + r"\n?",
            re.DOTALL,
        )
        if pattern.search(text):
            dest_path.write_text(pattern.sub(block, text))
            return "merge-update"
        dest_path.write_text(text.rstrip() + "\n\n" + block)
        return "merge-append"
    dest_path.write_text(f"# {repo_name} — Agent Instructions\n\n{block}")
    return "merge-create"

templates = plugin_root / "templates"
actions: list[str] = []

if overlay:
    agents_tpl = templates / "AGENTS.overlay.md.template"
    gemini_tpl = templates / "GEMINI.overlay.md.template"
    actions.append(
        f"AGENTS.md: {merge_overlay(target_root / 'AGENTS.md', render_template(agents_tpl))}"
    )
    actions.append(
        f"GEMINI.md: {merge_overlay(target_root / 'GEMINI.md', render_template(gemini_tpl))}"
    )
else:
    actions.append(
        f"AGENTS.md: {write_file(target_root / 'AGENTS.md', render_template(templates / 'AGENTS.md.template'))}"
    )
    actions.append(
        f"GEMINI.md: {write_file(target_root / 'GEMINI.md', render_template(templates / 'GEMINI.md.template'))}"
    )

infra_files = [
    (
        templates / ".codex/settings.toml.template",
        target_root / ".codex/settings.toml",
    ),
    (
        templates / ".codex/hooks/user-plan-approval.toml.template",
        target_root / ".codex/hooks/user-plan-approval.toml",
    ),
    (
        templates / ".agents/hooks.json.template",
        target_root / ".agents/hooks.json",
    ),
    (
        templates / ".cursor/hooks.json.template",
        target_root / ".cursor/hooks.json",
    ),
    (
        templates / ".cursor/hooks/gap-closure-gate.sh.template",
        target_root / ".cursor/hooks/gap-closure-gate.sh",
    ),
    (
        templates / "tools/antigravity/pre-tool-use-gap-closure.sh.template",
        target_root / Path(hook_script),
    ),
]

for src, dest in infra_files:
    action = write_file(dest, render_template(src))
    actions.append(f"{dest.relative_to(target_root)}: {action}")

gemini_dest = target_root / ".gemini/settings.json"
(target_root / ".gemini").mkdir(parents=True, exist_ok=True)
if force or not gemini_dest.exists():
    shutil.copy2(templates / ".gemini/settings.json", gemini_dest)
    actions.append(".gemini/settings.json: write")
else:
    actions.append(".gemini/settings.json: skip")

skills_src = plugin_root / "skills"
skills_dest = target_root / ".cursor/skills"

def sync_skill(skill: str) -> str:
    src = skills_src / skill
    dest = skills_dest / skill
    project_md = dest / "PROJECT.md"
    preserved = project_md.read_text() if project_md.exists() else None

    if overlay and (dest / "SKILL.md").exists() and not force:
        dest.mkdir(parents=True, exist_ok=True)
        shutil.copy2(src / "SKILL.md", dest / "SKILL.md")
        agents_src = src / "agents"
        agents_dest = dest / "agents"
        if agents_dest.exists():
            shutil.rmtree(agents_dest)
        if agents_src.exists():
            shutil.copytree(agents_src, agents_dest)
        if preserved is not None:
            project_md.write_text(preserved)
        return f".cursor/skills/{skill}: sync-skill"

    if dest.exists():
        shutil.rmtree(dest)
    shutil.copytree(
        src,
        dest,
        ignore=shutil.ignore_patterns("PROJECT.md.template"),
    )
    if preserved is not None:
        project_md.write_text(preserved)
    else:
        template = src / "PROJECT.md.template"
        if template.exists() and not project_md.exists():
            shutil.copy2(template, project_md)
    return f".cursor/skills/{skill}: write"

for skill in ("gap-closure-contract", "task-registry-flow"):
    actions.append(sync_skill(skill))

agents_skills = target_root / ".agents/skills"
agents_skills.mkdir(parents=True, exist_ok=True)
for skill in ("gap-closure-contract", "task-registry-flow"):
    link = agents_skills / skill
    if link.is_symlink() or link.exists():
        link.unlink()
    link.symlink_to(Path("../../.cursor/skills") / skill)
actions.append(".agents/skills symlinks: write")

governance_env = target_root / ".codex/governance-cli.env"
governance_env.parent.mkdir(parents=True, exist_ok=True)
governance_env.write_text(f'GOVERNANCE_VERIFY_HOOK_CMD="{verify_hook}"\n')
actions.append(".codex/governance-cli.env: write")

plugin_link = target_root / ".agents/plugins/agent-governance"
plugin_link.parent.mkdir(parents=True, exist_ok=True)
if plugin_link.is_symlink() or plugin_link.exists():
    if force:
        plugin_link.unlink(missing_ok=True)
if not plugin_link.exists():
    plugin_link.symlink_to(Path("../../plugins/agent-governance"))
    actions.append(".agents/plugins/agent-governance: write-symlink")
else:
    actions.append(".agents/plugins/agent-governance: skip")

hook_path = target_root / hook_script
if hook_path.exists() or force or not overlay:
    hook_path.chmod(0o755)

cursor_hook = target_root / ".cursor/hooks/gap-closure-gate.sh"
if cursor_hook.exists() or force or not overlay:
    cursor_hook.chmod(0o755)

mode = "overlay" if overlay else "full"
print(f"Rendered agent-governance ({mode}) into {target_root}")
for line in actions:
    print(f"  {line}")
PY
