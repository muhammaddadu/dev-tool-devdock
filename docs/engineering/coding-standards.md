# Coding Standards

## Rust

- Format with `cargo fmt`. Lint with `cargo clippy -- -D warnings`. Both
  are required to pass before merge.
- Prefer `?` over `unwrap`; reserve `unwrap` / `expect` for genuinely
  infallible cases and document why with a one-line comment.
- Errors use `thiserror` enums per layer. Convert at boundaries; do not
  leak adapter errors into command return types.
- No OS calls in `core::`. Go through `PlatformAdapter`.
- Public functions and types in `core::` and `platform::` have rustdoc
  comments explaining intent, not mechanics.
- Module-level comments are allowed at the top of a file when they
  explain *why* the module exists.
- No inline comments unless they explain a non-obvious "why".

## TypeScript

- Strict mode in `tsconfig.json` (`"strict": true`).
- Format with Prettier. Lint with ESLint. Both required to pass.
- No `any`. Use `unknown` and narrow.
- Component files export a single default component plus its prop type.
- Hooks live in `src/lib/hooks/`. Co-locate component-specific hooks
  only when they are not reused.
- API wrappers around `invoke` live in `src/lib/api/` and are the only
  place `invoke` is called.
- No inline comments unless they explain a non-obvious "why".

## Naming

- Rust: `snake_case` for functions/files, `CamelCase` for types,
  `SCREAMING_SNAKE_CASE` for consts.
- TS: `camelCase` for functions/vars, `PascalCase` for components and
  types, `kebab-case` for non-component file names.

## Imports

- Rust: group `std`, external, then crate-local with blank lines.
- TS: group external, then `@/` aliases, then relative. No deep
  relative imports (`../../../`) — use the `@/` alias.

## Commits

- Conventional commits are preferred but not enforced.
- Each PR updates docs touched by behavior changes — see
  [`AGENTS.md`](../../AGENTS.md).
