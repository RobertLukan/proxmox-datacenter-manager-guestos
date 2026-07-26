//! GuestOS sidecar deep-link helpers + task list fetch (lab bake-in).
//!
//! GUESTOS_BASE / secrets must match the GuestOS Compose `.env`. Rebuild the UI
//! package after changing these values.

use anyhow::{bail, format_err};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;

/// Public GuestOS base URL (no trailing slash) — TLS via Caddy on the sidecar.
pub const GUESTOS_BASE: &str = "https://192.168.123.197";

/// Shared HMAC secret (hex) — must match GuestOS ``GUESTOS_LAUNCH_SECRET``.
pub const GUESTOS_LAUNCH_SECRET: &str =
    "7c8f3a1e9b2d4f6a0c5e8b1d3f7a9c2e4b6d8f0a1c3e5b7d9f2a4c6e8b0d1f3a";

/// Machine API token — must match GuestOS ``GUESTOS_API_TOKEN`` (task list).
pub const GUESTOS_API_TOKEN: &str = "jozjij-forno";

/// Launch token lifetime in seconds.
pub const GUESTOS_LAUNCH_TTL_SECS: u64 = 300;

type HmacSha256 = Hmac<Sha256>;

fn encode_component(value: &str) -> percent_encoding::PercentEncode<'_> {
    percent_encoding::utf8_percent_encode(value, percent_encoding::NON_ALPHANUMERIC)
}

fn unix_now() -> u64 {
    js_sys::Date::now() as u64 / 1000
}

fn random_jti() -> String {
    let a = js_sys::Math::random();
    let b = js_sys::Math::random();
    format!(
        "{:08x}{:08x}",
        (a * 0xffff_ffffu32 as f64) as u32,
        (b * 0xffff_ffffu32 as f64) as u32
    )
}

fn sign_launch(exp: u64, template_vmid: u32, remote_id: &str, jti: &str) -> String {
    let msg = format!("{exp}.{template_vmid}.{remote_id}.{jti}");
    let mut mac =
        HmacSha256::new_from_slice(GUESTOS_LAUNCH_SECRET.as_bytes()).expect("HMAC key");
    mac.update(msg.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

/// Deep-link: signed one-click launch → clone+Sysprep wizard (skips GuestOS login).
pub fn sysprep_from_template_url(template_vmid: u32, remote_id: &str) -> String {
    let exp = unix_now() + GUESTOS_LAUNCH_TTL_SECS;
    let jti = random_jti();
    let sig = sign_launch(exp, template_vmid, remote_id, &jti);
    let remote = encode_component(remote_id);
    let jti_enc = encode_component(&jti);
    let sig_enc = encode_component(&sig);
    format!(
        "{GUESTOS_BASE}/launch?template_vmid={template_vmid}&remote_id={remote}&exp={exp}&jti={jti_enc}&sig={sig_enc}"
    )
}

/// Workflow detail page for a GuestOS task id.
pub fn workflow_url(task_id: &str) -> String {
    format!("{GUESTOS_BASE}/workflow/{task_id}")
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct GuestOsTask {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub progress: i32,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub timestamp: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub result_vmid: Option<i32>,
    #[serde(default)]
    pub result_ip_address: Option<String>,
    #[serde(default)]
    pub remote_id: Option<String>,
    #[serde(default)]
    pub template_vmid: Option<i32>,
    #[serde(default)]
    pub hostname: Option<String>,
}

#[derive(Deserialize)]
struct TaskListResponse {
    tasks: Vec<GuestOsTask>,
}

/// Fetch customization jobs from GuestOS (optionally filtered by PDM remote id).
pub async fn fetch_customization_tasks(
    remote_id: Option<&str>,
) -> Result<Vec<GuestOsTask>, anyhow::Error> {
    let mut url = format!("{GUESTOS_BASE}/api/tasks?kind=customization&limit=150");
    if let Some(remote) = remote_id {
        url.push_str("&remote_id=");
        url.push_str(&encode_component(remote).to_string());
    }
    let resp = gloo_net::http::Request::get(&url)
        .header("Authorization", &format!("Bearer {GUESTOS_API_TOKEN}"))
        .send()
        .await
        .map_err(|e| format_err!("GuestOS request failed: {e}"))?;
    if !resp.ok() {
        bail!(
            "GuestOS /api/tasks returned HTTP {} {}",
            resp.status(),
            resp.status_text()
        );
    }
    let body: TaskListResponse = resp
        .json()
        .await
        .map_err(|e| format_err!("GuestOS JSON decode failed: {e}"))?;
    Ok(body.tasks)
}
