use gtk4::glib::subclass::InitializingObject;
use gtk4::subclass::prelude::*;
use gtk4::{glib, Button, CheckButton, CompositeTemplate, ProgressBar, SpinButton};
use libadwaita as adw;

#[derive(CompositeTemplate, Default)]
#[template(file = "left_panel.ui")]
pub struct LeftPanel {
    #[template_child]
    pub execute_button: TemplateChild<Button>,
    #[template_child]
    pub progress_bar: TemplateChild<ProgressBar>,
    #[template_child]
    pub pairlist_combo: TemplateChild<adw::ComboRow>,
    #[template_child]
    pub win_rate_check: TemplateChild<CheckButton>,
    #[template_child]
    pub win_rate_value: TemplateChild<SpinButton>,
    // ... añade aquí los demás widgets que necesites controlar por ID
}

#[glib::object_subclass]
impl ObjectSubclass for LeftPanel {
    const NAME: &'static str = "LeftPanel";
    type Type = super::LeftPanel;
    type ParentType = gtk4::Box;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }
    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}
impl ObjectImpl for LeftPanel {}
impl WidgetImpl for LeftPanel {}
impl BoxImpl for LeftPanel {}

glib::wrapper! {
    pub struct LeftPanel(ObjectSubclass<super::LeftPanel>)
        @extends gtk4::Widget, gtk4::Box;
}

impl LeftPanel {
    pub fn new() -> Self {
        glib::Object::new()
    }
}