#!/bin/bash

pnpm build
rsync -Pavz -e "ssh -i ~/.ssh/id_ed25519" ./dist/ 70.34.244.173:/var/www/kamoshi.org --delete
echo "Done"
