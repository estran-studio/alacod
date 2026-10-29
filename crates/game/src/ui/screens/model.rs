//! Modèle pur des écrans (M2-T18, `docs/conventions.md` §56).
//!
//! La [`ScreenView`] décrit des sections nommées (texte, liste avec focus, jauge), indépendamment
//! de leur mise en page dans les gabarits `bevy_hui`.
//! Présentation seule : rien ici ne touche la simulation.

use std::collections::BTreeMap;

use bevy::prelude::Resource;

/// Écran ouvert.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ScreenId {
    Pause,
    Inventory,
    /// Marchand de déblocages du hub (M2-T13, `docs/conventions.md` §57).
    Merchant,
}

impl ScreenId {
    pub fn name(self) -> &'static str {
        match self {
            Self::Pause => "pause",
            Self::Inventory => "inventory",
            Self::Merchant => "merchant",
        }
    }
}

/// Une commande de navigation, quelle que soit sa source (clavier, manette, script de scénario).
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum UiAction {
    /// Ouvre l'écran, ou le ferme s'il est déjà ouvert.
    Toggle(ScreenId),
    Up,
    Down,
    /// Gauche / droite : règle le volume global dans « Réglages » de la pause.
    Left,
    Right,
    Confirm,
    /// Ferme l'écran ouvert.
    Back,
}

/// Un pas du script d'un scénario : l'action est jouée à la frame `at_frame` (format RON :
/// `(at_frame: 20, action: Toggle(Pause))`).
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UiStep {
    pub at_frame: u32,
    pub action: UiAction,
}

/// Script d'actions d'UI d'un scénario (ressource posée par le runner ; absente en jeu).
#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub struct ScreenScript {
    pub steps: Vec<UiStep>,
    /// Dernière frame dont les pas ont été joués : un pas ne l'est qu'une fois.
    pub played_until: Option<u32>,
}

/// Les entrées du menu de pause, dans l'ordre d'affichage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PauseEntry {
    Resume,
    Restart,
    Lobby,
    Settings,
    /// Écran d'aide des touches (S1-1, `screens::help`).
    Help,
}

pub const PAUSE_ENTRIES: [PauseEntry; 5] = [
    PauseEntry::Resume,
    PauseEntry::Restart,
    PauseEntry::Lobby,
    PauseEntry::Settings,
    PauseEntry::Help,
];

impl PauseEntry {
    pub fn label(self) -> &'static str {
        match self {
            Self::Resume => "Reprendre",
            Self::Restart => "Recommencer",
            Self::Lobby => "Lobby",
            Self::Settings => "Réglages",
            Self::Help => "Touches",
        }
    }
}

/// Ce que la présentation demande au reste du jeu (le seul chemin hors de l'écran).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScreenCommand {
    Restart,
    ToLobby,
    /// Acheter la ligne `n` du marchand (M2-T13), hors simulation.
    Buy(usize),
    /// Décale le volume global de `n` crans (S1-7).
    Volume(i8),
}

/// État local des écrans (présentation, hors rollback).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScreensState {
    pub open: Option<ScreenId>,
    pub pause_focus: usize,
    pub inventory_focus: usize,
    pub merchant_focus: usize,
    /// « Réglages » validé : l'écran des réglages est vide, on l'affiche.
    pub settings_shown: bool,
    /// « Touches » validé : l'aide des touches remplace le détail de la pause.
    pub help_shown: bool,
}

/// Déplace le focus de `step` (−1 / +1), en boucle sur `len` entrées.
pub fn focus_step(focus: usize, len: usize, step: i32) -> usize {
    if len == 0 {
        return 0;
    }
    (focus as i64 + step as i64).rem_euclid(len as i64) as usize
}

