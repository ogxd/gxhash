use plotters::prelude::*;

pub trait ResultProcessor {
    fn on_start(&mut self, name: &str);
    fn on_result(&mut self, input_size: usize, throughput: f64);
    fn on_end(&mut self);
    fn finish(&self);
}

#[derive(Default)]
pub struct OutputCsv {
    header_written: bool,
    row_data: Vec<String>,
    header_data: Vec<String>,
}

impl ResultProcessor for OutputCsv {
    fn on_start(&mut self, name: &str) {
        self.row_data.push(name.to_string());
        if !self.header_written {
            self.header_data.push("Throughput (MiB/s)".to_string());
        }
    }

    fn on_result(&mut self, input_size: usize, throughput: f64) {
        self.row_data.push(format!("{:.2}", throughput));
        if !self.header_written {
            self.header_data.push(input_size.to_string());
        }
    }

    fn on_end(&mut self) {
        if !self.header_written {
            println!("{}", self.header_data.join(", "));
            self.header_written = true;
        }
        println!("{}", self.row_data.join(", "));
        self.row_data.clear();
    }

    fn finish(&self) {}
}

#[derive(Default)]
pub struct OutputMd {
    header_written: bool,
    row_data: Vec<String>,
    header_data: Vec<String>,
}

impl ResultProcessor for OutputMd {
    fn on_start(&mut self, name: &str) {
        self.row_data.push(name.to_string());
        if !self.header_written {
            self.header_data.push("Throughput (MiB/s)".to_string());
        }
    }

    fn on_result(&mut self, input_size: usize, throughput: f64) {
        self.row_data.push(format!("{:.2}", throughput));
        if !self.header_written {
            self.header_data.push(input_size.to_string());
        }
    }

    fn on_end(&mut self) {
        if !self.header_written {
            println!("| {} |", self.header_data.join(" | "));
            let separator: Vec<String> = self.header_data.iter().map(|_| "---".to_string()).collect();
            println!("|{}|", separator.join("|"));
            self.header_written = true;
        }
        println!("| {} |", self.row_data.join(" | "));
        self.row_data.clear();
    }

    fn finish(&self) {}
}

#[derive(Default)]
pub struct OutputSimple;

impl ResultProcessor for OutputSimple {
    fn on_start(&mut self, name: &str) {
        println!("{}", name);
    }

    fn on_result(&mut self, input_size: usize, throughput: f64) {
        println!("  | {} > {:.2}", input_size, throughput);
    }

    fn on_end(&mut self) {
        println!();
    }

    fn finish(&self) {
        println!("Finished");
    }
}

#[derive(Default)]
pub struct OutputPlot {
    series: Vec<(String, Vec<(usize, f64)>)>
}

impl ResultProcessor for OutputPlot {
    fn on_start(&mut self, name: &str) {
        println!("Started '{}'...", name);
        self.series.push((name.to_string(), Vec::new()));
    }

    fn on_result(&mut self, input_size: usize, throughput: f64) {
        let len = self.series.len();
        let serie = self.series.get_mut(len - 1).unwrap();
        
        serie.1.push((input_size, throughput));
    }

    fn on_end(&mut self) {}

    fn finish(&self) {
        let mut arch = std::env::consts::ARCH.to_string();
        if cfg!(feature = "hybrid") {
            println!("Hybrid feature enabled");
            arch += "-hybrid";
        }
        let file_name = format!("benches/throughput/{}.svg", arch);

        let canvas = SVGBackend::new(file_name.as_str(), (600, 400)).into_drawing_area();
        canvas.fill(&WHITE).unwrap();

        let x_min = self.series.iter().next().unwrap().1.iter().map(|(x, _)| *x as u32).min().unwrap();
        let x_max = self.series.iter().next().unwrap().1.iter().map(|(x, _)| *x as u32).max().unwrap();

        let y_min = 0u32;
        let y_max = self.series.iter().flat_map(|inner_map| inner_map.1.iter()).map(|(_, y)| (1.05 * *y) as u32).max().unwrap();

        let mut chart = ChartBuilder::on(&canvas)
            .caption(format!("Throughput ({})", arch), ("sans-serif", (5).percent_height()))
            .set_label_area_size(LabelAreaPosition::Left, (14).percent())
            .set_label_area_size(LabelAreaPosition::Bottom, (10).percent())
            .margin_right((5).percent())
            .build_cartesian_2d(
                (x_min..x_max)
                    .log_scale()
                    .with_key_points(self.series.iter().next().unwrap().1.iter().map(|(x, _)| *x as u32).collect::<Vec<u32>>()),
                    y_min..y_max
                    //.log_scale(),
            ).unwrap();

        chart
            .configure_mesh()
            .max_light_lines(1)
            .x_desc("Input Size (bytes)")
            .y_desc("Throughput (MiB/s)")
            .draw().unwrap();

        let mut color_idx = 0;
        for (name, values) in self.series.iter() {
            let color = Palette99::pick(color_idx);
            color_idx += 1;
            let data: Vec<_> = values.iter().map(|(x, y)| (*x as u32, *y as u32)).collect();
            chart
                .draw_series(LineSeries::new(data,
                    color.stroke_width(2),
                )).unwrap()
                .label(name)
                .legend(move |(x, y)| Rectangle::new([(x, y - 5), (x + 10, y + 5)], color.filled()));
        }

        chart
            .configure_series_labels()
            .border_style(BLACK)
            .background_style(RGBAColor(255, 255, 255, 0.7f64))
            .draw().unwrap();

        // To avoid the IO failure being ignored silently, we manually call the present function
        canvas.present().expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");

        println!("Finished: '{}'", file_name);
    }
}