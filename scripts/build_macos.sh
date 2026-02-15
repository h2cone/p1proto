#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/build_macos.sh [options]

Build Rust GDExtension, export Godot macOS app, and package it as zip.

Options:
  --build <release|both>      Rust build mode (default: release)
  --godot-exe <path>          Godot executable name/path (default: godot)
  --preset-name <name>        Godot export preset name (default: macOS)
  --out-dir <path>            Output directory relative to repo root (default: build/macos)
  --app-name <name>           App bundle name without extension (default: p1proto)
  --zip-name <name>           Zip file name (default: <app-name>-macos.zip)
  --no-zip                    Skip zip packaging step
  --no-recovery-mode          Do not pass --recovery-mode to Godot
  -h, --help                  Show help
EOF
}

assert_command_exists() {
  local cmd="$1"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "Error: command not found: $cmd" >&2
    exit 1
  fi
}

trim_line() {
  local s="$1"
  s="${s#"${s%%[![:space:]]*}"}"
  s="${s%"${s##*[![:space:]]}"}"
  printf '%s' "$s"
}

normalize_godot_extension_list() {
  local godot_dir="$1"
  local extension_list_path="$godot_dir/.godot/extension_list.cfg"
  local default_ext="res://rust.gdextension"

  [[ -f "$extension_list_path" ]] || return 0

  local lines=()
  while IFS= read -r line; do
    line="${line//$'\r'/}"
    line="$(trim_line "$line")"
    [[ -n "$line" ]] || continue

    if [[ "$line" == res://* ]]; then
      local rel="${line#res://}"
      if [[ -e "$godot_dir/$rel" ]]; then
        lines+=("$line")
      fi
    fi
  done <"$extension_list_path"

  if [[ -f "$godot_dir/rust.gdextension" ]]; then
    local found=0
    for entry in "${lines[@]:-}"; do
      if [[ "$entry" == "$default_ext" ]]; then
        found=1
        break
      fi
    done

    if [[ "$found" -eq 0 ]]; then
      lines=("$default_ext" "${lines[@]:-}")
    fi
  fi

  : >"$extension_list_path"
  if [[ "${#lines[@]}" -gt 0 ]]; then
    printf '%s\n' "${lines[@]}" >"$extension_list_path"
  fi
}

ensure_export_preset_exists() {
  local export_presets_path="$1"
  local preset_name="$2"
  if [[ ! -f "$export_presets_path" ]]; then
    echo "Error: missing $export_presets_path" >&2
    echo "Create a macOS export preset named '$preset_name' in Godot first." >&2
    exit 1
  fi
  if ! grep -Fq "name=\"$preset_name\"" "$export_presets_path"; then
    echo "Error: export preset '$preset_name' not found in $export_presets_path" >&2
    echo "Create it in Godot: Project -> Export -> Add macOS preset." >&2
    exit 1
  fi
}

build_mode="release"
godot_exe="godot"
preset_name="macOS"
out_dir="build/macos"
app_name="p1proto"
zip_name=""
no_zip=0
no_recovery_mode=1

while [[ $# -gt 0 ]]; do
  case "$1" in
  --build)
    [[ $# -ge 2 ]] || {
      echo "Error: --build requires a value." >&2
      exit 1
    }
    build_mode="$2"
    shift 2
    ;;
  --godot-exe)
    [[ $# -ge 2 ]] || {
      echo "Error: --godot-exe requires a value." >&2
      exit 1
    }
    godot_exe="$2"
    shift 2
    ;;
  --preset-name)
    [[ $# -ge 2 ]] || {
      echo "Error: --preset-name requires a value." >&2
      exit 1
    }
    preset_name="$2"
    shift 2
    ;;
  --out-dir)
    [[ $# -ge 2 ]] || {
      echo "Error: --out-dir requires a value." >&2
      exit 1
    }
    out_dir="$2"
    shift 2
    ;;
  --app-name)
    [[ $# -ge 2 ]] || {
      echo "Error: --app-name requires a value." >&2
      exit 1
    }
    app_name="$2"
    shift 2
    ;;
  --zip-name)
    [[ $# -ge 2 ]] || {
      echo "Error: --zip-name requires a value." >&2
      exit 1
    }
    zip_name="$2"
    shift 2
    ;;
  --no-zip)
    no_zip=1
    shift
    ;;
  --no-recovery-mode)
    no_recovery_mode=1
    shift
    ;;
  -h | --help)
    usage
    exit 0
    ;;
  *)
    echo "Error: unknown argument: $1" >&2
    usage >&2
    exit 1
    ;;
  esac
done

case "$build_mode" in
release | both) ;;
*)
  echo "Error: --build must be 'release' or 'both'." >&2
  exit 1
  ;;
esac

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
rust_dir="$repo_root/rust"
godot_dir="$repo_root/godot"
out_dir_abs="$repo_root/$out_dir"
app_bundle_path="$out_dir_abs/${app_name}.app"

if [[ -z "$zip_name" ]]; then
  zip_name="${app_name}-macos.zip"
fi
zip_path="$out_dir_abs/$zip_name"

assert_command_exists cargo
assert_command_exists "$godot_exe"
if [[ "$no_zip" -eq 0 ]]; then
  if command -v ditto >/dev/null 2>&1; then
    :
  else
    assert_command_exists zip
  fi
fi

mkdir -p "$out_dir_abs"

echo "Building Rust GDExtension ($build_mode)..."
(
  cd "$rust_dir"
  cargo build --release --locked
  if [[ "$build_mode" == "both" ]]; then
    cargo build --locked
  fi
)

echo "Validating Godot export preset ($preset_name)..."
ensure_export_preset_exists "$godot_dir/export_presets.cfg" "$preset_name"
normalize_godot_extension_list "$godot_dir"

echo "Exporting macOS app..."
rm -rf "$app_bundle_path"
godot_args=(--headless)
if [[ "$no_recovery_mode" -eq 0 ]]; then
  godot_args+=(--recovery-mode)
fi
godot_args+=(--path "$godot_dir" --export-release "$preset_name" "$app_bundle_path")
"$godot_exe" "${godot_args[@]}"

if [[ ! -d "$app_bundle_path" ]]; then
  echo "Error: export failed, app bundle not found: $app_bundle_path" >&2
  exit 1
fi

# Ensure the extension dylib is present in the app bundle at the same path
# used by res://../rust/target/release/librust.dylib in rust.gdextension.
export_dylib_dir="$app_bundle_path/Contents/rust/target/release"
mkdir -p "$export_dylib_dir"
cp -f "$rust_dir/target/release/librust.dylib" "$export_dylib_dir/librust.dylib"

if [[ "$no_zip" -eq 0 ]]; then
  echo "Packaging zip..."
  rm -f "$zip_path"
  if command -v ditto >/dev/null 2>&1; then
    ditto -c -k --sequesterRsrc --keepParent "$app_bundle_path" "$zip_path"
  else
    (
      cd "$out_dir_abs"
      zip -r "$zip_name" "${app_name}.app" >/dev/null
    )
  fi
fi

echo "Done."
echo "App: $app_bundle_path"
if [[ "$no_zip" -eq 0 ]]; then
  echo "Zip: $zip_path"
fi
