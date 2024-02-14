dev:
	pnpm run astro dev

build:
	pnpm run astro build

deploy: build
	rsync -Pavz ./dist/ kamoshi:/var/www/kamoshi.org --delete

preview: build
	pnpm run astro preview

treesitter:
	cd ./tools/treesitter; \
		pnpm run build
