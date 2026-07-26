//! PDM tab: GuestOS customization job history (active + past).

use std::rc::Rc;

use gloo_timers::callback::Interval;
use yew::virtual_dom::{Key, VComp, VNode};
use yew::{AttrValue, Component, Context, Html, Properties, html};

use pwt::css::{FlexFit, FontColor, JustifyContent};
use pwt::prelude::*;
use pwt::props::{ContainerBuilder, CssPaddingBuilder, WidgetBuilder};
use pwt::state::{Selection, Store};
use pwt::widget::data_table::{DataTable, DataTableColumn, DataTableHeader};
use pwt::widget::{Button, Column, Container, Fa, Panel, Row, Toolbar, Tooltip};

use crate::guestos::{self, GuestOsTask};
use crate::pdm_client;

#[derive(Clone, PartialEq, Properties)]
pub struct GuestOsCustomizationList {
    /// When set, only show jobs for this PDM remote_id.
    #[prop_or_default]
    pub remote: Option<AttrValue>,
}

impl GuestOsCustomizationList {
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    pub fn remote(mut self, remote: impl Into<AttrValue>) -> Self {
        self.remote = Some(remote.into());
        self
    }
}

impl From<GuestOsCustomizationList> for VNode {
    fn from(val: GuestOsCustomizationList) -> Self {
        VComp::new::<GuestOsCustomizationListComp>(Rc::new(val), None).into()
    }
}

fn is_running(status: &str) -> bool {
    matches!(status, "PENDING" | "STARTED" | "PROGRESS")
}

enum Msg {
    Reload,
    Loaded(Result<(Vec<GuestOsTask>, Option<String>), String>),
    SelectionChange,
}

struct GuestOsCustomizationListComp {
    store: Store<GuestOsTask>,
    selection: Selection,
    columns: Rc<Vec<DataTableHeader<GuestOsTask>>>,
    load_error: Option<String>,
    loading: bool,
    guestos_base: Option<String>,
    _interval: Option<Interval>,
}

fn columns() -> Rc<Vec<DataTableHeader<GuestOsTask>>> {
    Rc::new(vec![
        DataTableColumn::new(tr!("Started"))
            .width("160px")
            .render(|t: &GuestOsTask| {
                html! { t.timestamp.clone().unwrap_or_else(|| "—".into()) }
            })
            .into(),
        DataTableColumn::new(tr!("Hostname"))
            .width("minmax(120px, 1fr)")
            .render(|t: &GuestOsTask| {
                html! { t.hostname.clone().unwrap_or_else(|| "—".into()) }
            })
            .into(),
        DataTableColumn::new(tr!("Remote"))
            .width("100px")
            .render(|t: &GuestOsTask| {
                html! { t.remote_id.clone().unwrap_or_else(|| "—".into()) }
            })
            .into(),
        DataTableColumn::new(tr!("Template"))
            .width("90px")
            .render(|t: &GuestOsTask| match t.template_vmid {
                Some(v) => html! { v },
                None => html! { "—" },
            })
            .into(),
        DataTableColumn::new(tr!("Result VM"))
            .width("90px")
            .render(|t: &GuestOsTask| match t.result_vmid {
                Some(v) => html! { v },
                None => html! { "—" },
            })
            .into(),
        DataTableColumn::new(tr!("Job"))
            .flex(2)
            .render(|t: &GuestOsTask| html! { &t.name })
            .into(),
        DataTableColumn::new(tr!("Status"))
            .width("minmax(160px, 1fr)")
            .render(|t: &GuestOsTask| {
                if is_running(&t.status) {
                    Row::new()
                        .gap(2)
                        .class(JustifyContent::Start)
                        .with_child(Fa::new("").class("pwt-loading-icon"))
                        .with_child(format!("{} {}%", t.status, t.progress))
                        .into()
                } else {
                    html! { format!("{} — {}", t.status, t.message) }
                }
            })
            .into(),
    ])
}

impl GuestOsCustomizationListComp {
    fn reload(&mut self, ctx: &Context<Self>) {
        self.loading = true;
        self.load_error = None;
        let remote = ctx.props().remote.as_ref().map(|r| r.to_string());
        let link = ctx.link().clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = pdm_client()
                .guestos_list_tasks(remote.as_deref())
                .await
                .map(|list| (list.tasks, list.base_url))
                .map_err(|e| e.to_string());
            link.send_message(Msg::Loaded(result));
        });
    }
}

impl Component for GuestOsCustomizationListComp {
    type Message = Msg;
    type Properties = GuestOsCustomizationList;

    fn create(ctx: &Context<Self>) -> Self {
        let store = Store::with_extract_key(|t: &GuestOsTask| Key::from(t.id.as_str()));
        let selection =
            Selection::new().on_select(ctx.link().callback(|_| Msg::SelectionChange));

        let mut me = Self {
            store,
            selection,
            columns: columns(),
            load_error: None,
            loading: false,
            guestos_base: None,
            _interval: None,
        };
        me.reload(ctx);
        let link = ctx.link().clone();
        me._interval = Some(Interval::new(5_000, move || {
            link.send_message(Msg::Reload);
        }));
        me
    }

    fn changed(&mut self, ctx: &Context<Self>, old: &Self::Properties) -> bool {
        if ctx.props().remote != old.remote {
            self.reload(ctx);
        }
        true
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Reload => {
                self.reload(ctx);
                true
            }
            Msg::Loaded(Ok((tasks, base_url))) => {
                self.loading = false;
                self.load_error = None;
                if base_url.is_some() {
                    self.guestos_base = base_url;
                }
                self.store.set_data(tasks);
                true
            }
            Msg::Loaded(Err(err)) => {
                self.loading = false;
                self.load_error = Some(err);
                true
            }
            Msg::SelectionChange => true,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let selected = self
            .selection
            .selected_key()
            .and_then(|k| self.store.read().lookup_record(&k).cloned());

        let open_btn = selected.as_ref().and_then(|t| {
            let base = self.guestos_base.as_ref()?;
            let url = guestos::workflow_url(base, &t.id);
            Some(
                Tooltip::new(
                    Button::new(tr!("Open in GuestOS"))
                        .icon_class("fa fa-external-link")
                        .onclick(move |_| {
                            let _ = web_sys::window().unwrap().open_with_url(&url);
                        }),
                )
                .tip(tr!("Open the GuestOS workflow page for this job.")),
            )
        });

        let toolbar = Toolbar::new()
            .class("pwt-border-bottom")
            .with_child(
                Button::refresh(self.loading).onclick({
                    let link = ctx.link().clone();
                    move |_| link.send_message(Msg::Reload)
                }),
            )
            .with_flex_spacer()
            .with_optional_child(open_btn);

        let table = DataTable::new(self.columns.clone(), self.store.clone())
            .selection(self.selection.clone())
            .class(FlexFit);

        let err = self.load_error.as_ref().map(|e| {
            Container::new()
                .padding(2)
                .class(FontColor::Error)
                .with_child(e.clone())
        });

        let hint = Container::new().padding(2).with_child(tr!(
            "Shows GuestOS clone+Sysprep jobs{0}. Select a row and open the live workflow in GuestOS.",
            match &ctx.props().remote {
                Some(r) => format!(" for remote '{r}'"),
                None => String::new(),
            }
        ));

        Panel::new()
            .title(tr!("GuestOS Customizations"))
            .class(FlexFit)
            .with_child(
                Column::new()
                    .class(FlexFit)
                    .with_child(toolbar)
                    .with_child(hint)
                    .with_optional_child(err)
                    .with_child(table),
            )
            .into()
    }
}
