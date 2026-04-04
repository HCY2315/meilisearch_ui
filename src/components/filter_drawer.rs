use crate::{input_value, select_value, App, Msg};
use wasm_bindgen::JsCast;
use yew::{html, Callback, Context, Html, MouseEvent};

/// Filter drawer component
pub fn render_filter_drawer(app: &App, ctx: &Context<App>) -> Html {
    let drawer_class = if app.filters_drawer_open {
        "filters-drawer active"
    } else {
        "filters-drawer"
    };
    html! {
        <div class={drawer_class} onclick={ctx.link().callback(|e: MouseEvent| {
            let target = e.target().and_then(|t| t.dyn_into::<web_sys::Element>().ok());
            if let Some(el) = target {
                if el.class_list().contains("filters-drawer") {
                    return Msg::OpenFilters(false);
                }
            }
            Msg::OpenFilters(true)
        })}>
            <div class="filters-drawer-content" onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}>
                <div class="filters-drawer-header">
                    <h3 style="font-size: 1.1rem;">{ "筛选与排序" }</h3>
                    <button class="btn btn-secondary" style="padding: 4px 12px;" onclick={ctx.link().callback(|_| Msg::OpenFilters(false))}>{ "关闭" }</button>
                </div>
                <div class="filters-drawer-body">
                    <aside class="filters-sidebar">
                        <div class="filter-section">
                            <h3 class="filter-title">
                                { "搜索字段" }
                                <button class="btn btn-secondary" style="padding: 4px 12px; font-size: 0.85rem;" onclick={ctx.link().callback(|_| Msg::OpenFieldConfig(true))}>{ "配置" }</button>
                            </h3>
                            <ul class="filter-list">
                                { app.render_search_fields(ctx) }
                            </ul>
                        </div>

                        <div class="filter-section" style={if app.facets_available() { "" } else { "display:none;" }}>
                            <h3 class="filter-title">{ "📊 分面筛选" }</h3>
                            <div>
                                { app.render_facets(ctx) }
                            </div>
                        </div>

                        <div class="filter-section">
                            <h3 class="filter-title">
                                { "📝 搜索历史" }
                                <span class="clear-history" onclick={ctx.link().callback(|_| Msg::ClearHistory)}>{ "清空" }</span>
                            </h3>
                            <ul class="history-list">
                                { app.render_history(ctx) }
                            </ul>
                        </div>

                        <div class="filter-section">
                            <h3 class="filter-title">{ "🔥 热门搜索" }</h3>
                            <div class="form-group" style="margin-bottom: 12px;">
                                <label style="font-size: 0.85rem; color: var(--text-secondary);">{ "热门字段" }</label>
                                <select class="form-control" onchange={ctx.link().callback(|e: yew::events::Event| Msg::SelectPopularField(select_value(e)))} value={app.popular_search_field.clone()}>
                                    { app.render_popular_field_options() }
                                </select>
                            </div>
                            <ul class="suggestions-list">
                                { app.render_popular_searches(ctx) }
                            </ul>
                        </div>

                        <div class="filter-section">
                            <h3 class="filter-title">{ "📈 排序方式" }</h3>
                            <select class="form-control" onchange={ctx.link().callback(|e: yew::events::Event| Msg::SortSelect(select_value(e)))} value={app.sort_value.clone()}>
                                { app.render_sort_options() }
                            </select>
                        </div>

                        <div class="filter-section">
                            <h3 class="filter-title">{ "⚙️ 结果限制" }</h3>
                            <div class="form-group" style="margin-bottom: 12px;">
                                <label style="font-size: 0.85rem; color: var(--text-secondary);">{ "每页最大结果数" }</label>
                                <input
                                    class="form-control"
                                    type="number"
                                    min="1"
                                    max="10000"
                                    value={app.max_results_per_page.to_string()}
                                    oninput={ctx.link().callback(|e: yew::events::InputEvent| Msg::UpdateMaxResults(input_value(e)))}
                                />
                                <div style="font-size: 0.75rem; color: var(--text-secondary); margin-top: 4px;">
                                    { "MeiliSearch 默认上限 1000，可自定义最大 10000" }
                                </div>
                            </div>
                        </div>
                    </aside>
                </div>
            </div>
        </div>
    }
}
