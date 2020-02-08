#[macro_use]
extern crate log;
#[macro_use]
extern crate relm;
#[macro_use]
extern crate relm_derive;

use cairo::FontSlant;
use cairo::FontWeight;
use gdk_pixbuf::Pixbuf;
use gtk;
use gtk::ButtonBoxExt;
use gtk::ButtonBoxStyle;
use gtk::ButtonExt;
use gtk::ButtonsType;
use gtk::ContainerExt;
use gtk::CssProviderExt;
use gtk::DialogExt;
use gtk::DialogFlags;
use gtk::DrawingArea;
use gtk::FileChooserAction;
use gtk::FileChooserExt;
use gtk::GtkWindowExt;
use gtk::Inhibit;
use gtk::Label;
use gtk::LabelExt;
use gtk::MessageType;
use gtk::Orientation;
use gtk::Orientation::Horizontal;
use gtk::Orientation::Vertical;
use gtk::OverlayExt;
use gtk::ResponseType;
use gtk::SpinnerExt;
use gtk::StyleContextExt;
use gtk::ToggleButtonExt;
use gtk::WidgetExt;
use gtk::Window;
use gtk::WindowType;
use gtk::{Button, CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION};
use itertools_num::linspace;
use relm::DrawHandler;
use relm::Relm;
use relm::Update;
use relm::Widget;
use relm::{Channel, ContainerWidget};
use structopt::StructOpt;

use core::borrow::Borrow;
use ks_curve_tracer::dut::trace::NullTrace;
use ks_curve_tracer::dut::trace::{GuiTrace, ShareableTrace};
use ks_curve_tracer::dut::DeviceType;
use ks_curve_tracer::dut::SomeDevice;
use ks_curve_tracer::dut::SomeDeviceType;
use ks_curve_tracer::dut::TwoTerminalDevice;
use ks_curve_tracer::dut::{
    CurrentBiasedDeviceType, Device, TwoTerminalDeviceType, VoltageBiasedDeviceType,
};
use ks_curve_tracer::gui::widgets::{DeviceConfig, DeviceConfigMsg, DeviceConfigWidget};
use ks_curve_tracer::options::GuiOpt;
use ks_curve_tracer::options::Opt;
use ks_curve_tracer::util::VERSION;
use ks_curve_tracer::Result;
use std::sync::Arc;
use std::thread;

struct Model {
    relm: Relm<Win>,
    draw_handler: DrawHandler<DrawingArea>,
    trace: Box<dyn GuiTrace>,
    opt: GuiOpt,
    v_zoom: f64,
    i_zoom: f64,
    device: SomeDevice,
}

struct ModelParam {
    opt: GuiOpt,
}

#[derive(Msg, Clone, Debug)]
enum Msg {
    Trace,
    TraceSucceeded(Arc<dyn ShareableTrace>),
    TraceFailed(Arc<failure::Error>),
    FitModel,
    LoadTrace,
    SaveTrace,
    UpdateDrawBuffer,
    Quit,
    VZoom(f64),
    IZoom(f64),
    DeviceType(SomeDeviceType),
    UpdateConfig(DeviceConfig),
}

#[derive(Clone)]
struct Widgets {
    window: Window,
    drawing_area: DrawingArea,
    drawing_area_overlay: gtk::Overlay,
    device_config: relm::Component<DeviceConfigWidget>,
    connection_hint_text: Label,
    legend_text: Label,
    model_text: Label,
}

struct Win {
    model: Model,
    widgets: Widgets,
}

impl Win {
    fn error_box(&self, msg: &str) {
        let error_msg = gtk::MessageDialog::new(
            Some(&self.widgets.window),
            DialogFlags::MODAL,
            MessageType::Error,
            ButtonsType::Close,
            msg,
        );

        error_msg.run();
        error_msg.close();
    }

    fn error_box_error(&self, err: &failure::Error) {
        self.error_box(&format!("Error: {}\nBacktrace: {}", err, err.backtrace()));
    }

    fn handle_error<T>(&self, res: Result<T>) -> Option<T> {
        match res {
            Ok(r) => Some(r),
            Err(e) => {
                self.error_box_error(&e);
                None
            }
        }
    }
}

impl Update for Win {
    type Model = Model;
    type ModelParam = ModelParam;
    type Msg = Msg;

