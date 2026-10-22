//! Grupr desktop (Tauri 2).
//!
//! The app is app.grupr.ai in a native window: no second front end to keep
//! in step with the web app, nothing the web app does not already do. What
//! the shell adds is what a browser tab cannot be:
//!
//! - one window that stays put: closing it hides it, the tray brings it back,
//!   a second launch focuses the first (single instance);
//! - deep links: `grupr://g/<room>`, `grupr://agents`, `grupr://agents/waiting`
//!   open the app on that page (the web app can emit them in emails and
//!   webhooks later);
//! - links that leave Grupr open in the system browser, so the window never
//!   turns into a general-purpose browser;
//! - the window remembers its size and position.
//!
//! The web page gets no Tauri IPC (no `remote` capability): it runs exactly as
//! it does in a browser. Notifications and badges come in a later version
//! through a small, declared bridge.

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Url, WebviewUrl, WebviewWindowBuilder,
};
use tauri_plugin_deep_link::DeepLinkExt;

/// The web app this shell wraps. Override with GRUPR_APP_URL for staging.
fn app_url() -> String {
    std::env::var("GRUPR_APP_URL").unwrap_or_else(|_| "https://app.grupr.ai".to_string())
}

/// Hosts that stay inside the window; everything else opens in the system
/// browser. GitHub is here so the GitHub App install round-trip completes.
fn stays_inside(url: &Url) -> bool {
    let host = url.host_str().unwrap_or("");
    host == "grupr.ai"
        || host.ends_with(".grupr.ai")
        || host == "github.com"
        || host.ends_with(".github.com")
        || host == "localhost"
        || host == "127.0.0.1"
}

fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

/// `grupr://g/<id>?x=1` -> `https://app.grupr.ai/g/<id>?x=1`. The scheme's
/// "host" is the first path segment of the web app.
fn target_for(link: &str) -> Option<String> {
    let u = Url::parse(link).ok()?;
    if u.scheme() != "grupr" {
        return None;
    }
    let mut path = String::new();
    if let Some(host) = u.host_str() {
        if !host.is_empty() {
            path.push('/');
            path.push_str(host);
        }
    }
    path.push_str(u.path());
    if let Some(q) = u.query() {
        path.push('?');
        path.push_str(q);
    }
    if path.is_empty() {
        path.push('/');
    }
    Some(format!("{}{}", app_url(), path))
}

fn navigate(app: &AppHandle, link: &str) {
    if let Some(target) = target_for(link) {
        if let Some(w) = app.get_webview_window("main") {
            // location.assign keeps the web app's own routing in charge.
            let js = format!("window.location.assign({})", serde_json::to_string(&target).unwrap_or_default());
            let _ = w.eval(&js);
        }
    }
    show_main(app);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            // A second launch, or a deep link handed to a new process on
            // Windows/Linux, lands here: route it to the running window.
            for a in args.iter().skip(1) {
                if a.starts_with("grupr://") {
                    navigate(app, a);
                    return;
                }
            }
            show_main(app);
        }))
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|app| {
            let start = Url::parse(&app_url())?;
            let window = WebviewWindowBuilder::new(app, "main", WebviewUrl::External(start))
                .title("Grupr")
                .inner_size(1280.0, 840.0)
                .min_inner_size(900.0, 600.0)
                .on_navigation(|url| {
                    let web = url.scheme() == "http" || url.scheme() == "https";
                    if web && !stays_inside(url) {
                        let _ = tauri_plugin_opener::open_url(url.as_str(), None::<&str>);
                        return false;
                    }
                    true
                })
                .build()?;

            // Closing the window keeps the app (and its tray) alive.
            let hide_on_close = window.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = hide_on_close.hide();
                }
            });

            // Deep links delivered to the running process (macOS, and
            // Windows/Linux when the OS reuses the instance).
            let for_links = app.handle().clone();
            app.deep_link().on_open_url(move |event| {
                for u in event.urls() {
                    navigate(&for_links, u.as_str());
                }
            });
            #[cfg(any(target_os = "linux", all(debug_assertions, windows)))]
            app.deep_link().register_all()?;

            // Tray: open, two shortcuts, quit.
            let open = MenuItem::with_id(app, "open", "Open Grupr", true, None::<&str>)?;
            let agents = MenuItem::with_id(app, "agents", "Agents", true, None::<&str>)?;
            let waiting = MenuItem::with_id(app, "waiting", "Waiting on you", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit Grupr", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open, &agents, &waiting, &quit])?;
            let mut tray = TrayIconBuilder::with_id("grupr-tray")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .tooltip("Grupr")
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "open" => show_main(app),
                    "agents" => navigate(app, "grupr://agents"),
                    "waiting" => navigate(app, "grupr://agents/waiting"),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        show_main(tray.app_handle());
                    }
                });
            if let Some(icon) = app.default_window_icon() {
                tray = tray.icon(icon.clone());
            }
            tray.build(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Grupr");
}

#[cfg(test)]
mod tests {
    use super::target_for;

    #[test]
    fn deep_links_map_to_app_paths() {
        assert_eq!(target_for("grupr://g/abc?x=1").unwrap(), "https://app.grupr.ai/g/abc?x=1");
        assert_eq!(target_for("grupr://agents/waiting").unwrap(), "https://app.grupr.ai/agents/waiting");
        assert_eq!(target_for("grupr://").unwrap(), "https://app.grupr.ai/");
        assert!(target_for("https://evil.example/").is_none());
    }
}
