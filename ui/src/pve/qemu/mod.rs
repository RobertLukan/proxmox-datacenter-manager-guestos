mod overview;
use overview::QemuOverviewPanel;

use std::rc::Rc;

use yew::virtual_dom::{VComp, VNode};

use proxmox_deb_version::Version;
use pwt::css::FlexFit;
use pwt::prelude::*;
use pwt::widget::{Button, Column, Container, Fa, Row, TabBarItem, TabPanel, Tooltip};
use pwt_macros::builder;

use proxmox_yew_comp::configuration::pve::{QemuHardwarePanel, QemuOptionsPanel};

use pdm_api_types::resource::PveQemuResource;

use crate::pve::utils::render_qemu_name;
use crate::pve::{GuestInfo, GuestType};
use crate::renderer::render_title_row;
use crate::widget::SnapshotWindow;

#[derive(Clone, Debug, Properties, PartialEq)]
#[builder]
pub struct QemuPanel {
    remote: String,
    node: String,
    info: PveQemuResource,

    #[prop_or_default]
    #[builder]
    /// The nodes pve-manager version, used to feature gate some entries.
    pve_manager_version: Option<Version>,

    #[prop_or(60_000)]
    /// The interval for refreshing the rrd data
    pub rrd_interval: u32,

    #[prop_or(10_000)]
    /// The interval for refreshing the status data
    pub status_interval: u32,
}

impl QemuPanel {
    pub fn new(remote: String, node: String, info: PveQemuResource) -> Self {
        yew::props!(Self { remote, node, info })
    }
}

pub enum Msg {
    WindowsCheck(Result<bool, String>),
}

pub struct QemuPanelComp {
    /// None while loading; Some(true) only for Windows ostype templates.
    is_windows_template: Option<bool>,
}

impl QemuPanelComp {
    fn load_windows_check(&self, ctx: &yew::Context<Self>) {
        let props = ctx.props();
        if !props.info.template {
            return;
        }
        let remote = props.remote.clone();
        let node = props.node.clone();
        let vmid = props.info.vmid;
        let link = ctx.link().clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = crate::guestos::template_is_windows(&remote, Some(&node), vmid)
                .await
                .map_err(|e| e.to_string());
            link.send_message(Msg::WindowsCheck(result));
        });
    }
}

impl yew::Component for QemuPanelComp {
    type Message = Msg;
    type Properties = QemuPanel;

    fn create(ctx: &yew::Context<Self>) -> Self {
        let me = Self {
            is_windows_template: if ctx.props().info.template {
                None
            } else {
                Some(false)
            },
        };
        me.load_windows_check(ctx);
        me
    }

    fn changed(&mut self, ctx: &yew::Context<Self>, old: &Self::Properties) -> bool {
        let props = ctx.props();
        if props.remote != old.remote
            || props.node != old.node
            || props.info.vmid != old.info.vmid
            || props.info.template != old.info.template
        {
            self.is_windows_template = if props.info.template {
                None
            } else {
                Some(false)
            };
            self.load_windows_check(ctx);
        }
        true
    }

    fn update(&mut self, _ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::WindowsCheck(Ok(is_win)) => {
                self.is_windows_template = Some(is_win);
                true
            }
            Msg::WindowsCheck(Err(err)) => {
                log::warn!("GuestOS ostype check failed: {err}");
                self.is_windows_template = Some(false);
                true
            }
        }
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        let props = ctx.props();
        let show_customize = props.info.template && self.is_windows_template == Some(true);

        let title: Html = Row::new()
            .gap(2)
            .class(pwt::css::AlignItems::Baseline)
            .with_child(Fa::new("desktop"))
            .with_child(tr! {"VM '{0}'", render_qemu_name(&props.info, true)})
            .into();

