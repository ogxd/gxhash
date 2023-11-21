#[cfg(feature = "bench-csv")]
#[derive(Default)]
pub struct ResultProcessor {
    header_written: bool,
    row_data: Vec<String>,
    header_data: Vec<String>,
}

#[cfg(feature = "bench-csv")]
impl ResultProcessor {
    pub fn on_start(&mut self, name: &str) {
        self.row_data.push(name.to_string());
        if !self.header_written {
            self.header_data.push("Throughput (MiB/s)".to_string());
        }
    }

    pub fn on_result(&mut self, input_size: usize, throughput: f64) {
        self.row_data.push(format!("{:.2}", throughput));
        if !self.header_written {
            self.header_data.push(input_size.to_string());
        }
    }

    pub fn on_end(&mut self) {
        if !self.header_written {
            println!("{}", self.header_data.join(", "));
            self.header_written = true;
        }
        println!("{}", self.row_data.join(", "));
        self.row_data.clear();
    }
}

#[cfg(feature = "bench-md")]
#[derive(Default)]
pub struct ResultProcessor {
    header_written: bool,
    row_data: Vec<String>,
    header_data: Vec<String>,
}

#[cfg(feature = "bench-md")]
impl ResultProcessor {
    pub fn on_start(&mut self, name: &str) {
        self.row_data.push(name.to_string());
        if !self.header_written {
            self.header_data.push("Throughput (MiB/s)".to_string());
        }
    }

    pub fn on_result(&mut self, input_size: usize, throughput: f64) {
        self.row_data.push(format!("{:.2}", throughput));
        if !self.header_written {
            self.header_data.push(input_size.to_string());
        }
    }

    pub fn on_end(&mut self) {
        if !self.header_written {
            println!("| {} |", self.header_data.join(" | "));
            let separator: Vec<String> = self.header_data.iter().map(|_| "---".to_string()).collect();
            println!("|{}|", separator.join("|"));
            self.header_written = true;
        }
        println!("| {} |", self.row_data.join(" | "));
        self.row_data.clear();
    }
}

#[cfg(all(not(feature = "bench-csv"), not(feature = "bench-md")))]
#[derive(Default)]
pub struct ResultProcessor;

#[cfg(all(not(feature = "bench-csv"), not(feature = "bench-md")))]
impl ResultProcessor {
    pub fn on_start(&mut self, name: &str) {
        println!("{}", name);
    }

    pub fn on_result(&mut self, input_size: usize, throughput: f64) {
        println!("  | {} > {:.2}", input_size, throughput);
    }

    pub fn on_end(&mut self) {
        println!();
    }
}