package com.gregorycarnegie.ironinsights.ui.navigation

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.navigation.NavHostController
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import com.gregorycarnegie.ironinsights.config.AppConfig
import com.gregorycarnegie.ironinsights.data.model.PublishedDataContract
import com.gregorycarnegie.ironinsights.domain.calculators.WeightUnit
import com.gregorycarnegie.ironinsights.ui.calculators.CalculatorsScreen
import com.gregorycarnegie.ironinsights.ui.comparison.ComparisonScreen
import com.gregorycarnegie.ironinsights.ui.equipment.EquipmentScreen
import com.gregorycarnegie.ironinsights.ui.equipment.EquipmentViewModel
import com.gregorycarnegie.ironinsights.ui.home.HomeScreen
import com.gregorycarnegie.ironinsights.ui.home.HomeUiState
import com.gregorycarnegie.ironinsights.ui.home.LookupFilterField
import com.gregorycarnegie.ironinsights.ui.log.WorkoutLogScreen
import com.gregorycarnegie.ironinsights.ui.log.WorkoutLogViewModel
import com.gregorycarnegie.ironinsights.ui.programmes.ProgrammeViewModel
import com.gregorycarnegie.ironinsights.ui.programmes.ProgrammesScreen
import com.gregorycarnegie.ironinsights.ui.progress.ProgressScreen
import com.gregorycarnegie.ironinsights.ui.progress.ProgressViewModel
import com.gregorycarnegie.ironinsights.ui.trends.TrendsScreen

@Composable
fun IronInsightsNavHost(
    navController: NavHostController,
    uiState: HomeUiState,
    onRefresh: () -> Unit,
    onFilterChange: (LookupFilterField, String) -> Unit,
    onRouteChange: (AppRoute) -> Unit,
    workoutLogViewModel: WorkoutLogViewModel,
    programmeViewModel: ProgrammeViewModel,
    progressViewModel: ProgressViewModel,
    equipmentViewModel: EquipmentViewModel,
    modifier: Modifier = Modifier,
) {
    val analyticsOnRouteChange: (AppRoute) -> Unit = { route ->
        onRouteChange(route)
        navController.navigate(route.navRoute) {
            popUpTo(navController.graph.startDestinationId) {
                saveState = true
            }
            launchSingleTop = true
            restoreState = true
        }
    }

    NavHost(
        navController = navController,
        startDestination = NavRoutes.LOOKUP,
        modifier = modifier,
    ) {
        composable(NavRoutes.LOOKUP) {
            HomeScreen(
                environment = AppConfig.environment,
                endpoints = PublishedDataContract.bootstrapSequence,
                milestones = PublishedDataContract.nextMilestones,
                uiState = uiState,
                selectedRoute = AppRoute.LOOKUP,
                onRouteChange = analyticsOnRouteChange,
                onRefresh = onRefresh,
                onFilterChange = onFilterChange,
            )
        }

        composable(NavRoutes.COMPARE) {
            ComparisonScreen(
                uiState = uiState,
                selectedRoute = AppRoute.COMPARE,
                onRouteChange = analyticsOnRouteChange,
                onRefresh = onRefresh,
                onFilterChange = onFilterChange,
            )
        }

        composable(NavRoutes.TRENDS) {
            TrendsScreen(
                uiState = uiState,
                selectedRoute = AppRoute.TRENDS,
                onRouteChange = analyticsOnRouteChange,
                onRefresh = onRefresh,
                onFilterChange = onFilterChange,
            )
        }

        composable(NavRoutes.CALCULATORS) {
            CalculatorsScreen(
                selectedRoute = AppRoute.CALCULATORS,
                onRouteChange = analyticsOnRouteChange,
            )
        }

        composable(NavRoutes.LOG) {
            WorkoutLogScreen(viewModel = workoutLogViewModel)
        }

        composable(NavRoutes.PROGRAMMES) {
            val state = programmeViewModel.uiState
            ProgrammesScreen(
                uiState = state,
                onCreateProgramme = programmeViewModel::createProgramme,
                onSelectProgramme = programmeViewModel::selectProgramme,
                onDeleteProgramme = programmeViewModel::deleteProgramme,
            )
        }

        composable(NavRoutes.PROGRESS) {
            ProgressScreen(
                uiState = progressViewModel.uiState,
                weightUnit = WeightUnit.KG,
            )
        }

        composable(NavRoutes.EQUIPMENT) {
            EquipmentScreen(viewModel = equipmentViewModel)
        }
    }
}
