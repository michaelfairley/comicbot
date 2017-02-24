linux:
	docker run -v $$PWD:/volume -w /volume -t clux/muslrust:stable cargo build --release
