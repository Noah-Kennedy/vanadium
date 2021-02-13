use indicatif::{ProgressBar, ProgressStyle};

fn get_style() -> ProgressStyle {
    ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} [{eta_precise}] {msg}")
        .progress_chars("##-")
}

pub fn config_bar(bar: &ProgressBar, message: &str) {
    bar.set_style(get_style());
    bar.enable_steady_tick(200);
    bar.set_message(message);
}