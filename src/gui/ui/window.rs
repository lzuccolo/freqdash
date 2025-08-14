use gtk4::glib::subclass::InitializingObject;
use gtk4::subclass::prelude::*;
use gtk4::{glib, Application, CompositeTemplate};
use libadwaita as adw;
use tokio::sync::mpsc;
use crate::gui::app::{DatabaseCommand, DatabaseResult};
use crate::gui::components::{AppToolbar, BacktestPage};

#[derive(CompositeTemplate, Default)]
#[template(file = "window.ui")]
pub struct AppWindow {
    #[template_child]
    pub header_bar: TemplateChild<AppToolbar>,
    #[template_child]
    pub tab_view: TemplateChild<adw::TabView>,
}

#[glib::object_subclass]
impl ObjectSubclass for AppWindow {
    const NAME: &'static str = "AppWindow";
    type Type = super::AppWindow;
    type ParentType = adw::ApplicationWindow;
    fn class_init(klass: &mut Self::Class) { klass.bind_template(); }
    fn instance_init(obj: &InitializingObject<Self>) { obj.init_template(); }
}

impl ObjectImpl for AppWindow {
    fn constructed(&self) {
        self.parent_constructed();
        // Conectamos el botón de "nueva pestaña" aquí
        let obj = self.obj();
        obj.imp().header_bar.imp().new_tab_button.connect_clicked(clone!(@weak obj => move |_| {
            // Lógica para añadir pestaña (necesita el command_tx)
        }));
    }
}
// ... (resto de impls y wrapper)

impl AppWindow {
    pub fn new(app: &Application, command_tx: mpsc::Sender<DatabaseCommand>) -> Self {
        let window: Self = glib::Object::builder().property("application", app).build();
        window.imp().header_bar.imp().new_tab_button.connect_clicked(clone!(@weak window, @strong command_tx => move |_| {
            window.add_new_tab(command_tx.clone());
        }));
        window.add_new_tab(command_tx); // Añadir la primera pestaña
        window
    }

    fn add_new_tab(&self, command_tx: mpsc::Sender<DatabaseCommand>) {
        let page = BacktestPage::new(command_tx);
        let tab = self.imp().tab_view.add_page(&page, None);
        tab.set_title("Nuevo Análisis");
    }
    
    pub fn route_result(&self, tab_id: u64, result: Result<Vec<crate::backtest::model::StrategyGridRow>, String>) {
        // ...
    }
}