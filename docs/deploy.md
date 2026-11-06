# Deploying Scratch-me

Scratch-me runs in Docker next to AnyCardz on the same server: its own
Postgres, API and SvelteKit server, in their own Compose project (`scratchme`),
network and volume. The host's nginx, which already serves AnyCardz, serves it
on 80/443 too. Nothing of Scratch-me is published beyond the host's loopback.

| | AnyCardz | Scratch-me |
| --- | --- | --- |
| API | 127.0.0.1:8080 | 127.0.0.1:8081 (`API_PORT`) |
| Site | 127.0.0.1:3000 | 127.0.0.1:3001 (`WEB_PORT`) |
| Metrics | 127.0.0.1:9100 | 127.0.0.1:9101 (`METRICS_PUBLISH`) |
| Docker network | 172.30.0.0/24 | 172.31.0.0/24 (`SUBNET`) |

- [What you need](#what-you-need)
- [1. Get the code](#1-get-the-code)
- [2. Configure](#2-configure)
- [3. Start the containers](#3-start-the-containers)
- [4. Certificate and nginx](#4-certificate-and-nginx)
- [5. Check it works](#5-check-it-works)
- [6. Make yourself an admin](#6-make-yourself-an-admin)
- [Updating, backups, logs](#updating-backups-logs)
- [Monitoring](#monitoring)

## What you need

- The AnyCardz server: Docker Engine with the Compose plugin, nginx and certbot on the host
- A domain name (or subdomain) whose A/AAAA records point at the server
- About 2 GB of free RAM while the images build (the Rust build is the heaviest step), 300 MB to run them

## 1. Get the code

```sh
git clone https://github.com/ef3d0c3e/scratch-me.git
cd scratch-me
```

## 2. Configure

```sh
cp .env.prod.example .env.prod
chmod 600 .env.prod
```

| Variable | What to put |
| -------- | ----------- |
| `DOMAIN` | The public host name, e.g. `scratch.example.org` |
| `POSTGRES_PASSWORD` | A long random string: `openssl rand -hex 24` |
| `API_PORT`, `WEB_PORT` | Leave at 8081 and 3001 unless taken on the host (`ss -ltn`) |
| `METRICS_PUBLISH`, `METRICS_TOKEN` | Leave as is for a Prometheus on the same server, see [Monitoring](#monitoring) |
| `RUST_LOG` | `info` (default), `debug` to investigate |

Session cookies are `Secure` (`SECURE_COOKIES` defaults to 1), as the site is
served over HTTPS. Never set `SCRATCHME_RELAXED_LIMITS` here: it lifts the
login and sign-up rate limits, for tests only.

To type plain `docker compose …` in this directory without `-f` and
`--env-file` (and never start the development `compose.yaml` by mistake):

```sh
echo 'COMPOSE_FILE=compose.prod.yaml' >> .env.prod
ln -s .env.prod .env
```

## 3. Start the containers

```sh
make prod-up    # docker compose -f compose.prod.yaml --env-file .env.prod up -d --build
```

The first build takes several minutes. Database migrations run when the API starts.
Check both answer on the host:

```sh
curl -s 127.0.0.1:8081/api/health     # {"ok":true}
curl -sI 127.0.0.1:3001/login | head -1   # HTTP/1.1 200 OK
```

**The network.** If `up` fails with `Pool overlaps with other one on this
address space`, 172.31.0.0/24 is taken on the server. Pick a free range and
set both lines in `.env.prod`, e.g. `SUBNET=172.32.0.0/24` and
`TRUSTED_PROXIES=172.32.0.1` (the range's `.1`), then `make prod-up` again.

## 4. Certificate and nginx

**Certificate** (nginx already answers the ACME challenge from `/var/www/html`
for AnyCardz):

```sh
sudo certbot certonly --webroot -w /var/www/html -d scratch.example.org
```

If no site serves that path for the new name yet, use `sudo certbot certonly --nginx -d scratch.example.org` instead.

**Site:**

```sh
sudo cp deploy/nginx.conf /etc/nginx/sites-available/scratchme
sudo sed -i 's/scratch.example.org/YOUR.DOMAIN/g' /etc/nginx/sites-available/scratchme
sudo ln -s /etc/nginx/sites-available/scratchme /etc/nginx/sites-enabled/
sudo nginx -t && sudo systemctl reload nginx
```

On distributions without `sites-available`, copy it to `/etc/nginx/conf.d/scratchme.conf`.
If you changed `API_PORT` or `WEB_PORT`, change the two `upstream` lines to match.

What the file does:

- `/api/` goes to the API, with the WebSocket upgrade the live feed and chat need,
  and a long read timeout so the socket stays open
- everything else goes to the SvelteKit server
- HTTP redirects to HTTPS; gzip for text; a week of caching for the favicon
- its WebSocket `map` uses its own variable (`$scratchme_upgrade`): nginx
  refuses the same `map` twice, and AnyCardz's file defines `$connection_upgrade`

**Client addresses.** Login and sign-up limits are per client address. nginx
reaches the containers from the Docker network's gateway (172.31.0.1), the
only proxy the API believes, and passes the player's address in
`X-Forwarded-For`. Check it once: try a wrong password on the site, then

```sh
docker compose -f compose.prod.yaml --env-file .env.prod logs api | grep "login failed"
# … login failed client=203.0.113.7   ← your public address: good
```

- `client=172.31.0.1` on every line: Docker forwards from another address on
  this server. Put the address you see in `TRUSTED_PROXIES` and `make prod-up`.
- Any other address you did not expect: nginx is not setting the header. Every
  `location` in the file sets its `proxy_set_header` lines on purpose; keep it
  that way when you edit it.

## 5. Check it works

```sh
curl -s https://YOUR.DOMAIN/api/health   # {"ok":true}
```

Then open the site on a phone and play as a guest. In another browser, open
the chat. Messages appear in both without a reload, which shows the WebSocket
works through nginx.

## 6. Make yourself an admin

Admins post announcements from the Chat page. Sign up on the site, then:

```sh
docker compose -f compose.prod.yaml --env-file .env.prod exec api scratchme-api make-admin <username>
```

## Updating, backups, logs

```sh
# Update: rebuilds the images; migrations run when the API starts
git pull && make prod-up

# Back up the database
docker compose -f compose.prod.yaml --env-file .env.prod exec -T db \
  pg_dump -U scratchme -Fc scratchme > scratchme-$(date +%F).dump

# Restore into the running database
docker compose -f compose.prod.yaml --env-file .env.prod exec -T db \
  pg_restore -U scratchme -d scratchme --clean --if-exists < scratchme-2026-11-08.dump

# Logs
make prod-logs
```

Keep `.env.prod` somewhere safe: it holds the database password. A daily
backup from cron, kept two weeks:

```cron
30 4 * * * cd /path/to/scratch-me && docker compose -f compose.prod.yaml --env-file .env.prod exec -T db pg_dump -U scratchme -Fc scratchme > backups/scratchme-$(date +\%F).dump && find backups -name '*.dump' -mtime +14 -delete
```

(Make the `backups` directory first.)

## Monitoring

The API serves Prometheus metrics on a listener of its own, published on
`METRICS_PUBLISH` (default `127.0.0.1:9101`), never through nginx.

**Prometheus.** Add the job from `deploy/prometheus/scrape.yml` to the
`prometheus.yml` that already scrapes AnyCardz. With the default (host only),
no token is needed: drop its `authorization` block.

```yaml
scrape_configs:
  - job_name: scratchme
    scrape_interval: 30s
    static_configs:
      - targets: ['127.0.0.1:9101']
```

A Prometheus in a container reaches it the same way AnyCardz's is reached: with
`network_mode: host`. Prometheus on another machine needs `METRICS_PUBLISH=0.0.0.0:9101`,
a `METRICS_TOKEN` (`openssl rand -hex 32`, put the same value in
`/etc/prometheus/scratchme.token` there), and a firewall rule letting only
that machine reach port 9101.

Check it, then look for the `scratchme` job under Status → Targets in Prometheus:

```sh
curl -s 127.0.0.1:9101/metrics | grep scratchme_players
```

**Grafana.** Dashboards → New → Import, upload `deploy/grafana/scratchme.json`,
then pick the Prometheus data source in the menu at the top. Worth an alert:
`sum(rate(scratchme_http_requests_total{status=~"5.."}[5m])) > 0.1` (server
errors) and `up{job="scratchme"} == 0` (the API is down).
