use std::cell::RefCell;
use std::rc::Rc;

use memory_pak_core::{
    CollectionStats, EntryId, FilterBy, ItemKind, MemoryPakApp, QuerySnapshot, QuerySpec, RowView,
    SetItemNotesInput, SetItemStatusInput, SortKey,
};
use slint::{ComponentHandle, Model, ModelNotify, ModelRc, ModelTracker, SharedString, VecModel};

use crate::persistence::SaveStore;
use crate::services::{
    DocumentService, PlatformDocuments, PlatformUpdates, UpdateService, UpdateState,
};
use crate::{AppWindow, UiRow, UiStats};

struct QueryModel {
    app: Rc<RefCell<MemoryPakApp>>,
    snapshot: RefCell<QuerySnapshot>,
    notify: ModelNotify,
}

impl QueryModel {
    fn new(app: Rc<RefCell<MemoryPakApp>>, snapshot: QuerySnapshot) -> Self {
        Self {
            app,
            snapshot: RefCell::new(snapshot),
            notify: ModelNotify::default(),
        }
    }

    fn replace(&self, snapshot: QuerySnapshot) {
        *self.snapshot.borrow_mut() = snapshot;
        self.notify.reset();
    }

    fn row_for_id(&self, id: &str) -> Option<UiRow> {
        let app = self.app.borrow();
        self.snapshot
            .borrow()
            .items
            .iter()
            .copied()
            .find_map(|reference| {
                let row = app.row_view(reference)?;
                (row_id(&row) == id).then(|| into_ui_row(row))
            })
    }

    fn notify_id(&self, id: &str) {
        let app = self.app.borrow();
        let archive = &app.catalog().archive;
        let (kind, ordinal) = if id.starts_with("console:") {
            (
                ItemKind::Console,
                archive
                    .consoles
                    .binary_search_by(|record| archive.string(record.id).cmp(id)),
            )
        } else if id.starts_with("game:") {
            (
                ItemKind::Game,
                archive
                    .games
                    .binary_search_by(|record| archive.string(record.id).cmp(id)),
            )
        } else {
            (
                ItemKind::Collectible,
                archive
                    .collectibles
                    .binary_search_by(|record| archive.string(record.id).cmp(id)),
            )
        };
        let Ok(ordinal) = ordinal else {
            return;
        };
        let reference = memory_pak_core::CatalogRef {
            kind,
            ordinal: ordinal as u32,
        };
        drop(app);
        if let Some(row) = self
            .snapshot
            .borrow()
            .items
            .iter()
            .position(|item| *item == reference)
        {
            self.notify.row_changed(row);
        }
    }
}

impl Model for QueryModel {
    type Data = UiRow;

    fn row_count(&self) -> usize {
        self.snapshot.borrow().items.len()
    }

    fn row_data(&self, row: usize) -> Option<Self::Data> {
        let reference = *self.snapshot.borrow().items.get(row)?;
        self.app.borrow().row_view(reference).map(into_ui_row)
    }

    fn model_tracker(&self) -> &dyn ModelTracker {
        &self.notify
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Clone)]
struct QueryState {
    kind: ItemKind,
    search: String,
    filter: FilterBy,
    sort_index: usize,
    group_index: usize,
    sorts: Vec<SortKey>,
    group_ids: Vec<String>,
}

impl Default for QueryState {
    fn default() -> Self {
        Self {
            kind: ItemKind::Console,
            search: String::new(),
            filter: FilterBy::All,
            sort_index: 0,
            group_index: 0,
            sorts: sort_keys(ItemKind::Console),
            group_ids: vec!["all".into()],
        }
    }
}

