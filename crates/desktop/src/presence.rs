//! Presence calendar (port of `presence-calendar.tsx`,
//! `presence-day-dialog.tsx` and `use-day-clipboard.ts`): the month grid, the
//! two-step day dialog (type, then commute) and the copy / paste of days.

use basic_presence_core::application::dto::commute_dto::CommuteDto;
use basic_presence_core::application::dto::emission_factor_dto::EmissionFactorDto;
use basic_presence_core::application::dto::presence_dto::PresenceDto;
use basic_presence_core::application::dto::profile_settings_dto::ProfileSettingsDto;
use basic_presence_core::application::dto::trip_dto::TripInputDto;
use basic_presence_core::application::view::format::{
    Lang, capitalize, format_co2, format_date, format_long_date, format_minutes, format_month_year,
    weekday_initials,
};
use basic_presence_core::application::view::period_stats::{civil_date, day_ms, month_totals};
use basic_presence_core::domain::entities::presence::PresenceType;
use chrono::{Datelike, Days, Months, NaiveDate};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

use crate::context::Ctx;
use crate::errors::UiError;
use crate::i18n::{lookup, tr};
use crate::{
    AppState, AppWindow, CommuteChoice, DayCell, ModeOption, Page, PresenceState, SegmentDraft,
};

/// Display order of the transport categories in the mode picker.
const CATEGORY_ORDER: [&str; 6] = ["car", "active", "public_transport", "rail", "air", "other"];

/// Copy / paste of days.
#[derive(Debug, Clone, Default, PartialEq)]
pub enum Clipboard {
    #[default]
    Idle,
    /// Waiting for the source day.
    Picking,
    /// A day is copied; clicked days receive it.
    Pasting { source_id: String },
}

/// A commute leg being edited, as typed.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Draft {
    pub mode_id: String,
    pub distance: String,
    pub occupants: String,
}

/// The open day dialog.
#[derive(Debug, Clone)]
pub struct DayDialog {
    pub day: i64,
    pub presence: Option<PresenceDto>,
    pub commute_step: bool,
    pub kind: PresenceType,
    /// `"none"`, `"custom"` or a saved commute id; `None` = nothing chosen.
    pub choice: Option<String>,
    pub segments: Vec<Draft>,
    pub round_trip: bool,
}

/// Data behind the calendar (per active profile).
#[derive(Default)]
pub struct PresenceData {
    /// First day of the displayed month (`None` until the first render).
    pub month: Option<NaiveDate>,
    pub presences: Vec<PresenceDto>,
    pub settings: Option<ProfileSettingsDto>,
    pub commutes: Vec<CommuteDto>,
    pub factors: Vec<EmissionFactorDto>,
    pub clipboard: Clipboard,
    pub overwrite_approved: bool,
    pub pending_paste: Option<i64>,
    pub dialog: Option<DayDialog>,
    /// Day whose hours are being encoded (work-hours page, S7).
    pub work_day: Option<i64>,
}

// ---------------------------------------------------------------- pure logic

fn today() -> NaiveDate {
    chrono::Local::now().date_naive()
}

fn first_of_month(d: NaiveDate) -> NaiveDate {
    d.with_day(1).unwrap_or(d)
}

/// The visible grid: whole weeks, Monday first, around `month`.
pub fn grid_days(month: NaiveDate) -> Vec<NaiveDate> {
    let first = first_of_month(month);
    let start = first - Days::new(first.weekday().num_days_from_monday().into());
    let last = (first + Months::new(1)) - Days::new(1);
    let end = last + Days::new((6 - last.weekday().num_days_from_monday()).into());
    start.iter_days().take_while(|d| *d <= end).collect()
}

fn type_label(lang: Lang, kind: &str) -> String {
    tr(lang, &format!("presence.types.{kind}"))
}

/// Localized transport-mode name, falling back to the backend label.
pub fn mode_label(lang: Lang, mode_id: &str, fallback: &str) -> String {
    lookup(lang, &format!("commute.modes.{mode_id}"))
        .map(str::to_string)
        .unwrap_or_else(|| fallback.to_string())
}

