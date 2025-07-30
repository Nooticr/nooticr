```markdown
# Rust Production-Grade GraphQL Server – Coding Philosophy & Top-Down Workflow

> “Top-down, **no TODOs**, **no mocks**, **deliver today**.”

---

## 1. Top-Down Writing Order (Recommended Flow)

Write each new slice of functionality **top-to-bottom** in the exact order a future reader will mentally trace it:

1. **Entrypoint** (`main.rs`)  
   Wire the minimal happy path so the binary compiles and the server starts.

2. **Router / Handlers** (`handlers/mod.rs`)  
   Expose the HTTP route or GraphQL field signature **without** any body—return hard-coded `Ok("todo")`.

3. **Schema & Merged Objects** (`graphql/mutation/mod.rs`, etc.)  
   Add the new resolver stub to the appropriate `struct XxxRoot`.

4. **Guard** (`guards/verify_token.rs`)  
   Create or reuse the guard that checks auth/context before the resolver runs.

5. **Context Data** (`contexts/token.rs`, `user_uid.rs`)  
   Define the exact data structures you’ll `ctx.data_unchecked::<T>()`.

6. **Domain / Repository** (`database/chat_repository.rs`)  
   Implement the **real** SQL query or Redis call.

7. **Return Type** (`domain/chat_room.rs`)  
   Finalize the GraphQL object you’ll return.

8. **Integration Test** (`tests/graphql/chat.rs`)  
   Spin up Postgres with `testcontainers` and assert the full flow.

9. **Back-fill**  
   Replace the hard-coded `"todo"` with the real repository call.

> **Never write TODO comments**—write a **compiling stub** that returns a literal value, then replace it.

---

## 2. Code-Writing Best Practices (Rust-Specific)

| Rule | Explanation |
|------|-------------|
| **Write Top-Down** | Start in `main.rs`, move downward through layers. Reader’s mental stack matches the file order. |
| **No Mocks** | Use `testcontainers` + real Postgres / Redis in tests. Mocks hide integration bugs. |
| **No TODOs** | Stub with real types (`return Ok(Default::default())`)—the compiler keeps you honest. |
| **One File per Enum** | `src/enums/role.rs` → single source of truth for GraphQL, DB, JSON. |
| **One File per GraphQL Domain** | `src/graphql/mutation/user.rs` contains **all** user mutations. Keeps merge conflicts low. |
| **Context Injection via Schema.data()** | `.data::<Arc<Client>>()`, `.data::<Arc<Mutex<UserUID>>>()`—no thread-local or singletons. |
| **Guards as Pure Functions** | `VerifyTokenGuard` only returns `Result<(), Error>`; no side effects. |
| **Structured Logging** | `tracing::info!(user_id = %uid, "message")`—works with OpenTelemetry traces. |
| **Strong Typing Everywhere** | Newtype wrappers (`struct UserUID(String)`) prevent mixing up IDs. |
| **Migrations First** | Every PR ships with `migrations/000NNN_*.sql`. CI runs `sqlx migrate run` before tests. |
| **Feature Flags via Env** | `if cfg!(feature = "dev_routes") { … }`—compile-time toggles, not runtime `if`. |

---

## 3. Stub Template (Copy-Paste)

```rust
// 1. handler
#[handler]
pub async fn create_chat_room(_req: Json<CreateChatRoomInput>) -> Result<impl IntoResponse> {
    Ok(Json(ChatRoom::default())) // compiles, no TODO
}

// 2. graphql resolver
#[Object]
impl ChatMutations {
    #[graphql(guard = "VerifyTokenGuard")]
    async fn create_chat_room(
        &self,
        _ctx: &Context<'_>,
        _input: CreateChatRoomInput,
    ) -> Result<ChatRoom> {
        Ok(ChatRoom::default())
    }
}

// 3. test
#[tokio::test]
async fn create_chat_room_works() {
    let room = create_chat_room(Json::default()).await.unwrap();
    assert_eq!(room.id, Uuid::nil()); // hard-coded check
}
```

Replace `ChatRoom::default()` with the **real** repository call once the stub compiles and the test passes.

---

## 4. Cheat-Sheet: Where to Add What

| Add… | In File |
|------|---------|
| New GraphQL field | `src/graphql/mutation/<domain>.rs` |
| New enum | `src/enums/<name>.rs` |
| New auth rule | `src/guards/<name>.rs` |
| New SQL query | `src/database/<domain>_repository.rs` |
| New context | `src/contexts/<name>.rs` |
| New HTTP route (non-GraphQL) | `src/handlers/mod.rs` |

---

## 5. Quick Start Checklist

1. `cargo new --bin my app && cd myapp`
2. Paste `Cargo.toml` deps from context.
3. Copy directory layout above.
4. Run `docker compose up --build` → playground at `http://localhost:8000/graphql`.
5. Write **top-down** for every new features


<!--  -->

```markdown
# Rust GraphQL API – Final Production Context  
(enums split 1-file-per-enum, **one file per GraphQL field**, complete CI/CD delivery pipeline)

---

## 1. Directory Layout (GraphQL-first, **one file per field**)

```
src/
├── auth/
│   └── mod.rs
├── config.rs
├── database/
│   └── mod.rs
├── domain/
│   └── user.rs
├── enums/
│   ├── mod.rs
│   ├── role.rs
│   └── order_status.rs
├── graphql/
│   ├── mutation/
│   │   ├── mod.rs          // re-exports & MergedObject
│   │   └── user.rs         // one file per domain (contains ALL user mutations)
│   ├── queries/
│   │   ├── mod.rs
│   │   └── user.rs
│   └── subscriptions/
│       ├── mod.rs
│       └── user.rs
├── guards/
│   └── verify_token.rs
├── handlers/
│   └── mod.rs
├── contexts/
│   ├── token.rs
│   └── user_uid.rs
├── telemetry.rs
├── main.rs
migrations/
tests/
.github/
└── workflows/
    ├── ci.yml
    └── cd.yml            // NEW: delivery pipeline
