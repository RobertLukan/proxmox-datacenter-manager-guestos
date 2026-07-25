//! GuestOS sidecar deep-link helpers (lab bake-in).
//!
//! GUESTOS_BASE is the public URL of the GuestOS Compose stack as reached from
//! operator browsers on the PDM network. Rebuild the UI package to change it.

/// Public GuestOS base URL (no trailing slash).
pub const GUESTOS_BASE: &str = "http://192.168.123.197:5001";

/// Deep-link into the existing-VM sysprep wizard with PDM `remote_id`.
pub fn sysprep_existing_url(vmid: u32, remote_id: &str) -> String {
    let remote = percent_encoding::utf8_percent_encode(
        remote_id,
        percent_encoding::NON_ALPHANUMERIC,
    );
    format!(
        "{GUESTOS_BASE}/sysprep_existing_vm_form/{vmid}?remote_id={remote}"
    )
}
