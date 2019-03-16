use crate::dut::CurrentBiasedDeviceConfig;
use gtk::ContainerExt;
use gtk::SpinButtonExt;
use gtk::SpinButtonSignals;
use gtk::WidgetExt;
use gtk::LabelExt;
use noisy_float::prelude::r64;
use noisy_float::prelude::R64;
use relm::{Relm, Update, Widget};

#[derive(Msg)]
pub enum BJTOptionsMsg {
    MinBias(R64),
    MaxBias(R64),
    Updated(CurrentBiasedDeviceConfig),
}

struct Widgets {
    root: gtk::Box,
    min_spinner: gtk::SpinButton,
    max_spinner: gtk::SpinButton,
}

pub struct BJTOptionsWidget {
    model: BJTOptionsModel,
    widgets: Widgets,
}

pub struct BJTOptionsModel {
    relm: Relm<BJTOptionsWidget>,
    config: CurrentBiasedDeviceConfig,
}

impl Update for BJTOptionsWidget {
    type Model = BJTOptionsModel;
    type ModelParam = CurrentBiasedDeviceConfig;
    type Msg = BJTOptionsMsg;

    fn model(relm: &Relm<Self>, config: CurrentBiasedDeviceConfig) -> BJTOptionsModel {
        BJTOptionsModel {
            relm: relm.clone(),
            config,
        }
    }

    fn update(&mut self, event: Self::Msg) {
        match event {
            BJTOptionsMsg::MinBias(b) => {
                let mut config = &mut self.model.config;
                config.min_bias_current = b / 1_000_000.0;
                if config.max_bias_current < config.min_bias_current {
                    config.max_bias_current = config.min_bias_current;
                    self.widgets
                        .max_spinner
                        .set_value(config.max_bias_current.raw() * 1_000_000.0);
                }
                self.model
                    .relm
                    .stream()
                    .emit(BJTOptionsMsg::Updated(config.clone()));
            }
            BJTOptionsMsg::MaxBias(b) => {
                let mut config = &mut self.model.config;
                config.max_bias_current = b / 1_000_000.0;
                if config.max_bias_current < config.min_bias_current {
                    config.min_bias_current = config.max_bias_current;
                    self.widgets
                        .min_spinner
                        .set_value(config.min_bias_current.raw() * 1_000_000.0);
                }
                self.model
                    .relm
                    .stream()
                    .emit(BJTOptionsMsg::Updated(config.clone()));
            }
            BJTOptionsMsg::Updated(_) => {}
        }
    }
}

impl Widget for BJTOptionsWidget {
    type Root = gtk::Box;

    fn root(&self) -> gtk::Box {
        self.widgets.root.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);

        let bias_label = gtk::Label::new("");
        bias_label.set_markup("I<sub>BE</sub>");
        hbox.add(&bias_label);

        let min_spinner = gtk::SpinButton::new_with_range(0.0, 50.0, 1.0);
        min_spinner.set_numeric(true);
        min_spinner.set_value(model.config.min_bias_current.raw() * 1_000_000.0);
        hbox.add(&min_spinner);

        hbox.add(&gtk::Label::new("to"));

        let max_spinner = gtk::SpinButton::new_with_range(0.0, 50.0, 1.0);
        max_spinner.set_numeric(true);
        max_spinner.set_value(model.config.max_bias_current.raw() * 1_000_000.0);
        hbox.add(&max_spinner);

        hbox.add(&gtk::Label::new("µA"));

        connect!(
            relm,
            min_spinner,
            connect_value_changed(btn),
            BJTOptionsMsg::MinBias(r64(btn.get_value()))
        );
        connect!(
            relm,
            max_spinner,
            connect_value_changed(btn),
            BJTOptionsMsg::MaxBias(r64(btn.get_value()))
        );

        hbox.show_all();

        BJTOptionsWidget {
            model,
            widgets: Widgets {
                root: hbox,
                min_spinner,
                max_spinner,
            },
        }
    }
}
