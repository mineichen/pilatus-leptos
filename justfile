[parallel]
dev: dev-frontend dev-backend

dev-frontend:
    @cd {{justfile_directory()}}/app && trunk serve

dev-backend:
    @cd {{justfile_directory()}}/app-backend && cargo run --target x86_64-unknown-linux-gnu

test:
    cd {{justfile_directory()}} && cargo test --target x86_64-unknown-linux-gnu


