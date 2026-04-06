use crate::{
    get_cell_value, get_doc_key, get_id_string, input_value, sort_hits, value_to_string,
    value_to_string_for_edit, App, Msg, SearchHit, Toast, ToastType, ViewConfig,
};
use serde_json::Value;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement};
use yew::events::DragEvent;
use yew::{html, Context, Html, MouseEvent};

/// Render the results area (table or empty state)
pub fn render_results(app: &App, ctx: &Context<App>) -> Html {
    if app.last_results.is_none() {
        return html! {
            <div class="empty-state">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <circle cx="11" cy="11" r="8"></circle>
                    <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
                </svg>
                <p>{ "输入搜索关键词开始查询" }</p>
            </div>
        };
    }
    let Some(results) = &app.last_results else {
        return Html::default();
    };
    if results.hits.is_empty() {
        return html! {
            <div class="empty-state" style="grid-column: 1 / -1;">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <circle cx="11" cy="11" r="8"></circle>
                    <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
                </svg>
                <p>{ "未找到匹配结果" }</p>
            </div>
        };
    }

    let hits = results.hits.clone();
    if app.view_mode != "table" {
        if let Some(cfg) = app.active_view_config() {
            return render_custom_results(app, ctx, &hits, &cfg);
        }
    }
    let columns = app.apply_column_prefs(app.last_base_columns.clone());
    if columns.is_empty() {
        return html! {
            <div class="empty-state" style="grid-column: 1 / -1;">
                <p>{ "当前列已全部隐藏，请在列设置中勾选显示。" }</p>
            </div>
        };
    }

    let mut sorted_hits = hits;
    if !app.table_sort_field.is_empty() && columns.contains(&app.table_sort_field) {
        sorted_hits = sort_hits(sorted_hits, &app.table_sort_field, &app.table_sort_dir);
    }

    let action_col = "__action__".to_string();
    let mut all_cols = columns.clone();
    all_cols.push(action_col.clone());

    let colgroup = html! {
        <colgroup>
            { for all_cols.iter().map(|col| {
                let width = app.column_widths.get(col).copied();
                let style = width.map(|w| format!("width: {}px;", w));
                html! { <col data-col={col.clone()} style={style.unwrap_or_default()} /> }
            }) }
        </colgroup>
    };

    let header = render_table_header(app, ctx, &columns, action_col.clone());
    let body = render_table_body(app, ctx, &sorted_hits, &columns, action_col);

    html! {
        <div class="results-table-wrap">
            <table class="results-table">
                { colgroup }
                { header }
                { body }
            </table>
        </div>
    }
}