/// Applique une action (pur). `inventory_len` : nombre d'éléments qui peuvent prendre le focus
/// dans l'écran ouvert à liste (l'inventaire, ou les déblocages du marchand). Renvoie la commande à transmettre au jeu, s'il y en a une.
pub fn reduce(
    state: &mut ScreensState,
    action: UiAction,
    inventory_len: usize,
) -> Option<ScreenCommand> {
    match action {
        UiAction::Toggle(screen) => {
            if state.open == Some(screen) {
                state.open = None;
            } else {
                *state = ScreensState {
                    open: Some(screen),
                    ..ScreensState::default()
                };
            }
            None
        }
        UiAction::Back => {
            // L'aide des touches se referme d'abord sur le menu de pause, qui se ferme au retour suivant.
            if state.open == Some(ScreenId::Pause) && state.help_shown {
                state.help_shown = false;
                return None;
            }
            state.open = None;
            state.settings_shown = false;
            None
        }
        UiAction::Up | UiAction::Down => {
            let step = if action == UiAction::Up { -1 } else { 1 };
            match state.open {
                Some(ScreenId::Pause) => {
                    state.settings_shown = false;
                    state.help_shown = false;
                    state.pause_focus = focus_step(state.pause_focus, PAUSE_ENTRIES.len(), step);
                }
                Some(ScreenId::Inventory) => {
                    state.inventory_focus = focus_step(state.inventory_focus, inventory_len, step);
                }
                Some(ScreenId::Merchant) => {
                    state.merchant_focus = focus_step(state.merchant_focus, inventory_len, step);
                }
                None => {}
            }
            None
        }
        UiAction::Left | UiAction::Right => {
            if state.open == Some(ScreenId::Pause) && state.settings_shown {
                Some(ScreenCommand::Volume(if action == UiAction::Left {
                    -1
                } else {
                    1
                }))
            } else {
                None
            }
        }
        UiAction::Confirm => match state.open {
            Some(ScreenId::Pause) => match PAUSE_ENTRIES[state.pause_focus % PAUSE_ENTRIES.len()] {
                PauseEntry::Resume => {
                    state.open = None;
                    None
                }
                PauseEntry::Restart => {
                    state.open = None;
                    Some(ScreenCommand::Restart)
                }
                PauseEntry::Lobby => {
                    state.open = None;
                    Some(ScreenCommand::ToLobby)
                }
                PauseEntry::Settings => {
                    state.settings_shown = true;
                    state.help_shown = false;
                    None
                }
                PauseEntry::Help => {
                    state.help_shown = true;
                    state.settings_shown = false;
                    None
                }
            },
            Some(ScreenId::Merchant) if inventory_len > 0 => Some(ScreenCommand::Buy(
                state.merchant_focus.min(inventory_len - 1),
            )),
            _ => None,
        },
    }
}

/// Une ligne de liste.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ListItem {
    pub label: String,
    pub detail: String,
    pub focused: bool,
}

/// Le contenu d'une section nommée.
#[derive(Clone, Debug, PartialEq)]
pub enum Section {
    Text(String),
    List(Vec<ListItem>),
    /// Jauge : texte et fraction dans `[0, 1]`.
    Bar {
        label: String,
        fraction: f32,
    },
}

/// Ce qu'un écran affiche cette frame.
#[derive(Clone, Debug, PartialEq)]
pub struct ScreenView {
    pub screen: ScreenId,
    pub sections: BTreeMap<String, Section>,
}