/// `"30 km Intercity train + 5 km Bicycle"` (port of `summarizeSegments`).
pub fn summarize(commute: &CommuteDto, factors: &[EmissionFactorDto], lang: Lang) -> String {
    commute
        .segments
        .iter()
        .map(|s| {
            let fallback = factors
                .iter()
                .find(|f| f.mode_id == s.mode_id)
                .map_or(s.mode_id.as_str(), |f| f.label.as_str());
            format!(
                "{} km {}",
                s.distance_km,
                mode_label(lang, &s.mode_id, fallback)
            )
        })
        .collect::<Vec<_>>()
        .join(" + ")
}

/// Mode picker entries: a header per category, then its modes.
pub fn mode_options(factors: &[EmissionFactorDto], lang: Lang) -> Vec<ModeOption> {
    let category = |f: &EmissionFactorDto| {
        if CATEGORY_ORDER.contains(&f.category.as_str()) {
            f.category.clone()
        } else {
            "other".to_string()
        }
    };
    let mut out = Vec::new();
    for cat in CATEGORY_ORDER {
        let modes: Vec<_> = factors.iter().filter(|f| category(f) == cat).collect();
        if modes.is_empty() {
            continue;
        }
        out.push(ModeOption {
            id: SharedString::new(),
            label: tr(lang, &format!("commute.categories.{cat}")).into(),
            header: true,
        });
        out.extend(modes.into_iter().map(|f| ModeOption {
            id: f.mode_id.clone().into(),
            label: mode_label(lang, &f.mode_id, &f.label).into(),
            header: false,
        }));
    }
    out
}

fn parse_number(text: &str) -> Option<f64> {
    text.trim()
        .replace(',', ".")
        .parse::<f64>()
        .ok()
        .filter(|v| v.is_finite())
}

/// Trip legs of a custom trip: complete legs only; occupants default to 1.
pub fn custom_trips(segments: &[Draft], round_trip: bool) -> Vec<TripInputDto> {
    segments
        .iter()
        .filter_map(|s| {
            let distance = parse_number(&s.distance)?;
            (!s.mode_id.is_empty() && distance > 0.0).then(|| TripInputDto {
                mode_id: s.mode_id.clone(),
                distance_km: distance,
                round_trip,
                occupants: Some(
                    s.occupants
                        .trim()
                        .parse::<i64>()
                        .ok()
                        .filter(|n| *n > 0)
                        .unwrap_or(1),
                ),
            })
        })
        .collect()
}

impl DayDialog {
    fn trips(&self, commutes: &[CommuteDto]) -> Vec<TripInputDto> {
        match self.choice.as_deref() {
            Some("custom") => custom_trips(&self.segments, self.round_trip),
            Some("none") | None => Vec::new(),
            Some(id) => commutes
                .iter()
                .find(|c| c.id == id)
                .map(CommuteDto::to_trips)
                .unwrap_or_default(),
        }
    }

    fn can_confirm(&self, commutes: &[CommuteDto]) -> bool {
        match self.choice.as_deref() {
            Some("none") => true,
            Some("custom") => !custom_trips(&self.segments, self.round_trip).is_empty(),
            Some(id) => commutes.iter().any(|c| c.id == id),
            None => false,
        }
    }
}

// ---------------------------------------------------------------- rendering

fn set_error(ui: &AppWindow, error: Option<UiError>) {
    let state = ui.global::<PresenceState>();
    let error = error.unwrap_or(UiError {
        code: "",
        arg: String::new(),
    });
    state.set_error_code(error.code.into());
    state.set_error_arg(error.arg.into());
}

