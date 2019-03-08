#![feature(try_blocks)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate relm;
#[macro_use]
extern crate relm_derive;

use cairo::enums::FontSlant;
use cairo::enums::FontWeight;
use gdk_pixbuf::Pixbuf;
use gtk;
use gtk::Button;
use gtk::ButtonBoxExt;
use gtk::ButtonBoxStyle;
use gtk::ButtonExt;
use gtk::ButtonsType;
use gtk::ContainerExt;
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
use gtk::ResponseType;
use gtk::ToggleButtonExt;
use gtk::WidgetExt;
use gtk::Window;
use gtk::WindowType;
use itertools_num::linspace;
use relm::DrawHandler;
use relm::Relm;
use relm::Update;
use relm::Widget;
use structopt::StructOpt;

use ks_curve_tracer::backend::BiasedTrace;
use ks_curve_tracer::gui::DevicePlot;
use ks_curve_tracer::gui::GuiTrace;
use ks_curve_tracer::options::GuiOpt;
use ks_curve_tracer::options::Opt;
use ks_curve_tracer::trace::file::ImportableTrace;
use ks_curve_tracer::DeviceType;
use ks_curve_tracer::Result;
use ks_curve_tracer::ThreeTerminalTrace;
use ks_curve_tracer::TwoTerminalTrace;
use ks_curve_tracer::{NullTrace, ThreeTerminalDeviceType, TwoTerminalDeviceType};

struct Model {
    relm: Relm<Win>,
    draw_handler: DrawHandler<DrawingArea>,
    trace: Box<dyn GuiTrace>,
    opt: GuiOpt,
    v_zoom: f64,
    i_zoom: f64,
    device_type: DeviceType,
}

struct ModelParam {
    opt: GuiOpt,
}

#[derive(Msg, Clone)]
enum Msg {
    Trace,
    FitModel,
    LoadTrace,
    SaveTrace,
    UpdateDrawBuffer,
    Quit,
    VZoom(f64),
    IZoom(f64),
    DeviceType(DeviceType),
}

