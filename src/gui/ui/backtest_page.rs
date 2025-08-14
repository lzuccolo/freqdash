use gtk4::glib::subclass::InitializingObject;
use gtk4::subclass::prelude::*;
use gtk4::{glib, CompositeTemplate, TreeView};
use libadwaita as adw;
use std::cell::RefCell;
use std::rc::Rc;
use tokio::sync::mpsc;

use crate::gui_adw::app::{DatabaseCommand, get_runtime};
use crate::gui_adw::state::AppState;
use super::{left_panel, right_panel, table_view};

#[derive(CompositeTemplate, Default)]
#[template(file = "backtest_page.ui")]
pub struct BacktestPage {
    #[template_child]
    pub flap: TemplateChild<adw::Flap>,

    pub state: Rc<RefCell<AppState>>,
    pub id: std::cell::Cell<u64>,
    // Guardamos los paneles para acceder a sus widgets internos
    left_panel: RefCell<Option<left_panel::LeftPanel>>,
    right_panel: RefCell<Option<gtk4::Box>>,
    tree_view: RefCell<Option<TreeView>>,
}

#[glib::object_subclass]
impl ObjectSubclass for BacktestPage {
    const NAME: &'static str = "BacktestPage";
    type Type = super::BacktestPage;
    type ParentType = gtk4::Box;
    fn class_init(klass: &mut Self::Class) { klass.bind_template(); }
    fn instance_init(obj: &InitializingObject<Self>) { obj.init_template(); }
}
impl ObjectImpl for BacktestPage {}
impl WidgetImpl for BacktestPage {}
impl BoxImpl for BacktestPage {}

glib::wrapper! {
    pub struct BacktestPage(ObjectSubclass<super::BacktestPage>)
        @extends gtk4::Widget, gtk4::Box;
}

impl BacktestPage {
    pub fn new(command_tx: mpsc::Sender<DatabaseCommand>) -> Self {
        let page: Self = glib::Object::new();
        let imp = page.imp();
        imp.id.set(rand::random());

        let left_panel = left_panel::LeftPanel::new();
        let (right_panel, tree_view, filter_model) = right_panel::create();
        
        imp.flap.set_flap(Some(&left_panel));
        imp.flap.set_content(Some(&right_panel));
        
        let store = table_view::get_base_store(&filter_model);
        let app_state = Rc::new(RefCell::new(AppState::new(store, filter_model)));
        imp.state.set(app_state);

        imp.left_panel.replace(Some(left_panel));
        imp.right_panel.replace(Some(right_panel));
        imp.tree_view.replace(Some(tree_view));

        page.connect_events(command_tx);
        page
    }
    
    pub fn id(&self) -> u64 { self.imp().id.get() }

    fn connect_events(&self, command_tx: mpsc::Sender<DatabaseCommand>) {
        let imp = self.imp();
        let left_panel = imp.left_panel.borrow();
        let left_panel = left_panel.as_ref().unwrap();
        
        let execute_button = &left_panel.imp().execute_button;
        
        execute_button.connect_clicked(clone!(@weak self as page => move |_| {
            let imp = page.imp();
            let state = imp.state.get();
            let left_panel = imp.left_panel.borrow();
            let left_panel = left_panel.as_ref().unwrap();
            
            if state.borrow().is_loading { return; }

            // ... Lógica para poner la UI en modo "cargando" ...

            let query = events::query::get_query_params(left_panel);
            let tx = command_tx.clone();
            let page_id = page.id();
            
            get_runtime().spawn(async move {
                tx.send(DatabaseCommand::RunBacktest(page_id, query)).await.unwrap();
            });
        }));
    }

    pub fn handle_result(&self, result: Result<Vec<crate::backtest::model::StrategyGridRow>, String>) {
        // ... Lógica para poblar la tabla por lotes usando los widgets de `self.imp()` ...
        println!("La página {} recibió un resultado.", self.id());
    }
}