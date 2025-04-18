//! A model for system specific state which can be accessed by any model or view.
use crate::prelude::*;

use unic_langid::LanguageIdentifier;
use vizia_derive::Lens;
use web_time::Duration;

/// And enum which represents the current built-in theme mode.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    /// The built-in vizia dark theme.
    DarkMode,
    /// The built-in vizia light theme.
    #[default]
    LightMode,
}

use crate::{context::EventContext, events::Event};

/// Represents the theme used by the application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppTheme {
    /// System theme, if we choose this as our theme vizia
    /// will follow system theme in supported platforms.
    System,
    /// Built-in vizia themes.
    BuiltIn(ThemeMode),
    // Custom(String),
}

/// Represents the theme used by the application.
#[derive(Lens)]
pub struct Theme {
    /// The current application theme
    pub app_theme: AppTheme,
    /// The current system theme
    pub sys_theme: Option<ThemeMode>,
}

impl Default for Theme {
    fn default() -> Self {
        Self { app_theme: AppTheme::BuiltIn(ThemeMode::LightMode), sys_theme: None }
    }
}

impl Theme {
    /// Returns the current theme of the application.
    pub fn get_current_theme(&self) -> ThemeMode {
        match self.app_theme {
            AppTheme::System => self.sys_theme.unwrap_or_default(),
            AppTheme::BuiltIn(theme) => theme,
        }
    }
}

/// A model for system specific state which can be accessed by any model or view.
#[derive(Lens)]
pub struct Environment {
    /// The locale used for localization.
    pub locale: LanguageIdentifier,
    /// Current application and system theme.
    pub theme: Theme,
    /// The timer used to blink the caret of a textbox.
    pub(crate) caret_timer: Timer,
}

impl Environment {
    pub(crate) fn new(cx: &mut Context) -> Self {
        let locale = sys_locale::get_locale().and_then(|l| l.parse().ok()).unwrap_or_default();
        let caret_timer = cx.add_timer(Duration::from_millis(530), None, |cx, action| {
            if matches!(action, TimerAction::Tick(_)) {
                cx.emit(TextEvent::ToggleCaret);
            }
        });
        Self { locale, theme: Theme::default(), caret_timer }
    }
}

/// Events for setting the state in the [Environment].
pub enum EnvironmentEvent {
    /// Set the locale used for the whole application.
    SetLocale(LanguageIdentifier),
    /// Set the default theme mode.
    // TODO: add SetSysTheme event when the winit `set_theme` fixed.
    SetThemeMode(AppTheme),
    /// Reset the locale to use the system provided locale.
    UseSystemLocale,
    /// Alternate between dark and light theme modes.
    ToggleThemeMode,
}

impl Model for Environment {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.take(|event, _| match event {
            EnvironmentEvent::SetLocale(locale) => {
                self.locale = locale;
            }

            EnvironmentEvent::SetThemeMode(theme) => {
                theme.clone_into(&mut self.theme.app_theme);

                cx.set_theme_mode(self.theme.get_current_theme());
                cx.reload_styles().unwrap();
            }

            EnvironmentEvent::UseSystemLocale => {
                self.locale =
                    sys_locale::get_locale().map(|l| l.parse().unwrap()).unwrap_or_default();
            }

            EnvironmentEvent::ToggleThemeMode => {
                let theme_mode = match self.theme.get_current_theme() {
                    ThemeMode::DarkMode => ThemeMode::LightMode,
                    ThemeMode::LightMode => ThemeMode::DarkMode,
                };

                self.theme.app_theme = AppTheme::BuiltIn(theme_mode);

                cx.set_theme_mode(theme_mode);
                cx.reload_styles().unwrap();
            }
        });

        event.map(|event, _| match event {
            WindowEvent::ThemeChanged(theme) => {
                self.theme.sys_theme = Some(*theme);
                if self.theme.app_theme == AppTheme::System {
                    cx.set_theme_mode(*theme);
                    cx.reload_styles().unwrap();
                }
            }
            _ => (),
        })
    }
}
