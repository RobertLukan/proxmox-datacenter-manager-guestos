# Proxmox Datacenter Manager — GuestOS Sysprep fork

Thin **AGPL-3** fork of [proxmox/proxmox-datacenter-manager](https://github.com/proxmox/proxmox-datacenter-manager) (UI package) that adds GuestOS **Sysprep customize** entry points in PDM.

Corresponding GuestOS app: https://github.com/RobertLukan/proxmox-guestos-customization  
(**Sysprep** is the supported path; **WinRM reconfigure** in GuestOS standalone UI is legacy/deprecated and is not exposed from PDM.)

## What changed

- On a **Windows Proxmox template**: **Customize (GuestOS)** opens a signed  
  `{GUESTOS_BASE}/launch?template_vmid=…&remote_id=…&exp=…&jti=…&sig=…` deep-link  
  (HMAC; creates a GuestOS session and lands on the clone+Sysprep wizard).
- **GuestOS** tab (Remotes / per-remote): polls GuestOS `GET /api/tasks` for customization job history.
- Lab bake-in: `GUESTOS_BASE=https://192.168.123.197` (see `ui/src/guestos.rs`).
- Package version: `1.1.3+guestos.5` (based on upstream UI **1.1.3**).

No PDM API proxy — the browser talks to GuestOS HTTPS for launch tokens and the task list (lab API token baked into the UI wasm).

## Branch

- `guestos-sysprep` — fork work  
- Upstream pin: GitHub `proxmox/proxmox-datacenter-manager` @ `4c3b952b` (matches UI 1.1.3 / PDM 1.1.7 tree)

## Build (Debian trixie / PDM host)

Requires Proxmox UI build deps (`proxmox-wasm-builder`, `librust-proxmox-*`, `esbuild`, `rust-grass`, …) typically from Proxmox devel/build repositories, plus:

```bash
git submodule update --init --recursive   # ui/pwt-assets
cd ui && make deb
```

Or install built assets over `/usr/share/javascript/proxmox-datacenter-manager/` after a successful wasm build.

## Corresponding source (AGPL)

This repository is the corresponding source for the modified `proxmox-datacenter-manager-ui` packages we install in lab.

Upstream: https://git.proxmox.com/?p=proxmox-datacenter-manager.git  
Mirror: https://github.com/proxmox/proxmox-datacenter-manager  
GuestOS app: https://github.com/RobertLukan/proxmox-guestos-customization
