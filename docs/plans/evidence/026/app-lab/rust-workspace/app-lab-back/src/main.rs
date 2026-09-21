mod api;
mod types;
mod db;
async fn auto_media(
    method: axum::http::Method,
    uri: axum::http::Uri,
    headers: axum::http::HeaderMap,
) -> axum::response::Response {
    let mut response = auto_lang::ui::image_pipeline::media_http_response(
        auto_lang::ui::image_pipeline::global_media_registry(),
        method.as_str(),
        uri.path(),
        headers.get(axum::http::header::IF_NONE_MATCH).and_then(|value| value.to_str().ok()),
    );
    // A freshly queued rendition is expected to be pending for a short time.
    // Give the bounded worker a small grace window so browser <img> loads do
    // not turn a transient 503 into a terminal error callback.
    if response.status == 503 {
        tokio::time::sleep(std::time::Duration::from_millis(120)).await;
        response = auto_lang::ui::image_pipeline::media_http_response(
            auto_lang::ui::image_pipeline::global_media_registry(),
            method.as_str(),
            uri.path(),
            headers.get(axum::http::header::IF_NONE_MATCH).and_then(|value| value.to_str().ok()),
        );
    }
    let mut builder = axum::response::Response::builder().status(response.status);
    for (name, value) in response.headers {
        builder = builder.header(name, value);
    }
    builder
        .body(axum::body::Body::from(response.body.map(|bytes| bytes.to_vec()).unwrap_or_default()))
        .expect("media response is valid")
}
fn media_plain(status: u16, msg: String) -> axum::response::Response {
    axum::response::Response::builder()
        .status(status)
        .header("Content-Type", "text/plain; charset=utf-8")
        .body(axum::body::Body::from(msg))
        .expect("media plain response is valid")
}

fn media_json_str(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 2);
    o.push('"');
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

static MEDIA_INDEX: std::sync::OnceLock<auto_lang::ui::media_service::MediaIndex> =
    std::sync::OnceLock::new();

fn media_index() -> Option<&'static auto_lang::ui::media_service::MediaIndex> {
    let root = auto_lang::ui::media_service::resolve_root(None)?;
    Some(MEDIA_INDEX.get_or_init(|| {
        auto_lang::ui::media_service::index_directory(&root).unwrap_or_default()
    }))
}

async fn auto_media_scan() -> axum::response::Response {
    // PLAN-617 T-10: 三态如实回传 —— 根目录未配置 / 已配置但不存在 /
    // 正常。`root_missing` 让前端能把「目录不存在」与「目录为空」分开说；
    // 绝对路径本身不出后端（SD-02 安全边界）。
    let Some(root) = auto_lang::ui::media_service::resolve_root(None) else {
        // No configured root is an honest empty list, not a 500.
        return axum::response::Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from("{\"entries\":[],\"root_missing\":false}"))
            .expect("media scan response is valid");
    };
    if !root.exists() {
        return axum::response::Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from("{\"entries\":[],\"root_missing\":true}"))
            .expect("media scan response is valid");
    }
    let index = MEDIA_INDEX.get_or_init(|| {
        auto_lang::ui::media_service::index_directory(&root).unwrap_or_default()
    });
    let mut out = String::from("{\"entries\":[");
    for (i, e) in index.entries.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        let (artist, song_title) = auto_lang::ui::media_service::parse_artist_and_title(&e.name);
        let album = if e.rel_dir.is_empty() {
            "本地单曲".to_string()
        } else {
            e.rel_dir.clone()
        };
        let index_str = format!("{:02}", i + 1);
        out.push_str(&format!(
            "{{\"id\":{},\"index\":{},\"index_str\":{},\"title\":{},\"song_title\":{},\"artist\":{},\"album\":{},\"is_liked\":false,\"name\":{},\"rel_dir\":{},\"relative_path\":{},\"extension\":{},\"bytes\":{},\"size_str\":{},\"url\":\"/api/media/stream/{}\",\"audio_url\":\"/api/media/stream/{}\",\"video_url\":\"/api/media/stream/{}\"}}",
            media_json_str(&e.id),
            i + 1,
            media_json_str(&index_str),
            media_json_str(auto_lang::ui::media_service::display_title(&e.name)),
            media_json_str(&song_title),
            media_json_str(&artist),
            media_json_str(&album),
            media_json_str(&e.name),
            media_json_str(&e.rel_dir),
            media_json_str(&e.relative_path),
            media_json_str(&e.extension),
            e.bytes,
            media_json_str(&auto_lang::ui::media_service::human_size(e.bytes)),
            &e.id,
            &e.id,
            &e.id
        ));
    }
    out.push_str("],\"root_missing\":false}");
    axum::response::Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(out))
        .expect("media scan response is valid")
}

