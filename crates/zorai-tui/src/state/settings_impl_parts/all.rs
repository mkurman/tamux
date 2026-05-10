use super::*;
impl SettingsTab {
    const ALL: &'static [SettingsTab] = &[
        SettingsTab::Auth,
        SettingsTab::Agent,
        SettingsTab::Concierge,
        SettingsTab::Tools,
        SettingsTab::WebSearch,
        SettingsTab::Chat,
        SettingsTab::Gateway,
        SettingsTab::SubAgents,
        SettingsTab::Features,
        SettingsTab::Advanced,
        SettingsTab::Plugins,
        SettingsTab::About,
    ];

    pub fn all() -> &'static [SettingsTab] {
        Self::ALL
    }
}
