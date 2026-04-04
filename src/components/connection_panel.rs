use crate::{input_value, select_value, App, Msg};
use yew::{html, Context, Html};

/// Connection panel component - renders server connection configuration UI
pub fn render_connection_panel(app: &App, ctx: &Context<App>) -> Html {
    html! {
        <section class="connection-panel">
            <div class="panel-header">
                <h2 class="panel-title">{ "🔌 服务器连接配置" }</h2>
                <button class="btn btn-primary" onclick={ctx.link().callback(|_| Msg::Connect)}>{ "连接服务器" }</button>
            </div>
            <div class="form-grid">
                <div class="form-group">
                    <label>{ "服务器地址" }</label>
                    <input
                        class="form-control"
                        value={app.host_input.clone()}
                        oninput={ctx.link().callback(|e: yew::events::InputEvent| Msg::SetHost(input_value(e)))}
                        placeholder="http://192.168.2.27:7700"
                    />
                </div>
                <div class="form-group">
                    <label>{ "API 密钥" }</label>
                    <input
                        class="form-control"
                        type="password"
                        value={app.api_key_input.clone()}
                        oninput={ctx.link().callback(|e: yew::events::InputEvent| Msg::SetApiKey(input_value(e)))}
                        placeholder="请输入API密钥"
                    />
                </div>
                <div class="form-group">
                    <label>{ "选择索引" }</label>
                    <select
                        class="form-control"
                        onchange={ctx.link().callback(|e: yew::events::Event| Msg::SelectIndex(select_value(e)))}
                        value={app.current_index.clone()}
                    >
                        { app.render_index_options() }
                    </select>
                </div>
            </div>
        </section>
    }
}
