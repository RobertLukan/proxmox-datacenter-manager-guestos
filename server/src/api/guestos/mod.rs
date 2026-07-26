//! GuestOS sidecar proxy (`/api2/json/guestos/...`).
//!
//! Keeps launch HMAC secret and API token on the PDM server. The UI never sees them.

use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Error, bail, format_err};
use http::StatusCode;
use hmac::{Hmac, Mac};
use openssl::ssl::{SslConnector, SslMethod, SslVerifyMode};
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use proxmox_http::{Body, client::Client as HttpClient};
use proxmox_router::{Permission, Router, RpcEnvironment, list_subdirs_api_method};
use proxmox_schema::api;
use proxmox_sortable_macro::sortable;
use serde::Deserialize;
use sha2::Sha256;

use pdm_api_types::PRIV_RESOURCE_AUDIT;
use pdm_api_types::guestos::{GuestOsLaunchResponse, GuestOsTask, GuestOsTaskList};
use pdm_config::guestos as guestos_config;

type HmacSha256 = Hmac<Sha256>;

#[sortable]
const SUBDIRS: proxmox_router::SubdirMap = &sorted!([
    (
        "launch",
        &Router::new().post(&API_METHOD_CREATE_LAUNCH_URL),
    ),
    ("tasks", &Router::new().get(&API_METHOD_LIST_TASKS)),
]);

pub const ROUTER: Router = Router::new()
    .get(&list_subdirs_api_method!(SUBDIRS))
    .subdirs(SUBDIRS);

fn require_cfg() -> Result<pdm_api_types::guestos::GuestOsConfig, Error> {
    let (cfg, _) = guestos_config::config()?;
    if !guestos_config::is_configured(&cfg) {
        bail!(
            "GuestOS is not configured. Create /etc/proxmox-datacenter-manager/guestos.cfg \
             with base-url, api-token, and launch-secret."
        );
    }
    Ok(cfg)
}

fn base_url(cfg: &pdm_api_types::guestos::GuestOsConfig) -> Result<String, Error> {
    let base = cfg
        .base_url
        .as_ref()
        .map(|s| s.trim().trim_end_matches('/'))
        .filter(|s| !s.is_empty())
        .ok_or_else(|| format_err!("guestos.cfg missing base-url"))?;
    Ok(base.to_string())
}

fn http_client(verify_tls: bool) -> Result<HttpClient, Error> {
    let mut builder = SslConnector::builder(SslMethod::tls())?;
    if !verify_tls {
        builder.set_verify(SslVerifyMode::NONE);
    }
    Ok(HttpClient::with_ssl_connector(
        builder.build(),
        Default::default(),
    ))
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn random_jti() -> String {
    use openssl::rand::rand_bytes;
    let mut buf = [0u8; 8];
    let _ = rand_bytes(&mut buf);
    hex::encode(buf)
}

fn sign_launch(secret: &str, exp: u64, template_vmid: u32, remote_id: &str, jti: &str) -> Result<String, Error> {
    let msg = format!("{exp}.{template_vmid}.{remote_id}.{jti}");
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|e| format_err!("invalid launch-secret: {e}"))?;
    mac.update(msg.as_bytes());
    Ok(hex::encode(mac.finalize().into_bytes()))
}

#[api(
    input: {
        properties: {
            "template-vmid": {
                description: "Proxmox template VMID to customize (clone+Sysprep).",
                type: u32,
            },
            "remote-id": {
                description: "PDM remote id passed through to GuestOS.",
                type: String,
            },
        },
    },
    access: {
        permission: &Permission::Privilege(&["/"], PRIV_RESOURCE_AUDIT, false),
    },
    returns: { type: GuestOsLaunchResponse },
)]
/// Build a short-lived signed GuestOS /launch URL (secrets stay on the server).
pub fn create_launch_url(
    template_vmid: u32,
    remote_id: String,
    _rpcenv: &mut dyn RpcEnvironment,
) -> Result<GuestOsLaunchResponse, Error> {
    let cfg = require_cfg()?;
    let base = base_url(&cfg)?;
    let secret = cfg
        .launch_secret
        .as_ref()
        .map(|s| s.as_str())
        .ok_or_else(|| format_err!("guestos.cfg missing launch-secret"))?;
    let ttl = cfg.launch_ttl.unwrap_or(300).clamp(30, 3600);
    let exp = unix_now() + ttl;
    let jti = random_jti();
    let sig = sign_launch(secret, exp, template_vmid, &remote_id, &jti)?;
    let remote_enc = utf8_percent_encode(&remote_id, NON_ALPHANUMERIC);
    let jti_enc = utf8_percent_encode(&jti, NON_ALPHANUMERIC);
    let sig_enc = utf8_percent_encode(&sig, NON_ALPHANUMERIC);
    let url = format!(
        "{base}/launch?template_vmid={template_vmid}&remote_id={remote_enc}&exp={exp}&jti={jti_enc}&sig={sig_enc}"
    );
    Ok(GuestOsLaunchResponse { url })
}

#[derive(Deserialize)]
struct GuestOsTaskListRaw {
    tasks: Vec<GuestOsTask>,
}

#[api(
    input: {
        properties: {
            "remote-id": {
                description: "Optional PDM remote id filter.",
                type: String,
                optional: true,
            },
        },
    },
    access: {
        permission: &Permission::Privilege(&["/"], PRIV_RESOURCE_AUDIT, false),
    },
    returns: { type: GuestOsTaskList },
)]
/// Proxy GuestOS customization task history.
pub async fn list_tasks(
    remote_id: Option<String>,
    _rpcenv: &mut dyn RpcEnvironment,
) -> Result<GuestOsTaskList, Error> {
    let cfg = require_cfg()?;
    let base = base_url(&cfg)?;
    let token = cfg
        .api_token
        .as_ref()
        .map(|s| s.as_str())
        .ok_or_else(|| format_err!("guestos.cfg missing api-token"))?;
    let verify_tls = cfg.verify_tls.unwrap_or(true);

    let mut path = format!("{base}/api/tasks?kind=customization&limit=150");
    if let Some(remote) = remote_id.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        path.push_str("&remote_id=");
        path.push_str(&utf8_percent_encode(remote, NON_ALPHANUMERIC).to_string());
    }

    let client = http_client(verify_tls)?;
    let request = http::Request::builder()
        .method(http::Method::GET)
        .uri(&path)
        .header("Authorization", format!("Bearer {token}"))
        .header("Accept", "application/json")
        .body(Body::empty())
        .map_err(|e| format_err!("build GuestOS request: {e}"))?;

    let response = client
        .request(request)
        .await
        .map_err(|e| format_err!("GuestOS request failed: {e}"))?;
    let status = response.status();
    let body = HttpClient::response_body_string(response)
        .await
        .map_err(|e| format_err!("GuestOS response body: {e}"))?;
    if status != StatusCode::OK {
        bail!("GuestOS /api/tasks returned HTTP {status}: {body}");
    }
    let parsed: GuestOsTaskListRaw = serde_json::from_str(&body)
        .map_err(|e| format_err!("GuestOS JSON decode failed: {e}"))?;
    Ok(GuestOsTaskList {
        tasks: parsed.tasks,
        base_url: Some(base),
    })
}
