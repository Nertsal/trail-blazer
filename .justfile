list:
    just --list

game *ARGS:
    cargo run -- {{ARGS}}

web command *ARGS:
    cargo geng {{command}} --platform web --release -- {{ARGS}}