#[derive(Clone)]
struct Widgets {
    trace_button: Button,
    drawing_area: DrawingArea,
    model_text: Label,
    window: Window,
    connection_hint_text: Label,
    legend_text: Label,
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
            device_type: DeviceType::TwoTerminal(TwoTerminalDeviceType::Diode),
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::Trace => {
                let device = self.model.opt.device().unwrap();

                match self.model.device_type {
                    DeviceType::TwoTerminal(device_type) => match device.trace_2(device_type) {
                        Ok(trace) => {
                            self.model.trace = Box::new(TwoTerminalTrace::from(trace));
                            info!("Got the trace");

                            self.widgets.model_text.set_markup("");
                            self.widgets.drawing_area.queue_resize();

                            self.model.relm.stream().emit(Msg::FitModel);
                        }
                        Err(err) => {
                            self.error_box_error(&err);
                        }
                    },
                    DeviceType::ThreeTerminal(device_type) => match device.trace_3(device_type) {
                        Ok(traces) => {
                            self.model.trace = Box::new(ThreeTerminalTrace::from(
                                traces
                                    .into_iter()
                                    .map(|BiasedTrace { bias, trace }| (bias, trace)),
                            ));
                            info!("Got the trace");

                            self.widgets.model_text.set_markup("");
                            self.widgets.drawing_area.queue_resize();

                            self.model.relm.stream().emit(Msg::FitModel);
                        }
                        Err(err) => {
                            self.error_box_error(&err);
                        }
                    },
                }
            }
            Msg::FitModel => {
                self.model.trace.fill_model();
                info!("Fit model to the trace");
                self.widgets
                    .model_text
                    .set_markup(&self.model.trace.model_report());
                self.widgets.drawing_area.queue_resize();
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
                let max_v = self.model.v_zoom * self.model.device_type.polarity();

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

                self.model.trace.draw(&cr, v_factor, i_factor, height);

                cr.set_source_rgba(0.0, 0.0, 0.0, 1.0);

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
                        cr.show_text(&text);
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
                        cr.show_text(&text);
                    }
                }

                self.model.trace.draw_model(&cr, v_factor, i_factor, height);
            }
            Msg::DeviceType(device_type) => {
                self.model.device_type = device_type;
                self.widgets
                    .connection_hint_text
                    .set_markup(self.model.device_type.connection_hint());
                self.widgets
                    .legend_text
                    .set_markup(&self.model.device_type.legend());

                self.widgets.drawing_area.queue_resize();
            }
            Msg::VZoom(z) => {
                self.model.v_zoom = z;
                self.widgets.drawing_area.queue_resize();
            }
            Msg::IZoom(z) => {
                self.model.i_zoom = z;
                self.widgets.drawing_area.queue_resize();
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

                if dialog.run() == gtk::ResponseType::Accept.into() {
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

                if dialog.run() == gtk::ResponseType::Accept.into() {
                    if let Some(filename) = dialog.get_filename() {
                        let res = try {
                            self.model.trace = match self.model.device_type {
                                DeviceType::TwoTerminal(_) => {
                                    Box::new(TwoTerminalTrace::from_csv(filename)?)
                                }
                                DeviceType::ThreeTerminal(_) => {
                                    Box::new(ThreeTerminalTrace::from_csv(filename)?)
                                }
                            };
                            info!("Got the trace");

                            self.widgets.model_text.set_markup("");
                            self.widgets.drawing_area.queue_resize();

                            self.model.relm.stream().emit(Msg::FitModel);
                        };
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

        let help_text = gtk::Label::new("");
        help_text.set_xalign(0.0);
        help_text.set_markup("\
        This is a very early version of the software.\n\
        Only diode measurement is included\n\
        for demonstration purposes.\n\
        Please submit your suggestions and bug reports here:\n\
        <a href=\"https://github.com/knack-supply/curve-tracer\">https://github.com/knack-supply/curve-tracer</a>\n\
        \n\
        Usage:\n\
        1) Run trace at least once\n\
        2) Wait 5 seconds to let AD2 input offsets settle\n\
        3) Use some thermal insulator to insert the device\n\
        under test into the curve tracer\n\
        4) Press \"Trace\" button below");

        right_pane.add(&help_text);

        fn radio_button_box(
            relm: &Relm<Win>,
            options: impl Iterator<Item = (String, Msg)>,
            initial_option: usize,
        ) -> Option<gtk::ButtonBox> {
            let mut options = options.enumerate();
            let (ix, (label, msg)) = options.next()?;
            let button_box = gtk::ButtonBox::new(Orientation::Horizontal);
            button_box.set_layout(ButtonBoxStyle::Expand);

            fn setup_button(
                relm: &Relm<Win>,
                msg: Msg,
                button: &gtk::RadioButton,
                v_zoom_buttons: &gtk::ButtonBox,
            ) {
                button.set_mode(false);
                connect!(relm, button, connect_clicked(_), msg.clone());
                v_zoom_buttons.add(button);
            }

            let first_button = gtk::RadioButton::new_with_label(&label);
            setup_button(&relm, msg, &first_button, &button_box);
            if ix == initial_option {
                first_button.set_active(true);
            }

            for (ix, (label, msg)) in options {
                let zoom_button =
                    gtk::RadioButton::new_with_label_from_widget(&first_button, &label);
                setup_button(&relm, msg, &zoom_button, &button_box);
                if ix == initial_option {
                    zoom_button.set_active(true);
                }
            }

            Some(button_box)
        }

        {
            let options = [
                DeviceType::TwoTerminal(TwoTerminalDeviceType::Diode),
                DeviceType::ThreeTerminal(ThreeTerminalDeviceType::NPN),
                DeviceType::ThreeTerminal(ThreeTerminalDeviceType::PNP),
                DeviceType::ThreeTerminal(ThreeTerminalDeviceType::NFET),
                DeviceType::ThreeTerminal(ThreeTerminalDeviceType::PFET),
            ]
            .iter()
            .map(|&d| (format!("{}", d), Msg::DeviceType(d)));
            if let Some(buttons) = radio_button_box(&relm, options, 0) {
                right_pane.add(&buttons);
            }
        }

        let connection_hint_text = gtk::Label::new("");
        connection_hint_text.set_xalign(0.0);
        connection_hint_text.set_markup(model.device_type.connection_hint());
        right_pane.add(&connection_hint_text);

        {
            let options = [0.5, 1.0, 2.0, 5.0]
                .iter()
                .map(|&v| (format!("{:0.1}V", v), Msg::VZoom(v)));
            if let Some(buttons) = radio_button_box(&relm, options, 1) {
                right_pane.add(&buttons);
            }
        }

        {
            let options = [0.005, 0.01, 0.02, 0.05]
                .iter()
                .map(|&i| (format!("{:0.0}mA", i * 1000.0), Msg::IZoom(i)));
            if let Some(buttons) = radio_button_box(&relm, options, 3) {
                right_pane.add(&buttons);
            }
        }

        let action_box = gtk::Box::new(Orientation::Horizontal, 8);

        let trace_button = Button::new_with_label("Trace");
        trace_button.set_hexpand(true);
        action_box.add(&trace_button);

        let save_button = Button::new_from_icon_name("document-save", gtk::IconSize::Button.into());
        action_box.add(&save_button);

        let load_button = Button::new_from_icon_name("document-open", gtk::IconSize::Button.into());
        action_box.add(&load_button);

        right_pane.add(&action_box);

        let legend_text = gtk::Label::new("");
        legend_text.set_xalign(0.0);
        legend_text.set_markup(&model.device_type.legend());
        right_pane.add(&legend_text);

        let model_text = gtk::Label::new("");
        model_text.set_xalign(0.0);
        model_text.set_margin_top(8);
        model_text.set_margin_bottom(8);
        right_pane.add(&model_text);

        let drawing_area = DrawingArea::new();
        model.draw_handler.init(&drawing_area);
        drawing_area.set_hexpand(true);
        drawing_area.set_size_request(500, 500);
        hbox.add(&drawing_area);
        hbox.add(&right_pane);

        let window = Window::new(WindowType::Toplevel);

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
                trace_button,
                drawing_area,
                model_text,
                connection_hint_text,
                legend_text,
                window,
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
