use axum::{
    extract::{Path, State, Json, Query},
    http::StatusCode,
    Json as JsonResponse,
};
use crate::types::*;
use std::sync::{Arc, Mutex};

// db-covered scalar service endpoints (PLAN-013 T2)

#[derive(serde::Deserialize, Default)]
#[serde(default)]
pub struct MuxTabIdAtQuery {
    pub i: i64,
}

#[derive(serde::Deserialize, Default)]
#[serde(default)]
pub struct MuxTabIsActiveAtQuery {
    pub i: i64,
}

#[derive(serde::Deserialize, Default)]
#[serde(default)]
pub struct MuxTabTitleAtQuery {
    pub i: i64,
}

#[derive(serde::Deserialize, Default)]
#[serde(default)]
pub struct MuxPaneColsQuery {
    pub pane_id: i64,
}

#[derive(serde::Deserialize, Default)]
#[serde(default)]
pub struct MuxPaneRowsQuery {
    pub pane_id: i64,
}

#[derive(serde::Deserialize, Default)]
#[serde(default)]
pub struct MuxPaneCursorRowQuery {
    pub pane_id: i64,
}

#[derive(serde::Deserialize, Default)]
#[serde(default)]
pub struct MuxPaneCursorColQuery {
    pub pane_id: i64,
}

#[derive(serde::Deserialize, Default)]
#[serde(default)]
pub struct MuxSlotPaneIdQuery {
    pub slot: i64,
}

#[derive(serde::Deserialize, Default)]
#[serde(default)]
pub struct MuxSlotPaneKeyQuery {
    pub slot: i64,
}

pub async fn get_lines() -> JsonResponse<Vec<String>> {
    JsonResponse(crate::db::get_lines())
}

pub async fn tick_gate() -> JsonResponse<i64> {
    JsonResponse(crate::db::tick_gate())
}

pub async fn term_send(Json(body): Json<serde_json::Value>) -> axum::http::StatusCode {
    crate::db::term_send(body["line"].as_str().unwrap_or_default());
    StatusCode::OK
}

pub async fn term_interrupt() -> JsonResponse<i64> {
    JsonResponse(crate::db::term_interrupt())
}

pub async fn term_menu_take() -> JsonResponse<i64> {
    JsonResponse(crate::db::term_menu_take())
}

pub async fn term_pump_input() -> JsonResponse<i64> {
    JsonResponse(crate::db::term_pump_input())
}

pub async fn term_apply_resize() -> JsonResponse<i64> {
    JsonResponse(crate::db::term_apply_resize())
}

pub async fn term_cols() -> JsonResponse<i64> {
    JsonResponse(crate::db::term_cols())
}

pub async fn term_rows() -> JsonResponse<i64> {
    JsonResponse(crate::db::term_rows())
}

pub async fn term_cursor_row() -> JsonResponse<i64> {
    JsonResponse(crate::db::term_cursor_row())
}

pub async fn term_cursor_col() -> JsonResponse<i64> {
    JsonResponse(crate::db::term_cursor_col())
}

pub async fn term_backlog_sample() -> JsonResponse<i64> {
    JsonResponse(crate::db::term_backlog_sample())
}

pub async fn term_backlog_alerts_total() -> JsonResponse<i64> {
    JsonResponse(crate::db::term_backlog_alerts_total())
}

pub async fn term_backlog_pending_mb() -> JsonResponse<i64> {
    JsonResponse(crate::db::term_backlog_pending_mb())
}

pub async fn term_backlog_paused() -> JsonResponse<i64> {
    JsonResponse(crate::db::term_backlog_paused())
}

pub async fn term_backlog_dropped() -> JsonResponse<i64> {
    JsonResponse(crate::db::term_backlog_dropped())
}

pub async fn term_exited() -> JsonResponse<bool> {
    JsonResponse(crate::db::term_exited())
}

pub async fn mux_split(Json(body): Json<serde_json::Value>) -> JsonResponse<i64> {
    JsonResponse(crate::db::mux_split(body["axis"].as_i64().unwrap_or_default()))
}

pub async fn mux_close_pane(Json(body): Json<serde_json::Value>) -> JsonResponse<i64> {
    JsonResponse(crate::db::mux_close_pane(body["pane_id"].as_i64().unwrap_or_default()))
}