async fn auto_media_stream(
    axum::extract::Path(id): axum::extract::Path<String>,
    headers: axum::http::HeaderMap,
) -> axum::response::Response {
    use auto_lang::ui::media_service as ms;
    use tokio::io::{AsyncReadExt, AsyncSeekExt};

    let Some(index) = media_index() else {
        return media_plain(503, "no media root configured".to_string());
    };
    let Some(entry) = ms::find(index, &id) else {
        return media_plain(404, "unknown media id".to_string());
    };
    // Re-join under the index root: the caller only ever supplied a token.
    let Some(root) = auto_lang::ui::media_service::resolve_root(None) else {
        return media_plain(503, "no media root configured".to_string());
    };
    let path = ms::entry_path(&root, entry);
    let Ok(meta) = tokio::fs::metadata(&path).await else {
        return media_plain(404, "media file missing".to_string());
    };
    let len = meta.len();
    let range = headers
        .get(axum::http::header::RANGE)
        .and_then(|v| v.to_str().ok());
    let plan = ms::parse_range(range, len);

    if plan == ms::StreamPlan::Unsatisfiable {
        return axum::response::Response::builder()
            .status(416)
            .header("Content-Range", format!("bytes */{}", len))
            .header("Content-Length", "0")
            .body(axum::body::Body::empty())
            .expect("media 416 response is valid");
    }

    let (start, end) = match &plan {
        ms::StreamPlan::Full => (0u64, len.saturating_sub(1)),
        ms::StreamPlan::Partial { start, end } => (*start, *end),
        ms::StreamPlan::Unsatisfiable => unreachable!("handled above"),
    };

    let mut file = match tokio::fs::File::open(&path).await {
        Ok(f) => f,
        Err(_) => return media_plain(404, "media file unreadable".to_string()),
    };
    if start > 0 && file.seek(std::io::SeekFrom::Start(start)).await.is_err() {
        return media_plain(500, "seek failed".to_string());
    }
    // Bounded chunks: the whole (possibly multi-GB) file is never resident.
    let window = file.take(end - start + 1);
    let body = axum::body::Body::from_stream(tokio_util::io::ReaderStream::with_capacity(
        window,
        256 * 1024,
    ));

    let mut builder = axum::response::Response::builder()
        .status(plan.status())
        .header("Content-Type", ms::content_type(&entry.extension))
        .header("Accept-Ranges", "bytes")
        .header("Content-Length", (end - start + 1).to_string());
    if let ms::StreamPlan::Partial { start: s, end: e } = plan {
        builder = builder.header("Content-Range", ms::content_range(s, e, len));
    }
    builder.body(body).expect("media stream response is valid")
}

use tower_http::cors::{CorsLayer, Any};

