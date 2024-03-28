use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};
use reaboot_core::api::InstallationStage;
use reaboot_core::installer::{InstallerListener, InstallerTask};
use std::collections::HashMap;
use std::fmt::Display;
use std::sync::RwLock;

pub struct CliInstallerListener {
    multi_progress: MultiProgress,
    main_progress_bar: ProgressBar,
    task_progress_bars: RwLock<HashMap<u32, ProgressBar>>,
}

impl CliInstallerListener {
    pub fn new() -> Self {
        let multi_progress = MultiProgress::new();
        let main_progress_bar = multi_progress.add(create_main_progress_bar());
        Self {
            multi_progress,
            main_progress_bar,
            task_progress_bars: Default::default(),
        }
    }

    fn log(&self, msg: impl Display) {
        // let _ = self.multi_progress.println(msg.to_string());
    }
}

impl InstallerListener for CliInstallerListener {
    fn installation_stage_changed(&self, event: InstallationStage) {
        self.main_progress_bar.reset();
        self.main_progress_bar.set_message(event.to_string());
    }

    fn installation_stage_progressed(&self, progress: f64) {
        self.main_progress_bar
            .set_position(convert_progress(progress));
    }

    fn task_started(&self, task_id: u32, task: InstallerTask) {
        let pb = self
            .multi_progress
            .add(create_task_progress_bar(task_id, task));
        pb.tick();
        self.task_progress_bars.write().unwrap().insert(task_id, pb);
    }

    fn task_progressed(&self, task_id: u32, progress: f64) {
        let map = self.task_progress_bars.read().unwrap();
        if let Some(pb) = map.get(&task_id) {
            pb.set_position(convert_progress(progress));
        }
    }

    fn task_finished(&self, task_id: u32) {
        // If we wanted the bar to stay on screen, we would use `get` instead and
        // call `finish()` on the progress bar instead of removing it.
        if let Some(pb) = self.task_progress_bars.write().unwrap().remove(&task_id) {
            self.multi_progress.remove(&pb);
        }
    }

    fn warn(&self, message: impl Display) {
        self.log(message);
    }

    fn info(&self, message: impl Display) {
        self.log(message);
    }

    fn debug(&self, message: impl Display) {
        self.log(message);
    }
}

fn create_main_progress_bar() -> ProgressBar {
    let pb = ProgressBar::with_draw_target(Some(100), ProgressDrawTarget::hidden());
    pb.set_draw_target(ProgressDrawTarget::hidden());
    pb.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        )
        .unwrap()
        .progress_chars("##-"),
    );
    pb
}

fn create_task_progress_bar(task_id: u32, task: InstallerTask) -> ProgressBar {
    // When not creating the progress bar in hidden state, we will get many duplicate lines.
    // I don't know exactly why.
    let pb = ProgressBar::with_draw_target(Some(100), ProgressDrawTarget::hidden());
    pb.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.red/green} {pos:>7}/{len:7} {msg}",
        )
        .unwrap()
        .progress_chars("##-"),
    );
    pb.set_message(task.label);
    pb
}

fn convert_progress(progress: f64) -> u64 {
    (progress * 100.0).round() as u64
}
