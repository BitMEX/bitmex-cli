#!/bin/sh
set -e

REPO="BitMEX/bitmex-cli"
BINARY="bitmex"
INSTALL_DIR="${BITMEX_INSTALL_DIR:-$HOME/.local/bin}"

get_target() {
  os=$(uname -s)
  arch=$(uname -m)

  case "$os" in
    Darwin)
      case "$arch" in
        x86_64)  echo "x86_64-apple-darwin" ;;
        arm64)   echo "aarch64-apple-darwin" ;;
        *)       echo "Unsupported architecture: $arch" >&2; exit 1 ;;
      esac
      ;;
    Linux)
      case "$arch" in
        x86_64)  echo "x86_64-unknown-linux-gnu" ;;
        aarch64) echo "aarch64-unknown-linux-gnu" ;;
        *)       echo "Unsupported architecture: $arch" >&2; exit 1 ;;
      esac
      ;;
    *)
      echo "Unsupported OS: $os" >&2
      echo "For Windows, download the binary manually from:" >&2
      echo "  https://github.com/${REPO}/releases" >&2
      exit 1
      ;;
  esac
}

on_path() {
  case ":$PATH:" in
    *":$1:"*) return 0 ;;
    *)        return 1 ;;
  esac
}

install_skills() {
  src_tarball="$1"
  skills_tmpdir=$(mktemp -d)
  trap 'rm -rf "$skills_tmpdir"' EXIT

  # Extract skills/ from source tarball (top-level dir varies, strip it)
  tar xzf "$src_tarball" -C "$skills_tmpdir" --strip-components=1 2>/dev/null || return 1
  skills_src="$skills_tmpdir/skills"
  [ -d "$skills_src" ] || return 1

  installed=""

  # OpenClaw
  if [ -d "$HOME/.openclaw" ]; then
    mkdir -p "$HOME/.openclaw/skills"
    for skill in "$skills_src"/bitmex-*/; do
      cp -r "$skill" "$HOME/.openclaw/skills/"
    done
    installed="${installed} OpenClaw"
  fi

  # Claude Code
  if [ -d "$HOME/.claude" ]; then
    mkdir -p "$HOME/.claude/commands"
    for skill in "$skills_src"/bitmex-*/; do
      name=$(basename "$skill")
      cp "$skill/SKILL.md" "$HOME/.claude/commands/${name}.md"
    done
    installed="${installed} Claude"
  fi

  # Hermes
  if [ -d "$HOME/.hermes" ]; then
    mkdir -p "$HOME/.hermes/skills"
    for skill in "$skills_src"/bitmex-*/; do
      cp -r "$skill" "$HOME/.hermes/skills/"
    done
    installed="${installed} Hermes"
  fi

  echo ""
  if [ -n "$installed" ]; then
    echo "Installed bitmex skills for:${installed}"
    echo "Start a new conversation in your AI tool for skills to be recognized."
  else
    echo "No AI agents detected. To install skills manually:"
    echo "  OpenClaw : cp -r skills/bitmex-* ~/.openclaw/skills/"
    echo "  Claude   : for s in skills/bitmex-*; do cp \"\$s/SKILL.md\" ~/.claude/commands/\$(basename \$s).md; done"
    echo "  Hermes   : cp -r skills/bitmex-* ~/.hermes/skills/"
  fi
}

main() {
  target=$(get_target)

  tag=$(curl -sSfL -o /dev/null -w "%{url_effective}" "https://github.com/${REPO}/releases/latest" 2>/dev/null | grep -o 'v[0-9][^/]*$' || true)
  if [ -z "$tag" ]; then
    echo "Error: could not determine latest release" >&2
    exit 1
  fi

  tarball="bitmex-cli-${target}.tar.gz"
  checksums="sha256.sum"

  echo "Installing ${BINARY} ${tag} (${target}) to ${INSTALL_DIR}..."
  echo ""

  tmpdir=$(mktemp -d)
  trap 'rm -rf "$tmpdir"' EXIT

  curl -sSfL "https://github.com/${REPO}/releases/download/${tag}/${tarball}" -o "$tmpdir/$tarball"
  curl -sSfL "https://github.com/${REPO}/releases/download/${tag}/${checksums}" -o "$tmpdir/$checksums"

  expected_hash=$(grep "$tarball" "$tmpdir/$checksums" | awk '{print $1}')
  if [ -z "$expected_hash" ]; then
    echo "Error: no checksum found for $tarball" >&2
    exit 1
  fi

  if command -v sha256sum >/dev/null 2>&1; then
    actual_hash=$(sha256sum "$tmpdir/$tarball" | awk '{print $1}')
  elif command -v shasum >/dev/null 2>&1; then
    actual_hash=$(shasum -a 256 "$tmpdir/$tarball" | awk '{print $1}')
  else
    echo "Error: need sha256sum or shasum to verify download" >&2
    exit 1
  fi

  if [ "$actual_hash" != "$expected_hash" ]; then
    echo "Error: checksum mismatch!" >&2
    echo "  Expected: $expected_hash" >&2
    echo "  Got:      $actual_hash" >&2
    exit 1
  fi

  echo "Checksum verified."
  tar xzf "$tmpdir/$tarball" -C "$tmpdir"

  # cargo-dist nests the binary inside <name>-<target>/
  src="$tmpdir/bitmex-cli-${target}/$BINARY"
  if [ ! -f "$src" ]; then
    echo "Error: binary not found at $src" >&2
    exit 1
  fi

  mkdir -p "$INSTALL_DIR"
  mv "$src" "$INSTALL_DIR/$BINARY"
  chmod +x "$INSTALL_DIR/$BINARY"

  echo ""
  echo "Installed ${BINARY} to ${INSTALL_DIR}/${BINARY}"

  if ! on_path "$INSTALL_DIR"; then
    echo ""
    echo "Note: ${INSTALL_DIR} is not on your PATH. Add it with:"
    case "${SHELL##*/}" in
      zsh)  echo "  echo 'export PATH=\"${INSTALL_DIR}:\$PATH\"' >> ~/.zshrc && source ~/.zshrc" ;;
      bash) echo "  echo 'export PATH=\"${INSTALL_DIR}:\$PATH\"' >> ~/.bashrc && source ~/.bashrc" ;;
      fish) echo "  fish_add_path ${INSTALL_DIR}" ;;
      *)    echo "  export PATH=\"${INSTALL_DIR}:\$PATH\"" ;;
    esac
  fi

  # Install skills (non-fatal if it fails)
  src_tarball="$tmpdir/source.tar.gz"
  if curl -sSfL "https://github.com/${REPO}/archive/refs/tags/${tag}.tar.gz" -o "$src_tarball" 2>/dev/null; then
    install_skills "$src_tarball" || echo "Warning: skill installation failed (skipping)"
  else
    echo ""
    echo "Warning: could not download skills — skipping skill installation."
  fi

  echo ""
  echo "Run 'bitmex --help' to get started."
}

main