#[tokio::main]
async fn main() {
    let port: u16 = std::env::var("AUTO_HTTP_PORT")
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(8080);
    let addr = format!("127.0.0.1:{}", port);
    println!("Server running on http://{}", addr);
    println!("CORS enabled for all origins");

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = axum::Router::new()
        .route("/api/term/tick", axum::routing::post(api::get_lines))
        .route("/api/term/tick-gate", axum::routing::post(api::tick_gate))
        .route("/api/term/send", axum::routing::post(api::term_send))
        .route("/api/term/interrupt", axum::routing::post(api::term_interrupt))
        .route("/api/term/menu-take", axum::routing::post(api::term_menu_take))
        .route("/api/term/pump", axum::routing::post(api::term_pump_input))
        .route("/api/term/apply-resize", axum::routing::post(api::term_apply_resize))
        .route("/api/term/cols", axum::routing::get(api::term_cols))
        .route("/api/term/rows", axum::routing::get(api::term_rows))
        .route("/api/term/cursor-row", axum::routing::get(api::term_cursor_row))
        .route("/api/term/cursor-col", axum::routing::get(api::term_cursor_col))
        .route("/api/term/backlog-sample", axum::routing::post(api::term_backlog_sample))
        .route("/api/term/backlog-alerts-total", axum::routing::get(api::term_backlog_alerts_total))
        .route("/api/term/backlog-pending-mb", axum::routing::get(api::term_backlog_pending_mb))
        .route("/api/term/backlog-paused", axum::routing::get(api::term_backlog_paused))
        .route("/api/term/backlog-dropped", axum::routing::get(api::term_backlog_dropped))
        .route("/api/term/exited", axum::routing::get(api::term_exited))
        .route("/api/mux/split", axum::routing::post(api::mux_split))
        .route("/api/mux/close-pane", axum::routing::post(api::mux_close_pane))
        .route("/api/mux/focus", axum::routing::post(api::mux_focus))
        .route("/api/mux/focus-dir", axum::routing::post(api::mux_focus_dir))
        .route("/api/mux/zoom", axum::routing::post(api::mux_zoom))
        .route("/api/mux/new-tab", axum::routing::post(api::mux_new_tab))
        .route("/api/term/profiles", axum::routing::get(api::term_profiles))
        .route("/api/term/profile-names", axum::routing::get(api::term_profile_names))
        .route("/api/mux/close-tab", axum::routing::post(api::mux_close_tab))
        .route("/api/mux/activate-tab", axum::routing::post(api::mux_activate_tab))
        .route("/api/mux/snapshot", axum::routing::get(api::mux_snapshot))
        .route("/api/mux/cols", axum::routing::get(api::mux_cols))
        .route("/api/mux/rows", axum::routing::get(api::mux_rows))
        .route("/api/mux/focus-key", axum::routing::get(api::mux_focus_key))
        .route("/api/mux/tabs", axum::routing::get(api::mux_tabs))
        .route("/api/mux/tab-count", axum::routing::get(api::mux_tab_count))
        .route("/api/mux/tab-id-at", axum::routing::get(api::mux_tab_id_at))
        .route("/api/mux/tab-is-active-at", axum::routing::get(api::mux_tab_is_active_at))
        .route("/api/mux/tab-title-at", axum::routing::get(api::mux_tab_title_at))
        .route("/api/mux/pane-lines", axum::routing::post(api::mux_pane_lines))
        .route("/api/mux/pane-cols", axum::routing::get(api::mux_pane_cols))
        .route("/api/mux/pane-rows", axum::routing::get(api::mux_pane_rows))
        .route("/api/mux/pane-cursor-row", axum::routing::get(api::mux_pane_cursor_row))
        .route("/api/mux/pane-cursor-col", axum::routing::get(api::mux_pane_cursor_col))
        .route("/api/mux/visible-pane-count", axum::routing::get(api::mux_visible_pane_count))
        .route("/api/mux/split-axis", axum::routing::get(api::mux_split_axis))
        .route("/api/mux/slot-pane-id", axum::routing::get(api::mux_slot_pane_id))
        .route("/api/mux/slot-pane-key", axum::routing::get(api::mux_slot_pane_key))
        .route("/api/mux/focus-id", axum::routing::get(api::mux_focus_id))
        .route("/api/mux/zoom-active", axum::routing::get(api::mux_zoom_active))
        .route("/api/mux/layout", axum::routing::get(api::mux_layout))
        .route("/api/mux/enqueue", axum::routing::post(api::mux_enqueue))
        .route("/api/mux/rect-kind", axum::routing::post(api::mux_rect_kind))
        .route("/api/mux/rect-pane", axum::routing::post(api::mux_rect_pane))
        .route("/api/mux/rect-key", axum::routing::post(api::mux_rect_key))
        .route("/api/mux/rect-branch", axum::routing::post(api::mux_rect_branch))
        .route("/api/mux/rect-axis", axum::routing::post(api::mux_rect_axis))
        .route("/api/mux/rect-x", axum::routing::post(api::mux_rect_x))
        .route("/api/mux/rect-y", axum::routing::post(api::mux_rect_y))
        .route("/api/mux/rect-w", axum::routing::post(api::mux_rect_w))
        .route("/api/mux/rect-h", axum::routing::post(api::mux_rect_h))
        .route("/api/mux/layout-version", axum::routing::get(api::mux_layout_version))
        .route("/api/mux/window-width", axum::routing::get(api::mux_window_width))
        .route("/api/mux/window-height", axum::routing::get(api::mux_window_height))
        .route("/api/mux/resize-pane", axum::routing::post(api::mux_resize_pane))
        .route("/api/mux/resize-branch", axum::routing::post(api::mux_resize_branch))
        .route("/api/__auto/media/{id}/{revision}", axum::routing::get(auto_media).head(auto_media))
        .route("/api/media/scan", axum::routing::get(auto_media_scan))
        .route("/api/media/stream/:id", axum::routing::get(auto_media_stream).head(auto_media_stream))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// rust_sidecar: 用户 .rs 侧车模块声明(PLAN-013 T2;勿手改)
mod term;