fn render_table_header(
    app: &App,
    ctx: &Context<App>,
    columns: &[String],
    action_col: String,
) -> Html {
    html! {
        <thead>
            <tr>
                { for columns.iter().map(|col| {
                    let label = app.field_labels.get(col).cloned().unwrap_or_else(|| col.clone());
                    let is_active = app.table_sort_field == *col;
                    let dir_mark = if is_active { if app.table_sort_dir == "asc" { " ▲" } else { " ▼" } } else { "" };
                    let col_key = col.clone();
                    let drag_col = col.clone();
                    let over_col = col.clone();
                    let drop_col = col.clone();
                    let sort_col = col.clone();
                    let resize_col = col.clone();
                    let mut th_class = if app.drag_over_col.as_ref() == Some(col) { "sortable-th drag-over".to_string() } else { "sortable-th".to_string() };
                    if !app.primary_key_field.is_empty() && app.primary_key_field == *col {
                        th_class.push_str(" pk-col");
                    }
                    html! {
                        <th
                            class={th_class}
                            draggable="true"
                            data-col={col_key.clone()}
                            ondragstart={ctx.link().callback(move |_| Msg::DragStart(drag_col.clone()))}
                            ondragover={ctx.link().callback(move |e: DragEvent| { e.prevent_default(); Msg::DragOver(over_col.clone()) })}
                            ondrop={ctx.link().callback(move |e: DragEvent| { e.prevent_default(); Msg::DropOn(drop_col.clone()) })}
                            ondragend={ctx.link().callback(|_| Msg::DragEnd)}
                            onclick={ctx.link().callback(move |_| Msg::ToggleTableSort(sort_col.clone()))}
                        >
                            <span class="th-label">
                                { if !app.primary_key_field.is_empty() && app.primary_key_field == *col { "🔑 " } else { "" } }
                                { format!("{}{}", label, dir_mark) }
                            </span>
                            <span
                                class="column-resizer"
                                data-col={col_key.clone()}
                                title="拖动调整列宽"
                                onmousedown={ctx.link().callback(move |e: MouseEvent| {
                                    e.prevent_default();
                                    e.stop_propagation();
                                    let target = e.target().and_then(|t| t.dyn_into::<Element>().ok());
                                    let start_width = target
                                        .and_then(|el| el.closest("th").ok().flatten())
                                        .and_then(|th| th.dyn_into::<HtmlElement>().ok())
                                        .map(|el| el.get_bounding_client_rect().width() as i32)
                                        .unwrap_or(120);
                                    Msg::StartResize(resize_col.clone(), e.client_x(), start_width)
                                })}
                            ></span>
                        </th>
                    }
                }) }
                <th data-col={action_col.to_string()}>{ "操作" }
                    <span
                        class="column-resizer"
                        data-col={action_col.to_string()}
                        title="拖动调整列宽"
                        onmousedown={ctx.link().callback(move |e: MouseEvent| {
                            e.prevent_default();
                            e.stop_propagation();
                            let target = e.target().and_then(|t| t.dyn_into::<Element>().ok());
                            let start_width = target
                                .and_then(|el| el.closest("th").ok().flatten())
                                .and_then(|th| th.dyn_into::<HtmlElement>().ok())
                                .map(|el| el.get_bounding_client_rect().width() as i32)
                                .unwrap_or(120);
                            Msg::StartResize(action_col.to_string(), e.client_x(), start_width)
                        })}
                    ></span>
                </th>
            </tr>
        </thead>
    }
}

fn render_table_body(
    app: &App,
    ctx: &Context<App>,
    hits: &[SearchHit],
    columns: &[String],
    action_col: String,
) -> Html {
    html! {
        <tbody>
            { for hits.iter().map(|hit| {
                let id = get_id_string(hit);
            let row_cells = columns.iter().map(|col| {
                let doc_id = get_doc_key(hit, &app.primary_key_field);
                let is_primary_key = !app.primary_key_field.is_empty() && app.primary_key_field == *col;
                let is_editable = !app.edit_locked && !doc_id.is_empty() && !is_primary_key;
                let edited_value = app
                    .pending_edits
                    .get(&doc_id)
                    .and_then(|m| m.get(col))
                    .cloned();

                if is_editable {
                    let raw_value = edited_value.unwrap_or_else(|| hit.get(col).cloned().unwrap_or(Value::Null));
                    let display = value_to_string_for_edit(&raw_value);
                    let field = col.clone();
                    let doc_key = doc_id.clone();
                    return html! {
                        <td>
                            <input
                                class="form-control"
                                style="padding: 6px 8px; font-size: 0.9rem;"
                                value={display}
                                oninput={ctx.link().callback(move |e: yew::events::InputEvent| {
                                    Msg::UpdateCellEdit(doc_key.clone(), field.clone(), input_value(e))
                                })}
                            />
                        </td>
                    };
                }

                if let Some(edit_override) = edited_value {
                    let display = value_to_string_for_edit(&edit_override);
                    let class = if is_primary_key { "pk-cell" } else { "" };
                    return html! { <td class={class} title={display.clone()}>{ display }</td> };
                }

                if let Some(cell) = get_cell_value(hit, col, app.highlight_enabled, app.image_preview_enabled, app.image_preview_links_only) {
                    let class = if is_primary_key { "pk-cell" } else { "" };
                    html! { <td class={class} title={cell.title}>{ cell.html }</td> }
                } else {
                    let class = if is_primary_key { "pk-cell" } else { "" };
                    html! { <td class={class}></td> }
                }
            });
                let ranking = hit.ranking_score.map(|score| {
                    html! { <>
                        <br />
                        <small style="color: var(--text-secondary);">{ format!("评分: {:.1}%", score * 100.0) }</small>
                    </> }
                });
                html! {
                    <tr>
                        { for row_cells }
                        <td>
                            <button class="btn btn-secondary" onclick={ctx.link().callback(move |_| Msg::OpenResultModal(Some(id.clone())))}>{ "查看" }</button>
                            { ranking }
                        </td>
                    </tr>
                }
            }) }
        </tbody>
    }
}