/// Push the month grid, the header totals and the dialog to the UI.
pub fn render(ctx: &Ctx, ui: &AppWindow) {
    let mut shared = ctx.shared();
    let lang = shared.lang;
    let data = &mut shared.presence;
    let month = *data.month.get_or_insert_with(|| first_of_month(today()));
    let state = ui.global::<PresenceState>();

    let show_hours = data
        .settings
        .as_ref()
        .is_some_and(|s| s.cell_display_mode == "hours");
    state.set_show_hours(show_hours);
    state.set_month_title(capitalize(&format_month_year(month, lang)).into());
    let weekdays: Vec<SharedString> = weekday_initials(lang).into_iter().map(Into::into).collect();
    state.set_weekdays(ModelRc::new(VecModel::from(weekdays)));

    let today = today();
    let cells: Vec<DayCell> = grid_days(month)
        .into_iter()
        .map(|date| {
            let key = day_ms(date);
            let presence = data.presences.iter().find(|p| p.day == key);
            let kind = presence.map(|p| p.kind.clone()).unwrap_or_default();
            let value = match presence {
                Some(p) if show_hours && p.work_minutes > 0 => format_minutes(p.work_minutes),
                Some(p) if !show_hours => match p.co2_kg {
                    Some(co2) if co2 > 0.0 => {
                        format!(
                            "{}{}",
                            format_co2(co2, lang),
                            if p.is_estimated { " ~" } else { "" }
                        )
                    }
                    _ => String::new(),
                },
                _ => String::new(),
            };
            let mut label = format_date(date, lang);
            if !kind.is_empty() {
                label = format!("{label} — {}", type_label(lang, &kind));
            }
            DayCell {
                key: key.to_string().into(),
                day: date.day() as i32,
                in_month: date.month() == month.month(),
                today: date == today,
                weekend: date.weekday().num_days_from_monday() >= 5,
                kind: kind.into(),
                value: value.into(),
                label: label.into(),
            }
        })
        .collect();
    state.set_cells(ModelRc::new(VecModel::from(cells)));

    let totals = month_totals(&data.presences, month.year(), month.month());
    state.set_month_hours(format_minutes(totals.work_minutes).into());
    state.set_month_co2(format_co2(totals.co2_kg, lang).into());

    state.set_clipboard(
        match data.clipboard {
            Clipboard::Idle => "idle",
            Clipboard::Picking => "picking",
            Clipboard::Pasting { .. } => "pasting",
        }
        .into(),
    );

    // Day dialog.
    state.set_dialog_open(data.dialog.is_some());
    if let Some(dialog) = &data.dialog {
        let date = civil_date(dialog.day);
        state.set_dialog_title(capitalize(&format_long_date(date, lang)).into());
        state.set_dialog_step(
            if dialog.commute_step {
                "commute"
            } else {
                "type"
            }
            .into(),
        );
        state.set_dialog_kind(
            dialog
                .presence
                .as_ref()
                .map(|p| p.kind.clone())
                .unwrap_or_default()
                .into(),
        );
        state.set_dialog_choice(dialog.choice.clone().unwrap_or_default().into());
        state.set_round_trip(dialog.round_trip);
        state.set_can_confirm(dialog.can_confirm(&data.commutes));
        let choices: Vec<CommuteChoice> = data
            .commutes
            .iter()
            .map(|c| CommuteChoice {
                id: c.id.clone().into(),
                name: c.name.clone().into(),
                co2: c
                    .co2_kg
                    .map(|v| format_co2(v, lang))
                    .unwrap_or_default()
                    .into(),
                summary: summarize(c, &data.factors, lang).into(),
            })
            .collect();
        state.set_commutes(ModelRc::new(VecModel::from(choices)));
        state.set_modes(ModelRc::new(VecModel::from(mode_options(
            &data.factors,
            lang,
        ))));
        render_segments(ui, dialog, &data.factors, lang);
    }
}

fn render_segments(ui: &AppWindow, dialog: &DayDialog, factors: &[EmissionFactorDto], lang: Lang) {
    let drafts: Vec<SegmentDraft> = dialog
        .segments
        .iter()
        .map(|s| {
            let factor = factors.iter().find(|f| f.mode_id == s.mode_id);
            SegmentDraft {
                mode_id: s.mode_id.clone().into(),
                mode_label: factor
                    .map(|f| mode_label(lang, &f.mode_id, &f.label))
                    .unwrap_or_default()
                    .into(),
                distance: s.distance.clone().into(),
                occupants: s.occupants.clone().into(),
                is_car: factor.is_some_and(|f| f.category == "car"),
            }
        })
        .collect();
    ui.global::<PresenceState>()
        .set_segments(ModelRc::new(VecModel::from(drafts)));
}

