package com.gregorycarnegie.ironinsights.ui.theme

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable

private val IronInsightsColorScheme = darkColorScheme(
    primary = AccentLime,
    onPrimary = NightVoid,
    primaryContainer = AccentDim,
    onPrimaryContainer = Chalk,
    secondary = AccentTeal,
    onSecondary = NightVoid,
    secondaryContainer = AccentTealDeep,
    onSecondaryContainer = Chalk,
    tertiary = SteelMist,
    onTertiary = NightVoid,
    tertiaryContainer = AccentLimeDeep,
    onTertiaryContainer = Chalk,
    background = NightVoid,
    onBackground = Chalk,
    surface = NightRaised,
    onSurface = Chalk,
    surfaceVariant = NightCard,
    onSurfaceVariant = SteelMist,
    outline = BorderMid,
    outlineVariant = BorderSubtle,
    error = DangerHeat,
    onError = NightVoid,
    errorContainer = DangerHeatDeep,
    onErrorContainer = Chalk,
)

@Composable
fun IronInsightsTheme(content: @Composable () -> Unit) {
    MaterialTheme(
        colorScheme = IronInsightsColorScheme,
        typography = IronInsightsTypography,
        content = content,
    )
}