pub fn bind(window: &AppWindow, app: MemoryPakApp, store: Rc<dyn SaveStore>) {
    let app = Rc::new(RefCell::new(app));
    let initial = app.borrow().initial_state();
    window.set_console_count(initial.consoles.len() as i32);
    window.set_game_count(initial.total_games as i32);
    window.set_collectible_count(initial.total_collectibles as i32);

    let state = Rc::new(RefCell::new(QueryState::default()));
    let snapshot = app.borrow().query_snapshot(spec_for(&state.borrow()));
    let model = Rc::new(QueryModel::new(app.clone(), snapshot));
    let search_timer = Rc::new(slint::Timer::default());
    window.set_rows(ModelRc::from(model.clone()));
    configure_tab(window, &app.borrow(), &mut state.borrow_mut(), 0);
    refresh(window, &app.borrow(), &state.borrow(), &model);

    {
        let weak = window.as_weak();
        let app = app.clone();
        let state = state.clone();
        let model = model.clone();
        window.on_tab_selected(move |tab| {
            let Some(window) = weak.upgrade() else { return };
            configure_tab(&window, &app.borrow(), &mut state.borrow_mut(), tab);
            refresh(&window, &app.borrow(), &state.borrow(), &model);
        });
    }
    {
        let weak = window.as_weak();
        let app = app.clone();
        let state = state.clone();
        let model = model.clone();
        let search_timer = search_timer.clone();
        window.on_search_changed(move |value| {
            state.borrow_mut().search = value.to_string();
            search_timer.stop();
            let weak = weak.clone();
            let app = app.clone();
            let state = state.clone();
            let model = model.clone();
            search_timer.start(
                slint::TimerMode::SingleShot,
                std::time::Duration::from_millis(120),
                move || {
                    if let Some(window) = weak.upgrade() {
                        refresh(&window, &app.borrow(), &state.borrow(), &model);
                    }
                },
            );
        });
    }
    {
        let weak = window.as_weak();
        let app = app.clone();
        let state = state.clone();
        let model = model.clone();
        window.on_filter_selected(move |filter| {
            state.borrow_mut().filter = match filter {
                1 => FilterBy::Owned,
                2 => FilterBy::Favorites,
                3 => FilterBy::Wishlist,
                4 => FilterBy::NotOwned,
                _ => FilterBy::All,
            };
            if let Some(window) = weak.upgrade() {
                window.set_active_filter(filter);
                refresh(&window, &app.borrow(), &state.borrow(), &model);
            }
        });
    }
    {
        let weak = window.as_weak();
        let app = app.clone();
        let state = state.clone();
        let model = model.clone();
        window.on_sort_selected(move |index| {
            state.borrow_mut().sort_index = usize::try_from(index).unwrap_or(0);
            if let Some(window) = weak.upgrade() {
                refresh(&window, &app.borrow(), &state.borrow(), &model);
            }
        });
    }
    {
        let weak = window.as_weak();
        let app = app.clone();
        let state = state.clone();
        let model = model.clone();
        window.on_group_selected(move |index| {
            state.borrow_mut().group_index = usize::try_from(index).unwrap_or(0);
            if let Some(window) = weak.upgrade() {
                refresh(&window, &app.borrow(), &state.borrow(), &model);
            }
        });
    }
    {
        let weak = window.as_weak();
        let app = app.clone();
        let state = state.clone();
        let model = model.clone();
        let store = store.clone();
        window.on_toggle_status(move |id, field| {
            let input = SetItemStatusInput {
                id: EntryId::from_raw(id.to_string()),
                owned: (field == 0).then(|| !current_state(&app.borrow(), &id, 0)),
                favorite: (field == 1).then(|| !current_state(&app.borrow(), &id, 1)),
                wishlist: (field == 2).then(|| !current_state(&app.borrow(), &id, 2)),
            };
            if let Err(error) = app.borrow_mut().set_item_status(input) {
                set_error(&weak, error);
                return;
            }
            persist(&app.borrow(), &store, &weak);
            if let Some(window) = weak.upgrade() {
                if query_membership_is_stable(&state.borrow()) {
                    model.notify_id(&id);
                    window.set_stats(summary(
                        app.borrow().collection_stats(),
                        state.borrow().kind,
                    ));
                } else {
                    refresh(&window, &app.borrow(), &state.borrow(), &model);
                }
                if window.get_detail_open() {
                    if let Some(row) = model.row_for_id(&id) {
                        window.set_detail_data(row);
                    } else {
                        window.set_detail_open(false);
                    }
                }
            }
        });
    }
    {
        let weak = window.as_weak();
        let app = app.clone();
        let model = model.clone();
        let store = store.clone();
        window.on_save_notes(move |id, notes| {
            let result = app.borrow_mut().set_item_notes(SetItemNotesInput {
                id: EntryId::from_raw(id.to_string()),
                notes: notes.to_string(),
            });
            if let Err(error) = result {
                set_error(&weak, error);
                return;
            }
            persist(&app.borrow(), &store, &weak);
            model.notify_id(&id);
        });
    }
    {
        let weak = window.as_weak();
        let app = app.clone();
        window.on_backup(move || match app.borrow().export_json() {
            Ok(json) => {
                if let Err(error) = PlatformDocuments.export_backup(&json) {
                    set_error(&weak, error);
                }
            }
            Err(error) => set_error(&weak, error),
        });
    }
    {
        let weak = window.as_weak();
        let app = app.clone();
        let state = state.clone();
        let model = model.clone();
        let store = store.clone();
        window.on_restore(move || {
            let weak = weak.clone();
            let app = app.clone();
            let state = state.clone();
            let model = model.clone();
            let store = store.clone();
            PlatformDocuments.import_backup(Box::new(move |result| match result {
                Ok(Some(json)) => {
                    if let Err(error) = app.borrow_mut().import_json(&json) {
                        set_error(&weak, error);
                        return;
                    }
                    persist(&app.borrow(), &store, &weak);
                    if let Some(window) = weak.upgrade() {
                        refresh(&window, &app.borrow(), &state.borrow(), &model);
                    }
                }
                Ok(None) => {}
                Err(error) => set_error(&weak, error),
            }));
        });
    }
    {
        let weak = window.as_weak();
        window.on_check_updates(move || {
            let weak = weak.clone();
            PlatformUpdates.check(Box::new(move |state| {
                let _ = slint::invoke_from_event_loop(move || {
                    apply_update_state(&weak, state);
                });
            }));
        });
    }
    {
        let weak = window.as_weak();
        window.on_install_update(move || {
            let weak = weak.clone();
            PlatformUpdates.install(Box::new(move |state| {
                let _ = slint::invoke_from_event_loop(move || {
                    apply_update_state(&weak, state);
                });
            }));
        });
    }
}