// ---------------------------------------------------------------- backend

/// (Re)load the active profile's presences, settings, commutes and factors.
pub fn reload(ctx: &Ctx) {
    let Some(profile_id) = ctx.shared().profile_id.clone() else {
        return;
    };
    let c = ctx.clone();
    ctx.call_authed(
        move |s| async move {
            let presences = s.list_presences.execute(&profile_id).await?;
            let settings = s.get_profile_settings.execute(&profile_id).await?;
            let commutes = s.list_commutes.execute(&profile_id).await?;
            let factors = s.list_emission_factors.execute().await?;
            Ok((profile_id, presences, settings, commutes, factors))
        },
        move |ui, result| match result {
            Ok((profile_id, presences, settings, commutes, factors)) => {
                {
                    let mut shared = c.shared();
                    // A profile switch while loading: drop the stale result.
                    if shared.profile_id.as_deref() != Some(profile_id.as_str()) {
                        return;
                    }
                    let data = &mut shared.presence;
                    data.presences = presences;
                    data.settings = Some(settings);
                    data.commutes = commutes;
                    data.factors = factors;
                }
                render(&c, ui);
            }
            Err(e) => set_error(ui, Some(e)),
        },
    );
}

/// The active profile changed (or was first known): show its data.
pub fn set_profile(ctx: &Ctx, profile_id: Option<String>) {
    {
        let mut shared = ctx.shared();
        if shared.profile_id == profile_id {
            return;
        }
        shared.profile_id = profile_id;
        let month = shared.presence.month;
        shared.presence = PresenceData {
            month,
            ..PresenceData::default()
        };
    }
    if let Some(ui) = ctx.ui.upgrade() {
        render(ctx, &ui);
    }
    reload(ctx);
}

/// Forget everything (sign-out).
pub fn reset(ctx: &Ctx, ui: &AppWindow) {
    {
        let mut shared = ctx.shared();
        shared.profile_id = None;
        shared.presence = PresenceData::default();
    }
    let state = ui.global::<PresenceState>();
    state.set_dialog_open(false);
    state.set_paste_confirm_open(false);
    state.set_busy(false);
    set_error(ui, None);
    render(ctx, ui);
}

/// Run a write, then close the dialog and reload the month.
fn write<Fut>(
    ctx: &Ctx,
    work: impl FnOnce(std::sync::Arc<basic_presence_core::services::Services>) -> Fut + Send + 'static,
) where
    Fut: std::future::Future<Output = Result<(), basic_presence_core::domain::error::DomainError>>
        + Send
        + 'static,
{
    let Some(ui) = ctx.ui.upgrade() else { return };
    ui.global::<PresenceState>().set_busy(true);
    set_error(&ui, None);
    let c = ctx.clone();
    ctx.call_authed(work, move |ui, result| {
        ui.global::<PresenceState>().set_busy(false);
        match result {
            Ok(()) => {
                c.shared().presence.dialog = None;
                render(&c, ui);
                reload(&c);
            }
            Err(e) => set_error(ui, Some(e)),
        }
    });
}

fn profile_id(ctx: &Ctx) -> Option<String> {
    ctx.shared().profile_id.clone()
}

fn set_presence(ctx: &Ctx, day: i64, kind: PresenceType, trips: Vec<TripInputDto>) {
    let Some(profile_id) = profile_id(ctx) else {
        return;
    };
    write(ctx, move |s| async move {
        s.set_presence
            .execute(&profile_id, day, kind.as_str(), trips)
            .await
            .map(|_| ())
    });
}

fn paste_onto(ctx: &Ctx, day: i64) {
    let Clipboard::Pasting { source_id } = ctx.shared().presence.clipboard.clone() else {
        return;
    };
    write(ctx, move |s| async move {
        s.copy_day.execute(&source_id, day).await.map(|_| ())
    });
}

