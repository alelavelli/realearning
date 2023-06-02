use crate::model::registry::Registry;
use crate::plots::extraction::monthy_extraction;
use plotters::prelude::*;

use super::extraction::{extract_categories_split, extract_daily_transactions};
use super::plot_utils::palettes::Palette;

pub fn plot_daily_transactions(
    registry: &Registry,
    resolution: (u32, u32),
    folder: &str,
    palette: &Palette,
) -> Result<(), Box<dyn std::error::Error>> {
    let figure_path = format!("{folder}/daily_transactions.png");

    let account_vec = vec![String::from("Ale"), String::from("Giulia")];
    let daily_transactions =
        extract_daily_transactions(registry, Some(&account_vec), None, true).unwrap();

    let colors = palette.colors;

    // Create the root drawing area
    let root = BitMapBackend::new(&figure_path, resolution).into_drawing_area();
    root.fill(&palette.background)?;
    let root = root.titled("Daily transactions", ("sans-serif", 30))?;
    let (upper, lower) = root.split_vertically(resolution.1 / 2);

    //let root = root.margin(10, 10, 10, 10);
    // After this point, we should be able to draw construct a chart context
    let mut upper_chart = ChartBuilder::on(&upper)
        .x_label_area_size(50)
        .y_label_area_size(50)
        .margin_left(30)
        .margin_right(30)
        .caption("timeseries", ("sans-serif", 20))
        .build_cartesian_2d(
            (daily_transactions.days_idx_range.0..(daily_transactions.days_idx_range.1)).step(1.0),
            (daily_transactions.amounts_range.0..(daily_transactions.amounts_range.1)).step(500.0),
        )?;

    upper_chart
        .configure_mesh()
        .bold_line_style(ShapeStyle {
            color: palette.mesh,
            filled: false,
            stroke_width: 1,
        })
        .x_labels(30) // number of labels per axis
        .y_labels(20)
        .y_label_formatter(&|x| format!("{:.0}", x))
        .x_label_formatter(&|x| format!("{:.3}", daily_transactions.days.get(*x as usize).unwrap()))
        .y_desc("Euros")
        .x_desc("Days")
        .draw()?;
    upper_chart.draw_series(
        LineSeries::new(
            daily_transactions.days_idx.iter().map(|&x| (x, 0.0)).collect::<Vec<(f32, f32)>>(),
        ShapeStyle {
            color: RGBAColor(0, 0, 0, 1.0),
            filled: false,
            stroke_width: 1,
        }
    )
    ).unwrap();
    upper_chart.draw_series(
        LineSeries::new(
            daily_transactions.amounts_pairs,
            ShapeStyle {
                color: colors[0],
                filled: true,
                stroke_width: 2,
            },
        )
        .point_size(2),
    )?;
    

    let mut cumulative_chart = ChartBuilder::on(&lower)
        .caption("cumulative transactions", ("sans-serif", 20).into_font())
        .x_label_area_size(50)
        .y_label_area_size(50)
        .margin_left(30)
        .margin_right(30)
        .margin_bottom(20)
        .build_cartesian_2d(
            (daily_transactions.days_idx_range.0..(daily_transactions.days_idx_range.1)).step(1.0),
            (daily_transactions.cumsum_amounts_range.0
                ..(daily_transactions.cumsum_amounts_range.1))
                .step(1000.0),
        )?;

    cumulative_chart.draw_series(
        LineSeries::new(
            daily_transactions.amount_cumulative_pairs,
            ShapeStyle {
                color: colors[0],
                filled: true,
                stroke_width: 2,
            },
        )
        .point_size(2),
    )?;
    cumulative_chart
        .configure_mesh()
        .bold_line_style(ShapeStyle {
            color: palette.mesh,
            filled: false,
            stroke_width: 1,
        })
        .x_labels(30) // number of labels per axis
        .y_labels(20)
        .y_label_formatter(&|x| format!("{:.0}", x))
        .x_label_formatter(&|x| format!("{:.3}", daily_transactions.days.get(*x as usize).unwrap()))
        .y_desc("Euros")
        .x_desc("Days")
        .draw()?;

    root.present()?;

    Ok(())
}

