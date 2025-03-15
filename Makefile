build:
	cargo run --release

watch:
	cargo run --release -- watch

serve:
	echo "http://[::1]:1234/"
	python -m http.server 1234 -b ::1 -d ./dist

deploy: build
	rsync -Pavzq ./dist/ kamoshi:/var/www/kamoshi.org --delete

perf:
	cargo flamegraph
