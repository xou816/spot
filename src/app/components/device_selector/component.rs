use std::ops::Deref;
use std::rc::Rc;

use glib::Cast;

use crate::app::components::{Component, EventListener};
use crate::app::models::ConnectDevice;
use crate::app::state::{Device, LoginEvent, PlaybackAction, PlaybackEvent};
use crate::app::{ActionDispatcher, AppEvent, AppModel};

use super::widget::DeviceSelectorWidget;

pub struct DeviceSelectorModel {
    app_model: Rc<AppModel>,
    dispatcher: Box<dyn ActionDispatcher>,
}

impl DeviceSelectorModel {
    pub fn new(app_model: Rc<AppModel>, dispatcher: Box<dyn ActionDispatcher>) -> Self {
        Self {
            app_model,
            dispatcher,
        }
    }

    pub fn refresh_available_devices(&self) {
        let api = self.app_model.get_spotify();

        self.dispatcher
            .call_spotify_and_dispatch(move || async move {
                api.list_available_devices()
                    .await
                    .map(|devices| PlaybackAction::SetAvailableDevices(devices).into())
            });
    }

    pub fn get_available_devices(&self) -> impl Deref<Target = Vec<ConnectDevice>> + '_ {
        self.app_model.map_state(|s| s.playback.available_devices())
    }

    pub fn get_current_device(&self) -> impl Deref<Target = Device> + '_ {
        self.app_model.map_state(|s| s.playback.current_device())
    }

    pub fn set_current_device(&self, id: Option<String>) {
        let devices = self.get_available_devices();
        let connect_device = id
            .and_then(|id| devices.iter().find(|&d| d.id == id))
            .cloned();
        let device = connect_device.map(Device::Connect).unwrap_or(Device::Local);
        self.dispatcher
            .dispatch(PlaybackAction::SwitchDevice(device).into());
    }
}

pub struct DeviceSelector {
    widget: DeviceSelectorWidget,
    model: Rc<DeviceSelectorModel>,
}

impl DeviceSelector {
    pub fn new(widget: DeviceSelectorWidget, model: DeviceSelectorModel) -> Self {
        let model = Rc::new(model);

        widget.connect_refresh(clone!(@weak model => move || {
            model.refresh_available_devices();
        }));

        widget.connect_switch_device(clone!(@weak model => move |id| {
            model.set_current_device(id);
        }));

        Self { widget, model }
    }
}

impl Component for DeviceSelector {
    fn get_root_widget(&self) -> &gtk::Widget {
        self.widget.upcast_ref()
    }
}

impl EventListener for DeviceSelector {
    fn on_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::LoginEvent(LoginEvent::LoginCompleted(_)) => {
                self.model.refresh_available_devices();
            }
            AppEvent::PlaybackEvent(PlaybackEvent::AvailableDevicesChanged) => {
                self.widget
                    .update_devices_list(&*self.model.get_available_devices());
            }
            AppEvent::PlaybackEvent(PlaybackEvent::SwitchedDevice(_)) => {
                self.widget
                    .set_current_device(&*self.model.get_current_device());
            }
            _ => (),
        }
    }
}
