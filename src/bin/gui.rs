extern crate gtk;
#[macro_use]
extern crate relm;
#[macro_use]
extern crate relm_derive;

use std::f64::consts::PI;

use cairo::enums::{FontSlant, FontWeight};
use gtk::DrawingArea;
use gtk::Orientation::{Horizontal, Vertical};
use gtk::{
    Button, ButtonExt, ButtonsType, ContainerExt, DialogExt, DialogFlags, GtkWindowExt, Inhibit,
    Label, LabelExt, MessageType, WidgetExt, Window, WindowType,
};
use itertools_num::linspace;
use relm::DrawHandler;
use relm::{Relm, Update, Widget};
use structopt::StructOpt;

use curve_tracer::backend::RawTrace;
use curve_tracer::model::diode::diode_model;
use curve_tracer::model::diode::NullModel;
use curve_tracer::model::IVModel;
use curve_tracer::options::Opt;

struct Model {
    draw_handler: DrawHandler<DrawingArea>,
    raw_trace: RawTrace,
    device_model: Box<dyn IVModel>,
    opt: Opt,
}

struct ModelParam {
    opt: Opt,
}

#[derive(Msg)]
enum Msg {
    Trace,
    UpdateDrawBuffer,
    Quit,
}

#[derive(Clone)]
struct Widgets {
    trace_button: Button,
    drawing_area: DrawingArea,
    model_text: Label,
    window: Window,
}

struct Win {
    model: Model,
    widgets: Widgets,
}

impl Update for Win {
    type Model = Model;
    type ModelParam = ModelParam;
    type Msg = Msg;

    fn model(_: &Relm<Self>, param: ModelParam) -> Model {
        Model {
            draw_handler: DrawHandler::new().expect("draw handler"),
            raw_trace: RawTrace::default(),
            device_model: Box::new(NullModel {}),
            opt: param.opt,
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::Trace => match self.model.opt.device().trace() {
                Ok(trace) => {
                    self.model.raw_trace = trace;

                    let model = diode_model(&self.model.raw_trace);
                    self.model.device_model = Box::new(model);
                    self.widgets
                        .model_text
                        .set_markup(&format!("{}", &self.model.device_model));

                    self.widgets.drawing_area.queue_resize();
                }
                Err(err) => {
                    let error_msg = gtk::MessageDialog::new(
                        Some(&self.widgets.window),
                        DialogFlags::MODAL,
                        MessageType::Error,
                        ButtonsType::Close,
                        &format!("Error: {}", err),
                    );

                    error_msg.run();
                    error_msg.close();
                }
            },
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

                let max_i = 0.05;
                let max_v = 0.5;

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

                cr.set_source_rgba(0.0, 0.0, 0.0, 0.05);
                for (&i, &v) in self
                    .model
                    .raw_trace
                    .current
                    .iter()
                    .zip(self.model.raw_trace.voltage.iter())
                {
                    cr.arc(
                        v * v_factor,
                        height - 20.0 - i * i_factor,
                        1.0,
                        0.0,
                        PI * 2.0,
                    );
                    cr.fill();
                }

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
                        let text = format!("{:.3}V", v_gridline);
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

                cr.set_source_rgba(1.0, 0.0, 0.0, 0.8);

                for (ix, v) in linspace(0.0, max_v, 101).enumerate() {
                    if ix == 0 {
                        cr.move_to(
                            v * v_factor,
                            height - 20.0 - i_factor * self.model.device_model.evaluate(v),
                        );
                    } else {
                        cr.line_to(
                            v * v_factor,
                            height - 20.0 - i_factor * self.model.device_model.evaluate(v),
                        );
                    }
                }
                cr.stroke();
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
        let hbox = gtk::Box::new(Horizontal, 0);

        let right_pane = gtk::Box::new(Vertical, 0);
        right_pane.set_size_request(300, 500);
        right_pane.set_margin_top(8);
        right_pane.set_margin_bottom(8);
        right_pane.set_margin_start(4);
        right_pane.set_margin_end(4);

        let help_text = gtk::Label::new("");
        help_text.set_xalign(0.0);
        help_text.set_margin_top(8);
        help_text.set_margin_bottom(8);
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

        let trace_button = Button::new_with_label("Trace");
        right_pane.add(&trace_button);

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
                window,
            },
        }
    }
}

fn main() {
    let opt = Opt::from_args();

    Win::run(ModelParam { opt }).unwrap();
}
