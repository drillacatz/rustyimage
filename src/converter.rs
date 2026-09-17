use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Instant;

use crate::codec::static_codec::{convert_image, TargetFormat};
use crate::core::frame::Frame;

#[derive(Clone)]
pub struct ConvertItem {
    pub frames: Vec<Frame>,
    pub out_path: PathBuf,
}

#[derive(Clone)]
pub struct ConvertJob {
    pub items: Vec<ConvertItem>,
    pub format: TargetFormat,
    pub quality: u8,
    pub custom_delay_ms: Option<u32>,
    pub repeat_count: u32,
}

impl ConvertJob {
    pub fn single(
        frames: Vec<Frame>,
        out_path: PathBuf,
        format: TargetFormat,
        quality: u8,
        custom_delay_ms: Option<u32>,
        repeat_count: u32,
    ) -> Self {
        Self {
            items: vec![ConvertItem { frames, out_path }],
            format,
            quality,
            custom_delay_ms,
            repeat_count,
        }
    }
}

pub enum ConvertEvent {
    ItemSuccess {
        out_path: PathBuf,
        current: usize,
        total: usize,
    },
    ItemError {
        out_path: PathBuf,
        error: String,
        current: usize,
        total: usize,
    },
    AllFinished {
        successful: Vec<PathBuf>,
        errors: Vec<String>,
        duration_ms: u64,
    },
}

pub struct AsyncConverter {
    receiver: Option<Receiver<ConvertEvent>>,
    pub is_converting: bool,
    pub start_time: Option<Instant>,
    pub progress: Option<(usize, usize)>, // (current_done, total)
}

impl Default for AsyncConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl AsyncConverter {
    pub fn new() -> Self {
        Self {
            receiver: None,
            is_converting: false,
            start_time: None,
            progress: None,
        }
    }

    pub fn start(&mut self, job: ConvertJob) {
        let (tx, rx): (Sender<ConvertEvent>, Receiver<ConvertEvent>) = channel();
        self.receiver = Some(rx);
        self.is_converting = true;
        self.start_time = Some(Instant::now());
        self.progress = Some((0, job.items.len()));

        thread::spawn(move || {
            let start = Instant::now();
            let total = job.items.len();
            let mut successful = Vec::new();
            let mut errors = Vec::new();

            for (idx, item) in job.items.into_iter().enumerate() {
                let res = convert_image(
                    &item.frames,
                    &item.out_path,
                    job.format,
                    job.quality,
                    job.custom_delay_ms,
                    job.repeat_count,
                );

                match res {
                    Ok(()) => {
                        let _ = tx.send(ConvertEvent::ItemSuccess {
                            out_path: item.out_path.clone(),
                            current: idx + 1,
                            total,
                        });
                        successful.push(item.out_path);
                    }
                    Err(err) => {
                        let msg = format!("{}: {err}", item.out_path.display());
                        let _ = tx.send(ConvertEvent::ItemError {
                            out_path: item.out_path,
                            error: msg.clone(),
                            current: idx + 1,
                            total,
                        });
                        errors.push(msg);
                    }
                }
            }

            let duration_ms = start.elapsed().as_millis() as u64;
            let _ = tx.send(ConvertEvent::AllFinished {
                successful,
                errors,
                duration_ms,
            });
        });
    }

    pub fn poll(&mut self) -> Option<ConvertEvent> {
        if let Some(ref rx) = self.receiver {
            if let Ok(event) = rx.try_recv() {
                match &event {
                    ConvertEvent::ItemSuccess { current, total, .. }
                    | ConvertEvent::ItemError { current, total, .. } => {
                        self.progress = Some((*current, *total));
                    }
                    ConvertEvent::AllFinished { .. } => {
                        self.is_converting = false;
                        self.receiver = None;
                        self.progress = None;
                    }
                }
                return Some(event);
            }
        }
        None
    }
}
