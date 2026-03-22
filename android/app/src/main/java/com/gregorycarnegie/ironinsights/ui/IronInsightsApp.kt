package com.gregorycarnegie.ironinsights.ui

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import com.gregorycarnegie.ironinsights.config.AppConfig
import com.gregorycarnegie.ironinsights.data.model.PublishedDataContract
import com.gregorycarnegie.ironinsights.ui.calculators.CalculatorsScreen
import com.gregorycarnegie.ironinsights.ui.comparison.ComparisonScreen
import com.gregorycarnegie.ironinsights.ui.home.HomeScreen
import com.gregorycarnegie.ironinsights.ui.home.HomeUiState
import com.gregorycarnegie.ironinsights.ui.home.LookupFilterField
import com.gregorycarnegie.ironinsights.ui.navigation.AppRoute
import com.gregorycarnegie.ironinsights.ui.trends.TrendsScreen

@Composable
fun IronInsightsApp(
    uiState: HomeUiState,
    onRefresh: () -> Unit,
    onFilterChange: (LookupFilterField, String) -> Unit,
    onRouteChange: (AppRoute) -> Unit,
) {
    var selectedRoute by rememberSaveable { mutableStateOf(AppRoute.LOOKUP) }

    LaunchedEffect(selectedRoute) {
        onRouteChange(selectedRoute)
    }

    when (selectedRoute) {
        AppRoute.LOOKUP -> {
            HomeScreen(
                environment = AppConfig.environment,
                endpoints = PublishedDataContract.bootstrapSequence,
                milestones = PublishedDataContract.nextMilestones,
                uiState = uiState,
                selectedRoute = selectedRoute,
                onRouteChange = { selectedRoute = it },
                onRefresh = onRefresh,
                onFilterChange = onFilterChange,
            )
        }

        AppRoute.COMPARE -> {
            ComparisonScreen(
                uiState = uiState,
                selectedRoute = selectedRoute,
                onRouteChange = { selectedRoute = it },
                onRefresh = onRefresh,
                onFilterChange = onFilterChange,
            )
        }

        AppRoute.TRENDS -> {
            TrendsScreen(
                uiState = uiState,
                selectedRoute = selectedRoute,
                onRouteChange = { selectedRoute = it },
                onRefresh = onRefresh,
                onFilterChange = onFilterChange,
            )
        }

        AppRoute.CALCULATORS -> {
            CalculatorsScreen(
                selectedRoute = selectedRoute,
                onRouteChange = { selectedRoute = it },
            )
        }
    }
}