    fn model(relm: &Relm<Self>, param: ModelParam) -> Model {
        Model {
            relm: relm.clone(),
            draw_handler: DrawHandler::new().expect("draw handler"),
            trace: Box::new(NullTrace {}),
            opt: param.opt,
            v_zoom: 1.0,
            i_zoom: 0.05,
            device: SomeDevice::TwoTerminal(TwoTerminalDevice::Diode),
        }
    }

    fn update(&mut self, event: Msg) {
        debug!("Event: {:?}", &event);
        match event {
            Msg::Trace => {
                let drawing_area_overlay_style =
                    self.widgets.drawing_area_overlay.get_style_context();
                drawing_area_overlay_style.add_class("active");
                self.widgets.model_text.set_markup("");
                self.widgets
                    .legend_text
                    .set_markup(&self.model.device.legend());

                let stream = self.model.relm.stream().clone();
                let (_, sender) = Channel::new(move |msg| {
                    stream.emit(msg);
                });
                let res = (|| {
                    let capture_device = self.model.opt.device()?;
                    let dut = self.model.device.clone();

                    thread::spawn(move || {
                        let res = (|| {
                            let trace: Arc<dyn ShareableTrace> =
                                Arc::from(dut.trace(&*capture_device)?);
                            info!("Got the trace");
                            sender
                                .send(Msg::TraceSucceeded(trace))
                                .expect("send message");
                            Ok(())
                        })();
                        if let Err(err) = res {
                            sender.send(Msg::TraceFailed(err)).expect("send message");
                        }
                    });
                    Ok(())
                })();
                if let Err(err) = res {
                    self.model.relm.stream().emit(Msg::TraceFailed(err));
                }
            }
            Msg::TraceSucceeded(trace) => {
                self.model.trace = trace.as_gui_trace();
                self.model.relm.stream().emit(Msg::UpdateDrawBuffer);
                self.model.relm.stream().emit(Msg::FitModel);
            }
            Msg::TraceFailed(error) => {
                self.model.relm.stream().emit(Msg::UpdateDrawBuffer);
                self.model.relm.stream().emit(Msg::FitModel);
                self.error_box_error(error.borrow());
            }
            Msg::FitModel => {
                self.model.trace.fill_model();
                info!("Fit model to the trace");
                self.widgets
                    .model_text
                    .set_markup(&self.model.trace.model_report());
                self.model.relm.stream().emit(Msg::UpdateDrawBuffer);
                let drawing_area_overlay_style =
                    self.widgets.drawing_area_overlay.get_style_context();
                drawing_area_overlay_style.remove_class("active");
            }
            Msg::UpdateDrawBuffer => {
                let cr = self.model.draw_handler.get_context();
                let allocation = self.widgets.drawing_area.get_allocation();
                let width = f64::from(allocation.width);
                let height = f64::from(allocation.height);

                cr.identity_matrix();
                cr.translate(0.5, 0.5);

                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint();

                cr.set_source_rgb(0.0, 0.0, 0.0);
                cr.set_line_width(1.0);

                cr.translate(10.0, 10.0);

                let max_i = self.model.i_zoom;
                let max_v = self.model.v_zoom * self.model.trace.area_of_interest().v_polarity();

                let i_factor = (height - 20.0) / max_i;
                let v_factor = (width - 20.0) / max_v;

                for (ix, i_gridline) in linspace(0.0, max_i, 11).enumerate() {
                    match ix {
                        0 | 10 => {
                            cr.set_dash(&[], 0.0);
                        }
                        5 => {
                            cr.set_dash(&[1.0, 2.0], 0.0);
                        }
                        _ => {
                            cr.set_dash(&[1.0, 3.0], 0.0);
                        }
                    }

                    cr.move_to(0.0, height - 20.0 - i_gridline * i_factor);
                    cr.line_to(width - 20.0, height - 20.0 - i_gridline * i_factor);
                    cr.stroke();
                }

                for (ix, v_gridline) in linspace(0.0, max_v, 11).enumerate() {
                    match ix {
                        0 | 10 => {
                            cr.set_dash(&[], 0.0);
                        }
                        5 => {
                            cr.set_dash(&[1.0, 2.0], 0.0);
                        }
                        _ => {
                            cr.set_dash(&[1.0, 3.0], 0.0);
                        }
                    }

                    cr.move_to(v_gridline * v_factor, 0.0);
                    cr.line_to(v_gridline * v_factor, height - 20.0);
                    cr.stroke();
                }

                self.model.trace.draw(&*cr, v_factor, i_factor, height);

                for (ix, i_gridline) in linspace(0.0, max_i, 11).enumerate() {
                    if ix > 0 {
                        cr.select_font_face("Monospace", FontSlant::Normal, FontWeight::Normal);
                        cr.set_font_size(13.0);
                        let text = format!("{:04.1}mA", i_gridline * 1000.0);
                        let extents = cr.text_extents(&text);
                        cr.move_to(
                            2.0,
                            height - 20.0 - i_gridline * i_factor + extents.height + 2.0,
                        );
                        cr.text_path(&text);
                        cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
                        cr.set_line_width(2.0);
                        cr.stroke_preserve();
                        cr.set_source_rgba(0.0, 0.0, 0.0, 1.0);
                        cr.fill();
                    }
                }

                for (ix, v_gridline) in linspace(0.0, max_v, 11).enumerate() {
                    if ix > 0 {
                        cr.select_font_face("Monospace", FontSlant::Normal, FontWeight::Normal);
                        cr.set_font_size(13.0);
                        let text = format!("{:.2}V", v_gridline);
                        let extents = cr.text_extents(&text);
                        if ix == 10 {
                            cr.move_to(
                                v_gridline * v_factor - extents.width - 2.0,
                                extents.height + 2.0,
                            );
                        } else {
                            cr.move_to(
                                v_gridline * v_factor - extents.width / 2.0,
                                extents.height + 2.0,
                            );
                        }
                        cr.text_path(&text);
                        cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
                        cr.set_line_width(2.0);
                        cr.stroke_preserve();
                        cr.set_source_rgba(0.0, 0.0, 0.0, 1.0);
                        cr.fill();
                    }
                }

                self.model.trace.draw_model(&cr, v_factor, i_factor, height);
            }
            Msg::DeviceType(device_type) => {
                let device = device_type.to_device();
                self.widgets
                    .device_config
                    .stream()
                    .emit(DeviceConfigMsg::SetConfig(device.config()));

                self.model.device = device;
                self.widgets
                    .connection_hint_text
                    .set_markup(device_type.connection_hint());
            }
            Msg::UpdateConfig(config) => {
                self.model.device.set_config(&config);
            }
            Msg::VZoom(z) => {
                self.model.v_zoom = z;
                self.model.relm.stream().emit(Msg::UpdateDrawBuffer);
            }
            Msg::IZoom(z) => {
                self.model.i_zoom = z;
                self.model.relm.stream().emit(Msg::UpdateDrawBuffer);
            }
            Msg::SaveTrace => {
                let dialog = gtk::FileChooserDialog::with_buttons(
                    Some("Save trace"),
                    Some(&self.widgets.window),
                    FileChooserAction::Save,
                    &[
                        ("_Cancel", ResponseType::Cancel),
                        ("_Save", ResponseType::Accept),
                    ],
                );
                dialog.set_do_overwrite_confirmation(true);

                if dialog.run() == gtk::ResponseType::Accept {
                    if let Some(filename) = dialog.get_filename() {
                        let _ = self.model.trace.save_as_csv(&filename);
                    }
                }
                dialog.close();
            }
            Msg::LoadTrace => {
                let dialog = gtk::FileChooserDialog::with_buttons(
                    Some("Load trace"),
                    Some(&self.widgets.window),
                    FileChooserAction::Open,
                    &[
                        ("_Cancel", ResponseType::Cancel),
                        ("_Load", ResponseType::Accept),
                    ],
                );
                dialog.set_do_overwrite_confirmation(true);

                if dialog.run() == gtk::ResponseType::Accept {
                    if let Some(filename) = dialog.get_filename() {
                        let res = (|| {
                            self.model.trace =
                                self.model.device.load_from_csv(filename)?.as_gui_trace();
                            info!("Got the trace");

                            self.widgets.legend_text.set_markup("");
                            self.widgets.model_text.set_markup("");
                            self.model.relm.stream().emit(Msg::UpdateDrawBuffer);

                            self.model.relm.stream().emit(Msg::FitModel);
                            Ok(())
                        })();
                        let _ = self.handle_error(res);
                    }
                }
                dialog.close();
            }
            Msg::Quit => gtk::main_quit(),
        }
    }
}

