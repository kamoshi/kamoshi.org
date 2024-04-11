build:
	cargo run

deploy: build
	rsync -Pavz ./dist/ kamoshi:/var/www/kamoshi.org --delete --mkpath

serve:
	python -m http.server -d ./dist
