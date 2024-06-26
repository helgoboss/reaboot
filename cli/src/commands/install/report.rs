use reaboot_core::{
    PreparationReport, PreparationReportAsMarkdown, PreparationReportMarkdownOptions,
};

pub fn print_report(report: &PreparationReport, actually_installed_things: bool) {
    let opts = PreparationReportMarkdownOptions {
        include_main_heading: true,
        include_donation_links: false,
        actually_installed_things,
        optimize_for_termimad: true,
    };
    let markdown = PreparationReportAsMarkdown::new(report, opts).to_string();
    termimad::print_text(&markdown);
}
