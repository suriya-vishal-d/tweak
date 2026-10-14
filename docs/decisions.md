# Tweak — Architecture Decisions

## AD-001 — Default scheme for bare URLs

**Date:** 2026-10-14

When a user types a URL without a scheme (e.g. `localhost:3000` or `api.example.com`),
Tweak infers the scheme based on the host:

- **`http://`** for local/private addresses: `localhost`, `*.localhost`, `127.0.0.0/8`,
  `10.x`, `172.16–31.x`, `192.168.x`, `::1`, single-label hostnames (no dots, e.g. `myserver:8080`),
  `*.local`, `*.internal`, `*.lan`, `100.64.0.0/10` (CGNAT/Tailscale), and `169.254.0.0/16` (link-local).
- **`https://`** for every other host.

**Rationale:** Local development servers almost never have TLS certificates, so defaulting
to `http` prevents spurious connection errors.  Public-facing hosts on the internet nearly
always serve HTTPS, so defaulting to `https` avoids an extra redirect and matches browser
behaviour.  Users who need the non-default scheme can always type the full URL with an
explicit scheme.

