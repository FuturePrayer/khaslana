use std::{fs, thread};

use git2::Repository;
use gpui::{Context, IntoElement, div, prelude::*, px, rgb};
use khaslana::{WorkflowExecutor, WorkflowProgressEvent, WorkflowRunOptions, parse_workflow_json5};

use crate::{
    DialogState, RepositoryLoading, RepositorySnapshot, RepositoryView, ScrollbarMode, UiEvent,
    placeholder_row, scrollable_frame_when, send_ui_event,
    ui::{
        components::{dialog_actions, dialog_panel as ui_dialog_panel, section_title},
        theme as ui_theme,
    },
};

impl RepositoryView {
    pub(crate) fn open_workflow_dialog(&mut self) {
        if self.repo_path.is_none() {
            self.last_error = Some("请先打开一个仓库".into());
            return;
        }
        self.close_popups();
        self.active_dialog = Some(DialogState::WorkflowRunner);
        self.workflow_log.clear();
        self.last_error = None;
    }

    pub(crate) fn browse_workflow_file(&mut self) {
        self.status = "正在选择工作流文件".to_string();
        self.last_error = None;
        let tx = self.tx.clone();
        thread::spawn(move || {
            let path = rfd::FileDialog::new()
                .add_filter("JSON5 工作流", &["json5", "jsonc"])
                .pick_file();
            send_ui_event(&tx, UiEvent::WorkflowFileSelected { path });
        });
    }

    pub(crate) fn clear_workflow_file(&mut self) {
        self.workflow_definition = None;
        self.workflow_preview = None;
        self.workflow_file_path = None;
        self.workflow_log.clear();
        self.last_error = None;
    }

