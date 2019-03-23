use gtk::ContainerExt;
use gtk::LabelExt;
use gtk::SpinButtonExt;
use gtk::SpinButtonSignals;
use gtk::WidgetExt;
use noisy_float::prelude::r64;
use noisy_float::prelude::R64;
use relm::{Relm, Update, Widget};

use crate::dut::{VoltageBiasedDeviceConfig, VoltageBiasedDeviceType};

#[derive(Msg)]
pub enum FETOptionsMsg {
    MinBias(R64),
    MaxBias(R64),
    Updated(VoltageBiasedDeviceConfig),
}

struct Widgets {
    root: gtk::Box,
    min_spinner: gtk::SpinButton,
    max_spinner: gtk::SpinButton,
}

pub struct FETOptionsWidget {
    model: FETOptionsModel,
    widgets: Widgets,
}

pub struct FETOptionsModel {
    relm: Relm<FETOptionsWidget>,
    config: VoltageBiasedDeviceConfig,
}

impl Update for FETOptionsWidget {
    type Model = FETOptionsModel;
    type ModelParam = VoltageBiasedDeviceConfig;
    type Msg = FETOptionsMsg;

    fn model(relm: &Relm<Self>, config: VoltageBiasedDeviceConfig) -> FETOptionsModel {
        FETOptionsModel {
            relm: relm.clone(),
            config,
        }
    }

    fn update(&mut self, event: Self::Msg) {
        match event {
            FETOptionsMsg::MinBias(b) => {
                let mut config = &mut self.model.config;
                config.min_bias_voltage = b;
                if config.max_bias_voltage < config.min_bias_voltage {
                    config.max_bias_voltage = config.min_bias_voltage;
                    self.widgets
                        .max_spinner
                        .set_value(config.max_bias_voltage.raw());
                }
                self.model
                    .relm
                    .stream()
                    .emit(FETOptionsMsg::Updated(config.clone()));
            }
            FETOptionsMsg::MaxBias(b) => {
                let mut config = &mut self.model.config;
                config.max_bias_voltage = b;
                if config.max_bias_voltage < config.min_bias_voltage {
                    config.min_bias_voltage = config.max_bias_voltage;
                    self.widgets
                        .min_spinner
                        .set_value(config.min_bias_voltage.raw());
                }
                self.model
                    .relm
                    .stream()
                    .emit(FETOptionsMsg::Updated(config.clone()));
            }
            FETOptionsMsg::Updated(_) => {}
        }
    }
}

impl Widget for FETOptionsWidget {
    type Root = gtk::Box;

    fn root(&self) -> gtk::Box {
        self.widgets.root.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);

        let sign = match model.config.device_type {
            VoltageBiasedDeviceType::NEFET | VoltageBiasedDeviceType::PEFET => "",
            VoltageBiasedDeviceType::NDFET | VoltageBiasedDeviceType::PDFET => "-",
        };

        let bias_label = gtk::Label::new("");
        bias_label.set_markup(&format!("{}V<sub>GS</sub>", sign));
        hbox.add(&bias_label);

        let min_spinner = gtk::SpinButton::new_with_range(0.0, 5.0, 0.1);
        min_spinner.set_numeric(true);
        min_spinner.set_hexpand(true);
        min_spinner.set_value(model.config.min_bias_voltage.raw());
        hbox.add(&min_spinner);

        hbox.add(&gtk::Label::new("to"));

        let max_spinner = gtk::SpinButton::new_with_range(0.0, 5.0, 0.1);
        max_spinner.set_numeric(true);
        max_spinner.set_hexpand(true);
        max_spinner.set_value(model.config.max_bias_voltage.raw());
        hbox.add(&max_spinner);

        hbox.add(&gtk::Label::new("V"));

        connect!(
            relm,
            min_spinner,
            connect_value_changed(btn),
            FETOptionsMsg::MinBias(r64(btn.get_value()))
        );
        connect!(
            relm,
            max_spinner,
            connect_value_changed(btn),
            FETOptionsMsg::MaxBias(r64(btn.get_value()))
        );

        hbox.show_all();

        FETOptionsWidget {
            model,
            widgets: Widgets {
                root: hbox,
                min_spinner,
                max_spinner,
            },
        }
    }
}
