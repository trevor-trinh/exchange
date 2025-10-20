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

missing features

- perps
- pnl
- deposits / withdrawals

missing production

- helllaaa latency
- metrics / alerting
- backups

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
- create simple schema and run migrations

### frontend dev

just run-db
just run-backend
bun dev

### backend dev

just run-db
cargo run --release
