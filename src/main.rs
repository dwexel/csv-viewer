#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release


use eframe::egui;
use csv::{self, StringRecord};
use std::{default, error::Error, io, process};




fn main() -> Result<(), eframe::Error> {


    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| {
            // This gives us image support:
            // egui_extras::install_image_loaders(&cc.egui_ctx);


            let app = MyApp::from_path("src/test.csv");
            Box::<MyApp>::new(app)
        }),
    )
}

struct MyApp {
    name: String,
    age: u32,

    striped: bool,
    resizable: bool,
    clickable: bool,

    // num_rows: usize,
    // scroll_to_row_slider: usize,
    // scroll_to_row: Option<usize>,
    selection: std::collections::HashSet<usize>,
    checked: bool,

    records: Vec<StringRecord>,
    headers: Option<StringRecord>
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: "Arthur".to_owned(),
            age: 42,
            
            striped: false,
            resizable: false,
            clickable: false,

            // num_rows: 10_000,
            // scroll_to_row_slider: 0,
            // scroll_to_row: None,
            selection: Default::default(),
            checked: false,       

            records: Vec::new(),
            headers: None
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {

            ui.heading("My egui Application");

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.striped, "Striped");
                    ui.checkbox(&mut self.resizable, "Resizable columns");
                    ui.checkbox(&mut self.clickable, "Clickable rows");
                });
            });
            
            ui.horizontal(|ui| {
                let name_label = ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.name).labelled_by(name_label.id);
            });
            
            ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));

            if ui.button("Increment").clicked() {
                self.age += 1;
            }
            
            ui.label(format!("Hello '{}', age {}", self.name, self.age));

            // ui.image(egui::include_image!(
            //     "../../../crates/egui/assets/ferris.png"
            // ));


            // conditionally build table
            if self.records.len() > 0 {
                use egui_extras::{Size, StripBuilder};

                StripBuilder::new(ui)
                    .size(Size::remainder().at_least(100.0)) // for the table
                    .vertical(|mut strip| {
                        strip.cell(|ui| {
                            egui::ScrollArea::horizontal().show(ui, |ui| {
                                self.table_ui(ui);
                            });
                        });
                    });
    
            }
        });
    }
}

impl MyApp {
    fn from_path(p: &str) -> Self {
        let mut rdr = csv::Reader::from_path(p).unwrap();
        let headers = rdr.headers().unwrap().clone();


        let mut records = Vec::new();
        for record in rdr.into_records() {
            records.push(record.unwrap());
        }


        Self {
            headers: Some(headers),
            records: records,
            ..Default::default()
        }
    }

    fn table_ui(&mut self, ui: &mut egui::Ui) {
        use egui_extras::{Column, TableBuilder};

        let text_height = egui::TextStyle::Body
            .resolve(ui.style())
            .size
            .max(ui.spacing().interact_size.y);

        let mut table = TableBuilder::new(ui)
            .striped(self.striped)
            .resizable(self.resizable)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::initial(100.0).range(40.0..=300.0))
            .column(Column::initial(100.0).at_least(40.0).clip(true))
            .column(Column::remainder())
            .min_scrolled_height(0.0);

        if self.clickable {
            table = table.sense(egui::Sense::click());
        }

        table
            .header(20.0, |mut header| {
                if let Some(headers) = &self.headers {
                    header.col(|ui| {
                        ui.strong("Row");
                    });
                    header.col(|ui| {
                        ui.strong("Interaction");
                    });
                    header.col(|ui| {
                        ui.strong(headers.get(0).expect("col 0"));
                    });
                    header.col(|ui| {
                        ui.strong(headers.get(1).expect("col 1"));
                    });
                    header.col(|ui| {
                        ui.strong(headers.get(2).expect("col 2"));
                    });
                            
    
                }
            })
            .body(|body| {
                // temporary 10 rows
                body.rows(text_height, 10, |mut row| {
                    let row_index = row.index();
                    row.set_selected(self.selection.contains(&row_index));

                    row.col(|ui| {
                        ui.label(row_index.to_string());
                    });
                    row.col(|ui| {
                        ui.checkbox(&mut self.checked, "Click me");
                    });

                    if let Some(rec) = self.records.get(row_index) {
                        row.col(|ui| {
                            ui.label(rec.get(0).expect("no value in col 0"));
                        });
                        row.col(|ui| {
                            ui.label(rec.get(1).expect("no value in col 0"));
                        });
                        row.col(|ui| {
                            ui.label(rec.get(2).expect("no value in col 0"));
                        });
                    }


                    // row.col(|ui| {
                    //     expanding_content(ui);
                    // });
                    // row.col(|ui| {
                    //     ui.label(long_text(row_index));
                    // });
                    // row.col(|ui| {
                    //     ui.add(
                    //         egui::Label::new("Thousands of rows of even height").wrap(false),
                    //     );
                    // });
                
                });


            });
    }
}

fn expanding_content(ui: &mut egui::Ui) {
    let width = ui.available_width().clamp(20.0, 200.0);
    let height = ui.available_height();
    let (rect, _response) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    ui.painter().hline(
        rect.x_range(),
        rect.center().y,
        (1.0, ui.visuals().text_color()),
    );
}

fn long_text(row_index: usize) -> String {
    format!("Row {row_index} has some long text that you may want to clip, or it will take up too much horizontal space!")
}