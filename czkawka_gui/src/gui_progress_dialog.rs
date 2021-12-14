use crate::fl;
use crate::help_functions::get_custom_label_from_label_with_image;
use gtk4::prelude::*;
use gtk4::Inhibit;
use gtk4::{Builder, EventControllerKey, Window};

#[derive(Clone)]
pub struct GuiProgressDialog {
    pub window_progress: gtk4::Dialog,

    pub progress_bar_current_stage: gtk4::ProgressBar,
    pub progress_bar_all_stages: gtk4::ProgressBar,

    pub label_stage: gtk4::Label,
    pub label_progress_current_stage: gtk4::Label,
    pub label_progress_all_stages: gtk4::Label,

    pub grid_progress_stages: gtk4::Grid,

    pub button_stop_in_dialog: gtk4::Button,
    pub evk_button_stop_in_dialog: gtk4::EventControllerKey,
}

impl GuiProgressDialog {
    pub fn create_from_builder(window_main: &Window) -> Self {
        let glade_src = include_str!("../ui/progress.glade").to_string();
        let builder = Builder::from_string(glade_src.as_str());

        let window_progress: gtk4::Dialog = builder.object("window_progress").unwrap();
        window_progress.set_transient_for(Some(window_main));
        window_progress.set_title(Some("Czkawka"));

        let progress_bar_current_stage: gtk4::ProgressBar = builder.object("progress_bar_current_stage").unwrap();
        let progress_bar_all_stages: gtk4::ProgressBar = builder.object("progress_bar_all_stages").unwrap();

        let label_stage: gtk4::Label = builder.object("label_stage").unwrap();
        let label_progress_current_stage: gtk4::Label = builder.object("label_progress_current_stage").unwrap();
        let label_progress_all_stages: gtk4::Label = builder.object("label_progress_all_stages").unwrap();

        let grid_progress_stages: gtk4::Grid = builder.object("grid_progress_stages").unwrap();

        let button_stop_in_dialog: gtk4::Button = builder.object("button_stop_in_dialog").unwrap();
        let evk_button_stop_in_dialog = EventControllerKey::new();

        Self {
            window_progress,
            progress_bar_current_stage,
            progress_bar_all_stages,
            label_stage,
            label_progress_current_stage,
            label_progress_all_stages,
            grid_progress_stages,
            button_stop_in_dialog,
            evk_button_stop_in_dialog,
        }
    }
    pub fn update_language(&self) {
        get_custom_label_from_label_with_image(&self.button_stop_in_dialog.clone()).set_text(&fl!("progress_stop_button"));

        self.label_progress_current_stage.set_label(&fl!("progress_current_stage"));
        self.label_progress_all_stages.set_label(&fl!("progress_all_stages"));
    }
}
