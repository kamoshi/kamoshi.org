#!/bin/bash

cloc \
  --force-lang="agda",astro \
  --exclude-dir=node_modules,dist \
  .
