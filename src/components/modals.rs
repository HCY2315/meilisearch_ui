use crate::{collect_field_config, collect_hidden_columns, input_value, App, Msg};
use yew::{html, Context, Html};

/// Field config modal component
pub fn render_field_config_modal(app: &App, ctx: &Context<App>) -> Html {
    let modal_class = if app.field_config_open {
        "modal active"
    } else {
        "modal"
    };
    html! {
        <div class={modal_class}>
            <div class="modal-content">
                <div class="modal-header">
                    <h2 class="modal-title">{ "配置搜索字段" }</h2>
                    <button class="modal-close" onclick={ctx.link().callback(|_| Msg::OpenFieldConfig(false))}>{ "×" }</button>
                </div>
                <div style="display: flex; flex-wrap: wrap; gap: 8px; margin-bottom: 12px;">
                    <button class="btn btn-secondary" onclick={ctx.link().callback(|_| Msg::SelectAllSearchable(true))}>{ "全选可搜索" }</button>
                    <button class="btn btn-secondary" onclick={ctx.link().callback(|_| Msg::SelectAllSearchable(false))}>{ "取消可搜索" }</button>
                    <button class="btn btn-secondary" onclick={ctx.link().callback(|_| Msg::SelectAllHighlight(true))}>{ "全选高亮" }</button>
                    <button class="btn btn-secondary" onclick={ctx.link().callback(|_| Msg::SelectAllHighlight(false))}>{ "取消高亮" }</button>
                    <button class="btn btn-secondary" onclick={ctx.link().callback(|_| Msg::SelectAllDisplay(true))}>{ "全选显示" }</button>
                    <button class="btn btn-secondary" onclick={ctx.link().callback(|_| Msg::SelectAllDisplay(false))}>{ "取消显示" }</button>
                </div>
                <div>
                    { for app.render_field_config(ctx) }
                </div>
                <div style="margin-top: 20px; text-align: right;">
                    <button class="btn btn-secondary" onclick={ctx.link().callback(|_| Msg::OpenFieldConfig(false))}>{ "取消" }</button>
                    <button class="btn btn-primary" onclick={ctx.link().callback(|_| Msg::SaveFieldConfig(collect_field_config()))}>{ "保存配置" }</button>
                </div>
            </div>
        </div>
    }
}

/// Column config modal component
pub fn render_column_config_modal(app: &App, ctx: &Context<App>) -> Html {
    let modal_class = if app.column_config_open {
        "modal active"
    } else {
        "modal"
    };
    html! {
        <div class={modal_class}>
            <div class="modal-content">
                <div class="modal-header">
                    <h2 class="modal-title">{ "列设置" }</h2>
                    <button class="modal-close" onclick={ctx.link().callback(|_| Msg::OpenColumnConfig(false))}>{ "×" }</button>
                </div>
                <div>
                    <p style="color: var(--text-secondary); font-size: 0.9rem; margin-bottom: 8px;">{ "勾选控制显示/隐藏，列顺序可在表头拖拽调整。" }</p>
                    <div class="column-config-list">
                        { app.render_column_config(ctx) }
                    </div>
                </div>
                <div class="modal-footer">
                    <button class="btn btn-secondary" onclick={ctx.link().callback(|_| Msg::OpenColumnConfig(false))}>{ "取消" }</button>
                    <button class="btn btn-primary" onclick={ctx.link().callback(|_| Msg::SaveColumnConfig(collect_hidden_columns()))}>{ "保存设置" }</button>
                </div>
            </div>
        </div>
    }
}

/// View config modal component
pub fn render_view_config_modal(app: &App, ctx: &Context<App>) -> Html {
    let modal_class = if app.view_modal_open {
        "modal active"
    } else {
        "modal"
    };
    html! {
        <div class={modal_class}>
            <div class="modal-content view-modal">
                <div class="modal-header">
                    <h2 class="modal-title">{ "视图设置" }</h2>
                    <button class="modal-close" onclick={ctx.link().callback(|_| Msg::OpenViewConfig(false))}>{ "×" }</button>
                </div>
                <div style="margin-bottom: 16px; display: flex; gap: 12px; align-items: center;">
                    <input
                        class="form-control"
                        style="max-width: 240px;"
                        placeholder="视图名称"
                        value={app.view_name_input.clone()}
                        oninput={ctx.link().callback(|e: yew::events::InputEvent| Msg::UpdateViewName(input_value(e)))}
                    />
                    <button class="btn btn-primary" onclick={ctx.link().callback(|_| Msg::SaveViewConfig)}>{ "保存视图" }</button>
                    <button class="btn btn-secondary" onclick={ctx.link().callback(|_| Msg::AddViewColumn)}>{ "增加列" }</button>
                    <button class="btn btn-secondary" onclick={ctx.link().callback(|_| Msg::RemoveViewColumn)}>{ "删除列" }</button>
                </div>
                <div class="view-workspace">
                    <div class="view-fields">
                        <div class="view-title">{ "可用字段" }</div>
                        <div
                            class="view-field-list"
                            ondragover={yew::Callback::from(|e: yew::events::DragEvent| e.prevent_default())}
                            ondrop={ctx.link().callback(|e: yew::events::DragEvent| { e.prevent_default(); Msg::DropViewFieldToPool })}
                        >
                            { for app.render_view_field_pool(ctx) }
                        </div>
                    </div>
                    <div class="view-columns" style={format!("grid-template-columns: repeat({}, minmax(0, 1fr));", app.view_layout_working.len().max(1))}>
                        { for (0..app.view_layout_working.len()).map(|idx| app.render_view_column(ctx, idx)) }
                    </div>
                </div>
            </div>
        </div>
    }
}

/// Result detail modal component
pub fn render_result_modal(app: &App, ctx: &Context<App>) -> Html {
    let modal_class = if app.result_modal_open {
        "modal active"
    } else {
        "modal"
    };
    html! {
        <div class={modal_class}>
            <div class="modal-content">
                <div class="modal-header">
                    <h2 class="modal-title">{ "结果详情" }</h2>
                    <button class="modal-close" onclick={ctx.link().callback(|_| Msg::CloseResultModal)}>{ "×" }</button>
                </div>
                <div>
                    { app.render_result_detail() }
                </div>
            </div>
        </div>
    }
}