fn open_dialog(ctx: &Ctx, day: i64) {
    let presence = {
        let mut shared = ctx.shared();
        let presence = shared
            .presence
            .presences
            .iter()
            .find(|p| p.day == day)
            .cloned();
        shared.presence.dialog = Some(DayDialog {
            day,
            kind: presence
                .as_ref()
                .and_then(|p| PresenceType::parse(&p.kind).ok())
                .unwrap_or(PresenceType::Office),
            presence: presence.clone(),
            commute_step: false,
            choice: None,
            segments: vec![Draft::default()],
            round_trip: true,
        });
        presence
    };
    if let Some(ui) = ctx.ui.upgrade() {
        set_error(&ui, None);
        render(ctx, &ui);
    }

    // Pre-fill the custom trip from an already-encoded office / remote day.
    let Some(presence) = presence.filter(|p| matches!(p.kind.as_str(), "office" | "remote")) else {
        return;
    };
    let c = ctx.clone();
    ctx.call_authed(
        move |s| async move { s.get_presence_trips.execute(&presence.id).await },
        move |ui, result| {
            let Ok(trips) = result else { return };
            if trips.is_empty() {
                return;
            }
            {
                let mut shared = c.shared();
                let Some(dialog) = shared.presence.dialog.as_mut().filter(|d| d.day == day) else {
                    return;
                };
                dialog.segments = trips
                    .iter()
                    .map(|t| Draft {
                        mode_id: t.mode_id.clone(),
                        distance: t.distance_km.to_string(),
                        occupants: t.occupants.to_string(),
                    })
                    .collect();
                dialog.round_trip = trips[0].round_trip;
                dialog.choice = Some("custom".into());
            }
            render(&c, ui);
        },
    );
}

/// A day of the grid was clicked (`key` = epoch-ms at UTC midnight).
fn day_clicked(ctx: &Ctx, key: &str) {
    let Ok(day) = key.parse::<i64>() else { return };
    let (clipboard, filled, approved) = {
        let shared = ctx.shared();
        let data = &shared.presence;
        let filled = data.presences.iter().find(|p| p.day == day).cloned();
        (data.clipboard.clone(), filled, data.overwrite_approved)
    };
    match clipboard {
        Clipboard::Idle => open_dialog(ctx, day),
        // Copy a fully-encoded day; empty days have nothing to copy.
        Clipboard::Picking => {
            if let Some(source) = filled {
                let mut shared = ctx.shared();
                shared.presence.clipboard = Clipboard::Pasting {
                    source_id: source.id,
                };
                // A fresh source day re-requires the overwrite confirmation.
                shared.presence.overwrite_approved = false;
                drop(shared);
                if let Some(ui) = ctx.ui.upgrade() {
                    render(ctx, &ui);
                }
            }
        }
        // Empty days paste at once; the first filled day asks first.
        Clipboard::Pasting { .. } => {
            if filled.is_none() || approved {
                paste_onto(ctx, day);
            } else {
                ctx.shared().presence.pending_paste = Some(day);
                if let Some(ui) = ctx.ui.upgrade() {
                    ui.global::<PresenceState>().set_paste_confirm_open(true);
                }
            }
        }
    }
}

fn disarm(ctx: &Ctx, ui: &AppWindow) {
    {
        let mut shared = ctx.shared();
        shared.presence.clipboard = Clipboard::Idle;
        shared.presence.overwrite_approved = false;
        shared.presence.pending_paste = None;
    }
    ui.global::<PresenceState>().set_paste_confirm_open(false);
    render(ctx, ui);
}

/// Edit the open dialog, then re-render it (and only the segment rows when
/// `segments_only`, so text fields being typed in keep their state).
fn edit_dialog(ctx: &Ctx, segments_only: bool, f: impl FnOnce(&mut DayDialog)) {
    let Some(ui) = ctx.ui.upgrade() else { return };
    let can_confirm = {
        let mut shared = ctx.shared();
        let data = &mut shared.presence;
        let Some(dialog) = data.dialog.as_mut() else {
            return;
        };
        f(dialog);
        dialog.can_confirm(&data.commutes)
    };
    if segments_only {
        ui.global::<PresenceState>().set_can_confirm(can_confirm);
    } else {
        render(ctx, &ui);
    }
}

