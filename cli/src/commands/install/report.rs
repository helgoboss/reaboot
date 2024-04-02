use reaboot_core::{
    PreparationReport, PreparationReportAsMarkdown, PreparationReportMarkdownOptions,
};

pub fn print_report(report: PreparationReport, packages_have_been_installed: bool) {
    let opts = PreparationReportMarkdownOptions {
        packages_have_been_installed,
        optimize_for_termimad: true,
    };
    let markdown = PreparationReportAsMarkdown::new(&report, opts).to_string();
    termimad::print_text(&markdown);
}
