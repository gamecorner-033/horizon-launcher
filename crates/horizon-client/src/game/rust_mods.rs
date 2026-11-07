//! Built-in Rust client mods for the Horizon + Pumpkin Rust ecosystem:
//! - **Mod #4: WAILA / Jade Target Inspector & Enemy Health Bars**
//! - **Mod #5: Veinminer / Excavate Keybind & Smart Auto-Tool Switch**

use super::{Game, Screen, native_hooks::NativeMod};
use horizon_client::{extras_hud::Corners, hud::HudBatch, items::ItemStack, text::Text};
use horizon_protocol::packets::play::{self, Serverbound as S};
use horizon_world::registry::BlockStateId;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{LazyLock, Mutex},
};

const MOD_ID_WAILA: &str = "waila_inspector";
const MOD_ID_VEINMINER: &str = "veinminer_autotool";
pub const MOD_ID_BEDROCK_CRAFTING: &str = "bedrock_crafting";

struct RustClientModsState {
    initialized: bool,
    // Mod #1: Bedrock Crafting UI
    bedrock_crafting_enabled: bool,
    // Mod #4 settings
    waila_enabled: bool,
    health_bars_enabled: bool,
    vein_preview_enabled: bool,
    claim_badge_enabled: bool,
    // Mod #5 settings
    veinminer_enabled: bool,
    auto_tool_enabled: bool,
    save_durability_enabled: bool,
    // Server-synced chunk claims: (chunk_x, chunk_z) -> owner_name
    chunk_claims: HashMap<(i32, i32), String>,
    // Recently targeted or attacked mob for combat health bar retention: (mob_id, last_seen_tick)
    tracked_mob: Option<(u32, u64)>,
    // Cached connected ore vein count: (pos, state_id, count)
    cached_vein: Option<([i32; 3], u32, usize)>,
}

impl Default for RustClientModsState {
    fn default() -> Self {
        Self {
            initialized: false,
            bedrock_crafting_enabled: true,
            waila_enabled: true,
            health_bars_enabled: true,
            vein_preview_enabled: true,
            claim_badge_enabled: true,
            veinminer_enabled: true,
            auto_tool_enabled: true,
            save_durability_enabled: true,
            chunk_claims: HashMap::new(),
            tracked_mob: None,
            cached_vein: None,
        }
    }
}

static STATE: LazyLock<Mutex<RustClientModsState>> =
    LazyLock::new(|| Mutex::new(RustClientModsState::default()));

fn with_state<R>(f: impl FnOnce(&mut RustClientModsState) -> R) -> R {
    let mut guard = STATE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    f(&mut guard)
}

fn format_pretty_id(raw: &str) -> String {
    let key = raw.strip_prefix("minecraft:").unwrap_or(raw);
    let mut out = String::with_capacity(key.len() + 4);
    for (i, word) in key.split('_').enumerate() {
        if i > 0 {
            out.push(' ');
        }
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            for c in first.to_uppercase() {
                out.push(c);
            }
            out.push_str(chars.as_str());
        }
    }
    out
}

fn localized_block_name(block_id: &str) -> String {
    let bare = block_id.strip_prefix("minecraft:").unwrap_or(block_id);
    let key = format!("block.minecraft.{bare}");
    let translated = Text::translate(&key, &[]).plain();
    if translated != key && !translated.is_empty() {
        translated
    } else {
        format_pretty_id(bare)
    }
}

fn localized_mob_name(mob_id: &str, custom_name: Option<&str>) -> String {
    if let Some(custom) = custom_name
        && !custom.is_empty()
    {
        return custom.to_owned();
    }
    let bare = mob_id.strip_prefix("minecraft:").unwrap_or(mob_id);
    let key = format!("entity.minecraft.{bare}");
    let translated = Text::translate(&key, &[]).plain();
    if translated != key && !translated.is_empty() {
        translated
    } else {
        format_pretty_id(bare)
    }
}

fn is_veinmineable_ore(block_name: &str) -> bool {
    let bare = block_name.strip_prefix("minecraft:").unwrap_or(block_name);
    bare.ends_with("_ore") || bare == "ancient_debris" || bare == "gilded_blackstone"
}

fn ore_family(block_name: &str) -> &str {
    let bare = block_name.strip_prefix("minecraft:").unwrap_or(block_name);
    bare.strip_prefix("deepslate_").unwrap_or(bare)
}

fn bool_label(val: bool) -> &'static str {
    if val { "ON" } else { "OFF" }
}

