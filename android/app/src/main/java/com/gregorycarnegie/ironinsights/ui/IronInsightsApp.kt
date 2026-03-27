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
import com.gregorycarnegie.ironinsights.ui.navigation.NavRoutes
import com.gregorycarnegie.ironinsights.ui.onboarding.OnboardingViewModel
import com.gregorycarnegie.ironinsights.ui.profile.ProfileViewModel
import com.gregorycarnegie.ironinsights.ui.programmes.ProgrammeViewModel
import com.gregorycarnegie.ironinsights.ui.progress.ProgressViewModel

@Composable
fun IronInsightsApp(
    uiState: HomeUiState,
    onRefresh: () -> Unit,
    onFilterChange: (LookupFilterField, String) -> Unit,
    onLookupLiftInputChange: (String) -> Unit,
    onLookupBodyweightInputChange: (String) -> Unit,
    onResetLookupInputsToProfile: () -> Unit,
    onRouteChange: (AppRoute) -> Unit,
    workoutLogViewModel: WorkoutLogViewModel,
    programmeViewModel: ProgrammeViewModel,
    progressViewModel: ProgressViewModel,
    equipmentViewModel: EquipmentViewModel,
    onboardingViewModel: OnboardingViewModel,
    profileViewModel: ProfileViewModel,
    onboardingCompleted: Boolean,
) {
    val navController = rememberNavController()
    val navBackStackEntry = navController.currentBackStackEntryAsState()
    val currentRoute = navBackStackEntry.value?.destination?.route
    val showBottomBar = currentRoute != NavRoutes.ONBOARDING

    // Sync navigation destination changes back to HomeViewModel for analytics routes.
    LaunchedEffect(navBackStackEntry.value?.destination?.route) {
        val route = navBackStackEntry.value?.destination?.route
        val appRoute = AppRoute.fromNavRoute(route)
        if (appRoute != null) {
            onRouteChange(appRoute)
        }
    }

    val startDestination = if (onboardingCompleted) NavRoutes.LOOKUP else NavRoutes.ONBOARDING

    Scaffold(
        bottomBar = {
            if (showBottomBar) {
                BottomNavBar(navController = navController)
            }
        },
    ) { innerPadding ->
        IronInsightsNavHost(
            navController = navController,
            uiState = uiState,
            onRefresh = onRefresh,
            onFilterChange = onFilterChange,
            onLookupLiftInputChange = onLookupLiftInputChange,
            onLookupBodyweightInputChange = onLookupBodyweightInputChange,
            onResetLookupInputsToProfile = onResetLookupInputsToProfile,
            onRouteChange = onRouteChange,
            workoutLogViewModel = workoutLogViewModel,
            programmeViewModel = programmeViewModel,
            progressViewModel = progressViewModel,
            equipmentViewModel = equipmentViewModel,
            onboardingViewModel = onboardingViewModel,
            profileViewModel = profileViewModel,
            startDestination = startDestination,
            modifier = Modifier.padding(innerPadding),
        )
    }
}
