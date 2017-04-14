linux:
	docker run -v $$PWD:/volume -w /volume -t clux/muslrust:stable cargo build --release

push:
	scp target/x86_64-unknown-linux-musl/release/comicbot michael@m12y.com:/home/michael/comicbot/comicbot
