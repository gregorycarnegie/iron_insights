package com.gregorycarnegie.ironinsights

import android.content.Context
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.Surface
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.datastore.core.DataStore
import androidx.datastore.preferences.core.Preferences
import androidx.datastore.preferences.preferencesDataStore
import androidx.lifecycle.ViewModelProvider
import androidx.work.Data
import androidx.work.OneTimeWorkRequestBuilder
import androidx.work.WorkManager
import com.gregorycarnegie.ironinsights.config.AppConfig
import com.gregorycarnegie.ironinsights.data.cache.LatestDatasetVersionCache
import com.gregorycarnegie.ironinsights.data.cache.PublishedPayloadCache
import com.gregorycarnegie.ironinsights.data.db.IronInsightsDatabase
import com.gregorycarnegie.ironinsights.data.health.HealthConnectSyncWorker
import com.gregorycarnegie.ironinsights.data.preferences.UserPreferences
import com.gregorycarnegie.ironinsights.data.preferences.UserPreferencesRepository
import com.gregorycarnegie.ironinsights.data.repository.HttpPublishedDataRepository
import com.gregorycarnegie.ironinsights.data.repository.TrainingRepository
import com.gregorycarnegie.ironinsights.ui.IronInsightsApp
import com.gregorycarnegie.ironinsights.ui.equipment.EquipmentViewModel
import com.gregorycarnegie.ironinsights.ui.home.HomeViewModel
import com.gregorycarnegie.ironinsights.ui.home.ProfileLookupDefaults
import com.gregorycarnegie.ironinsights.ui.log.WorkoutLogViewModel
import com.gregorycarnegie.ironinsights.ui.onboarding.OnboardingViewModel
import com.gregorycarnegie.ironinsights.ui.profile.ProfileViewModel
import com.gregorycarnegie.ironinsights.ui.programmes.ProgrammeViewModel
import com.gregorycarnegie.ironinsights.ui.progress.ProgressViewModel
import com.gregorycarnegie.ironinsights.ui.theme.IronInsightsTheme
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.runBlocking

private val Context.dataStore: DataStore<Preferences> by preferencesDataStore(name = "user_preferences")

class MainActivity : ComponentActivity() {
    private lateinit var homeViewModel: HomeViewModel
    private lateinit var workoutLogViewModel: WorkoutLogViewModel
    private lateinit var programmeViewModel: ProgrammeViewModel
    private lateinit var progressViewModel: ProgressViewModel
    private lateinit var equipmentViewModel: EquipmentViewModel
    private lateinit var onboardingViewModel: OnboardingViewModel
    private lateinit var profileViewModel: ProfileViewModel

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        val database = IronInsightsDatabase.getInstance(applicationContext)
        val trainingRepository = TrainingRepository(database)
        val preferencesRepository = UserPreferencesRepository(dataStore = dataStore)

        // Read profile defaults synchronously so HomeViewModel can use them for initial filters.
        val initialPrefs = runBlocking { preferencesRepository.preferencesFlow.firstOrNull() }
        val profileDefaults = initialPrefs?.toProfileLookupDefaults() ?: ProfileLookupDefaults()

        val latestDatasetVersionCache = LatestDatasetVersionCache(applicationContext)
        val publishedPayloadCache = PublishedPayloadCache(applicationContext)
        homeViewModel = ViewModelProvider(
            this,
            HomeViewModel.factory(
                repository = HttpPublishedDataRepository(
                    environment = AppConfig.environment,
                    latestDatasetVersionCache = latestDatasetVersionCache,
                    payloadCache = publishedPayloadCache,
                ),
                profileDefaults = profileDefaults,
            ),
        )[HomeViewModel::class.java]

        workoutLogViewModel = ViewModelProvider(
            this,
            WorkoutLogViewModel.factory(
                repository = trainingRepository,
                exerciseDao = database.exerciseDefinitionDao(),
                programmeDao = database.programmeDao(),
                trainingStatsDao = database.trainingStatsDao(),
                preferencesRepository = preferencesRepository,
                onSessionFinished = { sessionId ->
                    progressViewModel.refresh()
                    // Enqueue Health Connect sync if enabled
                    val inputData = Data.Builder()
                        .putLong("session_id", sessionId)
                        .build()
                    val request = OneTimeWorkRequestBuilder<HealthConnectSyncWorker>()
                        .setInputData(inputData)
                        .build()
                    WorkManager.getInstance(applicationContext).enqueue(request)
                },
            ),
        )[WorkoutLogViewModel::class.java]

        programmeViewModel = ViewModelProvider(
            this,
            ProgrammeViewModel.factory(
                programmeDao = database.programmeDao(),
                plannedSessionDao = database.plannedSessionDao(),
                exerciseDefinitionDao = database.exerciseDefinitionDao(),
                trainingStatsDao = database.trainingStatsDao(),
                preferencesRepository = preferencesRepository,
            ),
        )[ProgrammeViewModel::class.java]

        progressViewModel = ViewModelProvider(
            this,
            ProgressViewModel.factory(
                repository = trainingRepository,
                database = database,
            ),
        )[ProgressViewModel::class.java]

        equipmentViewModel = ViewModelProvider(
            this,
            EquipmentViewModel.factory(
                equipmentDao = database.equipmentDao(),
                preferencesRepository = preferencesRepository,
            ),
        )[EquipmentViewModel::class.java]

        onboardingViewModel = ViewModelProvider(
            this,
            OnboardingViewModel.factory(
                preferencesRepository = preferencesRepository,
            ),
        )[OnboardingViewModel::class.java]

        profileViewModel = ViewModelProvider(
            this,
            ProfileViewModel.factory(
                preferencesRepository = preferencesRepository,
            ),
        )[ProfileViewModel::class.java]

        setContent {
            val uiState = homeViewModel.uiState
            val prefs by preferencesRepository.preferencesFlow.collectAsState(
                initial = UserPreferences()
            )
            LaunchedEffect(prefs) {
                homeViewModel.syncProfileDefaults(prefs.toProfileLookupDefaults())
            }
            IronInsightsTheme {
                Surface(modifier = Modifier.fillMaxSize()) {
                    IronInsightsApp(
                        uiState = uiState,
                        onRefresh = homeViewModel::refresh,
                        onFilterChange = homeViewModel::updateFilter,
                        onLookupLiftInputChange = homeViewModel::updateLookupLiftInput,
                        onLookupBodyweightInputChange = homeViewModel::updateLookupBodyweightInput,
                        onResetLookupInputsToProfile = homeViewModel::resetLookupInputsToProfile,
                        onRouteChange = homeViewModel::setRoute,
                        workoutLogViewModel = workoutLogViewModel,
                        programmeViewModel = programmeViewModel,
                        progressViewModel = progressViewModel,
                        equipmentViewModel = equipmentViewModel,
                        onboardingViewModel = onboardingViewModel,
                        profileViewModel = profileViewModel,
                        onboardingCompleted = prefs.onboardingCompleted,
                    )
                }
            }
        }
    }
}

private fun UserPreferences.toProfileLookupDefaults(): ProfileLookupDefaults {
    return ProfileLookupDefaults(
        sex = sex,
        equipment = equipment,
        tested = tested,
        bodyweightKg = bodyweightKg,
        squatKg = squatKg,
        benchKg = benchKg,
        deadliftKg = deadliftKg,
    )
}
