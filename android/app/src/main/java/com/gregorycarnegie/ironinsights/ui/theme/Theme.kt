package com.gregorycarnegie.ironinsights.ui.theme

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.ui.graphics.Color

private val IronInsightsLightColors = lightColorScheme(
    primary = PlateRed,
    onPrimary = Bone,
    primaryContainer = Color(0xFFF2D6C8),
    onPrimaryContainer = IronDark,
    secondary = Graphite,
    onSecondary = Bone,
    secondaryContainer = Color(0xFFE1E7E9),
    onSecondaryContainer = IronDark,
    tertiary = PlateCopper,
    onTertiary = IronDark,
    background = Bone,
    onBackground = IronDark,
    surface = Color(0xFFF7F3EE),
    onSurface = IronDark,
    surfaceVariant = Color(0xFFE8E0D9),
    onSurfaceVariant = Graphite,
    outline = NightSteel,
    outlineVariant = SteelMist,
)

private val IronInsightsDarkColors = darkColorScheme(
    primary = PlateCopper,
    onPrimary = IronDark,
    primaryContainer = IronPanel,
    onPrimaryContainer = Chalk,
    secondary = SteelMist,
    onSecondary = IronDark,
    secondaryContainer = Graphite,
    onSecondaryContainer = Chalk,
    tertiary = PlateRed,
    onTertiary = Chalk,
    background = IronDark,
    onBackground = Chalk,
    surface = IronSurface,
    onSurface = Chalk,
    surfaceVariant = IronPanel,
    onSurfaceVariant = SteelMist,
    outline = NightSteel,
    outlineVariant = Graphite,
)

@Composable
fun IronInsightsTheme(
    darkTheme: Boolean = isSystemInDarkTheme(),
    content: @Composable () -> Unit,
) {
    MaterialTheme(
        colorScheme = if (darkTheme) IronInsightsDarkColors else IronInsightsLightColors,
        typography = IronInsightsTypography,
        content = content,
    )
}