impl ScreenView {
    /// Texte brut d'une section (pour l'attente de scénario `ScreenText`) : une ligne par
    /// élément, « > » devant l'élément qui a le focus, « libellé  détail ».
    pub fn section_text(&self, name: &str) -> String {
        match self.sections.get(name) {
            None => String::new(),
            Some(Section::Text(text)) => text.clone(),
            Some(Section::Bar { label, .. }) => label.clone(),
            Some(Section::List(items)) => items
                .iter()
                .map(|item| {
                    let mark = if item.focused { "> " } else { "  " };
                    if item.detail.is_empty() {
                        format!("{mark}{}", item.label)
                    } else {
                        format!("{mark}{}  {}", item.label, item.detail)
                    }
                })
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }
}

fn text(value: impl Into<String>) -> Section {
    Section::Text(value.into())
}

/// Vue du menu de pause (pure), sans texte d'aide : « Touches » n'y montre rien.
pub fn pause_view(state: &ScreensState) -> ScreenView {
    pause_view_with_help(state, "")
}

/// Vue du menu de pause (pure). `help` : le texte de l'aide des touches, montré à la place du
/// détail quand « Touches » est validé (`screens::help::help_text`).
pub fn pause_view_with_help(state: &ScreensState, help: &str) -> ScreenView {
    pause_view_full(state, help, 100)
}

/// Texte du curseur de volume global : « Volume : 70 %  [#######---] (← → pour régler) ».
pub fn volume_text(percent: u8) -> String {
    let filled = usize::from(percent.min(100)) / 10;
    format!(
        "Volume : {percent} %  [{}{}]  (← → pour régler)",
        "#".repeat(filled),
        "-".repeat(10 - filled)
    )
}

/// Vue du menu de pause au volume global `volume_percent` (S1-7) et au texte d'aide `help`.
pub fn pause_view_full(state: &ScreensState, help: &str, volume_percent: u8) -> ScreenView {
    let focus = state.pause_focus % PAUSE_ENTRIES.len();
    let entries = PAUSE_ENTRIES
        .iter()
        .enumerate()
        .map(|(index, entry)| ListItem {
            label: entry.label().into(),
            detail: String::new(),
            focused: index == focus,
        })
        .collect();
    let mut sections = BTreeMap::new();
    sections.insert("title".into(), text("Pause"));
    sections.insert("entries".into(), Section::List(entries));
    sections.insert(
        "detail".into(),
        text(if state.help_shown {
            help.to_string()
        } else if state.settings_shown {
            volume_text(volume_percent)
        } else {
            String::new()
        }),
    );
    sections.insert(
        "hint".into(),
        text("↑ ↓ choisir · Entrée / A valider · Échap / B reprendre"),
    );
    ScreenView {
        screen: ScreenId::Pause,
        sections,
    }
}

/// Une arme de l'inventaire.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WeaponLine {
    pub name: String,
    /// Munitions du chargeur du mode actif, si l'arme en a.
    pub mag: Option<u32>,
    pub reserve: u32,
    pub active: bool,
}

/// L'objet actif tenu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActiveLine {
    pub name: String,
    pub charge: u32,
    pub required: u32,
}

/// Ce que l'inventaire lit de l'état du joueur (déjà résolu en noms par le système appelant).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InventoryInput {
    pub weapons: Vec<WeaponLine>,
    pub active: Option<ActiveLine>,
    /// Noms des passifs, dans l'ordre de ramassage.
    pub passives: Vec<String>,
    /// `(nom, quantité)`, dans l'ordre des ids.
    pub consumables: Vec<(String, u32)>,
}

/// Les sections de l'inventaire qui prennent le focus, dans l'ordre où la manette les parcourt.
pub const INVENTORY_FOCUS_ORDER: [&str; 4] = ["weapons", "active", "passives", "consumables"];

/// Nombre d'éléments qui prennent le focus.
pub fn inventory_focus_len(input: &InventoryInput) -> usize {
    input.weapons.len()
        + usize::from(input.active.is_some())
        + input.passives.len()
        + input.consumables.len()
}

/// Section et index dans la section de l'élément qui a le focus (`None` : inventaire vide).
pub fn inventory_focus_target(
    input: &InventoryInput,
    focus: usize,
) -> Option<(&'static str, usize)> {
    let counts = [
        input.weapons.len(),
        usize::from(input.active.is_some()),
        input.passives.len(),
        input.consumables.len(),
    ];
    let total: usize = counts.iter().sum();
    if total == 0 {
        return None;
    }
    let mut remaining = focus % total;
    for (name, count) in INVENTORY_FOCUS_ORDER.iter().zip(counts) {
        if remaining < count {
            return Some((name, remaining));
        }
        remaining -= count;
    }
    None
}

