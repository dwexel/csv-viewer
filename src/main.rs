#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release


use eframe::egui;
use csv::{self, StringRecord};
use egui::Color32;
use std::{
    sync::Arc
};
use nucleo::{
    Config, 
    Nucleo, 
    pattern::CaseMatching,
    pattern::Normalization
};


// i have the feeling that the indices for individual values should be stored in hash maps

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([500.0, 600.0]),
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

    num_rows: usize,
    // scroll_to_row_slider: usize,
    // scroll_to_row: Option<usize>,
    
    //selection: std::collections::HashSet<usize>,
    
    checked: bool,

    // records: Vec<StringRecord>,

    headers: Option<StringRecord>,

    input_buffer: Vec<String>,


    // matcher: Matcher,

    // indices to include 
    // idk if it makes sense to do it this way
    // match_indices: Vec<usize>,

    // remember what is matched for each column
    // match_indices_individual: Vec<Vec<usize>>,

    
    nucleo: Nucleo<StringRecord>,
    running: bool
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: "Arthur".to_owned(),
            age: 42,
            
            striped: false,
            resizable: false,
            clickable: false,

            checked: false,       

            // scroll_to_row_slider: 0,
            // scroll_to_row: None,
            
            num_rows: 0,
            headers: None,

            input_buffer: vec!["".to_owned(); 3],

            // 2 cols.
            // only using this code once
            // so far
            nucleo: Nucleo::new(Config::DEFAULT, Arc::new(||{}), None, 3),
            running: false
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // should find some way to cap framerate
        // if it helps
        self.running = self.nucleo.tick(10).running;

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

            let color = match self.running {true => Color32::GREEN, false => Color32::RED};

            ui.colored_label(color, "xxx");

            // ui.image(egui::include_image!(
            //     "../../../crates/egui/assets/ferris.png"
            // ));

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
            });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        println!("exiting...");
        // let n = self.nucleo
        //     .snapshot()
        //     .matched_item_count()
        //     .clamp(0, 100);

        // for item in self.nucleo.snapshot().matched_items(0..n) {
        //     println!("{:?}", item.data);
        // }
    }
}

impl MyApp {
    fn from_path(p: &str) -> Self {
        let mut new = Self::default();

        // get csv
        let mut rdr = csv::Reader::from_path(p).unwrap();

        let headers = match rdr.headers() {
            Ok(h) => Some(h.clone()),
            Err(_) => None
        };

        let inj = new.nucleo.injector();
        let mut n = 0;
        
        for record in rdr.into_records() {
            let rec = record.unwrap();
            n += 1;
            inj.push(rec, |value, columns| {
                columns[0] = value.get(0).expect("damn").into();
                columns[1] = value.get(1).expect("damn").into();
                columns[2] = value.get(2).expect("damn").into();
            });
        }

        new.num_rows = n;
        new.headers = headers;
        new
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
                body.rows(text_height, self.num_rows, |mut row| {
                    let row_index = row.index() as u32;
                    if row_index == 0 {
                        // spacers
                        row.col(|ui| { ui.add_space(ui.available_width()) });
                        row.col(|ui| { ui.add_space(ui.available_width()) });
                        row.col(|ui| {
                            let te = ui.text_edit_singleline(&mut self.input_buffer[0]);
                            if te.clicked() {}
                            if te.changed() { 
                                // find a way to check the type...
                                // if the buffer's been purely added to, 
                                // or deleted from or whatev
                                self.nucleo.pattern
                                    .reparse(0, self.input_buffer[0].trim(), CaseMatching::Ignore, Normalization::Smart, false);
                                // double tick
                                // on this frame
                                self.nucleo.tick(10);
                            }
                        });

                        row.col(|ui| {
                            let te = ui.text_edit_singleline(&mut self.input_buffer[1]);
                            if te.clicked() {}
                            if te.changed() {
                                self.nucleo.pattern
                                    .reparse(1, self.input_buffer[1].trim(), CaseMatching::Ignore, Normalization::Smart, false);
                                self.nucleo.tick(10);
                            }
                        });
                        row.col(|ui| {
                            let te = ui.text_edit_singleline(&mut self.input_buffer[2]);
                            if te.clicked() {}
                            if te.changed() {
                                self.nucleo.pattern
                                    .reparse(2, self.input_buffer[2].trim(), CaseMatching::Ignore, Normalization::Smart, false);
                                self.nucleo.tick(10);
                            }
                        });

                        return;
                    }

                    let res = match self.nucleo.snapshot().get_matched_item(row_index - 1) {
                        None => return,
                        Some(_res) => _res
                    };

                    // i think i can use either the "data" or "matcher columns"
                    // from res
                    // i should check to make sure they're the same

                    // hm
                    // row.set_selected(self.selection.contains(&row_index));

                    row.col(|ui| {
                        ui.label(row_index.to_string());
                    });
                    row.col(|ui| {
                        ui.checkbox(&mut self.checked, "Click me");
                    });
                    row.col(|ui| {
                        ui.label(res.data.get(0).expect("no value in col 0 (weird)"));
                    });
                    row.col(|ui| {
                        ui.label(res.data.get(1).expect("no value in col 1 (weird)"));
                    });
                    row.col(|ui| {
                        ui.label(res.data.get(2).expect("no value in col 2 (weird)"));
                    });


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



    // note: column is a number that selects a coumn
    // todo: use the sorting and scoring functions

    // fn search_column(&mut self, column: usize) {
    //     let needle = self.input_storage[column].as_str();

    //     let mut temp = Vec::<char>::new();
    //     let mut temp2 = Vec::<char>::new();
    //     let n = Utf32Str::new(needle, &mut temp2);


    //     self.match_indices.clear(); 

    //     for (i, rec) in self.records.iter().enumerate() {
    //         // note: panics?
    //         let haystack = &rec[column];

    //         let h = Utf32Str::new(haystack, &mut temp);
   
    //         if let Some(_) = self.matcher.substring_match(h, n) {
    //             self.match_indices.push(i);
    //         }
    //     }
    // }
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

// fn intersection(nums: Vec<Vec<usize>>) -> Vec<usize> {
//     let mut result = nums[0].clone();

//     // ??
//     for i in 1..nums.len() {
//         result = nums[i]
//             .iter()
//             .filter(|num| {result.contains(*num)})
//             .map(|n| *n)
//             .collect();
//     }

//     result
// }


// fn intersection_a(nums: &Vec<Vec<usize>>) -> Vec<usize> {
//     let mut result = nums[0].clone();

//     // ??
//     for i in 1..nums.len() {
//         result = nums[i]
//             .iter()
//             .filter(|num| {result.contains(*num)})
//             .map(|n| *n)
//             .collect();
//     }

//     result
// }
