use crate::gui_adw::app::DatabaseCommand;
use crate::gui_adw::ui::{AppToolbar, BacktestPage};
use gtk4::glib::subclass::InitializingObject;
use gtk4::subclass::prelude::*;
use gtk4::{glib, Application, CompositeTemplate};
use libadwaita as adw;
use tokio::sync::mpsc;

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
    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }
    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}
impl ObjectImpl for AppWindow {
    fn constructed(&self) {
        self.parent_constructed();
        let obj = self.obj();
        // Crear la primera pestaña
        obj.add_new_tab();
    }
}
impl WidgetImpl for AppWindow {}
impl WindowImpl for AppWindow {}
impl ApplicationWindowImpl for AppWindow {}
impl AdwApplicationWindowImpl for AppWindow {}

glib::wrapper! {
    pub struct AppWindow(ObjectSubclass<super::AppWindow>)
        @extends adw::ApplicationWindow, gtk4::ApplicationWindow, gtk4::Window, gtk4::Widget;
}

impl AppWindow {
    pub fn new(app: &Application, command_tx: mpsc::Sender<DatabaseCommand>) -> Self {
        let window: Self = glib::Object::builder().property("application", app).build();
        window
            .imp()
            .header_bar
            .imp()
            .new_tab_button
            .connect_clicked(clone!(@weak window, @strong command_tx => move |_| {
                window.add_new_tab(command_tx.clone());
            }));
        window
    }

    fn add_new_tab(&self, command_tx: mpsc::Sender<DatabaseCommand>) {
        let page = BacktestPage::new(command_tx);
        let tab = self.imp().tab_view.add_page(&page, None);
        tab.set_title("Nuevo Análisis");
    }

    // Función para redirigir resultados a la pestaña correcta
    pub fn route_result(
        &self,
        tab_id: u64,
        result: Result<Vec<crate::backtest::model::StrategyGridRow>, String>,
    ) {
        let imp = self.imp();
        for i in 0..imp.tab_view.n_pages() {
            if let Some(page) = imp
                .tab_view
                .nth_page(i)
                .child()
                .downcast_ref::<BacktestPage>()
            {
                if page.id() == tab_id {
                    page.handle_result(result);
                    break;
                }
            }
        }
    }

    pub fn update_all_pairlists(&self, pairs: Vec<String>) {
        // ... (lógica para iterar sobre las pestañas y actualizar sus combolists)
    }
}