fn efficiency_level(stack: &ItemStack) -> u32 {
    stack
        .extra
        .as_ref()
        .and_then(|d| {
            d.enchantments
                .iter()
                .find(|(id, _)| &**id == "minecraft:efficiency" || &**id == "efficiency")
                .map(|(_, lvl)| *lvl)
        })
        .unwrap_or(0)
}

impl Game<'_> {
    fn ensure_rust_mods_loaded(&self) {
        with_state(|st| {
            if st.initialized {
                return;
            }
            st.initialized = true;
            if let Some(v) = self.native_setting(MOD_ID_BEDROCK_CRAFTING, "enabled") {
                let enabled = v != "0";
                st.bedrock_crafting_enabled = enabled;
                super::bedrock_crafting::with_bedrock_state(|b| b.enabled = enabled);
            }
            if let Some(v) = self.native_setting(MOD_ID_BEDROCK_CRAFTING, "craftable_only") {
                let craftable_only = v == "1";
                super::bedrock_crafting::with_bedrock_state(|b| b.craftable_only = craftable_only);
            }
            if let Some(v) = self.native_setting(MOD_ID_WAILA, "waila_enabled") {
                st.waila_enabled = v != "0";
            }
            if let Some(v) = self.native_setting(MOD_ID_WAILA, "health_bars_enabled") {
                st.health_bars_enabled = v != "0";
            }
            if let Some(v) = self.native_setting(MOD_ID_WAILA, "vein_preview_enabled") {
                st.vein_preview_enabled = v != "0";
            }
            if let Some(v) = self.native_setting(MOD_ID_WAILA, "claim_badge_enabled") {
                st.claim_badge_enabled = v != "0";
            }
            if let Some(v) = self.native_setting(MOD_ID_VEINMINER, "veinminer_enabled") {
                st.veinminer_enabled = v != "0";
            }
            if let Some(v) = self.native_setting(MOD_ID_VEINMINER, "auto_tool_enabled") {
                st.auto_tool_enabled = v != "0";
            }
            if let Some(v) = self.native_setting(MOD_ID_VEINMINER, "save_durability_enabled") {
                st.save_durability_enabled = v != "0";
            }
        });
    }

    /// Registers Mod #1 (Bedrock Crafting), Mod #4 (WAILA), and Mod #5 (Veinminer) in `Options > Mods`.
    pub(super) fn rust_mods_list(&mut self, mods: &mut Vec<NativeMod>) {
        self.ensure_rust_mods_loaded();
        mods.push(NativeMod {
            id: MOD_ID_BEDROCK_CRAFTING.to_owned(),
            title: "Bedrock Crafting UI 1.0".to_owned(),
            description: "Bedrock Edition categorized crafting UI (Construction, Equipment, Items, Nature), one-click instant crafting, controller bumper navigation, and recipe book sync.".to_owned(),
            keys: vec![[
                "toggle_bedrock_crafting".to_owned(),
                "Toggle Bedrock Crafting UI".to_owned(),
                "key.keyboard.b".to_owned(),
            ]],
        });
        mods.push(NativeMod {
            id: MOD_ID_WAILA.to_owned(),
            title: "WAILA & Health Bars 1.0".to_owned(),
            description: "Top-center Jade/WAILA block inspector (tool tier, break %, ore vein preview, chunk claims) and dynamic Enemy Health Bars.".to_owned(),
            keys: vec![[
                "toggle_waila".to_owned(),
                "Toggle WAILA & Health HUD".to_owned(),
                "key.keyboard.h".to_owned(),
            ]],
        });
        mods.push(NativeMod {
            id: MOD_ID_VEINMINER.to_owned(),
            title: "Veinminer & Smart Tool 1.0".to_owned(),
            description: "3D connected ore Veinminer toggle, Smart Auto-Tool hotbar switcher (with 1-HP durability guard), and Auto-Refill.".to_owned(),
            keys: vec![
                [
                    "toggle_veinminer".to_owned(),
                    "Toggle Veinminer".to_owned(),
                    "key.keyboard.v".to_owned(),
                ],
                [
                    "toggle_autotool".to_owned(),
                    "Toggle Smart Auto-Tool".to_owned(),
                    "key.keyboard.g".to_owned(),
                ],
            ],
        });
    }

    /// Populates settings buttons for `Options > Mods`.
    pub(super) fn rust_mods_settings(&mut self, mod_id: &str, buttons: &mut Vec<String>) {
        self.ensure_rust_mods_loaded();
        with_state(|st| match mod_id {
            MOD_ID_BEDROCK_CRAFTING => {
                let (enabled, craftable_only) = super::bedrock_crafting::with_bedrock_state(|b| {
                    (b.enabled, b.craftable_only)
                });
                buttons.push(format!(
                    "Bedrock Crafting UI: {}",
                    bool_label(enabled)
                ));
                buttons.push(format!(
                    "Filter Craftable Only: {}",
                    bool_label(craftable_only)
                ));
            }
            MOD_ID_WAILA => {
                buttons.push(format!(
                    "Block Inspector HUD: {}",
                    bool_label(st.waila_enabled)
                ));
                buttons.push(format!(
                    "Enemy Health Bars: {}",
                    bool_label(st.health_bars_enabled)
                ));
                buttons.push(format!(
                    "Ore Vein Preview: {}",
                    bool_label(st.vein_preview_enabled)
                ));
                buttons.push(format!(
                    "Chunk Claim Badge: {}",
                    bool_label(st.claim_badge_enabled)
                ));
            }
            MOD_ID_VEINMINER => {
                buttons.push(format!(
                    "Veinminer (Server): {}",
                    bool_label(st.veinminer_enabled)
                ));
                buttons.push(format!(
                    "Smart Auto-Tool Switch: {}",
                    bool_label(st.auto_tool_enabled)
                ));
                buttons.push(format!(
                    "Save 1-Durability Tools: {}",
                    bool_label(st.save_durability_enabled)
                ));
            }
            _ => {}
        });
    }

    /// Handles button clicks in `Options > Mods`.
    pub(super) fn rust_mods_settings_click(&mut self, mod_id: &str, button: usize) {
        self.ensure_rust_mods_loaded();
        let mut send_vm_cmd: Option<bool> = None;
        with_state(|st| match (mod_id, button) {
            (MOD_ID_BEDROCK_CRAFTING, 0) => {
                let enabled = super::bedrock_crafting::with_bedrock_state(|b| {
                    b.enabled = !b.enabled;
                    b.enabled
                });
                st.bedrock_crafting_enabled = enabled;
                self.native_set_setting(
                    MOD_ID_BEDROCK_CRAFTING,
                    "enabled",
                    if enabled { "1" } else { "0" },
                );
            }
            (MOD_ID_BEDROCK_CRAFTING, 1) => {
                let craftable_only = super::bedrock_crafting::with_bedrock_state(|b| {
                    b.craftable_only = !b.craftable_only;
                    b.craftable_only
                });
                self.native_set_setting(
                    MOD_ID_BEDROCK_CRAFTING,
                    "craftable_only",
                    if craftable_only { "1" } else { "0" },
                );
            }
            (MOD_ID_WAILA, 0) => {
                st.waila_enabled = !st.waila_enabled;
                self.native_set_setting(
                    MOD_ID_WAILA,
                    "waila_enabled",
                    if st.waila_enabled { "1" } else { "0" },
                );
            }
            (MOD_ID_WAILA, 1) => {
                st.health_bars_enabled = !st.health_bars_enabled;
                self.native_set_setting(
                    MOD_ID_WAILA,
                    "health_bars_enabled",
                    if st.health_bars_enabled { "1" } else { "0" },
                );
            }
            (MOD_ID_WAILA, 2) => {
                st.vein_preview_enabled = !st.vein_preview_enabled;
                self.native_set_setting(
                    MOD_ID_WAILA,
                    "vein_preview_enabled",
                    if st.vein_preview_enabled { "1" } else { "0" },
                );
            }
            (MOD_ID_WAILA, 3) => {
                st.claim_badge_enabled = !st.claim_badge_enabled;
                self.native_set_setting(
                    MOD_ID_WAILA,
                    "claim_badge_enabled",
                    if st.claim_badge_enabled { "1" } else { "0" },
                );
            }
            (MOD_ID_VEINMINER, 0) => {
                st.veinminer_enabled = !st.veinminer_enabled;
                self.native_set_setting(
                    MOD_ID_VEINMINER,
                    "veinminer_enabled",
                    if st.veinminer_enabled { "1" } else { "0" },
                );
                send_vm_cmd = Some(st.veinminer_enabled);
            }
            (MOD_ID_VEINMINER, 1) => {
                st.auto_tool_enabled = !st.auto_tool_enabled;
                self.native_set_setting(
                    MOD_ID_VEINMINER,
                    "auto_tool_enabled",
                    if st.auto_tool_enabled { "1" } else { "0" },
                );
            }
            (MOD_ID_VEINMINER, 2) => {
                st.save_durability_enabled = !st.save_durability_enabled;
                self.native_set_setting(
                    MOD_ID_VEINMINER,
                    "save_durability_enabled",
                    if st.save_durability_enabled { "1" } else { "0" },
                );
            }
            _ => {}
        });

        if let Some(enabled) = send_vm_cmd
            && self.remote.is_some()
        {
            let cmd = if enabled {
                "veinminer on"
            } else {
                "veinminer off"
            };
            self.send(S::ChatCommand(play::ChatCommand {
                command: cmd.to_owned(),
            }));
        }
    }

    /// Handles custom keybinds while playing (`V` for Veinminer, `G` for Smart Auto-Tool, `H` for WAILA HUD).
    pub(super) fn rust_mods_key(&mut self, key: &str, pressed: bool) {
        if !pressed || !self.native_window() || !self.native_playing() {
            return;
        }
        self.ensure_rust_mods_loaded();

        let key_bedrock = self.native_mod_key(
            MOD_ID_BEDROCK_CRAFTING,
            "toggle_bedrock_crafting",
            "key.keyboard.b",
        );
        let key_waila = self.native_mod_key(MOD_ID_WAILA, "toggle_waila", "key.keyboard.h");
        let key_vm = self.native_mod_key(MOD_ID_VEINMINER, "toggle_veinminer", "key.keyboard.v");
        let key_autotool =
            self.native_mod_key(MOD_ID_VEINMINER, "toggle_autotool", "key.keyboard.g");

        if key == key_bedrock {
            let enabled = super::bedrock_crafting::with_bedrock_state(|b| {
                b.enabled = !b.enabled;
                b.enabled
            });
            with_state(|st| st.bedrock_crafting_enabled = enabled);
            self.native_set_setting(
                MOD_ID_BEDROCK_CRAFTING,
                "enabled",
                if enabled { "1" } else { "0" },
            );
            self.native_chat(&format!(
                "[Bedrock Crafting UI] {}",
                bool_label(enabled)
            ));
        } else if key == key_waila {
            let enabled = with_state(|st| {
                st.waila_enabled = !st.waila_enabled;
                self.native_set_setting(
                    MOD_ID_WAILA,
                    "waila_enabled",
                    if st.waila_enabled { "1" } else { "0" },
                );
                st.waila_enabled
            });
            self.native_chat(&format!(
                "[WAILA] Target Inspector HUD: {}",
                bool_label(enabled)
            ));
        } else if key == key_vm {
            let enabled = with_state(|st| {
                st.veinminer_enabled = !st.veinminer_enabled;
                self.native_set_setting(
                    MOD_ID_VEINMINER,
                    "veinminer_enabled",
                    if st.veinminer_enabled { "1" } else { "0" },
                );
                st.veinminer_enabled
            });
            if self.remote.is_some() {
                let cmd = if enabled {
                    "veinminer on"
                } else {
                    "veinminer off"
                };
                self.send(S::ChatCommand(play::ChatCommand {
                    command: cmd.to_owned(),
                }));
            } else {
                self.native_chat(&format!("[Veinminer] {}", bool_label(enabled)));
            }
        } else if key == key_autotool {
            let enabled = with_state(|st| {
                st.auto_tool_enabled = !st.auto_tool_enabled;
                self.native_set_setting(
                    MOD_ID_VEINMINER,
                    "auto_tool_enabled",
                    if st.auto_tool_enabled { "1" } else { "0" },
                );
                st.auto_tool_enabled
            });
            self.native_chat(&format!(
                "[Smart Auto-Tool] Hotbar Auto-Switch: {}",
                bool_label(enabled)
            ));
        }
    }

    /// Receives server-to-client `horizon:mod_net` packets (`waila:claims` and `veinminer:state`).
    pub(super) fn rust_mods_net(&mut self, channel: &str, data: &[u8]) {
        self.ensure_rust_mods_loaded();
        match channel {
            "waila:claims" => {
                let Ok(text) = std::str::from_utf8(data) else {
                    return;
                };
                with_state(|st| {
                    for line in text.lines() {
                        if let Some(rest) = line.strip_prefix("CENTER:") {
                            let mut parts = rest.split(',');
                            if let (Some(cx_s), Some(cz_s)) = (parts.next(), parts.next())
                                && let (Ok(cx), Ok(cz)) = (cx_s.parse::<i32>(), cz_s.parse::<i32>())
                            {
                                for dx in -2..=2 {
                                    for dz in -2..=2 {
                                        st.chunk_claims.remove(&(cx + dx, cz + dz));
                                    }
                                }
                            }
                        } else {
                            let mut parts = line.splitn(3, ',');
                            if let (Some(cx_s), Some(cz_s), Some(owner)) =
                                (parts.next(), parts.next(), parts.next())
                                && let (Ok(cx), Ok(cz)) = (cx_s.parse::<i32>(), cz_s.parse::<i32>())
                            {
                                st.chunk_claims.insert((cx, cz), owner.to_owned());
                            }
                        }
                    }
                });
            }
            "veinminer:state" => {
                let enabled = data.first().copied() != Some(b'0');
                with_state(|st| {
                    st.veinminer_enabled = enabled;
                    self.native_set_setting(
                        MOD_ID_VEINMINER,
                        "veinminer_enabled",
                        if enabled { "1" } else { "0" },
                    );
                });
            }
            _ => {}
        }
    }

    /// Records the mob currently under the crosshair so the Enemy Health Bar stays visible
    /// during combat maneuvers.
    pub(super) fn rust_mods_tick(&mut self) {
        if !self.native_window() || !self.native_playing() {
            return;
        }
        self.ensure_rust_mods_loaded();

        if let Some((idx, ..)) = self.mob_hit()
            && let Some(mob) = self.mobs.get(idx)
            && mob.alive()
        {
            let mob_id = mob.id;
            let now = self.ticks;
            with_state(|st| {
                st.tracked_mob = Some((mob_id, now));
            });
        }
    }

    /// Runs right before an attack/mine click or held-attack tick in `tick_local_hand`
    /// so Smart Auto-Tool selects the optimal hotbar slot before mining begins.
    pub(super) fn rust_mods_auto_tool_before_attack(&mut self) {
        if !self.native_window() || self.screen != Screen::Playing || self.creative {
            return;
        }
        self.ensure_rust_mods_loaded();
        let (auto_tool, save_durability) =
            with_state(|st| (st.auto_tool_enabled, st.save_durability_enabled));
        if !auto_tool {
            return;
        }

        // If targeting an entity/mob, do not switch to a mining tool
        if self.entity_targeted() {
            return;
        }

        let Some(hit) = self.pick() else {
            return;
        };
        let state = self.world.block(hit.pos[0], hit.pos[1], hit.pos[2]);
        let facts = self.res.facts().get(state);
        // Skip air, fluids, unbreakable blocks, or instant-break blocks (hardness <= 0)
        if facts.hardness <= 0.0 {
            return;
        }
        let block_name = facts.name.clone();
        let requires_tool = facts.requires_tool;

        let is_about_to_break = |g: &Self, slot: usize| -> bool {
            if !save_durability {
                return false;
            }
            let Some(stack) = g.inventory.get(slot).and_then(|s| s.as_ref()) else {
                return false;
            };
            let Some(info) = g.items().get(&stack.item) else {
                return false;
            };
            info.max_damage > 0 && stack.damage.saturating_add(1) >= info.max_damage
        };

        let score_slot = |g: &Self, slot: usize| -> (bool, f32) {
            let Some(stack) = g.inventory.get(slot).and_then(|s| s.as_ref()) else {
                return (!requires_tool, 1.0);
            };
            let correct =
                !requires_tool || g.items().correct_for_drops(Some(stack), &block_name);
            let mut speed = g.items().mining_speed(Some(stack), &block_name);
            if speed > 1.0 {
                let eff = efficiency_level(stack);
                if eff > 0 {
                    speed += (eff * eff + 1) as f32;
                }
            }
            (correct, speed)
        };

        let current_slot = self.selected;
        let current_endangered = is_about_to_break(self, current_slot);
        let (cur_correct, cur_speed) = if current_endangered {
            (false, 0.0)
        } else {
            score_slot(self, current_slot)
        };

        let mut best_slot = current_slot;
        let mut best_correct = cur_correct;
        let mut best_speed = cur_speed;

        for slot in 0..9 {
            if is_about_to_break(self, slot) {
                continue;
            }
            let (correct, speed) = score_slot(self, slot);
            let better = (correct && !best_correct)
                || (correct == best_correct && speed > best_speed + 0.05);
            if better {
                best_slot = slot;
                best_correct = correct;
                best_speed = speed;
            }
        }

        if best_slot != current_slot {
            self.selected = best_slot;
            self.hand_stack_replaced(best_slot);
            if self.remote.is_some() {
                self.send(S::SetCarriedItem(play::SetCarriedItem {
                    slot: best_slot as i16,
                }));
            }
        }
    }

    /// Counts connected matching ore blocks around `origin` (up to 32) for the WAILA Ore Vein Preview.
    fn count_connected_ore_vein(&self, origin: [i32; 3], state: BlockStateId) -> usize {
        let state_raw = state.0;
        if let Some((cached_pos, cached_state, count)) = with_state(|st| st.cached_vein)
            && cached_pos == origin
            && cached_state == state_raw
        {
            return count;
        }

        let origin_facts = self.res.facts().get(state);
        if !is_veinmineable_ore(&origin_facts.name) {
            return 0;
        }
        let target_family = ore_family(&origin_facts.name).to_owned();

        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        visited.insert(origin);
        queue.push_back(origin);
        let mut count = 0usize;

        while let Some(cur) = queue.pop_front() {
            count += 1;
            if count >= 32 {
                break;
            }
            for dx in -1..=1 {
                for dy in -1..=1 {
                    for dz in -1..=1 {
                        if dx == 0 && dy == 0 && dz == 0 {
                            continue;
                        }
                        let np = [cur[0] + dx, cur[1] + dy, cur[2] + dz];
                        if (np[0] - origin[0]).abs() > 8
                            || (np[1] - origin[1]).abs() > 8
                            || (np[2] - origin[2]).abs() > 8
                        {
                            continue;
                        }
                        if !visited.insert(np) {
                            continue;
                        }
                        let ns = self.world.block(np[0], np[1], np[2]);
                        let nf = self.res.facts().get(ns);
                        if is_veinmineable_ore(&nf.name) && ore_family(&nf.name) == target_family {
                            queue.push_back(np);
                        }
                    }
                }
            }
        }

        with_state(|st| {
            st.cached_vein = Some((origin, state_raw, count));
        });
        count
    }

    /// Determines a human-friendly recommended tool & tier label for `state`.
    fn classify_recommended_tool(&self, block_name: &str, state: BlockStateId) -> &'static str {
        let probes: [(&str, &str); 5] = [
            ("minecraft:netherite_pickaxe", "Pickaxe"),
            ("minecraft:netherite_axe", "Axe"),
            ("minecraft:netherite_shovel", "Shovel"),
            ("minecraft:netherite_hoe", "Hoe"),
            ("minecraft:shears", "Shears"),
        ];
        let mut best_tool = "Hand / Any";
        let mut best_speed = 1.0f32;
        for (item_id, label) in probes {
            let probe_stack = ItemStack::new(item_id, 1);
            let sp = self.items().mining_speed(Some(&probe_stack), block_name);
            if sp > best_speed {
                best_speed = sp;
                best_tool = label;
            }
        }
        if best_tool == "Pickaxe" && self.res.facts().get(state).requires_tool {
            let iron = ItemStack::new("minecraft:iron_pickaxe", 1);
            if !self.items().correct_for_drops(Some(&iron), block_name) {
                return "Pickaxe (Diamond+)";
            }
            let stone = ItemStack::new("minecraft:stone_pickaxe", 1);
            if !self.items().correct_for_drops(Some(&stone), block_name) {
                return "Pickaxe (Iron+)";
            }
            let wood = ItemStack::new("minecraft:wooden_pickaxe", 1);
            if !self.items().correct_for_drops(Some(&wood), block_name) {
                return "Pickaxe (Stone+)";
            }
        }
        best_tool
    }

    /// Renders the Top-Center WAILA / Jade Target Inspector & Enemy Health Bar HUD.
    pub(super) fn rust_mods_hud(&mut self, batch: &mut HudBatch, _corners: &mut Corners) {
        if !self.native_window() || self.player.is_none() || self.debug {
            return;
        }
        self.ensure_rust_mods_loaded();

        let (
            waila_enabled,
            health_bars_enabled,
            vein_preview_enabled,
            claim_badge_enabled,
            veinminer_enabled,
        ) = with_state(|st| {
            (
                st.waila_enabled,
                st.health_bars_enabled,
                st.vein_preview_enabled,
                st.claim_badge_enabled,
                st.veinminer_enabled,
            )
        });

        let (screen_w, _screen_h) = self.gui_size();
        let mut top_y = 5.0f32;

        // =====================================================================
        // 1. TOP-CENTER ENEMY HEALTH BAR HUD
        // =====================================================================
        if health_bars_enabled {
            let direct_mob_idx = self.mob_hit().map(|(idx, ..)| idx);
            let target_mob_idx = direct_mob_idx.or_else(|| {
                let (tracked_id, last_tick) = with_state(|st| st.tracked_mob)?;
                if self.ticks.saturating_sub(last_tick) > 70 {
                    return None;
                }
                self.mobs
                    .iter()
                    .position(|m| m.id == tracked_id && m.alive())
            });

            if let Some(idx) = target_mob_idx
                && let Some(mob) = self.mobs.get(idx)
                && mob.alive()
            {
                let player_pos = self.player.as_ref().map(|p| p.position);
                let dist = player_pos.map_or(0.0, |pp| {
                    let dx = mob.position.x - pp.x;
                    let dy = mob.position.y - pp.y;
                    let dz = mob.position.z - pp.z;
                    (dx * dx + dy * dy + dz * dz).sqrt()
                });

                if direct_mob_idx.is_some() || dist <= 20.0 {
                    let mob_name = localized_mob_name(&mob.kind.id, mob.custom_name.as_deref());
                    let max_hp = (mob.attrs.max_health as f32)
                        .max(mob.kind.max_health)
                        .max(1.0);
                    let hp = mob.health.clamp(0.0, max_hp);
                    let ratio = (hp / max_hp).clamp(0.0, 1.0);
                    let pct = (ratio * 100.0).round() as i32;
                    let armor = mob.kind.armor.round() as i32;
                    let hurt_flash = mob.hurt_time > 0;

                    let header_left = if let Some(owner) = self.remote_player_name(mob.id) {
                        owner.to_owned()
                    } else {
                        mob_name
                    };
                    let header_right = if armor > 0 {
                        format!("Armor:{armor}  {dist:.1}m")
                    } else {
                        format!("{dist:.1}m")
                    };
                    let hp_text = format!("{hp:.1} / {max_hp:.1} HP ({pct}%)");

                    let content_w = (self.native_text_width(&header_left)
                        + self.native_text_width(&header_right)
                        + 18.0)
                        .max(self.native_text_width(&hp_text) + 24.0)
                        .max(148.0);
                    let box_w = content_w + 10.0;
                    let box_h = 28.0;
                    let box_x = ((screen_w - box_w) * 0.5).floor();
                    let box_y = top_y;

                    // Outer border (flashes crimson when hurt) and dark glass panel
                    let border_col = if hurt_flash { 0xFFE04040 } else { 0xB03B4252 };
                    self.native_fill(
                        batch,
                        [box_x - 1.0, box_y - 1.0, box_w + 2.0, box_h + 2.0],
                        border_col,
                    );
                    self.native_fill(batch, [box_x, box_y, box_w, box_h], 0xCC11151C);

                    // Header row: Mob Name (left) + Armor & Distance (right)
                    self.native_text(
                        batch,
                        &header_left,
                        [box_x + 5.0, box_y + 3.0],
                        if hurt_flash { 0xFF9999 } else { 0xFFFFFF },
                        true,
                    );
                    let right_w = self.native_text_width(&header_right);
                    self.native_text(
                        batch,
                        &header_right,
                        [box_x + box_w - right_w - 5.0, box_y + 3.0],
                        0xA3BE8C,
                        true,
                    );

                    // Health bar track & dynamic fill
                    let bar_x = box_x + 5.0;
                    let bar_y = box_y + 14.0;
                    let bar_w = box_w - 10.0;
                    let bar_h = 10.0;
                    self.native_fill(batch, [bar_x, bar_y, bar_w, bar_h], 0xFF1E222A);

                    let bar_color = if hurt_flash {
                        0xFFFF6B6B
                    } else if ratio > 0.5 {
                        0xFF2ECC71
                    } else if ratio > 0.25 {
                        0xFFF1C40F
                    } else {
                        0xFFE74C3C
                    };
                    let fill_w = (bar_w * ratio).round();
                    if fill_w > 0.0 {
                        self.native_fill(batch, [bar_x, bar_y, fill_w, bar_h], bar_color);
                    }

                    // Centered HP text inside the health bar
                    let hp_tw = self.native_text_width(&hp_text);
                    self.native_text(
                        batch,
                        &hp_text,
                        [(bar_x + (bar_w - hp_tw) * 0.5).floor(), bar_y + 1.0],
                        0xFFFFFF,
                        true,
                    );

                    top_y += box_h + 4.0;
                }
            }
        }

        // =====================================================================
        // 2. TOP-CENTER WAILA / JADE BLOCK TARGET INSPECTOR HUD
        // =====================================================================
        if !waila_enabled || self.entity_targeted() {
            return;
        }
        let Some(hit) = self.pick() else {
            return;
        };
        let [bx, by, bz] = hit.pos;
        let state = self.world.block(bx, by, bz);
        let facts = self.res.facts().get(state);
        if matches!(
            &*facts.name,
            "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
        ) {
            return;
        }

        let block_id = facts.name.to_string();
        let icon_item = if facts.item.is_empty() {
            block_id.clone()
        } else {
            facts.item.to_string()
        };
        let hardness = facts.hardness;
        let requires_tool = facts.requires_tool;

        let display_name = localized_block_name(&block_id);
        let coords_str = format!("[{bx}, {by}, {bz}]");
        let line1 = format!("{display_name}  {coords_str}");

        // Harvestability & Tool speed evaluation
        let held_stack = self.held().cloned();
        let harvestable = if hardness < 0.0 {
            false
        } else if !requires_tool {
            true
        } else {
            self.items()
                .correct_for_drops(held_stack.as_ref(), &block_id)
        };

        let rec_tool = self.classify_recommended_tool(&block_id, state);
        let (line2, line2_color) = if hardness < 0.0 {
            ("Unbreakable Block".to_owned(), 0xFF6B6B)
        } else if harvestable {
            let speed = self.items().mining_speed(held_stack.as_ref(), &block_id);
            if rec_tool == "Hand / Any" && !requires_tool {
                (format!("Harvestable  (Tool: {rec_tool})"), 0x88DD88)
            } else {
                (
                    format!("Harvestable  ({rec_tool} | {speed:.0}x)"),
                    0x55FF55,
                )
            }
        } else {
            (format!("Needs {rec_tool} to Harvest"), 0xFF6666)
        };

        // Line 3: Connected Ore Vein Preview + Chunk Claim Owner
        let mut badges = Vec::new();
        if vein_preview_enabled && is_veinmineable_ore(&block_id) {
            let vein_count = self.count_connected_ore_vein([bx, by, bz], state);
            if vein_count > 0 {
                let vm_tag = if veinminer_enabled { "VM:ON" } else { "VM:OFF" };
                badges.push(format!("Vein: {vein_count} Ores ({vm_tag})"));
            }
        }
        if claim_badge_enabled {
            let cx = bx >> 4;
            let cz = bz >> 4;
            let claim_owner = with_state(|st| st.chunk_claims.get(&(cx, cz)).cloned());
            if let Some(owner) = claim_owner {
                badges.push(format!("Claim: {owner}"));
            } else if self.remote.is_some() {
                badges.push("Wilderness".to_owned());
            }
        }
        let line3 = badges.join("  |  ");

        // Active Mining Progress
        let mining_progress = self
            .mining
            .as_ref()
            .filter(|m| m.pos == [bx, by, bz])
            .map(|m| m.progress.clamp(0.0, 1.0));

        let icon_pad = 22.0f32;
        let mut text_w = self
            .native_text_width(&line1)
            .max(self.native_text_width(&line2));
        if !line3.is_empty() {
            text_w = text_w.max(self.native_text_width(&line3));
        }

        let box_w = (icon_pad + text_w + 10.0).max(148.0);
        let mut box_h = if line3.is_empty() { 24.0 } else { 34.0 };
        if mining_progress.is_some() {
            box_h += 8.0;
        }
        let box_x = ((screen_w - box_w) * 0.5).floor();
        let box_y = top_y;

        // Card border & background
        self.native_fill(
            batch,
            [box_x - 1.0, box_y - 1.0, box_w + 2.0, box_h + 2.0],
            0xB03B4252,
        );
        self.native_fill(batch, [box_x, box_y, box_w, box_h], 0xCC11151C);

        // Render 16x16 Block Item Icon
        let _ = self.native_item(batch, &icon_item, [box_x + 4.0, box_y + 4.0], 16.0);

        // Line 1: Block Name + Coords
        let name_w = self.native_text_width(&display_name);
        self.native_text(
            batch,
            &display_name,
            [box_x + icon_pad, box_y + 3.0],
            0xFFFFFF,
            true,
        );
        self.native_text(
            batch,
            &coords_str,
            [box_x + icon_pad + name_w + 6.0, box_y + 3.0],
            0x99A3B0,
            true,
        );

        // Line 2: Tool & Harvestability
        self.native_text(
            batch,
            &line2,
            [box_x + icon_pad, box_y + 13.0],
            line2_color,
            true,
        );

        // Line 3: Ore Vein & Chunk Claim badges
        if !line3.is_empty() {
            let badge_color = if is_veinmineable_ore(&block_id) {
                0x55FFFF
            } else {
                0xEBCB8B
            };
            self.native_text(
                batch,
                &line3,
                [box_x + icon_pad, box_y + 23.0],
                badge_color,
                true,
            );
        }

        // Bottom Mining Progress Bar (when actively breaking this block)
        if let Some(prog) = mining_progress {
            let bar_x = box_x + 4.0;
            let bar_y = box_y + box_h - 6.0;
            let bar_w = box_w - 8.0;
            self.native_fill(batch, [bar_x, bar_y, bar_w, 4.0], 0xFF222630);
            let fill_w = (bar_w * prog).round();
            if fill_w > 0.0 {
                let col = if harvestable { 0xFF55FF55 } else { 0xFFFF5555 };
                self.native_fill(batch, [bar_x, bar_y, fill_w, 4.0], col);
            }
        }
    }
}
