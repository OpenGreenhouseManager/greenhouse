# Backend recipes: `just backend <recipe>` (backend/justfile)
mod backend

# Web recipes: `just web <recipe>` (web/justfile)
mod web

default:
    @just --list --list-submodules

# Lint both projects
lint: backend::lint web::lint

# Test both projects
test: backend::test web::test
