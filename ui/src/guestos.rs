//! GuestOS Sysprep helpers — secrets stay on the PDM server (`guestos.cfg` + `/api2/.../guestos`).

use anyhow::{bail, format_err};
use pdm_api_types::ConfigurationState;

/// True when QEMU `ostype` is a Windows guest (Sysprep path only).
pub fn is_windows_ostype<T: ToString>(ostype: &Option<T>) -> bool {
    match ostype {
        Some(o) => {
            let s = o.to_string().to_ascii_lowercase();
            s.starts_with("win") || s.starts_with("w2k") || s == "wxp" || s == "wvista"
        }
        None => false,
    }
}

/// Fetch QEMU config and report whether the guest is Windows.
pub async fn template_is_windows(
    remote: &str,
    node: Option<&str>,
    vmid: u32,
) -> Result<bool, anyhow::Error> {
    let config = crate::pdm_client()
        .pve_qemu_config(remote, node, vmid, ConfigurationState::Active, None)
        .await
        .map_err(|e| format_err!("failed to load VM config: {e}"))?;
    Ok(is_windows_ostype(&config.ostype))
}

/// Server-signed launch URL for template → clone → Sysprep (Windows templates only).
pub async fn launch_sysprep_customize(
    remote: &str,
    node: Option<&str>,
    template_vmid: u32,
) -> Result<String, anyhow::Error> {
    if !template_is_windows(remote, node, template_vmid).await? {
        bail!("GuestOS Customize is only available for Windows templates (check QEMU ostype).");
    }
    let resp = crate::pdm_client()
        .guestos_launch(remote, template_vmid)
        .await
        .map_err(|e| format_err!("GuestOS launch failed: {e}"))?;
    if resp.url.is_empty() {
        bail!("GuestOS launch returned an empty URL");
    }
    Ok(resp.url)
}

/// Workflow detail page for a GuestOS task id.
pub fn workflow_url(base_url: &str, task_id: &str) -> String {
    let base = base_url.trim().trim_end_matches('/');
    format!("{base}/workflow/{task_id}")
}

pub use pdm_client::types::GuestOsTask;