    pub(crate) fn load_workflow_file(&mut self, path: std::path::PathBuf) {
        let Some(repo_path) = self.repo_path.clone() else {
            self.last_error = Some("请先打开一个仓库".into());
            return;
        };
        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(err) => {
                self.last_error = Some(format!("工作流文件读取失败：{err}"));
                return;
            }
        };
        let definition = match parse_workflow_json5(&content) {
            Ok(definition) => definition,
            Err(err) => {
                self.last_error = Some(err.to_string());
                return;
            }
        };
        let Some(tab_id) = self.active_tab_id() else {
            self.last_error = Some("请先打开一个仓库".into());
            return;
        };
        let service = self.service_for_tab(tab_id);
        let preview = match Repository::open(repo_path)
            .map_err(khaslana::GitError::from)
            .and_then(|repo| {
                WorkflowExecutor::new(&service).preview(
                    &repo,
                    &definition,
                    &WorkflowRunOptions {
                        default_remote: self.current_remote().unwrap_or_else(|| "origin".into()),
                    },
                )
            }) {
            Ok(preview) => preview,
            Err(err) => {
                self.last_error = Some(err.to_string());
                return;
            }
        };
        self.workflow_definition = Some(definition);
        self.workflow_preview = Some(preview);
        self.workflow_file_path = Some(path);
        self.workflow_log.clear();
        self.status = "工作流已加载".to_string();
        self.last_error = None;
    }

    pub(crate) fn run_workflow(&mut self) {
        let Some(tab_id) = self.active_tab_id() else {
            self.last_error = Some("请先打开一个仓库".into());
            return;
        };
        let Some(repo_path) = self.repo_path.clone() else {
            self.last_error = Some("请先打开一个仓库".into());
            return;
        };
        let Some(definition) = self.workflow_definition.clone() else {
            self.last_error = Some("请先选择工作流文件".into());
            return;
        };
        if self.busy {
            self.last_error = Some("已有操作正在运行".into());
            return;
        }
        self.workflow_log.clear();
        let service = self.service_for_tab(tab_id);
        let tx = self.tx.clone();
        let options = WorkflowRunOptions {
            default_remote: self.current_remote().unwrap_or_else(|| "origin".into()),
        };
        self.apply_status_event(Some(tab_id), |this| {
            this.repository_load_id = this.repository_load_id.wrapping_add(1);
            this.loading = RepositoryLoading::default();
            this.busy = true;
            this.status = "正在运行工作流".to_string();
            this.last_error = None;
        });
        thread::spawn(move || {
            let result = (|| -> khaslana::Result<(RepositorySnapshot, Vec<String>, String)> {
                let mut repo = Repository::open(repo_path)?;
                let mut log = Vec::new();
                let result = WorkflowExecutor::new(&service).run(
                    &mut repo,
                    &definition,
                    options,
                    |event| {
                        let message = workflow_progress_message(&event);
                        log.push(message.clone());
                        send_ui_event(&tx, UiEvent::WorkflowProgress { tab_id, message });
                    },
                )?;
                let message = format!("工作流“{}”已完成（{} 步）", result.name, result.steps_run);
                Ok((result.snapshot, log, message))
            })();
            match result {
                Ok((snapshot, log, message)) => {
                    send_ui_event(
                        &tx,
                        UiEvent::WorkflowFinished {
                            tab_id,
                            message,
                            snapshot,
                            log,
                        },
                    );
                }
                Err(err) => {
                    send_ui_event(
                        &tx,
                        UiEvent::WorkflowProgress {
                            tab_id,
                            message: format!("工作流失败：{err}"),
                        },
                    );
                    send_ui_event(
                        &tx,
                        UiEvent::OperationFailed {
                            tab_id: Some(tab_id),
                            error: err.to_string(),
                        },
                    );
                }
            }
        });
    }

    pub(crate) fn render_workflow_dialog(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let file_label = self
            .workflow_file_path
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "未选择工作流文件".to_string());
        let workflow_name = self
            .workflow_preview
            .as_ref()
            .map(|preview| preview.name.clone())
            .unwrap_or_else(|| "选择 .json5 或 .jsonc 文件后显示预览".to_string());

        ui_dialog_panel("运行工作流")
            .w(px(720.0))
            .max_h(px(660.0))
            .child(
                div().flex().items_center().justify_end().gap_2().child(
                    div()
                        .flex()
                        .gap_2()
                        .child(self.button(
                            "选择文件",
                            !self.busy,
                            |this, _, _| this.browse_workflow_file(),
                            cx,
                        ))
                        .child(self.button(
                            "清空",
                            self.workflow_definition.is_some() && !self.busy,
                            |this, _, _| this.clear_workflow_file(),
                            cx,
                        )),
                ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .text_size(px(12.0))
                    .child(
                        div()
                            .text_color(rgb(ui_theme::TEXT_MUTED))
                            .truncate()
                            .child(file_label),
                    )
                    .child(
                        div()
                            .text_color(rgb(ui_theme::ACCENT_STRONG))
                            .font_weight(gpui::FontWeight::BOLD)
                            .truncate()
                            .child(workflow_name),
                    ),
            )
            .child(self.render_workflow_preview(cx))
            .child(self.render_workflow_log(cx))
            .child(
                dialog_actions()
                    .child(self.button("关闭", !self.busy, |this, _, _| this.close_dialog(), cx))
                    .child(self.primary_button(
                        if self.busy { "运行中..." } else { "运行" },
                        self.workflow_definition.is_some() && !self.busy,
                        |this, _, _| this.run_workflow(),
                        cx,
                    )),
            )
    }

    fn render_workflow_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let rows = self
            .workflow_preview
            .as_ref()
            .map(|preview| {
                preview
                    .steps
                    .iter()
                    .map(|step| {
                        div()
                            .flex()
                            .flex_none()
                            .items_center()
                            .gap_2()
                            .px_2()
                            .py_2()
                            .border_b_1()
                            .border_color(rgb(ui_theme::BORDER))
                            .bg(rgb(ui_theme::SURFACE))
                            .text_size(px(12.0))
                            .child(
                                div()
                                    .flex_none()
                                    .w(px(46.0))
                                    .text_color(rgb(ui_theme::TEXT_FAINT))
                                    .child(format!("#{}", step.index + 1)),
                            )
                            .child(
                                div()
                                    .flex_none()
                                    .w(px(110.0))
                                    .text_color(rgb(ui_theme::ACCENT_STRONG))
                                    .child(step.op),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_w(px(0.0))
                                    .text_color(rgb(ui_theme::TEXT))
                                    .truncate()
                                    .child(step.summary.clone()),
                            )
                            .into_any_element()
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| vec![placeholder_row("暂无工作流预览").into_any_element()]);
        let content_present = self
            .workflow_preview
            .as_ref()
            .is_some_and(|preview| !preview.steps.is_empty());

        div()
            .flex()
            .flex_col()
            .min_h(px(170.0))
            .max_h(px(240.0))
            .border_1()
            .border_color(rgb(ui_theme::BORDER))
            .rounded_sm()
            .child(section_title("步骤预览"))
            .child({
                let handle = self.scroll_handle("workflow-preview-list");
                let content = div()
                    .id("workflow-preview-list")
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h(px(0.0))
                    .overflow_y_scroll()
                    .track_scroll(&handle)
                    .children(rows)
                    .into_any_element();
                scrollable_frame_when(
                    "workflow-preview-list",
                    ScrollbarMode::Vertical,
                    content,
                    handle,
                    content_present,
                    cx,
                )
            })
    }

    fn render_workflow_log(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let rows = if self.workflow_log.is_empty() {
            vec![placeholder_row("运行后显示步骤日志").into_any_element()]
        } else {
            self.workflow_log
                .iter()
                .map(|line| {
                    div()
                        .flex_none()
                        .px_2()
                        .py_1()
                        .border_b_1()
                        .border_color(rgb(ui_theme::BORDER))
                        .text_size(px(12.0))
                        .text_color(rgb(ui_theme::TEXT_MUTED))
                        .bg(rgb(ui_theme::SURFACE))
                        .child(line.clone())
                        .into_any_element()
                })
                .collect::<Vec<_>>()
        };
        let content_present = !self.workflow_log.is_empty();

        div()
            .flex()
            .flex_col()
            .min_h(px(130.0))
            .max_h(px(190.0))
            .border_1()
            .border_color(rgb(ui_theme::BORDER))
            .rounded_sm()
            .child(section_title("运行日志"))
            .child({
                let handle = self.scroll_handle("workflow-log-list");
                let content = div()
                    .id("workflow-log-list")
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h(px(0.0))
                    .overflow_y_scroll()
                    .track_scroll(&handle)
                    .children(rows)
                    .into_any_element();
                scrollable_frame_when(
                    "workflow-log-list",
                    ScrollbarMode::Vertical,
                    content,
                    handle,
                    content_present,
                    cx,
                )
            })
    }
}

fn workflow_progress_message(event: &WorkflowProgressEvent) -> String {
    match event {
        WorkflowProgressEvent::Started { name, total } => {
            format!("开始运行工作流“{name}”（{total} 步）")
        }
        WorkflowProgressEvent::StepStarted {
            index,
            total,
            label,
        } => format!("步骤 {}/{}：{label}", index + 1, total),
        WorkflowProgressEvent::StepFinished {
            index,
            total,
            label,
        } => format!("步骤 {}/{} 完成：{label}", index + 1, total),
        WorkflowProgressEvent::Finished { name, total } => {
            format!("工作流“{name}”已完成（{total} 步）")
        }
    }
}
