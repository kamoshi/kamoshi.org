build:
	cargo run

deploy: build
	rsync -Pavzq ./dist/ kamoshi:/var/www/kamoshi.org --delete

serve:
	python -m http.server -d ./dist