/// Render custom view-based results
pub fn render_custom_results(
    app: &App,
    ctx: &Context<App>,
    hits: &[SearchHit],
    cfg: &ViewConfig,
) -> Html {
    let columns = if cfg.columns.is_empty() {
        vec![vec![]]
    } else {
        cfg.columns.clone()
    };
    let widths = App::normalize_view_widths(cfg.widths.clone(), columns.len());
    let label_widths = App::normalize_label_widths(cfg.label_widths.clone(), columns.len());
    let gap_px = 12.0_f32;
    let gap_each = if columns.len() > 0 {
        gap_px * (columns.len() as f32 - 1.0) / columns.len() as f32
    } else {
        0.0
    };
    let grid_cols = widths
        .iter()
        .map(|w| format!("calc({}% - {:.2}px)", w, gap_each))
        .collect::<Vec<_>>()
        .join(" ");
    html! {
        <div class="custom-results">
            { for hits.iter().map(|hit| {
                let id = get_id_string(hit);
                html! {
                    <div class="custom-row">
                        <div class="custom-row-columns" style={format!("grid-template-columns: {};", grid_cols)}>
                            { for columns.iter().enumerate().map(|(col_idx, col_fields)| {
                                let widths = widths.clone();
                                let label_widths = label_widths.clone();
                                let label_px = label_widths.get(col_idx).cloned().unwrap_or(140);
                                html! {
                                    <div class="custom-col" style={format!("--label-width: {}px;", label_px)}>
                                        <span
                                            class="view-col-resizer"
                                            title="拖动调整列宽"
                                            onmousedown={ctx.link().callback(move |e: MouseEvent| {
                                                e.prevent_default();
                                                e.stop_propagation();
                                                let target = e.target().and_then(|t| t.dyn_into::<Element>().ok());
                                                let (start_width, container_width) = target
                                                    .and_then(|el| el.closest(".custom-row-columns").ok().flatten())
                                                    .and_then(|el| el.dyn_into::<HtmlElement>().ok())
                                                    .map(|el| {
                                                        let rect = el.get_bounding_client_rect();
                                                        (rect.width() as f32, rect.width() as f32)
                                                    })
                                                    .unwrap_or((0.0, 0.0));
                                                let current_percent = widths.get(col_idx).cloned().unwrap_or(0) as f32;
                                                let start_width_px = if container_width > 0.0 { container_width * current_percent / 100.0 } else { start_width };
                                                Msg::StartViewColumnResize(col_idx, e.client_x(), start_width_px, container_width)
                                            })}
                                        ></span>
                                        <span
                                            class="field-width-resizer"
                                            title="拖动调整字段宽度"
                                            onmousedown={ctx.link().callback(move |e: MouseEvent| {
                                                e.prevent_default();
                                                e.stop_propagation();
                                                let target = e.target().and_then(|t| t.dyn_into::<Element>().ok());
                                                let container_width = target
                                                    .and_then(|el| el.closest(".custom-col").ok().flatten())
                                                    .and_then(|el| el.dyn_into::<HtmlElement>().ok())
                                                    .map(|el| el.get_bounding_client_rect().width() as f32)
                                                    .unwrap_or(0.0);
                                                Msg::StartViewFieldResize(col_idx, e.client_x(), label_px as f32, container_width)
                                            })}
                                        ></span>
                                        { for col_fields.iter().map(|field| {
                                            let label = app.field_labels.get(field).cloned().unwrap_or_else(|| field.clone());
                                            let cell = get_cell_value(hit, field, app.highlight_enabled, app.image_preview_enabled, app.image_preview_links_only);
                                            html! {
                                                <div class="custom-field">
                                                    <span class="custom-label">{ label }</span>
                                                    <span class="custom-value" title={cell.as_ref().map(|c| c.title.to_string()).unwrap_or_default()}>{ cell.map(|c| c.html).unwrap_or_default() }</span>
                                                </div>
                                            }
                                        }) }
                                    </div>
                                }
                            }) }
                        </div>
                        <div class="custom-row-actions">
                            <button class="btn btn-secondary" onclick={ctx.link().callback(move |_| Msg::OpenResultModal(Some(id.clone())))}>{ "查看" }</button>
                        </div>
                    </div>
                }
            }) }
        </div>
    }
}