fn configure_tab(window: &AppWindow, app: &MemoryPakApp, state: &mut QueryState, tab: i32) {
    state.kind = match tab {
        1 => ItemKind::Game,
        2 => ItemKind::Collectible,
        _ => ItemKind::Console,
    };
    state.sort_index = 0;
    state.group_index = 0;
    state.sorts = sort_keys(state.kind);
    state.group_ids = vec!["all".into()];

    let initial = app.initial_state();
    let group_labels: Vec<SharedString> = match state.kind {
        ItemKind::Console => vec![],
        ItemKind::Game => {
            state.group_ids.extend(
                initial
                    .consoles_with_games
                    .iter()
                    .map(|console| console.id.to_string()),
            );
            std::iter::once("All consoles".into())
                .chain(
                    initial
                        .consoles_with_games
                        .iter()
                        .map(|console| SharedString::from(console.name.as_str())),
                )
                .collect()
        }
        ItemKind::Collectible => {
            state.group_ids.extend(
                initial
                    .collections
                    .iter()
                    .map(|collection| collection.id.clone()),
            );
            std::iter::once("All collections".into())
                .chain(
                    initial
                        .collections
                        .iter()
                        .map(|collection| SharedString::from(collection.name.as_str())),
                )
                .collect()
        }
    };
    let sort_labels: Vec<SharedString> = sort_labels(state.kind)
        .into_iter()
        .map(SharedString::from)
        .collect();
    window.set_active_tab(tab.clamp(0, 2));
    window.set_sort_index(0);
    window.set_group_index(0);
    window.set_sort_options(ModelRc::new(VecModel::from(sort_labels)));
    window.set_group_options(ModelRc::new(VecModel::from(group_labels)));
}

