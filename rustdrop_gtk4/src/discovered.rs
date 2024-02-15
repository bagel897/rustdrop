use adw::{prelude::*, subclass::prelude::*};

use crate::daemon::DiscoveryHandle;
mod imp {

    use std::cell::OnceCell;

    use super::*;
    use crate::daemon::DiscoveryHandle;
    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(file = "blueprints/discovered.blp")]
    pub struct DiscoveredRow {
        handle: OnceCell<DiscoveryHandle>,
    }
    #[glib::object_subclass]
    impl ObjectSubclass for DiscoveredRow {
        const NAME: &'static str = "DiscoveredRow";
        type Type = super::DiscoveredRow;
        type ParentType = adw::SpinRow;
        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            // klass.bind_template_callbacks();
        }
        // fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        //     obj.init_template();
        // }
    }
    impl DiscoveredRow {}
    impl WidgetImpl for DiscoveredRow {}
    impl ObjectImpl for DiscoveredRow {}
    impl ListBoxRowImpl for DiscoveredRow {}
    impl PreferencesRowImpl for DiscoveredRow {}
    impl ActionRowImpl for DiscoveredRow {}
    impl SpinRowImpl for DiscoveredRow {}
}
glib::wrapper! {
    pub struct DiscoveredRow(ObjectSubclass<imp::DiscoveredRow>) @extends adw::SpinRow, adw::ActionRow, adw::PreferencesRow,gtk::Widget,@implements gtk::Accessible;
}
impl DiscoveredRow {
    pub fn new_with_handle(handle: DiscoveryHandle) -> Self {
        Self::new()
    }
    pub fn init(&self, handle: DiscoveryHandle) {
        self.set_title(&handle.device.device_name);
        self.set_subtitle(&format!("{:?}", handle.device.discovery));
    }
}
