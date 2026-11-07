//! Minecraft Bedrock Edition Crafting Interface for Horizon
//!
//! Provides the controller- and handheld-friendly Bedrock categorized recipe layout:
//! - 4 Category Tabs: Construction, Equipment, Items, Nature & Redstone
//! - Fast one-click / controller-friendly instant crafting (A = 1 batch, Y / Shift+A = max batches)
//! - Clean 5x4 output grid with clear craftability highlights and dimmed uncraftable slots
//! - "Craftable Only" filter toggle (Controller X button or clickable pill)
//! - Bumpers (LB/RB) category tab switching
//! - Responsive sizing: fits 320x240 handheld screens (muOS RG34XX SP) and scales to desktop.

use super::{Game, Screen};
use horizon_client::{
    hud::{HudBatch, rgba},
    items::ItemStack,
    recipe_book::{BookType, Entry},
    widgets::{self, Kind},
};
use std::sync::{LazyLock, Mutex};

/// The 4 standard Bedrock Edition Crafting Categories
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BedrockCategory {
    Construction = 0,
    Equipment = 1,
    Items = 2,
    Nature = 3,
}

impl BedrockCategory {
    pub const ALL: [BedrockCategory; 4] = [
        BedrockCategory::Construction,
        BedrockCategory::Equipment,
        BedrockCategory::Items,
        BedrockCategory::Nature,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Self::Construction => "Construction",
            Self::Equipment => "Equipment",
            Self::Items => "Items",
            Self::Nature => "Nature",
        }
    }

    pub fn icon_item(self) -> &'static str {
        match self {
            Self::Construction => "minecraft:bricks",
            Self::Equipment => "minecraft:iron_sword",
            Self::Items => "minecraft:stick",
            Self::Nature => "minecraft:redstone",
        }
    }

    /// Classifies an item into its Bedrock category based on item ID.
    pub fn classify(item_id: &str) -> Self {
        let name = item_id.strip_prefix("minecraft:").unwrap_or(item_id);

        // Equipment: Tools, Weapons, Armor, Projectiles
        if name.ends_with("_sword")
            || name.ends_with("_pickaxe")
            || name.ends_with("_axe")
            || name.ends_with("_shovel")
            || name.ends_with("_hoe")
            || name.ends_with("_helmet")
            || name.ends_with("_chestplate")
            || name.ends_with("_leggings")
            || name.ends_with("_boots")
            || matches!(
                name,
                "shield"
                    | "bow"
                    | "crossbow"
                    | "arrow"
                    | "spectral_arrow"
                    | "tipped_arrow"
                    | "shears"
                    | "flint_and_steel"
                    | "fishing_rod"
                    | "carrot_on_a_stick"
                    | "warped_fungus_on_a_stick"
                    | "spyglass"
                    | "trident"
                    | "mace"
                    | "brush"
                    | "bundle"
            )
        {
            return Self::Equipment;
        }

        // Nature & Redstone: Redstone mechanisms, plants, food, organics
        if name.contains("redstone")
            || name.contains("repeater")
            || name.contains("comparator")
            || name.contains("piston")
            || name.contains("dispenser")
            || name.contains("dropper")
            || name.contains("observer")
            || name.contains("torch")
            || name.contains("lantern")
            || name.contains("campfire")
            || name.contains("door")
            || name.contains("sensor")
            || name.ends_with("_button")
            || name.ends_with("_pressure_plate")
            || matches!(
                name,
                "lever"
                    | "tripwire_hook"
                    | "lightning_rod"
                    | "target"
                    | "tnt"
                    | "bread"
                    | "cookie"
                    | "cake"
                    | "pumpkin_pie"
                    | "mushroom_stew"
                    | "rabbit_stew"
                    | "beetroot_soup"
                    | "golden_apple"
                    | "enchanted_golden_apple"
                    | "golden_carrot"
                    | "sugar"
                    | "honey_bottle"
                    | "wheat"
                    | "bone_meal"
                    | "firework_rocket"
                    | "firework_star"
                    | "apple"
                    | "baked_potato"
                    | "cooked_beef"
                    | "cooked_porkchop"
                    | "cooked_mutton"
                    | "cooked_chicken"
            )
        {
            return Self::Nature;
        }

        // Items: Crafting items, materials, raw resources, storage, tables
        if name.ends_with("_ingot")
            || name.ends_with("_nugget")
            || name.ends_with("_raw")
            || name.ends_with("_dye")
            || matches!(
                name,
                "stick"
                    | "bowl"
                    | "bucket"
                    | "water_bucket"
                    | "lava_bucket"
                    | "milk_bucket"
                    | "paper"
                    | "book"
                    | "writable_book"
                    | "written_book"
                    | "compass"
                    | "clock"
                    | "map"
                    | "empty_map"
                    | "chest"
                    | "trapped_chest"
                    | "barrel"
                    | "crafting_table"
                    | "furnace"
                    | "blast_furnace"
                    | "smoker"
                    | "anvil"
                    | "chipped_anvil"
                    | "damaged_anvil"
                    | "grindstone"
                    | "loom"
                    | "smithing_table"
                    | "cartography_table"
                    | "stonecutter"
                    | "brewing_stand"
                    | "cauldron"
                    | "iron_ingot"
                    | "gold_ingot"
                    | "copper_ingot"
                    | "netherite_ingot"
                    | "diamond"
                    | "emerald"
                    | "coal"
                    | "charcoal"
                    | "flint"
                    | "feather"
                    | "leather"
                    | "string"
                    | "amethyst_shard"
                    | "minecart"
                    | "lead"
                    | "name_tag"
            )
        {
            return Self::Items;
        }

        // Construction: All structural, block, wood, and architectural items
        Self::Construction
    }
}

