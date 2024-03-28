use reaboot_core::{PreparationReport, PreparationReportAsMarkdown};

pub fn print_report(report: PreparationReport, packages_have_been_installed: bool) {
    let markdown =
        PreparationReportAsMarkdown::new(&report, packages_have_been_installed).to_string();
    termimad::print_text(&markdown);
}