pub fn plot_category_pie(
    registry: &Registry,
    resolution: (u32, u32),
    max_categories: usize,
    folder: &str,
    palette: &Palette,
) -> Result<(), Box<dyn std::error::Error>> {
    let account_vec = vec![String::from("Ale"), String::from("Giulia")];
    let categories_split =
        extract_categories_split(registry, Some(&account_vec), None, Some(max_categories)).unwrap();

    let figure_path = format!("{folder}/transaction_pie.png");

    let root_area = BitMapBackend::new(&figure_path, resolution).into_drawing_area();
    root_area.fill(&WHITE).unwrap();
    let title_style = TextStyle::from(("sans-serif", 30).into_font()).color(&(BLACK));
    root_area
        .titled("Categories Pie Chart", title_style)
        .unwrap();
    let (left, right) = root_area.split_horizontally(resolution.0 / 2);
    left.titled("Expenses", ("sans-serif", 20).into_font())?;
    right.titled("Entries", ("sans-serif", 20).into_font())?;

    // Expenses
    let dims = left.dim_in_pixel();
    let center = (dims.0 as i32 / 2, dims.1 as i32 / 2);
    let radius = 250.0;
    let colors: Vec<RGBColor> = (0..categories_split.expense_categories.len())
        .map(|x| {
            let (r, g, b) = palette.colors[x].rgb();
            RGBColor(r, g, b)
        })
        .collect();

    let mut pie = Pie::new(
        &center,
        &radius,
        &categories_split.expense_percentages,
        &colors,
        &categories_split.expense_categories,
    );
    pie.start_angle(66.0);
    pie.label_style((("sans-serif", 20).into_font()).color(&(BLACK)));
    pie.percentages((("sans-serif", radius * 0.08).into_font()).color(&BLACK));
    left.draw(&pie)?;

    // Entries
    let dims = right.dim_in_pixel();
    let center = (
        dims.0 as i32 / 2 + resolution.0 as i32 / 2,
        dims.1 as i32 / 2,
    );
    let colors: Vec<RGBColor> = (0..categories_split.income_categories.len())
        .map(|x| {
            let (r, g, b) = palette.colors[x].rgb();
            RGBColor(r, g, b)
        })
        .collect();

    let mut pie = Pie::new(
        &center,
        &radius,
        &categories_split.income_percentages,
        &colors,
        &categories_split.income_categories,
    );
    pie.start_angle(66.0);
    pie.label_style((("sans-serif", 20).into_font()).color(&(BLACK)));
    pie.percentages((("sans-serif", radius * 0.08).into_font()).color(&BLACK));
    right.draw(&pie)?;
    Ok(())
}

