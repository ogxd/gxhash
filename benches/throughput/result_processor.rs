use plotters::prelude::*;

// For each algorithm, its throughput in MiB/s for each input size
pub type Results = Vec<(String, Vec<(usize, f64)>)>;

pub fn print_csv(results: &Results) {
    let sizes: Vec<String> = results[0].1.iter().map(|(size, _)| size.to_string()).collect();
    println!("Throughput (MiB/s), {}", sizes.join(", "));
    for (name, values) in results {
        let values: Vec<String> = values.iter().map(|(_, throughput)| format!("{throughput:.2}")).collect();
        println!("{}, {}", name, values.join(", "));
    }
}

// One row per input size, in GiB/s, with the fastest in bold, followed by the throughput of GxHash divided by the one
// of the fastest other algorithm
pub fn print_md(results: &Results) {
    let names: Vec<&str> = results.iter().map(|(name, _)| name.as_str()).collect();
    let gxhash = names.iter().position(|&name| name == "GxHash").unwrap();
    println!("| Throughput (GiB/s) | {} | GxHash speedup |", names.join(" | "));
    println!("|---|{}---:|", "---:|".repeat(names.len()));
    for (i, (size, _)) in results[0].1.iter().enumerate() {
        let row: Vec<f64> = results.iter().map(|(_, values)| values[i].1 / 1024.0).collect();
        let best = row.iter().copied().fold(0.0, f64::max);
        let best_other = row.iter().enumerate().filter(|&(j, _)| j != gxhash).map(|(_, &t)| t).fold(0.0, f64::max);
        let cells: Vec<String> = row.iter().map(|&t| if t == best { format!("**{t:.2}**") } else { format!("{t:.2}") }).collect();
        println!("| {} B | {} | {:.2}× |", size, cells.join(" | "), row[gxhash] / best_other);
    }
}

pub fn plot(results: &Results) {
    let arch = std::env::consts::ARCH;
    let file_name = format!("benches/throughput/{}.svg", arch);

    let canvas = SVGBackend::new(file_name.as_str(), (600, 400)).into_drawing_area();
    canvas.fill(&WHITE).unwrap();

    let x_min = results.iter().next().unwrap().1.iter().map(|(x, _)| *x as u32).min().unwrap();
    let x_max = results.iter().next().unwrap().1.iter().map(|(x, _)| *x as u32).max().unwrap();

    let y_min = results.iter().flat_map(|inner_map| inner_map.1.iter()).map(|(_, y)| (0.95 * *y) as u32).min().unwrap();
    let y_max = results.iter().flat_map(|inner_map| inner_map.1.iter()).map(|(_, y)| (1.05 * *y) as u32).max().unwrap();

    let mut chart = ChartBuilder::on(&canvas)
        .caption(format!("Throughput ({})", arch), ("sans-serif", (5).percent_height()))
        .set_label_area_size(LabelAreaPosition::Left, (14).percent())
        .set_label_area_size(LabelAreaPosition::Bottom, (10).percent())
        .margin_right((5).percent())
        .build_cartesian_2d(
            (x_min..x_max)
                .log_scale()
                .with_key_points(results.iter().next().unwrap().1.iter().map(|(x, _)| *x as u32).collect::<Vec<u32>>()),
                (y_min..y_max)
                .log_scale(),
        ).unwrap();

    chart
        .configure_mesh()
        .x_max_light_lines(1)
        .y_max_light_lines(2)
        .y_labels(10)
        .x_desc("Input Size (bytes)")
        .y_desc("Throughput (MiB/s)")
        .draw().unwrap();

    let mut color_idx = 0;
    for (name, values) in results.iter() {
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

    eprintln!("Finished: '{}'", file_name);
}
