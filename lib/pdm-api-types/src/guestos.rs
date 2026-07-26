//! GuestOS sidecar integration config (optional).

use serde::{Deserialize, Serialize};

use proxmox_schema::{Schema, StringSchema, Updater, api};

use crate::HTTP_URL_SCHEMA;

pub const GUESTOS_API_TOKEN_SCHEMA: Schema =
    StringSchema::new("GuestOS machine API token (Bearer) used by the PDM proxy.")
        .min_length(8)
        .max_length(512)
        .schema();

pub const GUESTOS_LAUNCH_SECRET_SCHEMA: Schema = StringSchema::new(
    "HMAC secret for GuestOS /launch deep-links (must match GuestOS GUESTOS_LAUNCH_SECRET).",
)
.min_length(16)
.max_length(512)
.schema();

#[api(
    properties: {
        "base-url": {
            schema: HTTP_URL_SCHEMA,
            optional: true,
        },
        "api-token": {
            schema: GUESTOS_API_TOKEN_SCHEMA,
            optional: true,
        },
        "launch-secret": {
            schema: GUESTOS_LAUNCH_SECRET_SCHEMA,
            optional: true,
        },
        "launch-ttl": {
            description: "Launch token lifetime in seconds (default 300).",
            optional: true,
            minimum: 30,
            maximum: 3600,
        },
        "verify-tls": {
            description: "Verify GuestOS TLS certificate (default true). Set false for lab self-signed.",
            optional: true,
            default: true,
        },
    },
)]
#[derive(Clone, Debug, Default, Deserialize, Serialize, Updater, PartialEq)]
#[serde(rename_all = "kebab-case")]
/// Optional GuestOS sidecar settings (`/etc/proxmox-datacenter-manager/guestos.cfg`).
pub struct GuestOsConfig {
    /// Public GuestOS base URL (no trailing slash), e.g. https://guestos.example.com
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,

    /// Bearer token for GuestOS /api/tasks (server-side only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_token: Option<String>,

    /// HMAC secret for signed /launch URLs (server-side only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub launch_secret: Option<String>,

    /// Launch token TTL seconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub launch_ttl: Option<u64>,

    /// Verify TLS when calling GuestOS (lab self-signed: false).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verify_tls: Option<bool>,
}

#[api]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
/// Signed GuestOS launch URL for template customize.
pub struct GuestOsLaunchResponse {
    /// Full HTTPS URL to open in a new tab.
    pub url: String,
}

#[api]
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
/// One GuestOS customization task (proxied from GuestOS /api/tasks).
pub struct GuestOsTask {
    /// GuestOS task id (UUID).
    pub id: String,
    /// Short display name.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
    /// Longer task description from the GuestOS ledger.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Task status (PENDING/STARTED/PROGRESS/SUCCESS/FAILURE/…).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub status: String,
    /// Progress percent 0–100.
    #[serde(default)]
    pub progress: i64,
    /// Latest status message.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub message: String,
    /// ISO timestamp when the task was created.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    /// ISO timestamp of the last update.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    /// Result VMID after clone (if any).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result_vmid: Option<i64>,
    /// Result guest IP (if known).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result_ip_address: Option<String>,
    /// PDM remote id associated with the job.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_id: Option<String>,
    /// Source template VMID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template_vmid: Option<i64>,
    /// Target hostname.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
}

#[api(
    properties: {
        tasks: {
            type: Array,
            items: {
                type: GuestOsTask,
            },
        },
        "base-url": {
            schema: HTTP_URL_SCHEMA,
            optional: true,
        },
    },
)]
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
/// GuestOS task list response.
pub struct GuestOsTaskList {
    /// Customization tasks from GuestOS.
    pub tasks: Vec<GuestOsTask>,
    /// GuestOS public base URL (no secrets) for deep-links such as /workflow/{id}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
}