pub fn plot_monthly_report(
    registry: &Registry,
    resolution: (u32, u32),
    max_categories: Option<usize>,
    folder: &str,
    palette: &Palette,
) -> Result<(), Box<dyn std::error::Error>> {
    let account_vec = vec![String::from("Ale"), String::from("Giulia")];
    let monthly_extraction = monthy_extraction(registry, Some(&account_vec), None, max_categories)?;

    let figure_path = format!("{folder}/monthly_net_ts.png");
    let colors = palette.colors;
    let root_area = BitMapBackend::new(&figure_path, resolution).into_drawing_area();
    root_area.fill(&WHITE).unwrap();
    root_area.titled("Monthly Plots", ("sans-serif", 30))?;

    let (upper, mid) = root_area.split_vertically(resolution.1 / 2);

    // UPPER
    let mut upper_chart = ChartBuilder::on(&upper)
        .x_label_area_size(50)
        .y_label_area_size(50)
        .margin_left(30)
        .margin_right(30)
        .margin_top(50)
        .caption("monthly net income", ("sans-serif", 20))
        .build_cartesian_2d(
            (monthly_extraction.months_idx_range.0..(monthly_extraction.months_idx_range.1))
                .step(1.0),
            (monthly_extraction.net_income_range.0..(monthly_extraction.net_income_range.1))
                .step(100.0),
        )?;

    upper_chart
        .configure_mesh()
        .x_labels(monthly_extraction.months_idx.len()) // number of labels per axis
        .y_labels(20)
        .y_label_formatter(&|x| format!("{:.0}", x))
        .x_label_formatter(&|x| format!("{}", monthly_extraction.months.get(*x as usize).unwrap()))
        .y_desc("Euros")
        .x_desc("Months")
        .draw()?;
    upper_chart.draw_series(
        LineSeries::new(
            monthly_extraction.net_income_pairs,
            ShapeStyle {
                color: colors[0],
                filled: true,
                stroke_width: 2,
            },
        )
        .point_size(3),
    )?;

    upper_chart.draw_series(
        LineSeries::new(
            monthly_extraction.months_idx.iter().map(|&x| (x, 0.0)).collect::<Vec<(f32, f32)>>(),
        ShapeStyle {
            color: RGBAColor(0, 0, 0, 1.0),
            filled: true,
            stroke_width: 2,
        }
    )
    ).unwrap();

    // MID
    let mut mid_chart = ChartBuilder::on(&mid)
        .x_label_area_size(50)
        .y_label_area_size(50)
        .margin_left(30)
        .margin_right(30)
        .caption("monthly spend for each category", ("sans-serif", 20))
        .build_cartesian_2d(
            (monthly_extraction.categories_months_idx_range.0
                ..(monthly_extraction.categories_months_idx_range.1))
                .step(1.0),
            (monthly_extraction.categories_amounts_range.0
                ..(monthly_extraction.categories_amounts_range.1))
                .step(100.0),
        )?;

    mid_chart
        .configure_mesh()
        .x_labels(5) // number of labels per axis
        .y_labels(30)
        //.y_label_formatter(&|x| format!("{:.0}", 10.0.pow(x))) logarithmic
        .y_label_formatter(&|x| format!("{:.0}", x))
        .x_label_formatter(&|x| {
            format!("{:.3}", monthly_extraction.months.get(*x as usize).unwrap())
        })
        .y_desc("Euros")
        .x_desc("Month")
        .draw()?;

    for (i, category) in monthly_extraction.categories.iter().enumerate() {
        let pairs = monthly_extraction.categories_pairs.get(i).unwrap().clone();
        mid_chart
            .draw_series(
                LineSeries::new(
                    pairs,
                    ShapeStyle {
                        color: colors[i],
                        filled: true,
                        stroke_width: 2,
                    },
                )
                .point_size(3),
            )
            .unwrap()
            .label(category)
            .legend(move |(x, y)| {
                PathElement::new(
                    vec![(x, y), (x + 20, y)],
                    ShapeStyle {
                        color: colors[i],
                        filled: true,
                        stroke_width: 2,
                    },
                )
            });
    }
    mid_chart
        .configure_series_labels()
        .border_style(BLACK)
        .background_style(WHITE.mix(0.8))
        .position(SeriesLabelPosition::Coordinate(
            (resolution.1 as f32 * 0.75) as i32,
            (resolution.0 as f32 * 0.02) as i32,
        ))
        .draw()
        .unwrap();

    root_area.present()?;

    let figure_path = format!("{folder}/monthly_category_pies.png");

    let root_area = BitMapBackend::new(&figure_path, resolution).into_drawing_area();
    root_area.fill(&WHITE).unwrap();
    //root_area.titled("Monthly Pies", ("sans-serif", 30))?;
    let n_months = monthly_extraction.months.len();
    let cols = 3;
    let rows = (n_months as f32 / cols as f32).ceil() as usize;
    let drawing_areas = root_area.split_evenly((rows, cols));
    let colors: Vec<RGBColor> = (0..monthly_extraction.categories.len())
        .map(|x| {
            let (r, g, b) = palette.colors[x].rgb();
            RGBColor(r, g, b)
        })
        .collect();

    for (i, month) in monthly_extraction
        .categories_amounts_perc_months
        .iter()
        .enumerate()
    {
        let da = drawing_areas.get(i).unwrap();
        da.titled(&month.to_string(), ("sans-serif", 20))?;

        let dims = da.dim_in_pixel();

        let base_pixel = da.get_base_pixel();

        let center = (
            (base_pixel.0 + dims.0 as i32 / 2),
            (base_pixel.1 + dims.1 as i32 / 2),
        );

        let radius: f64 = (dims.0 / 4) as f64;
        let mut labels: Vec<String> = Vec::new();
        for (j, category_name) in monthly_extraction
            .categories_amounts_perc_names
            .get(i)
            .unwrap()
            .iter()
            .enumerate()
        {
            let mut label = category_name.clone();
            //label.push_str(&monthly_extraction.categories_amounts_perc_value.get(i).unwrap().get(j).unwrap().to_string());
            label.push_str(&format!(
                "{:.0}",
                &monthly_extraction
                    .categories_amounts_perc_value
                    .get(i)
                    .unwrap()
                    .get(j)
                    .unwrap()
                    .abs()
            ));
            labels.push(label);
        }

        let mut pie = Pie::new(
            &center,
            &radius,
            monthly_extraction.categories_amounts_perc.get(i).unwrap(),
            &colors,
            &labels, //monthly_extraction.categories_amounts_perc_names.get(i).unwrap()
        );

        pie.start_angle(66.0);
        pie.label_style((("sans-serif", 20).into_font()).color(&(BLACK)));
        pie.percentages((("sans-serif", radius * 0.08).into_font()).color(&BLACK));
        da.draw(&pie)?;
    }

    root_area.present()?;
    Ok(())
}
