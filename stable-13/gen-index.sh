#!/bin/sh
find . -mindepth 2 -maxdepth 2 -name '*.toml' \
  | sed 's|^\./||; s|\.toml$||' \
  | sort > packages.index