Dockerfile
compose.yml
```

---

## 2. One-file-per-enum Convention (already shown)

`src/enums/role.rs`

```rust
use async_graphql::*;

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum Role {
    Admin,
    User,
}
```

---

## 3. One file per GraphQL **domain** (not per field)

`src/graphql/mutation/user.rs`

```rust
use async_graphql::*;
use sqlx::PgPool;

use crate::{auth, guards::verify_token::VerifyTokenGuard};

#[derive(Default)]
pub struct UserMutations;

#[Object]
impl UserMutations {
    /// Create a new user
    #[graphql(guard = "VerifyTokenGuard")]
    async fn sign_up(
        &self,
        ctx: &Context<'_>,
        email: String,
        password: String,
    ) -> Result<domain::User> {
        let pool = ctx.data_unchecked::<PgPool>();
        let hash = auth::hash_password(&password)?;
        let rec = sqlx::query_as!(
            domain::User,
            r#"
            INSERT INTO users (email, password_hash)
            VALUES ($1, $2)
            RETURNING id, email, created_at
            "#,
            email,
            hash
        )
        .fetch_one(pool)
        .await?;
        Ok(rec)
    }
}
```

`src/graphql/mutation/mod.rs`

```rust
pub mod user;

pub use user::UserMutations;
// … import other domain mutations …

use async_graphql::MergedObject;

#[derive(MergedObject, Default)]
pub struct MutationRoot(
    UserMutations,
    // MissionMutations,
    // …
);
```

Same pattern for **queries** and **subscriptions**.

---

## 4. Context wiring (`src/graphql/mod.rs`)

```rust
pub use mutations::MutationRoot;
pub use queries::QueryRoot;
pub use subscriptions::SubscriptionRoot;

use async_graphql::Schema;
use tokio_postgres::Client;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::contexts::user_uid::UserUID;

pub type PointIdSchema = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;

pub fn create_schema_dynamic_context(
    db: Arc<Client>,
) -> PointIdSchema {
    Schema::build(QueryRoot, MutationRoot, SubscriptionRoot)
        .data(db)
        .data(Arc::new(Mutex::new(UserUID::new())))
        .finish()
}
```

---

## 5. Guard (`src/guards/verify_token.rs`)

```rust
use async_graphql::*;
use crate::contexts::{token::Token, user_uid::UserUID};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct VerifyTokenGuard;

#[async_trait::async_trait]
impl Guard for VerifyTokenGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        let token = ctx
            .data::<Token>()
            .map_err(|_| Error::new("AUTH::Unauthorized"))?;

        let user_uid = ctx.data::<Arc<Mutex<UserUID>>>()?;
        let clean = token.0.trim_start_matches("Bearer ").trim();
        let (uid, _) = crate::auth::extract_user_from_token(clean)
            .map_err(|_| Error::new("AUTH::Unauthorized"))?;
        user_uid.lock().await.update(uid);
        Ok(())
    }
}
```

---

## 6. Handlers (`src/handlers/mod.rs`) – unchanged

Still serve `/graphql`, `/graphql/ws`, `/health`, plus optional download routes.

---

## 7. CI Pipeline (`.github/workflows/ci.yml`)

```yaml
name: ci
on: [push, pull_request]

env:
  CARGO_TERM_COLOR: always
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { components: "rustfmt, clippy" }
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --all -- --check
      - run: cargo clippy --all-targets --all-features -- -D warnings

  test:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:16
        env: { POSTGRES_PASSWORD: postgres }
        ports: ["5432:5432"]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --locked --all-features

  build-push:
    needs: [lint, test]
    runs-on: ubuntu-latest
    permissions:
      contents: read
      packages: write
    steps:
      - uses: actions/checkout@v4
      - uses: docker/setup-buildx-action@v3
      - uses: docker/login-action@v3
        with:
          registry: ${{ env.REGISTRY }}
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}
      - uses: docker/metadata-action@v5
        id: meta
        with: { images: "${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}" }
      - uses: docker/build-push-action@v5
        with:
          context: .
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
```

---

## 8. CD Pipeline (`.github/workflows/cd.yml`)

```yaml
name: cd
on:
  push:
    branches: [main]

env:
  IMAGE_NAME: ghcr.io/${{ github.repository }}
  KUBE_CONFIG: ${{ secrets.KUBE_CONFIG }}

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build & push production image
        uses: ./.github/actions/build-push   # re-use composite action
      - name: Install kubectl
        uses: azure/setup-kubectl@v3
      - name: Deploy to k8s
        run: |
          echo "${{ env.KUBE_CONFIG }}" | base64 -d > kubeconfig
          export KUBECONFIG=kubeconfig
          kubectl set image deployment/myapp myapp=${{ env.IMAGE_NAME }}:${{ github.sha }} -n prod
```

---

## 9. Dockerfile (unchanged)

```dockerfile
FROM rust:1.80-slim AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/myapp /usr/local/bin/myapp
ENTRYPOINT ["/usr/local/bin/myapp"]
```

---

## 10. Quick Commands

```bash
# Local dev
docker compose up --build

# Schema generation
cargo run --bin export-schema > schema.graphql
```

---

All **TODOs replaced with working code**, no mocks in tests, CI/CD delivers to **GHCR + Kubernetes**.