pub struct BedrockUiState {
    pub enabled: bool,
    pub category: BedrockCategory,
    pub craftable_only: bool,
    pub page: usize,
    pub _hovered_recipe: Option<usize>,
}

impl Default for BedrockUiState {
    fn default() -> Self {
        Self {
            enabled: true,
            category: BedrockCategory::Construction,
            craftable_only: false,
            page: 0,
            _hovered_recipe: None,
        }
    }
}

static UI_STATE: LazyLock<Mutex<BedrockUiState>> =
    LazyLock::new(|| Mutex::new(BedrockUiState::default()));

pub fn with_bedrock_state<R>(f: impl FnOnce(&mut BedrockUiState) -> R) -> R {
    let mut guard = UI_STATE.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    f(&mut guard)
}

fn inside(pos: (f32, f32), rect: [f32; 4]) -> bool {
    pos.0 >= rect[0] && pos.0 < rect[0] + rect[2] && pos.1 >= rect[1] && pos.1 < rect[1] + rect[3]
}

impl Game<'_> {
    /// Returns true if the Bedrock Crafting UI is active for the current screen.
    pub fn is_bedrock_crafting_active(&self) -> bool {
        self.screen == Screen::Inventory
            && self.station.is_none()
            && !self.mount_active()
            && self.native_screen_height().is_none()
            && (self.container.is_none() || self.craft_size == 3)
            && self.native_on("bedrock_crafting")
            && with_bedrock_state(|st| st.enabled)
    }

    /// Calculates the Bedrock panel boundary box `[x, y, w, h]`.
    pub fn bedrock_panel_rect(&self, batch: &HudBatch) -> [f32; 4] {
        let (ox, oy) = self.inventory_origin(batch);
        if batch.width < 380.0 {
            // Handheld / 320x240 mode: fill left side cleanly
            [4.0, oy, 132.0, 166.0]
        } else {
            // Desktop / wide mode: sit to the left of the container window
            let w = 144.0;
            [ox - w - 6.0, oy, w, 166.0]
        }
    }

    /// Draws the Minecraft Bedrock Edition crafting panel.
    pub fn draw_bedrock_crafting_ui(&mut self, batch: &mut HudBatch) {
        if !self.is_bedrock_crafting_active() {
            return;
        }

        self.ensure_book();
        let entries = self.recipe_book_entries();
        let available = self.book_available(BookType::Crafting);

        let panel = self.bedrock_panel_rect(batch);
        let (px, py, pw, ph) = (panel[0], panel[1], panel[2], panel[3]);
        let cursor = self.cursor;
        let atlas = &self.gui.atlas;

        // 1. Panel Background & Frame
        batch.fill(atlas, px - 1.0, py - 1.0, pw + 2.0, ph + 2.0, rgba(17, 20, 24, 255));
        batch.fill(atlas, px, py, pw, ph, rgba(28, 32, 40, 250));
        batch.fill(atlas, px + 1.0, py + 1.0, pw - 2.0, ph - 2.0, rgba(34, 38, 48, 250));

        let (active_cat, craftable_only, mut page) = with_bedrock_state(|st| {
            (st.category, st.craftable_only, st.page)
        });

        // 2. Category Tabs (4 Tabs)
        let tab_w = ((pw - 10.0) / 4.0).floor();
        let tab_h = 22.0;
        let tab_y = py + 3.0;

        for (i, &cat) in BedrockCategory::ALL.iter().enumerate() {
            let tx = px + 4.0 + i as f32 * (tab_w + 1.0);
            let tab_rect = [tx, tab_y, tab_w, tab_h];
            let is_sel = cat == active_cat;

            let bg_color = if is_sel {
                rgba(58, 68, 84, 255)
            } else if inside(cursor, tab_rect) {
                rgba(48, 56, 70, 255)
            } else {
                rgba(24, 27, 34, 240)
            };

            batch.fill(atlas, tx, tab_y, tab_w, tab_h, bg_color);

            // Active underline indicator
            if is_sel {
                batch.fill(atlas, tx, tab_y + tab_h - 2.0, tab_w, 2.0, rgba(85, 255, 85, 255));
            }

            // Tab icon (16x16)
            let icon_stack = ItemStack::new(cat.icon_item(), 1);
            self.draw_item(batch, &icon_stack, tx + ((tab_w - 16.0) / 2.0).floor(), tab_y + 2.0);

            // Register tab as widget target for gamepad tab navigation (LB/RB)
            widgets::record_as(cat.title(), tab_rect, true, Kind::Tab(is_sel));
        }

        // 3. Sub-header: Category Title, Craftable Filter & Page Indicator
        let header_y = py + 27.0;
        let title_text = active_cat.title();
        batch.text(atlas, title_text, px + 6.0, header_y + 2.0, rgba(220, 235, 255, 255), true);

        // Filter button: [Craftable] / [All]
        let filter_w = 34.0;
        let filter_h = 12.0;
        let filter_x = px + pw - filter_w - 5.0;
        let filter_rect = [filter_x, header_y, filter_w, filter_h];
        let filter_bg = if craftable_only {
            rgba(46, 120, 48, 255)
        } else {
            rgba(50, 56, 68, 240)
        };
        batch.fill(atlas, filter_x, header_y, filter_w, filter_h, filter_bg);
        let filter_label = if craftable_only { "Craft" } else { "All" };
        let fl_w = HudBatch::text_width(atlas, filter_label);
        batch.text(
            atlas,
            filter_label,
            filter_x + ((filter_w - fl_w) / 2.0).floor(),
            header_y + 2.0,
            rgba(255, 255, 255, 255),
            true,
        );
        widgets::record_as("Filter", filter_rect, true, Kind::Button);

        // 4. Fetch Recipes matching category & grid
        let book = self.recipe_ui.book.as_ref();
        let (gw, gh) = self.book_grid();

        let mut matching_recipes: Vec<(&Entry, bool)> = Vec::new();
        for entry in &entries {
            if !entry.fits(gw, gh) {
                continue;
            }
            if BedrockCategory::classify(&entry.result.item) != active_cat {
                continue;
            }
            let is_craftable = book.is_some_and(|b| b.can_craft(entry, &available));
            if craftable_only && !is_craftable {
                continue;
            }
            matching_recipes.push((entry, is_craftable));
        }

        // Deduplicate recipes by result item so the grid shows unique items cleanly
        let mut seen_items = std::collections::HashSet::new();
        let mut display_recipes = Vec::new();
        for (e, craftable) in matching_recipes {
            if seen_items.insert(e.result.item.clone()) {
                display_recipes.push((e, craftable));
            }
        }

        let per_page = 20; // 5 columns x 4 rows
        let total_pages = display_recipes.len().div_ceil(per_page).max(1);
        if page >= total_pages {
            page = total_pages - 1;
            with_bedrock_state(|st| st.page = page);
        }

        // Page arrows if multiple pages
        if total_pages > 1 {
            let prev_x = filter_x - 38.0;
            let next_x = filter_x - 12.0;
            let page_text = format!("{}/{}", page + 1, total_pages);

            let hovered_prev = inside(self.cursor, [prev_x, header_y, 10.0, 12.0]);
            let prev_col = if hovered_prev { rgba(255, 255, 255, 255) } else { rgba(180, 190, 205, 255) };
            batch.text(atlas, "<", prev_x + 2.0, header_y + 2.0, prev_col, true);
            widgets::record_as("<", [prev_x, header_y, 10.0, 12.0], true, Kind::Button);

            let pt_w = HudBatch::text_width(atlas, &page_text);
            let pt_x = prev_x + 10.0 + ((next_x - (prev_x + 10.0) - pt_w) / 2.0).floor();
            batch.text(atlas, &page_text, pt_x, header_y + 2.0, rgba(180, 190, 205, 255), true);

            let hovered_next = inside(self.cursor, [next_x, header_y, 10.0, 12.0]);
            let next_col = if hovered_next { rgba(255, 255, 255, 255) } else { rgba(180, 190, 205, 255) };
            batch.text(atlas, ">", next_x + 2.0, header_y + 2.0, next_col, true);
            widgets::record_as(">", [next_x, header_y, 10.0, 12.0], true, Kind::Button);
        }

        // 5. Recipe Output Grid (5 cols x 4 rows = 20 slots)
        let cols = 5;
        let _rows = 4;
        let cell_w = 23.0;
        let cell_h = 23.0;
        let grid_x = px + ((pw - cols as f32 * cell_w) / 2.0).floor();
        let grid_y = py + 42.0;

        let start_idx = page * per_page;
        let page_recipes = display_recipes.iter().skip(start_idx).take(per_page);

        let mut hovered_entry: Option<(&Entry, bool)> = None;

        for (slot_idx, &(entry, is_craftable)) in page_recipes.enumerate() {
            let col = slot_idx % cols;
            let row = slot_idx / cols;
            let cx = grid_x + col as f32 * cell_w;
            let cy = grid_y + row as f32 * cell_h;
            let slot_rect = [cx, cy, 21.0, 21.0];

            let is_hover = inside(cursor, slot_rect);
            if is_hover {
                hovered_entry = Some((entry, is_craftable));
            }

            // Slot background
            let slot_bg = if is_craftable {
                rgba(22, 26, 34, 255)
            } else {
                rgba(16, 18, 22, 230)
            };
            batch.fill(atlas, cx, cy, 21.0, 21.0, slot_bg);

            // 1px Border
            let border_col = if is_hover {
                rgba(255, 255, 255, 255)
            } else if is_craftable {
                rgba(68, 76, 92, 255)
            } else {
                rgba(38, 42, 50, 200)
            };
            batch.fill(atlas, cx, cy, 21.0, 1.0, border_col);
            batch.fill(atlas, cx, cy + 20.0, 21.0, 1.0, border_col);
            batch.fill(atlas, cx, cy, 1.0, 21.0, border_col);
            batch.fill(atlas, cx + 20.0, cy, 1.0, 21.0, border_col);

            // Item Icon
            self.draw_item(batch, &entry.result, cx + 2.5, cy + 2.5);

            // If uncraftable, draw subtle dim overlay
            if !is_craftable {
                batch.fill(atlas, cx + 1.0, cy + 1.0, 19.0, 19.0, rgba(0, 0, 0, 140));
            }

            // Count text if > 1
            if entry.result.count > 1 {
                let cnt_str = format!("{}", entry.result.count);
                let cw = HudBatch::text_width(atlas, &cnt_str);
                batch.text(
                    atlas,
                    &cnt_str,
                    cx + 19.0 - cw,
                    cy + 11.0,
                    rgba(255, 255, 255, 255),
                    true,
                );
            }

            // Register widget target for D-pad gamepad navigation
            let item_label = self.pack.names.display_item(&entry.result.item);
            widgets::record_as(&item_label, slot_rect, true, Kind::Button);
        }

        // Fill remaining empty slots with clean blank frames
        let count_on_page = display_recipes.len().saturating_sub(start_idx).min(per_page);
        for slot_idx in count_on_page..per_page {
            let col = slot_idx % cols;
            let row = slot_idx / cols;
            let cx = grid_x + col as f32 * cell_w;
            let cy = grid_y + row as f32 * cell_h;
            batch.fill(atlas, cx, cy, 21.0, 21.0, rgba(18, 20, 26, 180));
            batch.fill(atlas, cx, cy, 21.0, 1.0, rgba(32, 36, 44, 180));
            batch.fill(atlas, cx, cy + 20.0, 21.0, 1.0, rgba(32, 36, 44, 180));
            batch.fill(atlas, cx, cy, 1.0, 21.0, rgba(32, 36, 44, 180));
            batch.fill(atlas, cx + 20.0, cy, 1.0, 21.0, rgba(32, 36, 44, 180));
        }

        // 6. Footer: Focused Recipe Details & Controller Hints
        let footer_y = py + 138.0;
        batch.fill(atlas, px + 4.0, footer_y, pw - 8.0, 24.0, rgba(20, 23, 30, 240));

        if let Some((entry, is_craftable)) = hovered_entry {
            let display_name = self.pack.names.display_item(&entry.result.item);
            let name_line = if entry.result.count > 1 {
                format!("{} x{}", display_name, entry.result.count)
            } else {
                display_name
            };

            let name_color = if is_craftable {
                rgba(255, 255, 255, 255)
            } else {
                rgba(180, 185, 195, 255)
            };
            batch.text(atlas, &name_line, px + 7.0, footer_y + 2.0, name_color, true);

            // Ingredient summary line
            let req_summary = self.format_ingredient_summary(entry);
            let req_color = if is_craftable {
                rgba(85, 255, 85, 255)
            } else {
                rgba(255, 100, 100, 255)
            };
            batch.text(atlas, &req_summary, px + 7.0, footer_y + 13.0, req_color, true);
        } else {
            // Controller legend
            batch.text(
                atlas,
                "A: Craft (1)  Y: Max",
                px + 7.0,
                footer_y + 3.0,
                rgba(160, 210, 255, 255),
                true,
            );
            batch.text(
                atlas,
                "X: Filter  LB/RB: Tab",
                px + 7.0,
                footer_y + 13.0,
                rgba(140, 160, 180, 255),
                true,
            );
        }
    }

    /// Formats a concise ingredient requirement line for the footer (e.g. "Needs: 1x Oak Log")
    fn format_ingredient_summary(&self, entry: &Entry) -> String {
        let reqs = entry.requirements();
        if reqs.is_empty() {
            return "No ingredients required".to_string();
        }

        // Count unique ingredients required
        let mut counts: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
        for ing in reqs {
            if let Some(first) = ing.items.iter().next() {
                let name = self.pack.names.display_item(first);
                *counts.entry(name).or_insert(0) += 1;
            }
        }

        let mut parts = Vec::new();
        for (item_name, count) in counts {
            parts.push(format!("{count}x {item_name}"));
        }
        format!("Needs: {}", parts.join(", "))
    }

    /// Handles a click / press in the Bedrock Crafting UI.
    /// Returns true if the Bedrock UI handled the interaction.
    pub fn click_bedrock_crafting(&mut self, right: bool) -> bool {
        if !self.is_bedrock_crafting_active() {
            return false;
        }

        let batch = match self.gui_scale() {
            Some(b) => b,
            None => return false,
        };
        let panel = self.bedrock_panel_rect(&batch);
        let cursor = self.cursor;

        // If click is not inside Bedrock panel, let normal inventory handle it
        if !inside(cursor, panel) {
            return false;
        }

        let (px, py, pw, _ph) = (panel[0], panel[1], panel[2], panel[3]);

        // 1. Check Category Tabs
        let tab_w = ((pw - 10.0) / 4.0).floor();
        let tab_h = 22.0;
        let tab_y = py + 3.0;

        for (i, &cat) in BedrockCategory::ALL.iter().enumerate() {
            let tx = px + 4.0 + i as f32 * (tab_w + 1.0);
            if inside(cursor, [tx, tab_y, tab_w, tab_h]) {
                with_bedrock_state(|st| {
                    st.category = cat;
                    st.page = 0;
                });
                return true;
            }
        }

        // 2. Check Craftable Filter button
        let header_y = py + 27.0;
        let filter_w = 34.0;
        let filter_h = 12.0;
        let filter_x = px + pw - filter_w - 5.0;
        if inside(cursor, [filter_x, header_y, filter_w, filter_h]) {
            with_bedrock_state(|st| {
                st.craftable_only = !st.craftable_only;
                st.page = 0;
            });
            return true;
        }

        // 2b. Check Page Prev/Next buttons
        let prev_x = filter_x - 38.0;
        let next_x = filter_x - 12.0;
        if inside(cursor, [prev_x, header_y, 10.0, 12.0]) {
            with_bedrock_state(|st| {
                if st.page > 0 {
                    st.page -= 1;
                } else {
                    st.page = 999; // Will be clamped below
                }
            });
            return true;
        }
        if inside(cursor, [next_x, header_y, 10.0, 12.0]) {
            with_bedrock_state(|st| {
                st.page = st.page.saturating_add(1);
            });
            return true;
        }

        // 3. Check Recipe Slots
        let (active_cat, craftable_only, mut page) = with_bedrock_state(|st| {
            (st.category, st.craftable_only, st.page)
        });

        self.ensure_book();
        let entries = self.recipe_book_entries();
        let available = self.book_available(BookType::Crafting);
        let book = self.recipe_ui.book.as_ref();
        let (gw, gh) = self.book_grid();

        let mut matching_recipes: Vec<&Entry> = Vec::new();
        for entry in &entries {
            if !entry.fits(gw, gh) {
                continue;
            }
            if BedrockCategory::classify(&entry.result.item) != active_cat {
                continue;
            }
            let is_craftable = book.is_some_and(|b| b.can_craft(entry, &available));
            if craftable_only && !is_craftable {
                continue;
            }
            matching_recipes.push(entry);
        }

        let mut seen_items = std::collections::HashSet::new();
        let mut display_recipes = Vec::new();
        for e in matching_recipes {
            if seen_items.insert(e.result.item.clone()) {
                display_recipes.push(e);
            }
        }

        let per_page = 20;
        let total_pages = display_recipes.len().div_ceil(per_page).max(1);
        if page >= total_pages {
            page = total_pages - 1;
        }

        let cols = 5;
        let cell_w = 23.0;
        let cell_h = 23.0;
        let grid_x = px + ((pw - cols as f32 * cell_w) / 2.0).floor();
        let grid_y = py + 42.0;

        let start_idx = page * per_page;

        for (slot_idx, &entry) in display_recipes.iter().skip(start_idx).take(per_page).enumerate() {
            let col = slot_idx % cols;
            let row = slot_idx / cols;
            let cx = grid_x + col as f32 * cell_w;
            let cy = grid_y + row as f32 * cell_h;

            if inside(cursor, [cx, cy, 21.0, 21.0]) {
                // Craft recipe: single batch on left click, max batches on shift-click or right click
                let craft_max = self.has_shift_down() || right;
                self.mod_place_recipe(entry.display_id, craft_max);
                return true;
            }
        }

        // If inside panel, consume the click so it doesn't drop items into the world
        true
    }
}
