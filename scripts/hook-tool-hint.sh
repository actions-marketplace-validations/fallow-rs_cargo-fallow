#!/bin/sh
# Print one hint line for a tool that a git hook step did not find.
# hk.pkl calls this script. fallow reads the pinned version from mise.toml.
# This repository has no mise.toml, so CONTRIBUTING.md names the tools.
tool="$1"
echo "${tool} not found; install ${tool} (see the Quality gates section in CONTRIBUTING.md)" >&2