pub fn wire(ctx: &Ctx) {
    let Some(ui) = ctx.ui.upgrade() else { return };
    let state = ui.global::<PresenceState>();

    let c = ctx.clone();
    state.on_prev_month(move || {
        if let Some(m) = c.shared().presence.month.as_mut() {
            *m = *m - Months::new(1);
        }
        if let Some(ui) = c.ui.upgrade() {
            render(&c, &ui);
        }
    });
    let c = ctx.clone();
    state.on_next_month(move || {
        if let Some(m) = c.shared().presence.month.as_mut() {
            *m = *m + Months::new(1);
        }
        if let Some(ui) = c.ui.upgrade() {
            render(&c, &ui);
        }
    });

    let c = ctx.clone();
    state.on_day_clicked(move |key| day_clicked(&c, &key));

    let c = ctx.clone();
    state.on_toggle_copy(move || {
        let Some(ui) = c.ui.upgrade() else { return };
        let idle = c.shared().presence.clipboard == Clipboard::Idle;
        if idle {
            c.shared().presence.clipboard = Clipboard::Picking;
            render(&c, &ui);
        } else {
            disarm(&c, &ui);
        }
    });
    let c = ctx.clone();
    state.on_cancel_copy(move || {
        if let Some(ui) = c.ui.upgrade() {
            disarm(&c, &ui);
        }
    });
    let c = ctx.clone();
    state.on_confirm_paste(move || {
        let pending = {
            let mut shared = c.shared();
            shared.presence.overwrite_approved = true;
            shared.presence.pending_paste.take()
        };
        if let Some(ui) = c.ui.upgrade() {
            ui.global::<PresenceState>().set_paste_confirm_open(false);
        }
        if let Some(day) = pending {
            paste_onto(&c, day);
        }
    });

    let c = ctx.clone();
    state.on_set_show_hours(move |hours| {
        let Some(profile_id) = profile_id(&c) else {
            return;
        };
        let Some(mut settings) = c.shared().presence.settings.clone() else {
            return;
        };
        settings.cell_display_mode = if hours { "hours" } else { "co2" }.into();
        let c2 = c.clone();
        c.call_authed(
            move |s| async move { s.set_profile_settings.execute(&profile_id, settings).await },
            move |ui, result| match result {
                Ok(settings) => {
                    c2.shared().presence.settings = Some(settings);
                    render(&c2, ui);
                }
                Err(e) => set_error(ui, Some(e)),
            },
        );
    });

    // ----- Day dialog -----
    let c = ctx.clone();
    state.on_close_dialog(move || {
        c.shared().presence.dialog = None;
        if let Some(ui) = c.ui.upgrade() {
            render(&c, &ui);
        }
    });

    let c = ctx.clone();
    state.on_select_type(move |kind| {
        let Ok(kind) = PresenceType::parse(&kind) else {
            return;
        };
        if kind.is_work() {
            // Remote defaults to "no trip" (working from home); keep a
            // pre-filled choice (the day's existing trips) when there is one.
            edit_dialog(&c, false, |d| {
                d.kind = kind;
                d.commute_step = true;
                if d.choice.is_none() && kind == PresenceType::Remote {
                    d.choice = Some("none".into());
                }
            });
        } else {
            // Vacation / holiday: saved at once (no commute).
            let day = c.shared().presence.dialog.as_ref().map(|d| d.day);
            if let Some(day) = day {
                set_presence(&c, day, kind, Vec::new());
            }
        }
    });

    let c = ctx.clone();
    state.on_choose(move |choice| {
        edit_dialog(&c, false, |d| d.choice = Some(choice.to_string()));
    });
    let c = ctx.clone();
    state.on_back(move || edit_dialog(&c, false, |d| d.commute_step = false));

    let c = ctx.clone();
    state.on_confirm(move || {
        let pending = {
            let shared = c.shared();
            let data = &shared.presence;
            data.dialog
                .as_ref()
                .filter(|d| d.can_confirm(&data.commutes))
                .map(|d| (d.day, d.kind, d.trips(&data.commutes)))
        };
        if let Some((day, kind, trips)) = pending {
            set_presence(&c, day, kind, trips);
        }
    });

    let c = ctx.clone();
    state.on_delete(move || {
        let id = c
            .shared()
            .presence
            .dialog
            .as_ref()
            .and_then(|d| d.presence.as_ref())
            .map(|p| p.id.clone());
        if let Some(id) = id {
            write(
                &c,
                move |s| async move { s.delete_presence.execute(&id).await },
            );
        }
    });

    let c = ctx.clone();
    state.on_encode_hours(move || {
        {
            let mut shared = c.shared();
            let day = shared.presence.dialog.take().map(|d| d.day);
            shared.presence.work_day = day;
        }
        if let Some(ui) = c.ui.upgrade() {
            render(&c, &ui);
            ui.global::<AppState>().set_page(Page::WorkHours);
        }
    });

    // ----- Custom trip legs -----
    let c = ctx.clone();
    state.on_set_segment_mode(move |i, id| {
        edit_dialog(&c, false, |d| {
            if let Some(s) = d.segments.get_mut(i as usize) {
                s.mode_id = id.to_string();
            }
        });
    });
    let c = ctx.clone();
    state.on_set_segment_distance(move |i, text| {
        edit_dialog(&c, true, |d| {
            if let Some(s) = d.segments.get_mut(i as usize) {
                s.distance = text.to_string();
            }
        });
    });
    let c = ctx.clone();
    state.on_set_segment_occupants(move |i, text| {
        edit_dialog(&c, true, |d| {
            if let Some(s) = d.segments.get_mut(i as usize) {
                s.occupants = text.to_string();
            }
        });
    });
    let c = ctx.clone();
    state.on_add_segment(move || edit_dialog(&c, false, |d| d.segments.push(Draft::default())));
    let c = ctx.clone();
    state.on_remove_segment(move |i| {
        edit_dialog(&c, false, |d| {
            if (i as usize) < d.segments.len() {
                d.segments.remove(i as usize);
            }
        });
    });
    let c = ctx.clone();
    state.on_set_round_trip(move |value| edit_dialog(&c, true, |d| d.round_trip = value));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn grid_spans_whole_weeks_monday_first() {
        // November 2026: Sunday the 1st → grid starts Monday Oct 26.
        let days = grid_days(date(2026, 11, 15));
        assert_eq!(days.first(), Some(&date(2026, 10, 26)));
        assert_eq!(days.last(), Some(&date(2026, 12, 6)));
        assert_eq!(days.len() % 7, 0);
        // February 2027 starts on a Monday, ends on a Sunday: 4 exact weeks.
        let days = grid_days(date(2027, 2, 1));
        assert_eq!(days.len(), 28);
        assert_eq!(days[0], date(2027, 2, 1));
    }

    #[test]
    fn custom_trips_keep_complete_legs() {
        let legs = vec![
            Draft {
                mode_id: "car_petrol".into(),
                distance: "12,5".into(),
                occupants: "".into(),
            },
            Draft {
                mode_id: "".into(),
                distance: "3".into(),
                occupants: "".into(),
            },
            Draft {
                mode_id: "bike".into(),
                distance: "0".into(),
                occupants: "".into(),
            },
            Draft {
                mode_id: "car_ev".into(),
                distance: "4".into(),
                occupants: "3".into(),
            },
        ];
        let trips = custom_trips(&legs, false);
        assert_eq!(trips.len(), 2);
        assert_eq!(trips[0].distance_km, 12.5);
        assert_eq!(trips[0].occupants, Some(1));
        assert!(!trips[0].round_trip);
        assert_eq!(trips[1].occupants, Some(3));
    }

    #[test]
    fn mode_labels_fall_back_to_the_backend_label() {
        assert_eq!(mode_label(Lang::Fr, "no_such_mode", "Backend"), "Backend");
        assert_ne!(mode_label(Lang::Fr, "bike", "Backend"), "Backend");
    }
}
