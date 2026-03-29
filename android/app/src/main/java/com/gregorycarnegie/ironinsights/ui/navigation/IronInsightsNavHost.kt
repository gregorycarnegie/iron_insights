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
import com.gregorycarnegie.ironinsights.ui.onboarding.OnboardingScreen
import com.gregorycarnegie.ironinsights.ui.onboarding.OnboardingViewModel
import com.gregorycarnegie.ironinsights.ui.profile.ProfileScreen
import com.gregorycarnegie.ironinsights.ui.profile.ProfileViewModel
import com.gregorycarnegie.ironinsights.ui.programmes.ProgrammeDetailScreen
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
    startDestination: String,
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
        startDestination = startDestination,
        modifier = modifier,
    ) {
        composable(NavRoutes.ONBOARDING) {
            val onboardingState = onboardingViewModel.uiState
            val finishOnboarding: () -> Unit = {
                onboardingViewModel.finish {
                    navController.navigate(NavRoutes.LOOKUP) {
                        popUpTo(NavRoutes.ONBOARDING) { inclusive = true }
                    }
                }
            }
            OnboardingScreen(
                uiState = onboardingState,
                onUpdateSex = onboardingViewModel::updateSex,
                onUpdateBodyweight = onboardingViewModel::updateBodyweight,
                onUpdateHeight = onboardingViewModel::updateHeight,
                onUpdateAge = onboardingViewModel::updateAge,
                onUpdateEquipment = onboardingViewModel::updateEquipment,
                onUpdateTested = onboardingViewModel::updateTested,
                onUpdateSquat = onboardingViewModel::updateSquat,
                onUpdateBench = onboardingViewModel::updateBench,
                onUpdateDeadlift = onboardingViewModel::updateDeadlift,
                onNext = onboardingViewModel::nextStep,
                onBack = onboardingViewModel::previousStep,
                onFinish = finishOnboarding,
                onSkip = finishOnboarding,
            )
        }

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
                onLookupLiftInputChange = onLookupLiftInputChange,
                onLookupBodyweightInputChange = onLookupBodyweightInputChange,
                onResetLookupInputsToProfile = onResetLookupInputsToProfile,
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
            val selectedProgramme = state.selectedProgramme
            if (selectedProgramme == null) {
                ProgrammesScreen(
                    uiState = state,
                    onCreateProgramme = programmeViewModel::createProgramme,
                    onSelectProgramme = programmeViewModel::selectProgramme,
                    onDeleteProgramme = programmeViewModel::deleteProgramme,
                )
            } else {
                ProgrammeDetailScreen(
                    programmeWithBlocks = selectedProgramme,
                    generatedPlan = state.generatedPlan,
                    liftBaselines = state.liftBaselines,
                    weightUnit = state.weightUnit,
                    onSetActiveProgramme = {
                        programmeViewModel.setActiveProgramme(selectedProgramme.programme.id)
                    },
                    onAddBlock = { name, blockType, weekCount ->
                        programmeViewModel.addBlock(
                            programmeId = selectedProgramme.programme.id,
                            name = name,
                            blockType = blockType,
                            weekCount = weekCount,
                        )
                    },
                    onAddBlockSequence = { drafts ->
                        programmeViewModel.addBlockSequence(
                            programmeId = selectedProgramme.programme.id,
                            blocks = drafts,
                        )
                    },
                    onDeleteBlock = programmeViewModel::deleteBlock,
                    onNavigateBack = programmeViewModel::clearSelection,
                )
            }
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

        composable(NavRoutes.PROFILE) {
            val profileState = profileViewModel.uiState
            ProfileScreen(
                uiState = profileState,
                onNavigateBack = { navController.popBackStack() },
                onStartEditing = profileViewModel::startEditing,
                onCancelEditing = profileViewModel::cancelEditing,
                onSaveProfile = profileViewModel::saveProfile,
                onUpdateSex = profileViewModel::updateSex,
                onUpdateBodyweight = profileViewModel::updateBodyweight,
                onUpdateHeight = profileViewModel::updateHeight,
                onUpdateAge = profileViewModel::updateAge,
                onUpdateEquipment = profileViewModel::updateEquipment,
                onUpdateTested = profileViewModel::updateTested,
                onUpdateSquat = profileViewModel::updateSquat,
                onUpdateBench = profileViewModel::updateBench,
                onUpdateDeadlift = profileViewModel::updateDeadlift,
            )
        }
    }
}
