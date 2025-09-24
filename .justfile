list:
    just --list

game *ARGS:
    cargo run -- {{ARGS}}

client *ARGS:
    cargo run -- --connect ws://127.0.0.1:1155 {{ARGS}}

web command *ARGS:
    cargo geng {{command}} --platform web --release -- {{ARGS}}

server := "trail-blazer.nertsal.com"
server_user := "nertsal"

update-server:
    docker run --rm -it -e CARGO_TARGET_DIR=/target -v `pwd`/docker-target:/target -v `pwd`:/src -w /src ghcr.io/geng-engine/cargo-geng cargo geng build --release
    rsync -avz docker-target/geng/ {{server_user}}@{{server}}:trail-blazer/
    ssh {{server_user}}@{{server}} systemctl --user restart trail-blazer

publish-web:
    CONNECT=wss://{{server}} cargo geng build --release --platform web --out-dir target/geng
    butler -- push target/geng nertsal/trail-blazer:html5

deploy:
    just update-server
    just publish-web