/// Vue de l'inventaire gungeon (pure) : armes, actif et sa charge, passifs, consommables.
pub fn inventory_view(input: &InventoryInput, state: &ScreensState) -> ScreenView {
    let target = inventory_focus_target(input, state.inventory_focus);
    let is_focused =
        |section: &str, index: usize| target.is_some_and(|(s, i)| s == section && i == index);
    let weapons = input
        .weapons
        .iter()
        .enumerate()
        .map(|(index, weapon)| ListItem {
            label: if weapon.active {
                format!("{} (en main)", weapon.name)
            } else {
                weapon.name.clone()
            },
            detail: match weapon.mag {
                Some(mag) => format!("{mag} | {}", weapon.reserve),
                None => String::new(),
            },
            focused: is_focused("weapons", index),
        })
        .collect();
    let passives = input
        .passives
        .iter()
        .enumerate()
        .map(|(index, name)| ListItem {
            label: name.clone(),
            detail: String::new(),
            focused: is_focused("passives", index),
        })
        .collect();
    let consumables = input
        .consumables
        .iter()
        .enumerate()
        .map(|(index, (name, count))| ListItem {
            label: name.clone(),
            detail: format!("×{count}"),
            focused: is_focused("consumables", index),
        })
        .collect();
    let (active_items, charge) = match &input.active {
        Some(active) => (
            vec![ListItem {
                label: active.name.clone(),
                detail: String::new(),
                focused: is_focused("active", 0),
            }],
            Section::Bar {
                label: format!(
                    "{} / {}{}",
                    active.charge.min(active.required),
                    active.required,
                    if active.charge >= active.required {
                        " · prêt"
                    } else {
                        ""
                    }
                ),
                fraction: if active.required == 0 {
                    1.0
                } else {
                    (active.charge as f32 / active.required as f32).clamp(0.0, 1.0)
                },
            },
        ),
        None => (
            Vec::new(),
            Section::Bar {
                label: String::new(),
                fraction: 0.0,
            },
        ),
    };
    let mut sections = BTreeMap::new();
    sections.insert("title".into(), text("Inventaire"));
    sections.insert("weapons".into(), Section::List(weapons));
    sections.insert("active".into(), Section::List(active_items));
    sections.insert("charge".into(), charge);
    sections.insert("passives".into(), Section::List(passives));
    sections.insert("consumables".into(), Section::List(consumables));
    sections.insert("hint".into(), text("↑ ↓ parcourir · I / Échap / B fermer"));
    ScreenView {
        screen: ScreenId::Inventory,
        sections,
    }
}

/// État d'un déblocage pour le joueur au marchand.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MerchantStatus {
    Owned,
    /// Achetable maintenant.
    Available,
    /// Pas encore : la raison (prérequis, statistique ou essence manquants).
    Locked(String),
}

/// Une ligne du marchand : un `Unlock` du jeu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MerchantLine {
    pub id: String,
    pub name: String,
    pub cost: Option<i64>,
    pub status: MerchantStatus,
}

/// Les déblocages du jeu (ordre des ids) et leur état pour `profile` (pur). Un achat qui serait
/// refusé (`meta::Profile::purchase`) donne `Locked(raison)`.
pub fn merchant_lines(
    unlocks: &[(String, items::UnlockDef)],
    profile: &meta::Profile,
) -> Vec<MerchantLine> {
    unlocks
        .iter()
        .map(|(id, def)| {
            let status = if profile.has_unlock(id) {
                MerchantStatus::Owned
            } else {
                match profile.clone().purchase(
                    id,
                    &meta::PurchaseTerms {
                        cost: def.cost,
                        requires: &def.requires,
                        stat: def.stat.as_ref().map(|(stat, need)| (stat.as_str(), *need)),
                    },
                ) {
                    Ok(()) => MerchantStatus::Available,
                    Err(error) => MerchantStatus::Locked(error.to_string()),
                }
            };
            MerchantLine {
                id: id.clone(),
                name: if def.name.is_empty() {
                    id.clone()
                } else {
                    def.name.clone()
                },
                cost: def.cost,
                status,
            }
        })
        .collect()
}

/// Vue du marchand (pure) : l'essence du joueur, la liste (focus), le détail de la ligne
/// focalisée et `message` (résultat du dernier achat).
pub fn merchant_view(
    lines: &[MerchantLine],
    state: &ScreensState,
    essence: u64,
    message: &str,
) -> ScreenView {
    let focus = state.merchant_focus.min(lines.len().saturating_sub(1));
    let entries = lines
        .iter()
        .enumerate()
        .map(|(index, line)| ListItem {
            label: line.name.clone(),
            detail: match (&line.status, line.cost) {
                (MerchantStatus::Owned, _) => "possédé".into(),
                (_, Some(cost)) => format!("{cost} essence"),
                (_, None) => "gratuit".into(),
            },
            focused: index == focus,
        })
        .collect();
    let detail = match lines.get(focus).map(|line| &line.status) {
        Some(MerchantStatus::Locked(reason)) => reason.clone(),
        _ => String::new(),
    };
    let mut sections = BTreeMap::new();
    sections.insert("title".into(), text("Marchand"));
    sections.insert("essence".into(), text(format!("Essence : {essence}")));
    sections.insert("entries".into(), Section::List(entries));
    sections.insert("detail".into(), text(detail));
    sections.insert("message".into(), text(message));
    sections.insert(
        "hint".into(),
        text("↑ ↓ choisir · Entrée / A acheter · Échap / B fermer"),
    );
    ScreenView {
        screen: ScreenId::Merchant,
        sections,
    }
}

