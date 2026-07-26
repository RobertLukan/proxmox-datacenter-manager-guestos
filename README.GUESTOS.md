# Proxmox Datacenter Manager — GuestOS Sysprep fork

Thin **AGPL-3** fork of [proxmox/proxmox-datacenter-manager](https://github.com/proxmox/proxmox-datacenter-manager) that adds GuestOS **Sysprep customize** entry points in PDM (UI + small server proxy).

Corresponding GuestOS app: https://github.com/RobertLukan/proxmox-guestos-customization  
(**Sysprep** is the supported path; **WinRM reconfigure** in GuestOS standalone UI is legacy/deprecated and is not exposed from PDM.)

## What changed

- On a **Windows Proxmox template** (QEMU `ostype` win*): **Customize (GuestOS)** asks the PDM server for a signed launch URL, then opens GuestOS `/launch?…` (HMAC; session → clone+Sysprep wizard). Non-Windows templates do not show the button.
- **GuestOS** tab (Remotes / per-remote): polls PDM `GET /api2/extjs/guestos/tasks` (server proxies GuestOS with the machine token).
- Secrets and `base-url` live in `/etc/proxmox-datacenter-manager/guestos.cfg` — **not** in the UI wasm.
- Package versions: UI `1.1.3+guestos.8`, server `1.1.7+guestos.8` (based on
  upstream UI **1.1.3** / server tree **1.1.7**).

## Server config

Create `/etc/proxmox-datacenter-manager/guestos.cfg` (see [`docs/guestos.cfg.example`](docs/guestos.cfg.example)):

```
base-url: https://guestos.example.com
api-token: <GUESTOS_API_TOKEN>
launch-secret: <GUESTOS_LAUNCH_SECRET>
launch-ttl: 300
verify-tls: true
```

Lab self-signed Caddy: set `verify-tls: false`. File ownership should be `www-data:www-data` mode `640`.

API (authenticated PDM session):

| Method | Path | Purpose |
|--------|------|---------|
| POST | `/api2/extjs/guestos/launch?remote-id=&template-vmid=` | Sign launch URL |
| GET | `/api2/extjs/guestos/tasks?remote-id=` | Proxy task history |

## Branch

- `guestos-sysprep` — fork work  
- Upstream pin: GitHub `proxmox/proxmox-datacenter-manager` @ `4c3b952b` (matches UI 1.1.3 / PDM 1.1.7 tree)

## Build (Debian trixie / PDM host)

Requires Proxmox UI build deps (`proxmox-wasm-builder`, `librust-proxmox-*`, `esbuild`, `rust-grass`, …) typically from Proxmox devel/build repositories, plus:

```bash
git submodule update --init --recursive   # ui/pwt-assets
# Server package (includes guestos proxy):
make deb
# Or UI-only:
cd ui && make deb
```

Install the resulting `.deb` files on the PDM host, then:

```bash
sudo apt-mark hold proxmox-datacenter-manager proxmox-datacenter-manager-ui
```

so upstream upgrades do not replace the fork. There is no public APT repo yet —
copy artifacts by hand (or your internal package mirror).

Install `guestos.cfg` on the PDM host before using Customize / the GuestOS tab.

**End-to-end product install** (GuestOS Compose + this fork, production vs lab):
see GuestOS [`docs/INSTALL.md`](https://github.com/RobertLukan/proxmox-guestos-customization/blob/main/docs/INSTALL.md).

## Corresponding source (AGPL)

This repository is the corresponding source for the modified `proxmox-datacenter-manager` / `-ui` packages we install in lab.

Upstream: https://git.proxmox.com/?p=proxmox-datacenter-manager.git  
Mirror: https://github.com/proxmox/proxmox-datacenter-manager  
GuestOS app: https://github.com/RobertLukan/proxmox-guestos-customization
