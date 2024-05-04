build:
	cargo run --release

watch:
	cargo run --release -- watch

serve:
	python -m http.server -d ./dist

deploy: build
	rsync -Pavzq ./dist/ kamoshi:/var/www/kamoshi.org --delete
