[parallel]
dev: devf devb

devf:
    @cd {{justfile_directory()}}/pilatus-leptos-app && trunk serve --features examples

devb:
    @cd {{justfile_directory()}}/app-backend && cargo run --target x86_64-unknown-linux-gnu --release

test:
    cd {{justfile_directory()}} && cargo test --target x86_64-unknown-linux-gnu