fn refresh(window: &AppWindow, app: &MemoryPakApp, state: &QueryState, model: &QueryModel) {
    let snapshot = app.query_snapshot(spec_for(state));
    window.set_shown_count(snapshot.total as i32);
    model.replace(snapshot);
    window.set_stats(summary(app.collection_stats(), state.kind));
    window.set_error_message("".into());
}

fn spec_for(state: &QueryState) -> QuerySpec {
    QuerySpec {
        kind: state.kind,
        search: (!state.search.is_empty()).then(|| state.search.clone()),
        sort_by: state.sorts.get(state.sort_index).copied(),
        filter_by: Some(state.filter),
        group_id: state.group_ids.get(state.group_index).cloned(),
        offset: None,
        limit: None,
    }
}

fn query_membership_is_stable(state: &QueryState) -> bool {
    state.filter == FilterBy::All && state.sorts.get(state.sort_index) != Some(&SortKey::Status)
}

fn sort_keys(kind: ItemKind) -> Vec<SortKey> {
    match kind {
        ItemKind::Console => vec![SortKey::Name, SortKey::Manufacturer, SortKey::Status],
        ItemKind::Game => vec![SortKey::Title, SortKey::Year, SortKey::Status],
        ItemKind::Collectible => vec![
            SortKey::Name,
            SortKey::Collection,
            SortKey::Category,
            SortKey::Group,
            SortKey::Variant,
            SortKey::Year,
            SortKey::Status,
        ],
    }
}

fn sort_labels(kind: ItemKind) -> Vec<&'static str> {
    match kind {
        ItemKind::Console => vec!["Name", "Manufacturer", "Status"],
        ItemKind::Game => vec!["Title", "Year", "Status"],
        ItemKind::Collectible => vec![
            "Name",
            "Collection",
            "Category",
            "Pack / Game",
            "Variant",
            "Year",
            "Status",
        ],
    }
}

fn summary(stats: CollectionStats, kind: ItemKind) -> UiStats {
    match kind {
        ItemKind::Console => UiStats {
            owned: stats.owned_consoles as i32,
            favorite: stats.favorite_consoles as i32,
            wishlist: stats.wishlist_consoles as i32,
            total: stats.total_consoles as i32,
        },
        ItemKind::Game => UiStats {
            owned: stats.owned_games as i32,
            favorite: stats.favorite_games as i32,
            wishlist: stats.wishlist_games as i32,
            total: stats.total_games as i32,
        },
        ItemKind::Collectible => UiStats {
            owned: stats.owned_collectibles as i32,
            favorite: stats.favorite_collectibles as i32,
            wishlist: stats.wishlist_collectibles as i32,
            total: stats.total_collectibles as i32,
        },
    }
}

