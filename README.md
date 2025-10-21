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

infra

- [ ] railway testing?

devx

- [ ] github actions
- [ ] devcontainers
- [ ] video walkthrough

missing features

- perps
- pnl
- deposits / withdrawals

missing production

- helllaaa latency
- write ahead log lmao
- metrics / alerting
- backups / disaster recovery

### process

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
-

### frontend dev

just db-run
just db-setup
just run-backend
bun dev

### backend dev

just db-run
just db-setup
cargo run --release
