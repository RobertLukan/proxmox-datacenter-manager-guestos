# Proxmox Datacenter Manager — GuestOS Sysprep fork

Thin **AGPL-3** fork of [proxmox/proxmox-datacenter-manager](https://github.com/proxmox/proxmox-datacenter-manager) (UI package) that adds a **Sysprep (GuestOS)** action on the QEMU guest panel.

## What changed

- QEMU panel toolbar: **Sysprep (GuestOS)** opens  
  `{GUESTOS_BASE}/sysprep_existing_vm_form/{vmid}?remote_id={pdm_remote}`  
  in a new tab.
- Lab default: `GUESTOS_BASE=http://192.168.123.197:5001` (see `ui/src/guestos.rs`).
- Package version: `1.1.3+guestos.1` (based on upstream UI **1.1.3**).

No PDM API proxy and no GuestOS API token in the browser — operators use a normal GuestOS login session for the wizard.

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
