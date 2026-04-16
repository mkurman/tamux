use super::*;
use crossterm::event::{
    KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::ExecutableCommand;

#[derive(Debug, Clone, serde::Deserialize)]
pub(super) struct ProviderAuthState {
    pub provider_id: String,
    pub provider_name: String,
    #[allow(dead_code)]
    pub authenticated: bool,
    pub auth_source: String,
    pub model: String,
    pub base_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum WhatsAppAllowlistPromptOutcome {
    Submitted(String),
    Cancelled,
    EndOfInput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum WhatsAppAllowlistPromptResolution<'a> {
    Accept(String),
    Retry(&'a str),
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ConfigWrite {
    pub key_path: String,
    pub value_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum WhatsAppLinkAttemptOutcome {
    Linked(Option<String>),
    TimedOut,
    CancelledByUser,
}

pub(super) fn wizard_keyboard_enhancement_flags() -> KeyboardEnhancementFlags {
    // Only request DISAMBIGUATE_ESCAPE_CODES so that arrow-key sequences are
    // unambiguous.  The more aggressive flags (REPORT_EVENT_TYPES,
    // REPORT_ALTERNATE_KEYS, REPORT_ALL_KEYS_AS_ESCAPE_CODES) re-encode Enter
    // as a CSI-u escape sequence that some terminals (e.g. Yakuake/Konsole)
    // emit in a shape crossterm does not reliably map back to KeyCode::Enter,
    // causing the wizard to hang after the first selection — see issue #53.
    KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
}

pub(super) struct RawModeGuard {
    keyboard_enhancement_enabled: bool,
}

impl RawModeGuard {
    pub(super) fn new() -> Result<Self> {
        terminal::enable_raw_mode().context("Failed to enable raw mode")?;
        let keyboard_enhancement_enabled = matches!(
            crossterm::terminal::supports_keyboard_enhancement(),
            Ok(true)
        );

        if keyboard_enhancement_enabled {
            let mut stdout = io::stdout();
            let _ = stdout.execute(PushKeyboardEnhancementFlags(
                wizard_keyboard_enhancement_flags(),
            ));
        }

        Ok(Self {
            keyboard_enhancement_enabled,
        })
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        if self.keyboard_enhancement_enabled {
            let mut stdout = io::stdout();
            let _ = stdout.execute(PopKeyboardEnhancementFlags);
        }
        let _ = terminal::disable_raw_mode();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostSetupAction {
    LaunchTui,
    LaunchElectron,
    NotNow,
}

#[derive(Debug, Clone)]
pub(super) struct ProviderSelection {
    pub provider_id: String,
    pub provider_name: String,
    pub base_url: String,
    pub default_model: String,
    pub auth_source: String,
}

#[derive(Debug, Default)]
pub(super) struct SetupSummary {
    pub model: Option<String>,
    pub web_search: Option<String>,
    pub gateway: Option<String>,
    pub whatsapp_linked: bool,
    pub concierge: Option<String>,
    pub weles: Option<String>,
}
