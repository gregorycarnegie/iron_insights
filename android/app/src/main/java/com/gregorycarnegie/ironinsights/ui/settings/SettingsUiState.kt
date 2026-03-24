package com.gregorycarnegie.ironinsights.ui.settings

import com.gregorycarnegie.ironinsights.data.preferences.UserPreferences

data class SettingsUiState(
    val preferences: UserPreferences = UserPreferences(),
)
