// src/gui_adw/ui/right_panel.rs

use gtk4::glib::subclass::InitializingObject;
use gtk4::subclass::prelude::*;
use gtk4::{glib, CompositeTemplate, ListStore, TreeModelFilter, TreeView, TreeViewColumn, CellRendererText};
use std::cell::RefCell;
use super::status_bar;

// 1. Define el struct principal del widget
#[derive(CompositeTemplate, Default)]
#[template(file = "right_panel.ui")]
pub struct RightPanel {
    #[template_child]
    pub tree_view: TemplateChild<TreeView>,
    #[template_child]
    pub status_bar_container: TemplateChild<gtk4::Box>,

    // Guardamos los modelos de datos como estado interno
    pub store: RefCell<Option<ListStore>>,
    pub filter_model: RefCell<Option<TreeModelFilter>>,
}

// 2. Implementa la lógica de la subclase
#[glib::object_subclass]
impl ObjectSubclass for RightPanel {
    const NAME: &'static str = "RightPanel";
    type Type = super::RightPanel;
    type ParentType = gtk4::Box;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

// 3. Lógica de inicialización del objeto
impl ObjectImpl for RightPanel {
    fn constructed(&self) {
        self.parent_constructed();
        let obj = self.obj(); // Obtiene la instancia de RightPanel

        // Creamos los modelos
        let store = obj.create_list_store();
        let filter_model = TreeModelFilter::new(&store, None);
        filter_model.set_visible_column(30);

        // Asignamos los modelos a nuestros widgets y estado interno
        obj.imp().tree_view.set_model(Some(&filter_model));
        obj.imp().store.replace(Some(store));
        obj.imp().filter_model.replace(Some(filter_model));

        // Configuramos las columnas y la barra de estado
        obj.setup_columns();
        obj.imp().status_bar_container.append(&status_bar::create());
    }
}

impl WidgetImpl for RightPanel {}
impl BoxImpl for RightPanel {}

// 4. "Envoltorio" público y constructor
glib::wrapper! {
    pub struct RightPanel(ObjectSubclass<super::RightPanel>)
        @extends gtk4::Widget, gtk4::Box;
}

impl RightPanel {
    pub fn new() -> Self {
        glib::Object::new()
    }
    
    // --- LÓGICA DE `table_view.rs` AHORA VIVE AQUÍ ---

    fn create_list_store(&self) -> ListStore {
        ListStore::new(&[
            glib::Type::STRING, // 0: Estrategia
            // ... (todas las 31 columnas, como las tenías definidas)
            glib::Type::BOOL,   // 30: Visible
        ])
    }

    fn setup_columns(&self) {
        let tree_view = self.imp().tree_view.get();
        let columns = [
            ("📊 Estrategia", 0),
            ("⏱ TF", 1),
            // ... (todas las demás columnas que tenías)
        ];

        for (title, column_id) in columns {
            let column = TreeViewColumn::new();
            let cell = CellRendererText::new();
            column.pack_start(&cell, true);
            column.set_title(title);
            column.add_attribute(&cell, "text", column_id);
            tree_view.append_column(&column);
        }
    }
}