use gtk4::glib::subclass::InitializingObject;
use gtk4::subclass::prelude::*;
use gtk4::{glib, Button, CompositeTemplate};
use libadwaita as adw;

#[derive(CompositeTemplate, Default)]
#[template(file = "toolbar.ui")]
pub struct AppToolbar {
    #[template_child]
    pub new_tab_button: TemplateChild<Button>,
}

#[glib::object_subclass]
impl ObjectSubclass for AppToolbar {
    const NAME: &'static str = "AppToolbar";
    type Type = super::AppToolbar;
    type ParentType = adw::HeaderBar;

    fn class_init(klass: &mut Self::Class) { klass.bind_template(); }
    fn instance_init(obj: &InitializingObject<Self>) { obj.init_template(); }
}

impl ObjectImpl for AppToolbar {}
impl WidgetImpl for AppToolbar {}
impl HeaderBarImpl for AppToolbar {}

glib::wrapper! {
    pub struct AppToolbar(ObjectSubclass<imp::AppToolbar>)
        @extends adw::HeaderBar, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl AppToolbar {
    pub fn new() -> Self {
        glib::Object::new()
    }
}