        TabPanel::new()
            .router(true)
            .class(pwt::css::FlexFit)
            .title(title)
            .tool(
                Row::new()
                    .gap(1)
                    .with_child(
                        Tooltip::new(
                            Button::new(tr!("Open Web UI"))
                                .icon_class("fa fa-external-link")
                                .aria_label(tr!("Open the web UI of VM {0}.", props.info.vmid))
                                .on_activate({
                                    let link = ctx.link().clone();
                                    let remote = props.remote.clone();
                                    let node = props.node.clone();
                                    let vmid = props.info.vmid;
                                    move |_| {
                                        let id = format!("qemu/{vmid}");
                                        if let Some(url) =
                                            crate::get_deep_url(&link, &remote, Some(&node), &id)
                                        {
                                            let _ =
                                                web_sys::window().unwrap().open_with_url(&url.href());
                                        }
                                    }
                                }),
                        )
                        .tip(tr!("Open the web UI of VM {0}.", props.info.vmid)),
                    )
                    .with_optional_child(show_customize.then(|| {
                        Tooltip::new(
                            Button::new(tr!("Customize (GuestOS)"))
                                .icon_class("fa fa-cogs")
                                .aria_label(tr!(
                                    "Clone template {0} and Sysprep in GuestOS.",
                                    props.info.vmid
                                ))
                                .on_activate({
                                    let remote = props.remote.clone();
                                    let node = props.node.clone();
                                    let vmid = props.info.vmid;
                                    move |_| {
                                        let remote = remote.clone();
                                        let node = node.clone();
                                        wasm_bindgen_futures::spawn_local(async move {
                                            match crate::guestos::launch_sysprep_customize(
                                                &remote,
                                                Some(&node),
                                                vmid,
                                            )
                                            .await
                                            {
                                                Ok(url) => {
                                                    let _ = web_sys::window()
                                                        .unwrap()
                                                        .open_with_url(&url);
                                                }
                                                Err(err) => {
                                                    let _ = web_sys::window()
                                                        .unwrap()
                                                        .alert_with_message(&format!(
                                                            "GuestOS Customize failed: {err}"
                                                        ));
                                                }
                                            }
                                        });
                                    }
                                }),
                        )
                        .tip(tr!(
                            "GuestOS: clone Windows template {0} then Sysprep the clone (remote {1}). Never runs on production VMs.",
                            props.info.vmid,
                            props.remote.clone()
                        ))
                    })),
            )
            .with_item_builder(
                TabBarItem::new()
                    .key("status_view")
                    .label(tr!("Overview"))
                    .icon_class("fa fa-tachometer"),
                {
                    let remote = props.remote.clone();
                    let node = props.node.clone();
                    let info = props.info.clone();
                    move |_| {
                        QemuOverviewPanel::new(remote.clone(), node.clone(), info.clone()).into()
                    }
                },
            )
            .with_item_builder(
                TabBarItem::new()
                    .key("config")
                    .label(tr!("Config"))
                    .icon_class("fa fa-file-text-o"),
                {
                    let remote = props.remote.clone();
                    let node = props.node.clone();
                    let vmid = props.info.vmid;
                    let pve_manager_version = props.pve_manager_version.clone();
                    move |_| {
                        Container::new()
                            .class(FlexFit)
                            .with_child(
                                Column::new()
                                    .padding(4)
                                    .gap(2)
                                    .with_child(render_title_row(tr!("Hardware"), "desktop"))
                                    .with_child(html! {<hr/>})
                                    .with_child(
                                        QemuHardwarePanel::new(node.clone(), vmid)
                                            .readonly(true)
                                            .remote(remote.clone()),
                                    )
                                    .with_child(
                                        render_title_row(tr!("Options"), "gear").margin_top(6),
                                    )
                                    .with_child(html! {<hr/>})
                                    .with_child(
                                        QemuOptionsPanel::new(node.clone(), vmid)
                                            .pve_manager_version(pve_manager_version.clone())
                                            .readonly(true)
                                            .remote(remote.clone()),
                                    ),
                            )
                            .into()
                    }
                },
            )
            .with_item_builder(
                TabBarItem::new()
                    .key("snapshots")
                    .label(tr!("Snapshots"))
                    .icon_class("fa fa-history"),
                {
                    let remote = props.remote.clone();
                    let vmid = props.info.vmid;
                    move |_| {
                        SnapshotWindow::new(
                            remote.clone(),
                            GuestInfo {
                                guest_type: GuestType::Qemu,
                                vmid,
                            },
                        )
                        .into()
                    }
                },
            )
            .with_item_builder(
                TabBarItem::new()
                    .key("novnc")
                    .label(tr!("novnc"))
                    .icon_class("fa fa-terminal"), // FIXME
                {
                    let remote = props.remote.clone();
                    let node = props.node.clone();
                    let supported = props
                        .pve_manager_version
                        .as_ref()
                        .map(|ver| ver >= &Version::new("9.1.0", None))
                        .unwrap_or(true);
                    let vmid = props.info.vmid;
                    move |_| {
                        if supported {
                            let mut xtermjs = proxmox_yew_comp::XTermJs::new();
                            xtermjs.set_vnc(true);
                            xtermjs.set_node_name(node.clone());
                            xtermjs.set_console_type(proxmox_yew_comp::ConsoleType::RemotePveKVM(
                                remote.clone(),
                                vmid as u64,
                            ));
                            xtermjs.into()
                        } else {
                            Row::new()
                                .class(pwt::css::FlexFit)
                                .class(pwt::css::JustifyContent::Center)
                                .class(pwt::css::AlignItems::Center)
                                .with_child(html! { tr!("pve-manager version too old") })
                                .into()
                        }
                    }
                },
            )
            .with_item_builder(
                TabBarItem::new()
                    .key("serial_console")
                    .label(tr!("xterm.js"))
                    .icon_class("fa fa-terminal"),
                {
                    let remote = props.remote.clone();
                    let node = props.node.clone();
                    let supported = props
                        .pve_manager_version
                        .as_ref()
                        .map(|ver| ver >= &Version::new("9.1.0", None))
                        .unwrap_or(true);
                    let vmid = props.info.vmid;
                    move |_| {
                        if supported {
                            let mut xtermjs = proxmox_yew_comp::XTermJs::new();
                            xtermjs.set_node_name(node.clone());
                            xtermjs.set_console_type(proxmox_yew_comp::ConsoleType::RemotePveKVM(
                                remote.clone(),
                                vmid as u64,
                            ));
                            xtermjs.into()
                        } else {
                            Row::new()
                                .class(pwt::css::FlexFit)
                                .class(pwt::css::JustifyContent::Center)
                                .class(pwt::css::AlignItems::Center)
                                .with_child(html! { tr!("pve-manager version too old") })
                                .into()
                        }
                    }
                },
            )
            .into()
    }
}

impl From<QemuPanel> for VNode {
    fn from(val: QemuPanel) -> Self {
        VComp::new::<QemuPanelComp>(Rc::new(val), None).into()
    }
}