/// Touches de navigation, déjà lues (clavier ou manette).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum NavKey {
    /// Échap / Start.
    Menu,
    /// I / Select.
    Inventory,
    Up,
    Down,
    /// Entrée, Espace / A.
    Confirm,
    /// Retour arrière / B.
    Back,
    Left,
    Right,
}

/// Les actions déclenchées par les touches pressées cette frame (pur, ordre de `NavKey`).
/// Échap / Start ouvre la pause, ou ferme l'écran ouvert quel qu'il soit.
pub fn actions_from_keys(open: Option<ScreenId>, pressed: &[NavKey]) -> Vec<UiAction> {
    let mut keys = pressed.to_vec();
    keys.sort();
    keys.dedup();
    keys.into_iter()
        .map(|key| match key {
            NavKey::Menu => match open {
                None => UiAction::Toggle(ScreenId::Pause),
                Some(_) => UiAction::Back,
            },
            NavKey::Inventory => UiAction::Toggle(ScreenId::Inventory),
            NavKey::Up => UiAction::Up,
            NavKey::Down => UiAction::Down,
            NavKey::Left => UiAction::Left,
            NavKey::Right => UiAction::Right,
            NavKey::Confirm => UiAction::Confirm,
            NavKey::Back => UiAction::Back,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> InventoryInput {
        InventoryInput {
            weapons: vec![
                WeaponLine {
                    name: "Pistolet".into(),
                    mag: Some(8),
                    reserve: 40,
                    active: true,
                },
                WeaponLine {
                    name: "Fusil".into(),
                    mag: None,
                    reserve: 0,
                    active: false,
                },
            ],
            active: Some(ActiveLine {
                name: "Fiole de rage".into(),
                charge: 1,
                required: 2,
            }),
            passives: vec!["Amulette".into(), "Bottes".into()],
            consumables: vec![("Blank".into(), 2), ("Clé".into(), 1)],
        }
    }

    #[test]
    fn le_focus_boucle() {
        assert_eq!(focus_step(0, 4, -1), 3);
        assert_eq!(focus_step(3, 4, 1), 0);
        assert_eq!(focus_step(1, 4, 1), 2);
        assert_eq!(focus_step(0, 0, 1), 0);
    }

    #[test]
    fn ordre_du_focus_armes_actif_passifs_consommables() {
        let input = input();
        assert_eq!(inventory_focus_len(&input), 7);
        let order: Vec<_> = (0..7)
            .map(|focus| inventory_focus_target(&input, focus).unwrap())
            .collect();
        assert_eq!(
            order,
            vec![
                ("weapons", 0),
                ("weapons", 1),
                ("active", 0),
                ("passives", 0),
                ("passives", 1),
                ("consumables", 0),
                ("consumables", 1),
            ]
        );
        // Le focus déborde : il reboucle.
        assert_eq!(inventory_focus_target(&input, 7), Some(("weapons", 0)));
        assert_eq!(inventory_focus_target(&InventoryInput::default(), 3), None);
    }

    #[test]
    fn un_seul_element_a_le_focus_dans_l_inventaire() {
        let input = input();
        for focus in 0..7 {
            let view = inventory_view(
                &input,
                &ScreensState {
                    open: Some(ScreenId::Inventory),
                    inventory_focus: focus,
                    ..ScreensState::default()
                },
            );
            let focused: usize = ["weapons", "active", "passives", "consumables"]
                .iter()
                .map(|name| match &view.sections[*name] {
                    Section::List(items) => items.iter().filter(|i| i.focused).count(),
                    _ => 0,
                })
                .sum();
            assert_eq!(focused, 1, "focus {focus}");
        }
    }

    #[test]
    fn texte_de_l_inventaire() {
        let view = inventory_view(&input(), &ScreensState::default());
        assert_eq!(
            view.section_text("weapons"),
            "> Pistolet (en main)  8 | 40\n  Fusil"
        );
        assert_eq!(view.section_text("charge"), "1 / 2");
        assert_eq!(view.section_text("consumables"), "  Blank  ×2\n  Clé  ×1");
        match view.sections["charge"] {
            Section::Bar { fraction, .. } => assert_eq!(fraction, 0.5),
            _ => panic!("jauge"),
        }
    }

    #[test]
    fn actif_pret_et_sans_actif() {
        let mut full = input();
        full.active.as_mut().unwrap().charge = 5;
        let view = inventory_view(&full, &ScreensState::default());
        assert_eq!(view.section_text("charge"), "2 / 2 · prêt");
        let mut none = input();
        none.active = None;
        let view = inventory_view(&none, &ScreensState::default());
        assert_eq!(view.section_text("active"), "");
        assert_eq!(view.section_text("charge"), "");
    }

    #[test]
    fn pause_navigation_et_commandes() {
        let mut state = ScreensState::default();
        assert_eq!(
            reduce(&mut state, UiAction::Toggle(ScreenId::Pause), 0),
            None
        );
        assert_eq!(state.open, Some(ScreenId::Pause));
        assert_eq!(
            pause_view(&state).section_text("entries"),
            "> Reprendre\n  Recommencer\n  Lobby\n  Réglages\n  Touches"
        );
        reduce(&mut state, UiAction::Down, 0);
        assert_eq!(
            reduce(&mut state, UiAction::Confirm, 0),
            Some(ScreenCommand::Restart)
        );
        assert_eq!(state.open, None);

        reduce(&mut state, UiAction::Toggle(ScreenId::Pause), 0);
        reduce(&mut state, UiAction::Up, 0); // boucle sur « Touches »
        assert_eq!(state.pause_focus, 4);
        assert_eq!(reduce(&mut state, UiAction::Confirm, 0), None);
        assert!(state.help_shown);
        assert_eq!(
            pause_view_with_help(&state, "aide").section_text("detail"),
            "aide"
        );
        // Retour : l'aide se referme sur la pause, le retour suivant ferme la pause.
        reduce(&mut state, UiAction::Back, 0);
        assert!(!state.help_shown);
        assert_eq!(state.open, Some(ScreenId::Pause));
        reduce(&mut state, UiAction::Up, 0); // « Réglages »
        assert_eq!(state.pause_focus, 3);
        assert_eq!(reduce(&mut state, UiAction::Confirm, 0), None);
        assert!(state.settings_shown);
        assert!(pause_view(&state)
            .section_text("detail")
            .contains("Volume : 100 %"));
        reduce(&mut state, UiAction::Up, 0);
        assert!(!state.settings_shown);
        assert_eq!(
            reduce(&mut state, UiAction::Confirm, 0),
            Some(ScreenCommand::ToLobby)
        );
    }

    #[test]
    fn reprendre_et_retour_ferment() {
        let mut state = ScreensState::default();
        reduce(&mut state, UiAction::Toggle(ScreenId::Pause), 0);
        assert_eq!(reduce(&mut state, UiAction::Confirm, 0), None);
        assert_eq!(state.open, None);
        reduce(&mut state, UiAction::Toggle(ScreenId::Inventory), 3);
        reduce(&mut state, UiAction::Down, 3);
        assert_eq!(state.inventory_focus, 1);
        reduce(&mut state, UiAction::Toggle(ScreenId::Inventory), 3);
        assert_eq!(state.open, None);
        reduce(&mut state, UiAction::Toggle(ScreenId::Inventory), 3);
        assert_eq!(state.inventory_focus, 0, "rouvrir remet le focus à 0");
        reduce(&mut state, UiAction::Back, 3);
        assert_eq!(state.open, None);
    }

    #[test]
    fn touches_vers_actions() {
        assert_eq!(
            actions_from_keys(None, &[NavKey::Menu]),
            vec![UiAction::Toggle(ScreenId::Pause)]
        );
        assert_eq!(
            actions_from_keys(Some(ScreenId::Inventory), &[NavKey::Menu]),
            vec![UiAction::Back]
        );
        assert_eq!(
            actions_from_keys(
                Some(ScreenId::Pause),
                &[NavKey::Down, NavKey::Confirm, NavKey::Down]
            ),
            vec![UiAction::Down, UiAction::Confirm]
        );
    }
}
