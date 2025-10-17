# exchange

- for learning purposes

### todo

db -> me -> api -> web / bot

frontend

- [ ] websockets
- [ ] embedded wallet

backend

- [ ] supabase with foreign clickhouse table

infra

- [ ] local docker-compose
- [ ] railway
- [ ] vercel

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
-
