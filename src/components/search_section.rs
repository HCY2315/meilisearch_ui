use crate::query_editor;
use crate::results_table;
use crate::{checkbox_checked, input_value, select_value, App, Msg};
use yew::{html, Callback, Context, Html, MouseEvent};

/// Search section component - renders search box, query builder, advanced settings, and results
pub fn render_search_section(app: &App, ctx: &Context<App>) -> Html {
    let ai_active = app.ai_config.ai_enabled && app.ai_config.ai_weight > 0;
    let ai_weight = app.ai_config.ai_weight.min(100);
    let base_weight = 100 - ai_weight;
    let ai_dropdown_class = if app.ai_dropdown_open {
        "ai-dropdown active"
    } else {
        "ai-dropdown"
    };
    let ai_badge_class = if ai_active {
        "ai-badge active"
    } else {
        "ai-badge"
    };

    html! {
        <section class="search-section">
            <div class="search-panel">
                { render_search_box(app, ctx, ai_dropdown_class, ai_badge_class, base_weight, ai_weight) }
                { render_query_builder(app, ctx) }
                { render_advanced_settings(app, ctx) }
                { render_results_panel(app, ctx) }
            </div>
        </section>
    }
}

fn render_search_box(
    app: &App,
    ctx: &Context<App>,
    ai_dropdown_class: &'static str,
    ai_badge_class: &'static str,
    base_weight: u32,
    ai_weight: u32,
) -> Html {
    html! {
        <div class="search-panel">
            <div class="search-box">
                <span class="search-icon">{ "🔍" }</span>
                <input
                    class="search-input"
                    value={app.search_input.clone()}
                    placeholder="输入搜索关键词..."
                    oninput={ctx.link().callback(|e: yew::events::InputEvent| Msg::SetSearchInput(input_value(e)))}
                />
                <span
                    class={ai_badge_class}
                    onclick={ctx.link().callback(|e: MouseEvent| {
                        e.stop_propagation();
                        Msg::ToggleAiDropdown
                    })}
                >{ "AI" }</span>
                <div
                    class={ai_dropdown_class}
                    onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}
                >
                    <div class="dropdown-title">{ "AI 权重" }</div>
                    <label class="dropdown-row" style="margin-bottom: 10px; cursor: pointer;">
                        <span>{ "启用 AI" }</span>
                        <input
                            type="checkbox"
                            checked={app.ai_config.ai_enabled}
                            onchange={ctx.link().callback(|e: yew::events::Event| Msg::SetAiEnabled(checkbox_checked(e)))}
                        />
                    </label>
                    <div class="dropdown-row">
                        <span>{ "基础" }</span>
                        <span class="ai-weight-value">{ format!("{}%", base_weight) }</span>
                    </div>
                    <div class="dropdown-row">
                        <span>{ "AI" }</span>
                        <span class="ai-weight-value">{ format!("{}%", ai_weight) }</span>
                    </div>
                    <input
                        type="range"
                        min="0"
                        max="100"
                        value={ai_weight.to_string()}
                        oninput={ctx.link().callback(|e: yew::events::InputEvent| Msg::SetAiWeight(input_value(e).parse::<u32>().unwrap_or(0)))}
                    />
                    <div class="ai-weight-hint">{ format!("基础 {}% · AI {}%", base_weight, ai_weight) }</div>
                </div>
            </div>
        </div>
    }
}

fn render_query_builder(app: &App, ctx: &Context<App>) -> Html {
    html! {
        <div class="query-builder">
            <h3 style="margin-bottom: 16px; font-size: 1.1rem;">{ "📋 查询条件构建器" }</h3>
            <div>
                { for app.query_rows.iter().map(|row| query_editor::render_query_row(app, ctx, row)) }
            </div>
            <div class="query-actions" style="margin-top: 16px;">
                <button class="btn btn-secondary" onclick={ctx.link().callback(|_| Msg::AddQueryRow)}>{ "➕ 添加查询条件" }</button>
                <button class="btn btn-primary" onclick={ctx.link().callback(|_| Msg::ApplyQuery)}>{ "✅ 应用查询" }</button>
                <button class="btn btn-secondary" onclick={ctx.link().callback(|_| Msg::ClearQuery)}>{ "🗑️ 清空查询" }</button>
            </div>
            <div style="margin-top: 10px; font-size: 0.85rem; color: var(--text-secondary);">
                { "当前过滤: " }{ query_editor::filter_preview_text(app) }
            </div>
        </div>
    }
}

