use crate::api::MultiDownloadInfo;
use atomic::Atomic;
use bytemuck::NoUninit;
use std::sync::atomic::Ordering;
use std::sync::Arc;

type TaskRecordHandle = Arc<TaskRecord>;

pub struct TaskTracker {
    records: Vec<TaskRecordHandle>,
}

pub struct Task<P> {
    pub record: TaskRecordHandle,
    pub payload: P,
}

#[derive(Default)]
pub struct TaskRecord {
    status: Atomic<TaskStatus>,
    progress: Atomic<f64>,
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

pub struct Summary {
    pub in_progress_count: u32,
    pub success_count: u32,
    pub error_count: u32,
    pub total_count: u32,
    pub total_progress: f64,
}

impl Summary {
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

impl TaskTracker {
    pub fn new<P>(task_payloads: impl IntoIterator<Item = P>) -> (Self, Vec<Task<P>>) {
        let (tasks, records): (Vec<_>, Vec<_>) = task_payloads
            .into_iter()
            .map(|payload| {
                let record = Arc::new(TaskRecord::default());
                let task = Task {
                    record: record.clone(),
                    payload,
                };
                (task, record)
            })
            .unzip();
        let tracker = Self { records };
        (tracker, tasks)
    }

    pub fn summary(&self) -> Summary {
        let total_count = self.records.len() as u32;
        let mut progress_sum = 0.0;
        let mut summary = Summary::new(total_count);
        if total_count == 0 {
            return summary;
        }
        for task in &self.records {
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
}

impl TaskRecord {
    pub fn status(&self) -> TaskStatus {
        self.status.load(Ordering::Relaxed)
    }

    pub fn progress(&self) -> f64 {
        self.progress.load(Ordering::Relaxed)
    }

    pub fn start(&self) {
        self.set_status(TaskStatus::InProgress);
    }

    pub fn fail(&self) {
        self.set_status(TaskStatus::Error);
    }

    pub fn finish(&self) {
        self.set_status(TaskStatus::Done);
        self.set_progress(1.0);
    }

    pub fn set_progress(&self, progress: f64) {
        self.progress.store(progress, Ordering::Relaxed);
    }

    fn set_status(&self, status: TaskStatus) {
        self.status.store(status, Ordering::Relaxed);
    }
}
