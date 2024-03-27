use crate::api::MultiDownloadInfo;
use crate::downloader::Download;
use atomic::Atomic;
use bytemuck::NoUninit;
use std::sync::atomic::Ordering;
use std::sync::{Arc, OnceLock};

pub struct Task<L, P> {
    record: TaskRecordHandle<L>,
    pub payload: P,
}

pub fn track_tasks<P, L>(task_payloads: impl IntoIterator<Item = P>, listener: L) -> Vec<Task<L, P>>
where
    L: TaskTrackerListener,
{
    let tracker = TaskTracker {
        records: OnceLock::new(),
        listener,
    };
    let tracker = Arc::new(tracker);
    let (tasks, records): (Vec<_>, Vec<_>) = task_payloads
        .into_iter()
        .enumerate()
        .map(|(i, payload)| {
            let record = Arc::new(TaskRecord::new(i, tracker.clone()));
            let task = Task {
                record: record.clone(),
                payload,
            };
            (task, record)
        })
        .unzip();
    tracker.records.get_or_init(|| records);
    tasks
}

type TaskRecordHandle<L> = Arc<TaskRecord<L>>;

struct TaskTracker<L> {
    records: OnceLock<Vec<TaskRecordHandle<L>>>,
    listener: L,
}

struct TaskRecord<L> {
    index: usize,
    status: Atomic<TaskStatus>,
    progress: Atomic<f64>,
    tracker: Arc<TaskTracker<L>>,
}

impl<L> TaskRecord<L> {
    fn new(index: usize, tracker: Arc<TaskTracker<L>>) -> Self {
        Self {
            index,
            status: Default::default(),
            progress: Default::default(),
            tracker,
        }
    }
}

pub trait TaskTrackerListener {
    type Payload;

    fn summary_changed(&self, info: TaskSummary);
    fn total_progressed(&self, progress: f64);
    fn task_started(&self, task_index: usize, payload: &Self::Payload);
    fn task_progressed(&self, task_index: usize, progress: f64);
    fn task_finished(&self, task_index: usize);
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default, NoUninit)]
#[repr(u8)]
enum TaskStatus {
    #[default]
    Todo,
    InProgress,
    Error,
    Done,
}

pub struct TaskSummary {
    pub in_progress_count: u32,
    pub success_count: u32,
    pub error_count: u32,
    pub total_count: u32,
    pub total_progress: f64,
}

impl TaskSummary {
    fn new(total_count: u32) -> Self {
        Self {
            success_count: 0,
            error_count: 0,
            in_progress_count: 0,
            total_count,
            total_progress: 0.0,
        }
    }

    pub fn done(&self) -> bool {
        self.success_count + self.error_count == self.total_count
    }
}

impl<L> TaskTracker<L>
where
    L: TaskTrackerListener,
{
    fn update_summary(&self) {
        let summary = self.summary();
        let progress = summary.total_progress;
        self.listener.summary_changed(summary);
        self.listener.total_progressed(progress);
    }

    fn summary(&self) -> TaskSummary {
        let total_count = self.records().len() as u32;
        let mut progress_sum = 0.0;
        let mut summary = TaskSummary::new(total_count);
        if total_count == 0 {
            return summary;
        }
        for task in self.records() {
            progress_sum += task.progress();
            match task.status() {
                TaskStatus::Todo => {}
                TaskStatus::InProgress => {
                    summary.in_progress_count += 1;
                }
                TaskStatus::Error => {
                    summary.error_count += 1;
                }
                TaskStatus::Done => {
                    summary.success_count += 1;
                }
            }
        }
        summary.total_progress = progress_sum / total_count as f64;
        summary
    }

    fn records(&self) -> &[TaskRecordHandle<L>] {
        self.records.get().expect("records have not been set yet")
    }
}

impl<L, P> Task<L, P>
where
    L: TaskTrackerListener<Payload = P>,
{
    pub fn status(&self) -> TaskStatus {
        self.record.status()
    }

    pub fn progress(&self) -> f64 {
        self.record.progress()
    }

    pub fn start(&self) {
        self.record
            .tracker
            .listener
            .task_started(self.record.index, &self.payload);
        self.record.change_status(TaskStatus::InProgress);
    }

    pub fn fail(&self) {
        self.record
            .tracker
            .listener
            .task_finished(self.record.index);
        self.record.change_status(TaskStatus::Error);
    }

    pub fn finish(&self) {
        self.record
            .tracker
            .listener
            .task_finished(self.record.index);
        self.record.change_status(TaskStatus::Done);
    }

    pub fn set_progress(&self, progress: f64) {
        self.record
            .tracker
            .listener
            .task_progressed(self.record.index, progress);
        self.record.progress.store(progress, Ordering::Relaxed);
    }
}

impl<L> TaskRecord<L>
where
    L: TaskTrackerListener,
{
    fn status(&self) -> TaskStatus {
        self.status.load(Ordering::Relaxed)
    }

    fn change_status(&self, status: TaskStatus) {
        self.status.store(status, Ordering::Relaxed);
        self.tracker.update_summary();
    }

    fn progress(&self) -> f64 {
        self.progress.load(Ordering::Relaxed)
    }
}
