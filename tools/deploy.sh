#!/bin/bash

pnpm build
rsync -Pavz ./dist/ kamoshi:/var/www/kamoshi.org --delete
echo "Done"

