use relm::ContainerWidget;
use relm::{Relm, Update, Widget};

use crate::dut::{CurrentBiasedDeviceConfig, TwoTerminalDeviceConfig, VoltageBiasedDeviceConfig};
use crate::gui::widgets::bjt::{BJTOptionsMsg, BJTOptionsWidget};
use crate::gui::widgets::fet::{FETOptionsMsg, FETOptionsWidget};

pub mod bjt;
pub mod fet;

#[derive(Clone, Debug)]
pub enum DeviceConfig {
    None,
    BJT(CurrentBiasedDeviceConfig),
    FET(VoltageBiasedDeviceConfig),
}

impl DeviceConfig {
    pub fn downcast<'a, T>(&'a self) -> Option<T>
    where
        &'a Self: Into<Option<T>>,
    {
        Into::<Option<T>>::into(self)
    }
}

impl Into<Option<TwoTerminalDeviceConfig>> for &DeviceConfig {
    fn into(self) -> Option<TwoTerminalDeviceConfig> {
        if let DeviceConfig::None = self {
            Some(TwoTerminalDeviceConfig {})
        } else {
            None
        }
    }
}

impl Into<Option<CurrentBiasedDeviceConfig>> for &DeviceConfig {
    fn into(self) -> Option<CurrentBiasedDeviceConfig> {
        if let DeviceConfig::BJT(config) = self {
            Some(config.clone())
        } else {
            None
        }
    }
}

impl Into<Option<VoltageBiasedDeviceConfig>> for &DeviceConfig {
    fn into(self) -> Option<VoltageBiasedDeviceConfig> {
        if let DeviceConfig::FET(config) = self {
            Some(config.clone())
        } else {
            None
        }
    }
}

enum DeviceConfigModel {
    None,
    BJT(relm::Component<BJTOptionsWidget>, CurrentBiasedDeviceConfig),
    FET(relm::Component<FETOptionsWidget>, VoltageBiasedDeviceConfig),
}

pub struct Model {
    relm: Relm<DeviceConfigWidget>,
    device_config_model: DeviceConfigModel,
}

#[derive(Msg, Debug)]
pub enum DeviceConfigMsg {
    SetConfig(DeviceConfig),
    ConfigUpdated(DeviceConfig),
}

pub struct DeviceConfigWidget {
    model: Model,
    widget: gtk::Box,
}

impl Update for DeviceConfigWidget {
    type Model = Model;
    type ModelParam = DeviceConfig;
    type Msg = DeviceConfigMsg;

    fn model(relm: &Relm<Self>, initial_config: DeviceConfig) -> Model {
        relm.stream()
            .emit(DeviceConfigMsg::SetConfig(initial_config));
        Model {
            relm: relm.clone(),
            device_config_model: DeviceConfigModel::None,
        }
    }

    fn update(&mut self, event: DeviceConfigMsg) {
        match event {
            DeviceConfigMsg::SetConfig(config) => {
                match &self.model.device_config_model {
                    DeviceConfigModel::None => {}
                    DeviceConfigModel::BJT(c, _) => {
                        self.root().remove_widget(c.clone());
                    }
                    DeviceConfigModel::FET(c, _) => {
                        self.root().remove_widget(c.clone());
                    }
                };

                self.model.device_config_model = match config {
                    DeviceConfig::None => DeviceConfigModel::None,
                    DeviceConfig::BJT(c) => {
                        let comp = self.root().add_widget::<BJTOptionsWidget>(c.clone());
                        let stream = self.model.relm.stream().clone();
                        comp.stream().observe(move |msg| {
                            match msg {
                                BJTOptionsMsg::Updated(config) => {
                                    stream.emit(DeviceConfigMsg::ConfigUpdated(DeviceConfig::BJT(
                                        config.clone(),
                                    )))
                                }
                                _ => {}
                            };
                        });
                        DeviceConfigModel::BJT(comp, c)
                    }
                    DeviceConfig::FET(c) => {
                        let comp = self.root().add_widget::<FETOptionsWidget>(c.clone());
                        let stream = self.model.relm.stream().clone();
                        comp.stream().observe(move |msg| {
                            match msg {
                                FETOptionsMsg::Updated(config) => {
                                    stream.emit(DeviceConfigMsg::ConfigUpdated(DeviceConfig::FET(
                                        config.clone(),
                                    )))
                                }
                                _ => {}
                            };
                        });
                        DeviceConfigModel::FET(comp, c)
                    }
                }
            }
            DeviceConfigMsg::ConfigUpdated(_) => {}
        }
    }
}

impl Widget for DeviceConfigWidget {
    type Root = gtk::Box;

    fn root(&self) -> gtk::Box {
        self.widget.clone()
    }

    fn view(_: &Relm<Self>, model: Model) -> Self {
        let hbox = gtk::Box::new(gtk::Orientation::Vertical, 8);

        DeviceConfigWidget {
            model,
            widget: hbox,
        }
    }
}

impl DeviceConfigWidget {
    pub fn config(&self) -> DeviceConfig {
        match &self.model.device_config_model {
            DeviceConfigModel::None => DeviceConfig::None,
            DeviceConfigModel::BJT(_, c) => DeviceConfig::BJT(c.clone()),
            DeviceConfigModel::FET(_, c) => DeviceConfig::FET(c.clone()),
        }
    }
}
