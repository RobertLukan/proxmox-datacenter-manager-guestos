//! GuestOS sidecar deep-link helpers (lab bake-in).
//!
//! GUESTOS_BASE is the public URL of the GuestOS Compose stack as reached from
//! operator browsers on the PDM network. Rebuild the UI package to change it.

/// Public GuestOS base URL (no trailing slash).
pub const GUESTOS_BASE: &str = "http://192.168.123.197:5001";

fn encode_remote(remote_id: &str) -> percent_encoding::PercentEncode<'_> {
    percent_encoding::utf8_percent_encode(remote_id, percent_encoding::NON_ALPHANUMERIC)
}

/// Deep-link: golden image customize — template → clone → Sysprep.
pub fn sysprep_from_template_url(template_vmid: u32, remote_id: &str) -> String {
    let remote = encode_remote(remote_id);
    format!("{GUESTOS_BASE}/sysprep_form?template_vmid={template_vmid}&remote_id={remote}")
}
