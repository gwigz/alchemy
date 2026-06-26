use eframe::egui;

#[derive(Clone, Copy)]
pub enum Level {
    Info,
    Success,
    Error,
}

#[derive(Clone)]
pub struct Notice {
    pub level: Level,
    pub text: String,
}

fn queue_id() -> egui::Id {
    egui::Id::new("notify_queue")
}

fn push(ctx: &egui::Context, level: Level, text: impl Into<String>) {
    let notice = Notice {
        level,
        text: text.into(),
    };

    ctx.data_mut(|d| {
        d.get_temp_mut_or_default::<Vec<Notice>>(queue_id())
            .push(notice);
    });
}

pub fn info(ctx: &egui::Context, text: impl Into<String>) {
    push(ctx, Level::Info, text);
}

pub fn success(ctx: &egui::Context, text: impl Into<String>) {
    push(ctx, Level::Success, text);
}

pub fn error(ctx: &egui::Context, text: impl Into<String>) {
    push(ctx, Level::Error, text);
}

pub fn drain(ctx: &egui::Context) -> Vec<Notice> {
    ctx.data_mut(|d| std::mem::take(d.get_temp_mut_or_default::<Vec<Notice>>(queue_id())))
}
