list:
    just --list

run *ARGS:
    cargo run {{ARGS}}

web command *ARGS:
    cargo geng {{command}} --platform web --release -- {{ARGS}}