pub async fn mux_focus(Json(body): Json<serde_json::Value>) -> JsonResponse<i64> {
    JsonResponse(crate::db::mux_focus(body["pane_id"].as_i64().unwrap_or_default()))
}

pub async fn mux_focus_dir(Json(body): Json<serde_json::Value>) -> JsonResponse<i64> {
    JsonResponse(crate::db::mux_focus_dir(body["d"].as_i64().unwrap_or_default()))
}

pub async fn mux_zoom(Json(body): Json<serde_json::Value>) -> JsonResponse<i64> {
    JsonResponse(crate::db::mux_zoom(body["pane_id"].as_i64().unwrap_or_default()))
}

pub async fn mux_new_tab(Json(body): Json<serde_json::Value>) -> JsonResponse<i64> {
    JsonResponse(crate::db::mux_new_tab(body["profile"].as_str().unwrap_or_default()))
}

pub async fn term_profiles() -> JsonResponse<Vec<String>> {
    JsonResponse(crate::db::term_profiles())
}

pub async fn term_profile_names() -> JsonResponse<Vec<String>> {
    JsonResponse(crate::db::term_profile_names())
}

pub async fn mux_close_tab(Json(body): Json<serde_json::Value>) -> JsonResponse<i64> {
    JsonResponse(crate::db::mux_close_tab(body["tab_id"].as_i64().unwrap_or_default()))
}

pub async fn mux_activate_tab(Json(body): Json<serde_json::Value>) -> JsonResponse<i64> {
    JsonResponse(crate::db::mux_activate_tab(body["tab_id"].as_i64().unwrap_or_default()))
}

pub async fn mux_snapshot() -> JsonResponse<String> {
    JsonResponse(crate::db::mux_snapshot())
}

pub async fn mux_cols() -> JsonResponse<i64> {
    JsonResponse(crate::db::mux_cols())
}

pub async fn mux_rows() -> JsonResponse<i64> {
    JsonResponse(crate::db::mux_rows())
}

pub async fn mux_focus_key() -> JsonResponse<String> {
    JsonResponse(crate::db::mux_focus_key())
}

pub async fn mux_tabs() -> JsonResponse<Vec<String>> {
    JsonResponse(crate::db::mux_tabs())
}

pub async fn mux_tab_count() -> JsonResponse<i64> {
    JsonResponse(crate::db::mux_tab_count())
}

pub async fn mux_tab_id_at(Query(query): Query<MuxTabIdAtQuery>) -> JsonResponse<i64> {
    JsonResponse(crate::db::mux_tab_id_at(query.i))
}

pub async fn mux_tab_is_active_at(Query(query): Query<MuxTabIsActiveAtQuery>) -> JsonResponse<i64> {
    JsonResponse(crate::db::mux_tab_is_active_at(query.i))
}

pub async fn mux_tab_title_at(Query(query): Query<MuxTabTitleAtQuery>) -> JsonResponse<String> {
    JsonResponse(crate::db::mux_tab_title_at(query.i))
}

pub async fn mux_pane_lines(Json(body): Json<serde_json::Value>) -> JsonResponse<Vec<String>> {
    JsonResponse(crate::db::mux_pane_lines(body["pane_id"].as_i64().unwrap_or_default()))
}

pub async fn mux_pane_cols(Query(query): Query<MuxPaneColsQuery>) -> JsonResponse<i64> {
    JsonResponse(crate::db::mux_pane_cols(query.pane_id))
}

pub async fn mux_pane_rows(Query(query): Query<MuxPaneRowsQuery>) -> JsonResponse<i64> {
    JsonResponse(crate::db::mux_pane_rows(query.pane_id))
}

pub async fn mux_pane_cursor_row(Query(query): Query<MuxPaneCursorRowQuery>) -> JsonResponse<i64> {
    JsonResponse(crate::db::mux_pane_cursor_row(query.pane_id))
}

pub async fn mux_pane_cursor_col(Query(query): Query<MuxPaneCursorColQuery>) -> JsonResponse<i64> {
    JsonResponse(crate::db::mux_pane_cursor_col(query.pane_id))
}

pub async fn mux_visible_pane_count() -> JsonResponse<i64> {
    JsonResponse(crate::db::mux_visible_pane_count())
}

