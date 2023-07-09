#!/bin/bash

pnpm build
rsync -Pavz ./dist/ kamoshi.org:/var/www/kamoshi.org --delete
echo "Done"
