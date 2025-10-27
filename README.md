# exchange

- for learning purposes

[![Deploy on Railway](https://railway.com/button.svg)](https://railway.com/deploy/U5i6Da?referralCode=trevor)

### todo

db -> me -> api -> web / bot

frontend

- [ ] websockets
- [ ] embedded wallet

backend

- [ ] make db schema
- [ ] make api endpoints
- [ ] db migrations
- [ ] env logger vs tracing
- [ ] socketio consideration?

infra

- [ ] railway testing?
- [ ] test with testcontainers. then local fullstack. then railway

devx

- [ ] github actions
- [ ] devcontainers
- [ ] video walkthrough

bots

- [ ] copy binance book. one maker copy entire orderbook. other taker on trade.
- [ ] ai bot. similar to nof1 or krafer nueral net trading.

missing features

- perps
- pnl
- deposits / withdrawals

missing production

- helllaaa latency
- write ahead log lmao
- design for concurrency
- metrics / alerting
- backups / disaster recovery
- scaling / k8s
- mm channel prioritization
- cancel prioritization
- nice rust/python/typescript sdks

# process

### process framework

- culmination of my frontend skills and new backend learning. made this as clean as possible while keeping scope tight.
- decide on structure, schema, and write e2e tests fast so iterate fast. stub out models just enough of the different ones to test.
- when starting from scratch, important to: decide structure. decide data shape. decide data flow. fill in rest of details. edge cases and optimization require more thinking

### main learnings

- setting up databases. using sqlx.
- designing domain/api/db models.
- using channels for engine input and websocket broadcast.

### log

- want to build exchange
- reference previous code
- talk to claude a lot
- scaffold project with the cleanest things i know
- decide on nextjs
- integrate openapi
- started designing api layer, rest and ws
- then db tables
- considered bot mechanics
- considered different db setup and ease of deployment. opted to not use supabase because dont want to manage that and clickhouse account. prefer one click deploy
- decide on clickhouse for candles, postgres for everything else
- decide to add wallet to make it crypto
- make one-click deploy on railway
- play with pgcli and usql. get nice syntax highlighting and basics of sql
- create simple postgres schema and run migrations
- start clickhouse integration
- make setup-db script and cleanup justfile
- use models for type management
- understand db <> domain <> api From and Into conversions
- use u128 integers and keep everything in atoms
- continue refining understanding of me and api layer architecture. consider socketio for ws.
- consider hexagon architecture
- opt for simpler impl rather than repository pattern
- use sqlx prepare
- decide on bigdecimal or u128. make conversion from db and domain
- clickhouse candles table a bit different than postgres
- add testcontainers for e2e testing, containers need to pass from utils to stay alive
- fix candles for half open interval. fix market creation with missing tokens.
- add error handling, thiserror and anyhow. learning about Result/Error/unwrap_or_else. use from macro.
- add db/app state to axum.
- add e2e api tests.
- setup ws api axum. naieve implemetnation. incoming/outgoing tasks, shared state, subscriptions is a set and filter braodcasted events through the set before sending.
- scaffold matching engine parts and their state, and api state
- use oneshot pattern for engine rest responses
- implement api endpoints and their db interactions
- hookup ws api to spawn two tasks to handle incoming and outgoing messages
- use tokio tungstenite for ws e2e testing
- add ws integration tests and improve utils
- consider orderbook snapshot emission from engine task
- implement engine. concurrency limitation for multiple markets since all pass through same engine.
- add tick/lot validation to engine and api
- hookup engine to db
- exchange e2e from api to engine to db
- fill out missing api endpoints and db functions
- address executor and orderbook consistency with db using transactions

- add trading signatures
- add benchmarks
- add cancel priority

### frontend dev

just db-run
just db-setup
just run-backend
bun dev

### backend dev

just db-run
just db-setup
cargo run --release