fn render_advanced_settings(app: &App, ctx: &Context<App>) -> Html {
    html! {
        <div class="advanced-settings">
            <h3 style="margin-bottom: 16px; font-size: 1.1rem;">{ "⚙️ 高级搜索设置" }</h3>
            <div class="settings-grid">
                <div class="checkbox-group">
                    <input
                        type="checkbox"
                        checked={app.highlight_enabled}
                        onchange={ctx.link().callback(|e: yew::events::Event| Msg::ToggleHighlight(checkbox_checked(e)))}
                    />
                    <label>{ "启用高亮显示" }</label>
                </div>
                <div class="checkbox-group">
                    <input
                        type="checkbox"
                        checked={app.show_ranking_score}
                        onchange={ctx.link().callback(|e: yew::events::Event| Msg::ToggleRanking(checkbox_checked(e)))}
                    />
                    <label>{ "显示评分" }</label>
                </div>
                <div class="checkbox-group">
                    <input
                        type="number"
                        value={app.crop_length.to_string()}
                        min="0"
                        style="width: 80px;"
                        oninput={ctx.link().callback(|e: yew::events::InputEvent| Msg::UpdateCropLength(input_value(e)))}
                    />
                    <label>{ "内容截断长度" }</label>
                </div>
                <div class="form-group" style="flex: 1;">
                    <label>{ "每页结果数" }</label>
                    <input
                        type="number"
                        class="form-control"
                        value={app.page_size.to_string()}
                        min="1"
                        max="100"
                        oninput={ctx.link().callback(|e: yew::events::InputEvent| Msg::UpdatePageSize(input_value(e)))}
                    />
                </div>
            </div>
        </div>
    }
}

fn render_results_panel(app: &App, ctx: &Context<App>) -> Html {
    html! {
        <div class="results-panel" style="margin-top: 24px;">
            <div class="results-stats">
                <span class="results-count">{ "找到 " }<strong>{ app.results_count_text() }</strong>{ " 条结果" }</span>
                <div class="results-actions">
                    <span>{ app.search_time_text() }</span>
                    <button class="btn btn-secondary" onclick={ctx.link().callback(|_| Msg::OpenFilters(true))}>{ "筛选入口" }</button>
                    <button class="btn btn-secondary" onclick={ctx.link().callback(|_| Msg::OpenColumnConfig(true))}>{ "列设置" }</button>
                    <button class="btn btn-secondary" onclick={ctx.link().callback(|_| Msg::OpenViewConfig(true))}>{ "视图设置" }</button>
                    <select
                        class="form-control"
                        style="min-width: 140px;"
                        onchange={ctx.link().callback(|e: yew::events::Event| Msg::SetViewMode(select_value(e)))}
                        value={app.view_mode.clone()}
                    >
                        { app.render_view_options() }
                    </select>
                    <button class="btn btn-secondary" onclick={ctx.link().callback(|_| Msg::ToggleEditLock)}>
                        { if app.edit_locked { "🔒 已锁定" } else { "🔓 可编辑" } }
                    </button>
                    { if !app.edit_locked {
                        html! {
                            <select
                                class="form-control"
                                style="min-width: 140px;"
                                onchange={ctx.link().callback(|e: yew::events::Event| Msg::SetPrimaryKey(select_value(e)))}
                                value={app.primary_key_field.clone()}
                            >
                                { app.render_primary_key_options() }
                            </select>
                        }
                    } else { Html::default() } }
                    <button
                        class="btn btn-primary"
                        disabled={app.edit_locked || app.pending_edits.is_empty()}
                        onclick={ctx.link().callback(|_| Msg::SaveEdits)}
                    >
                        { "保存修改" }
                    </button>
                </div>
            </div>
            <div id="resultsContainer" class={if app.view_mode == "table" { "results-grid" } else { "results-grid custom-grid" }}>
                { results_table::render_results(app, ctx) }
            </div>
            <div id="pagination" class="pagination">
                { results_table::render_pagination(app, ctx) }
            </div>
        </div>
    }
}