impl Widget for Win {
    // Specify the type of the root widget.
    type Root = Window;

    // Return the root widget.
    fn root(&self) -> Self::Root {
        self.widgets.window.clone()
    }

    // Create the widgets.
    fn view(relm: &Relm<Self>, mut model: Self::Model) -> Self {
        Window::set_default_icon(
            &Pixbuf::new_from_inline(include_bytes!("../../res/icon256.pixbuf"), false).unwrap(),
        );

        let hbox = gtk::Box::new(Horizontal, 0);

        let right_pane = gtk::Box::new(Vertical, 8);
        right_pane.set_size_request(300, 500);
        right_pane.set_margin_top(8);
        right_pane.set_margin_bottom(8);
        right_pane.set_margin_start(4);
        right_pane.set_margin_end(4);
        right_pane.set_hexpand(false);

        let help_text = gtk::Label::new(Some(""));
        help_text.set_xalign(0.0);
        help_text.set_markup(&format!("\
        Version: {}\n\
        Please submit your suggestions and bug reports here:\n\
        <a href=\"https://github.com/knack-supply/curve-tracer/issues\">https://github.com/knack-supply/curve-tracer/issues</a>\n\
        \n\
        Usage:\n\
        1) Select the device type\n\
        2) Set the trace settings, such as bias levels\n\
        3) Press \"Trace\" button below", VERSION));

        right_pane.add(&help_text);

        fn radio_button_box(
            relm: &Relm<Win>,
            options: impl Iterator<Item = (String, Msg)>,
            initial_option: usize,
            last_button: Option<gtk::RadioButton>,
        ) -> Option<(gtk::ButtonBox, gtk::RadioButton)> {
            let mut options = options.enumerate();
            let button_box = gtk::ButtonBox::new(Orientation::Horizontal);
            button_box.set_layout(ButtonBoxStyle::Expand);

            fn setup_button(
                relm: &Relm<Win>,
                msg: Msg,
                button: &gtk::RadioButton,
                button_box: &gtk::ButtonBox,
            ) {
                button.set_mode(false);
                connect!(relm, button, connect_toggled(btn), {
                    if btn.get_active() {
                        Some(msg.clone())
                    } else {
                        None
                    }
                });
                button_box.add(button);
            }

            let last_button: gtk::RadioButton = match last_button {
                Some(last_button) => last_button,
                None => {
                    let (ix, (label, msg)) = options.next()?;
                    let last_button = gtk::RadioButton::new_with_label(&label);
                    setup_button(&relm, msg, &last_button, &button_box);
                    if ix == initial_option {
                        last_button.set_active(true);
                    }
                    last_button
                }
            };

            for (ix, (label, msg)) in options {
                let button = gtk::RadioButton::new_with_label_from_widget(&last_button, &label);
                setup_button(&relm, msg, &button, &button_box);
                if ix == initial_option {
                    button.set_active(true);
                }
            }

            Some((button_box, last_button))
        }

        {
            let options = [
                SomeDeviceType::TwoTerminal(TwoTerminalDeviceType::Diode),
                SomeDeviceType::CurrentBiased(CurrentBiasedDeviceType::NPN),
                SomeDeviceType::CurrentBiased(CurrentBiasedDeviceType::PNP),
            ]
            .iter()
            .copied()
            .map(|d| (format!("{}", d), Msg::DeviceType(d)));
            let (buttons, last_button) = radio_button_box(&relm, options, 0, None).unwrap();
            right_pane.add(&buttons);

            let options = [
                SomeDeviceType::VoltageBiased(VoltageBiasedDeviceType::NEFET),
                SomeDeviceType::VoltageBiased(VoltageBiasedDeviceType::PEFET),
                SomeDeviceType::VoltageBiased(VoltageBiasedDeviceType::NDFET),
                SomeDeviceType::VoltageBiased(VoltageBiasedDeviceType::PDFET),
            ]
            .iter()
            .copied()
            .map(|d| (format!("{}", d), Msg::DeviceType(d)));
            let (buttons, _) = radio_button_box(&relm, options, 0, Some(last_button)).unwrap();
            right_pane.add(&buttons);
        }

        let connection_hint_text = gtk::Label::new(Some(""));
        connection_hint_text.set_xalign(0.0);
        connection_hint_text.set_markup(model.device.device_type().connection_hint());
        right_pane.add(&connection_hint_text);

        let action_box = gtk::Box::new(Orientation::Horizontal, 8);

        let trace_button = Button::new_with_label("Trace");
        trace_button.set_hexpand(true);
        action_box.add(&trace_button);

        let save_button = Button::new_from_icon_name(Some("document-save"), gtk::IconSize::Button);
        action_box.add(&save_button);

        let load_button = Button::new_from_icon_name(Some("document-open"), gtk::IconSize::Button);
        action_box.add(&load_button);

        right_pane.add(&action_box);

        let device_config = right_pane.add_widget::<DeviceConfigWidget>(model.device.config());
        {
            let relm = relm.clone();
            #[allow(clippy::single_match)]
            device_config.stream().observe(move |m| match m {
                DeviceConfigMsg::ConfigUpdated(c) => {
                    relm.stream().emit(Msg::UpdateConfig(c.clone()));
                }
                _ => {}
            });
        }

        let filler = gtk::Box::new(gtk::Orientation::Vertical, 0);
        filler.set_vexpand(true);
        right_pane.add(&filler);

        let model_text = gtk::Label::new(Some(""));
        model_text.set_xalign(0.0);
        model_text.set_margin_top(8);
        model_text.set_margin_bottom(8);
        right_pane.add(&model_text);

        let legend_text = gtk::Label::new(Some(""));
        legend_text.set_xalign(0.0);
        legend_text.set_markup(&model.device.legend());
        right_pane.add(&legend_text);

        {
            let options = [0.5, 1.0, 2.0, 5.0]
                .iter()
                .map(|&v| (format!("{:0.1}V", v), Msg::VZoom(v)));
            if let Some((buttons, _)) = radio_button_box(&relm, options, 1, None) {
                right_pane.add(&buttons);
            }
        }

        {
            let options = [0.005, 0.01, 0.02, 0.05]
                .iter()
                .map(|&i| (format!("{:0.0}mA", i * 1000.0), Msg::IZoom(i)));
            if let Some((buttons, _)) = radio_button_box(&relm, options, 3, None) {
                right_pane.add(&buttons);
            }
        }

        let drawing_area_overlay = gtk::Overlay::new();
        drawing_area_overlay.set_widget_name("iv-curve");
        drawing_area_overlay.set_hexpand(true);
        drawing_area_overlay.set_vexpand(true);
        let drawing_area = DrawingArea::new();

        model.draw_handler.init(&drawing_area);

        drawing_area.set_hexpand(true);
        drawing_area.set_size_request(500, 500);
        drawing_area_overlay.add(&drawing_area);

        let lightbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        lightbox.set_hexpand(true);
        lightbox.set_vexpand(true);

        let spinner = gtk::Spinner::new();
        spinner.set_size_request(256, 256);
        spinner.set_halign(gtk::Align::Center);
        spinner.set_valign(gtk::Align::Center);
        spinner.start();

        drawing_area_overlay.add_overlay(&lightbox);
        drawing_area_overlay.add_overlay(&spinner);

        drawing_area_overlay.show_all();

        hbox.add(&drawing_area_overlay);
        hbox.add(&right_pane);

        let window = Window::new(WindowType::Toplevel);

        let style = include_bytes!("../../res/main.css");
        let css_provider = CssProvider::new();
        css_provider.load_from_data(style).unwrap();
        gtk::StyleContext::add_provider_for_screen(
            &gdk::Screen::get_default().unwrap(),
            &css_provider,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        window.add(&hbox);

        window.show_all();

        connect!(
            relm,
            drawing_area,
            connect_size_allocate(_, _),
            Msg::UpdateDrawBuffer
        );
        connect!(relm, trace_button, connect_clicked(_), Msg::Trace);
        connect!(relm, save_button, connect_clicked(_), Msg::SaveTrace);
        connect!(relm, load_button, connect_clicked(_), Msg::LoadTrace);
        connect!(
            relm,
            window,
            connect_delete_event(_, _),
            return (Some(Msg::Quit), Inhibit(false))
        );

        Win {
            model,
            widgets: Widgets {
                window,
                drawing_area,
                drawing_area_overlay,
                device_config,
                model_text,
                connection_hint_text,
                legend_text,
            },
        }
    }
}

fn main() -> Result<()> {
    let opt = GuiOpt::from_args();
    opt.initialize_logging()?;

    Win::run(ModelParam { opt }).unwrap();
    Ok(())
}