/// Render pagination controls
pub fn render_pagination(app: &App, ctx: &Context<App>) -> Html {
    let total_hits = app.results_count;
    let total_pages = (total_hits as f64 / app.page_size as f64).ceil() as u32;
    if total_pages <= 1 {
        return Html::default();
    }
    let mut items: Vec<Html> = vec![];
    let prev_disabled = app.current_page == 1;
    let prev_page = app.current_page.saturating_sub(1);
    items.push(html! {
        <button class="pagination-btn" disabled={prev_disabled} onclick={ctx.link().callback(move |_| Msg::GoToPage(prev_page))}>{ "上一页" }</button>
    });
    let start_page = app.current_page.saturating_sub(2).max(1);
    let end_page = (app.current_page + 2).min(total_pages);
    if start_page > 1 {
        items.push(html! { <button class="pagination-btn" onclick={ctx.link().callback(|_| Msg::GoToPage(1))}>{ "1" }</button> });
        if start_page > 2 {
            items.push(html! { <span>{ "..." }</span> });
        }
    }
    for i in start_page..=end_page {
        let class = if i == app.current_page {
            "pagination-btn active"
        } else {
            "pagination-btn"
        };
        items.push(html! { <button class={class} onclick={ctx.link().callback(move |_| Msg::GoToPage(i))}>{ i }</button> });
    }
    if end_page < total_pages {
        if end_page + 1 < total_pages {
            items.push(html! { <span>{ "..." }</span> });
        }
        let last_page = total_pages;
        items.push(html! { <button class="pagination-btn" onclick={ctx.link().callback(move |_| Msg::GoToPage(last_page))}>{ last_page }</button> });
    }
    let next_disabled = app.current_page >= total_pages;
    let next_page = app.current_page + 1;
    items.push(html! {
        <button class="pagination-btn" disabled={next_disabled} onclick={ctx.link().callback(move |_| Msg::GoToPage(next_page))}>{ "下一页" }</button>
    });
    html! { for items }
}

/// Render toast notification
pub fn render_toast(toast: &Toast) -> Html {
    let class = match toast.kind {
        ToastType::Success => "toast success",
        ToastType::Error => "toast error",
        ToastType::Warning => "toast warning",
    };
    let icon = match toast.kind {
        ToastType::Success => "✅",
        ToastType::Error => "❌",
        ToastType::Warning => "⚠️",
    };
    html! {
        <div class={class}>
            <span>{ icon }</span>
            <span>{ toast.message.clone() }</span>
        </div>
    }
}

pub fn init() {
    // no-op for skeleton
}