fn into_ui_row(row: RowView) -> UiRow {
    match row {
        RowView::Console(view) => UiRow {
            id: view.id.to_string().into(),
            title: view.name.into(),
            subtitle: format!(
                "{}{}",
                view.manufacturer,
                (view.generation > 0)
                    .then(|| format!(" / Gen {}", view.generation))
                    .unwrap_or_default()
            )
            .into(),
            mobile_subtitle: format!(
                "{}{}",
                view.manufacturer,
                (view.generation > 0)
                    .then(|| format!(" / Gen {}", view.generation))
                    .unwrap_or_default()
            )
            .into(),
            meta: format!(
                "{} owned / {} favorite / {} wishlist",
                view.game_counts.owned, view.game_counts.favorite, view.game_counts.wishlist
            )
            .into(),
            mobile_meta: format!("{} games", view.game_counts.total).into(),
            note: view.state.notes.into(),
            kind: "console".into(),
            owned: view.state.owned,
            favorite: view.state.favorite,
            wishlist: view.state.wishlist,
        },
        RowView::Game(view) => UiRow {
            id: view.id.to_string().into(),
            title: view.title.into(),
            subtitle: format!(
                "{} / {}{} / {}",
                view.console_name,
                (!view.developer.is_empty())
                    .then(|| format!("{} / ", view.developer))
                    .unwrap_or_default(),
                if view.publisher.is_empty() {
                    "Unknown publisher"
                } else {
                    &view.publisher
                },
                if view.year == 0 {
                    "Unknown year".to_string()
                } else {
                    view.year.to_string()
                }
            )
            .into(),
            mobile_subtitle: format!(
                "{} / {}",
                view.console_name,
                if view.year == 0 {
                    "Unknown year".to_string()
                } else {
                    view.year.to_string()
                }
            )
            .into(),
            meta: status_meta(&view.state).into(),
            mobile_meta: [view.developer.as_str(), view.publisher.as_str()]
                .into_iter()
                .filter(|value| !value.is_empty())
                .collect::<Vec<_>>()
                .join(" / ")
                .into(),
            note: view.state.notes.into(),
            kind: "game".into(),
            owned: view.state.owned,
            favorite: view.state.favorite,
            wishlist: view.state.wishlist,
        },
        RowView::Collectible(view) => UiRow {
            id: view.id.to_string().into(),
            title: view.name.into(),
            subtitle: [
                view.collection_name.as_str(),
                view.category.as_str(),
                view.group.as_str(),
                view.variant.as_str(),
            ]
            .into_iter()
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>()
            .join(" / ")
            .into(),
            mobile_subtitle: [view.collection_name.as_str(), view.category.as_str()]
                .into_iter()
                .filter(|value| !value.is_empty())
                .collect::<Vec<_>>()
                .join(" / ")
                .into(),
            meta: status_meta(&view.state).into(),
            mobile_meta: [view.group.as_str(), view.variant.as_str()]
                .into_iter()
                .filter(|value| !value.is_empty())
                .collect::<Vec<_>>()
                .join(" / ")
                .into(),
            note: view.state.notes.into(),
            kind: "collectible".into(),
            owned: view.state.owned,
            favorite: view.state.favorite,
            wishlist: view.state.wishlist,
        },
    }
}

fn status_meta(state: &memory_pak_core::EntryState) -> String {
    format!(
        "{} · {} · {}",
        if state.owned { "owned" } else { "not owned" },
        if state.favorite { "fav" } else { "not fav" },
        if state.wishlist { "wish" } else { "not wish" }
    )
}

fn row_id(row: &RowView) -> &str {
    match row {
        RowView::Console(view) => view.id.as_str(),
        RowView::Game(view) => view.id.as_str(),
        RowView::Collectible(view) => view.id.as_str(),
    }
}

fn current_state(app: &MemoryPakApp, id: &str, field: i32) -> bool {
    let Some(state) = app.persisted_state().entries.get(id) else {
        return false;
    };
    match field {
        0 => state.owned,
        1 => state.favorite,
        _ => state.wishlist,
    }
}

fn persist(app: &MemoryPakApp, store: &Rc<dyn SaveStore>, window: &slint::Weak<AppWindow>) {
    match app.save_json() {
        Ok(json) => {
            if let Err(error) = store.save(&json) {
                set_error(window, error);
            }
        }
        Err(error) => set_error(window, error),
    }
}

fn set_error(window: &slint::Weak<AppWindow>, error: impl std::fmt::Display) {
    if let Some(window) = window.upgrade() {
        window.set_error_message(error.to_string().into());
    }
}

fn apply_update_state(window: &slint::Weak<AppWindow>, state: UpdateState) {
    let Some(window) = window.upgrade() else {
        return;
    };
    match state {
        UpdateState::Unsupported => {
            window.set_update_ready(false);
            window.set_update_message(
                "Updates are managed by the store or are unavailable in this build.".into(),
            );
        }
        UpdateState::Current => {
            window.set_update_ready(false);
            window.set_update_message("Memory Pak is up to date.".into());
        }
        UpdateState::Available(version) => {
            window.set_update_ready(true);
            window.set_update_message(format!("Memory Pak {version} is available.").into());
        }
        UpdateState::Installed => {
            window.set_update_ready(false);
            window.set_update_message("Update installed. Restart Memory Pak.".into());
        }
        UpdateState::Error(error) => {
            window.set_update_ready(false);
            window.set_update_message(format!("Update check failed: {error}").into());
        }
    }
}
