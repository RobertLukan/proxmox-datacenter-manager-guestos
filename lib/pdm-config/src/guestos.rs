//! GuestOS sidecar config (`/etc/proxmox-datacenter-manager/guestos.cfg`).

use anyhow::Error;

use proxmox_schema::ApiType;

use pdm_api_types::ConfigDigest;
use pdm_api_types::guestos::GuestOsConfig;
use pdm_buildcfg::configdir;

const CONF_FILE: &str = configdir!("/guestos.cfg");

/// Read GuestOS config. Missing file yields empty/default config.
pub fn config() -> Result<(GuestOsConfig, ConfigDigest), Error> {
    let content = proxmox_sys::fs::file_read_optional_string(CONF_FILE)?.unwrap_or_default();
    let digest = openssl::sha::sha256(content.as_bytes());
    let data: GuestOsConfig =
        proxmox_simple_config::from_str(&content, &GuestOsConfig::API_SCHEMA)?;
    Ok((data, digest.into()))
}

/// True when base-url, api-token, and launch-secret are all set.
pub fn is_configured(cfg: &GuestOsConfig) -> bool {
    cfg.base_url
        .as_ref()
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false)
        && cfg
            .api_token
            .as_ref()
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false)
        && cfg
            .launch_secret
            .as_ref()
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false)
}
