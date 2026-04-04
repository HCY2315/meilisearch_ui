use crate::App;
use yew::{html, Html};

/// Toast container component
pub fn render_toasts(app: &App) -> Html {
    html! {
        <div class="toast-container">
            { for app.toasts.iter().map(|toast| app.render_toast(toast)) }
        </div>
    }
}

/// Loading overlay component
pub fn render_loading(app: &App) -> Html {
    let loading_class = if app.loading {
        "loading-overlay active"
    } else {
        "loading-overlay"
    };
    html! {
        <div class={loading_class}>
            <div class="spinner"></div>
        </div>
    }
}
