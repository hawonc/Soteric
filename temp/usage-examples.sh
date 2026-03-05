#!/usr/bin/env bash

# JSON blacklist list (profile root is temp/, list entries are file names)
soteric add-profile demo-json --path temp --blacklist-from blacklist-files.json

# TOML blacklist list
soteric add-profile demo-toml --path temp --blacklist-from blacklist-files.toml

# YAML blacklist list
soteric add-profile demo-yaml --path temp --blacklist-from blacklist-files.yaml

# Plain text blacklist list
soteric add-profile demo-txt --path temp --blacklist-from blacklist-files.txt

# Glob-only blacklist (relative to profile root)
soteric add-profile demo-glob --path temp --blacklist-glob "*.txt"

# Mix: explicit + glob + file
soteric add-profile demo-mixed --path temp \
  --blacklist codex.txt \
  --blacklist-glob "*.txt" \
  --blacklist-from blacklist-files.txt

soteric list-profiles