pub async fn mux_split_axis() -> JsonResponse<i64> {
    JsonResponse(crate::db::mux_split_axis())
}

pub async fn mux_slot_pane_id(Query(query): Query<MuxSlotPaneIdQuery>) -> JsonResponse<i64> {
    JsonResponse(crate::db::mux_slot_pane_id(query.slot))
}

pub async fn mux_slot_pane_key(Query(query): Query<MuxSlotPaneKeyQuery>) -> JsonResponse<String> {
    JsonResponse(crate::db::mux_slot_pane_key(query.slot))
}

pub async fn mux_focus_id() -> JsonResponse<i64> {
    JsonResponse(crate::db::mux_focus_id())
}

pub async fn mux_zoom_active() -> JsonResponse<i64> {
    JsonResponse(crate::db::mux_zoom_active())
}

pub async fn mux_layout() -> JsonResponse<String> {
    JsonResponse(crate::db::mux_layout())
}

pub async fn mux_enqueue(Json(body): Json<serde_json::Value>) -> JsonResponse<i64> {
    JsonResponse(crate::db::mux_enqueue(body["code"].as_i64().unwrap_or_default(), body["arg"].as_i64().unwrap_or_default()))
}

pub async fn mux_rect_kind(Json(body): Json<serde_json::Value>) -> JsonResponse<i64> {
    JsonResponse(crate::db::mux_rect_kind(body["k"].as_i64().unwrap_or_default()))
}

pub async fn mux_rect_pane(Json(body): Json<serde_json::Value>) -> JsonResponse<i64> {
    JsonResponse(crate::db::mux_rect_pane(body["k"].as_i64().unwrap_or_default()))
}

pub async fn mux_rect_key(Json(body): Json<serde_json::Value>) -> JsonResponse<String> {
    JsonResponse(crate::db::mux_rect_key(body["k"].as_i64().unwrap_or_default()))
}

pub async fn mux_rect_branch(Json(body): Json<serde_json::Value>) -> JsonResponse<i64> {
    JsonResponse(crate::db::mux_rect_branch(body["k"].as_i64().unwrap_or_default()))
}

pub async fn mux_rect_axis(Json(body): Json<serde_json::Value>) -> JsonResponse<i64> {
    JsonResponse(crate::db::mux_rect_axis(body["k"].as_i64().unwrap_or_default()))
}

pub async fn mux_rect_x(Json(body): Json<serde_json::Value>) -> JsonResponse<i64> {
    JsonResponse(crate::db::mux_rect_x(body["k"].as_i64().unwrap_or_default()))
}

pub async fn mux_rect_y(Json(body): Json<serde_json::Value>) -> JsonResponse<i64> {
    JsonResponse(crate::db::mux_rect_y(body["k"].as_i64().unwrap_or_default()))
}

pub async fn mux_rect_w(Json(body): Json<serde_json::Value>) -> JsonResponse<i64> {
    JsonResponse(crate::db::mux_rect_w(body["k"].as_i64().unwrap_or_default()))
}

pub async fn mux_rect_h(Json(body): Json<serde_json::Value>) -> JsonResponse<i64> {
    JsonResponse(crate::db::mux_rect_h(body["k"].as_i64().unwrap_or_default()))
}

pub async fn mux_layout_version() -> JsonResponse<i64> {
    JsonResponse(crate::db::mux_layout_version())
}

pub async fn mux_window_width() -> JsonResponse<i64> {
    JsonResponse(crate::db::mux_window_width())
}

pub async fn mux_window_height() -> JsonResponse<i64> {
    JsonResponse(crate::db::mux_window_height())
}

pub async fn mux_resize_pane(Json(body): Json<serde_json::Value>) -> JsonResponse<i64> {
    JsonResponse(crate::db::mux_resize_pane(body["pane_id"].as_i64().unwrap_or_default(), body["ratio"].as_i64().unwrap_or_default()))
}

pub async fn mux_resize_branch(Json(body): Json<serde_json::Value>) -> JsonResponse<i64> {
    JsonResponse(crate::db::mux_resize_branch(body["branch_id"].as_i64().unwrap_or_default(), body["px"].as_i64().unwrap_or_default(), body["py"].as_i64().unwrap_or_default()))
}