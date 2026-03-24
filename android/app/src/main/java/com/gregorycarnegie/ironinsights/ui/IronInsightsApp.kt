package com.gregorycarnegie.ironinsights.ui

import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Scaffold
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Modifier
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import com.gregorycarnegie.ironinsights.ui.home.HomeUiState
import com.gregorycarnegie.ironinsights.ui.home.LookupFilterField
import com.gregorycarnegie.ironinsights.ui.equipment.EquipmentViewModel
import com.gregorycarnegie.ironinsights.ui.log.WorkoutLogViewModel
import com.gregorycarnegie.ironinsights.ui.navigation.AppRoute
import com.gregorycarnegie.ironinsights.ui.navigation.BottomNavBar
import com.gregorycarnegie.ironinsights.ui.navigation.IronInsightsNavHost
import com.gregorycarnegie.ironinsights.ui.programmes.ProgrammeViewModel
import com.gregorycarnegie.ironinsights.ui.progress.ProgressViewModel

@Composable
fun IronInsightsApp(
    uiState: HomeUiState,
    onRefresh: () -> Unit,
    onFilterChange: (LookupFilterField, String) -> Unit,
    onRouteChange: (AppRoute) -> Unit,
    workoutLogViewModel: WorkoutLogViewModel,
    programmeViewModel: ProgrammeViewModel,
    progressViewModel: ProgressViewModel,
    equipmentViewModel: EquipmentViewModel,
) {
    val navController = rememberNavController()
    val navBackStackEntry = navController.currentBackStackEntryAsState()

    // Sync navigation destination changes back to HomeViewModel for analytics routes.
    LaunchedEffect(navBackStackEntry.value?.destination?.route) {
        val currentRoute = navBackStackEntry.value?.destination?.route
        val appRoute = AppRoute.fromNavRoute(currentRoute)
        if (appRoute != null) {
            onRouteChange(appRoute)
        }
    }

    Scaffold(
        bottomBar = {
            BottomNavBar(navController = navController)
        },
    ) { innerPadding ->
        IronInsightsNavHost(
            navController = navController,
            uiState = uiState,
            onRefresh = onRefresh,
            onFilterChange = onFilterChange,
            onRouteChange = onRouteChange,
            workoutLogViewModel = workoutLogViewModel,
            programmeViewModel = programmeViewModel,
            progressViewModel = progressViewModel,
            equipmentViewModel = equipmentViewModel,
            modifier = Modifier.padding(innerPadding),
        )
    }
}
