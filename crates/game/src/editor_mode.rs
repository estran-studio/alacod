//! Mode éditeur (K5-1, `docs/taches/plan-editeur-vocal.md` D1, `docs/editeur-vocal.md`).
//!
//! `ALACOD_EDITOR=1` (`make editor`) : la partie se joue au clavier **et** se pilote par le remote
//! BRP (`ALACOD_REMOTE` est implicite). Le mode force `check_distance = 0` (voir
//! [`crate::args::play_check_distance_from_env`]) : ni snapshot ni resimulation, donc écrire une
//! donnée hors `GgrsSchedule` (modificateur, registre) ne fait pas diverger la session. Les
//! commandes d'édition de `remote.rs` passent toutes par [`hot_edit_allowed`] : en ligne ou à
//! `check_distance > 0`, elles sont refusées avec la cause nommée.
//!
//! Présentation/outillage seulement : rien ici n'est lu par `GgrsSchedule`.

use bevy::prelude::*;

/// Posée (par `CoreAppPlugin`) quand `ALACOD_EDITOR=1`. Absente sinon : le remote garde alors son
/// comportement d'origine (partie en pause, inputs entièrement pilotés).
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditorMode;

impl EditorMode {
    /// `ALACOD_EDITOR=1` ?
    pub fn enabled_by_env() -> bool {
        std::env::var("ALACOD_EDITOR").is_ok_and(|v| v == "1")
    }

    /// `Some` si `ALACOD_EDITOR=1`.
    pub fn from_env() -> Option<Self> {
        Self::enabled_by_env().then_some(Self)
    }
}

/// Pourquoi une commande d'édition à chaud est refusée.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorRefusal {
    /// Session en ligne (p2p) : une écriture locale diverge entre pairs, sans canal pour la répliquer.
    Online,
    /// Synctest avec resimulation (`check_distance` > 0) : les frames rejouées liraient la donnée
    /// modifiée et le checksum diffère.
    CheckDistance(usize),
}

impl std::fmt::Display for EditorRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Online => write!(
                f,
                "commande d'édition refusée : online (session en ligne, p2p non pris en charge)"
            ),
            Self::CheckDistance(n) => write!(
                f,
                "commande d'édition refusée : check_distance={n} (le mode éditeur exige 0 : relancer sans ALACOD_CHECK_DISTANCE ni ALACOD_STATE_TRACE explicite)"
            ),
        }
    }
}

/// Une commande qui écrit dans la simulation hors `GgrsSchedule` n'est sûre qu'en session locale
/// sans resimulation (D1). `online` prime sur `check_distance` dans le message.
pub fn hot_edit_allowed(online: bool, check_distance: usize) -> Result<(), EditorRefusal> {
    if online {
        Err(EditorRefusal::Online)
    } else if check_distance > 0 {
        Err(EditorRefusal::CheckDistance(check_distance))
    } else {
        Ok(())
    }
}

/// Garde sur l'état de la session : `online` est testé **avant** de lire `check_distance` (une session
/// en ligne sans configuration lisible doit se dire « online », pas « session inconnue »). Renvoie la
/// cause en texte ; `check_distance` absent (hors ligne) est une erreur, jamais « autorisé ».
pub fn session_hot_edit_allowed(
    online: Option<crate::session::OnlineState>,
    check_distance: Option<usize>,
) -> Result<(), String> {
    if online == Some(crate::session::OnlineState::Online) {
        return Err(EditorRefusal::Online.to_string());
    }
    let check_distance =
        check_distance.ok_or_else(|| "session inconnue : check_distance illisible".to_string())?;
    hot_edit_allowed(false, check_distance).map_err(|refusal| refusal.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_without_resimulation_is_allowed() {
        assert_eq!(hot_edit_allowed(false, 0), Ok(()));
    }

    #[test]
    fn online_is_refused_whatever_the_distance() {
        assert_eq!(hot_edit_allowed(true, 0), Err(EditorRefusal::Online));
        assert_eq!(hot_edit_allowed(true, 2), Err(EditorRefusal::Online));
    }

    #[test]
    fn synctest_distance_is_refused_and_named() {
        let refusal = hot_edit_allowed(false, 2).unwrap_err();
        assert_eq!(refusal, EditorRefusal::CheckDistance(2));
        assert!(refusal.to_string().contains("check_distance=2"));
        assert!(EditorRefusal::Online.to_string().contains("online"));
    }

    #[test]
    fn online_is_named_before_unknown_session() {
        use crate::session::OnlineState;
        let err = session_hot_edit_allowed(Some(OnlineState::Online), None).unwrap_err();
        assert!(err.contains("online"), "{err}");
        assert!(session_hot_edit_allowed(Some(OnlineState::Offline), None)
            .unwrap_err()
            .contains("session inconnue"));
        assert!(session_hot_edit_allowed(None, Some(0)).is_ok());
        assert!(session_hot_edit_allowed(None, Some(2))
            .unwrap_err()
            .contains("check_distance=2"));
    }
}
