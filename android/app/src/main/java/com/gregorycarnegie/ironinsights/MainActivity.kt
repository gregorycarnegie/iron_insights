package com.gregorycarnegie.ironinsights

import android.content.Context
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.Surface
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
import com.gregorycarnegie.ironinsights.data.preferences.UserPreferencesRepository
import com.gregorycarnegie.ironinsights.data.repository.HttpPublishedDataRepository
import com.gregorycarnegie.ironinsights.data.repository.TrainingRepository
import com.gregorycarnegie.ironinsights.ui.IronInsightsApp
import com.gregorycarnegie.ironinsights.ui.equipment.EquipmentViewModel
import com.gregorycarnegie.ironinsights.ui.home.HomeViewModel
import com.gregorycarnegie.ironinsights.ui.log.WorkoutLogViewModel
import com.gregorycarnegie.ironinsights.ui.programmes.ProgrammeViewModel
import com.gregorycarnegie.ironinsights.ui.progress.ProgressViewModel
import com.gregorycarnegie.ironinsights.ui.theme.IronInsightsTheme

private val Context.dataStore: DataStore<Preferences> by preferencesDataStore(name = "user_preferences")

class MainActivity : ComponentActivity() {
    private lateinit var homeViewModel: HomeViewModel
    private lateinit var workoutLogViewModel: WorkoutLogViewModel
    private lateinit var programmeViewModel: ProgrammeViewModel
    private lateinit var progressViewModel: ProgressViewModel
    private lateinit var equipmentViewModel: EquipmentViewModel

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
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
            ),
        )[HomeViewModel::class.java]

        val database = IronInsightsDatabase.getInstance(applicationContext)
        val trainingRepository = TrainingRepository(database)
        val preferencesRepository = UserPreferencesRepository(dataStore = dataStore)

        workoutLogViewModel = ViewModelProvider(
            this,
            WorkoutLogViewModel.factory(
                repository = trainingRepository,
                exerciseDao = database.exerciseDefinitionDao(),
                onSessionFinished = { sessionId ->
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

        setContent {
            val uiState = homeViewModel.uiState
            IronInsightsTheme {
                Surface(modifier = Modifier.fillMaxSize()) {
                    IronInsightsApp(
                        uiState = uiState,
                        onRefresh = homeViewModel::refresh,
                        onFilterChange = homeViewModel::updateFilter,
                        onRouteChange = homeViewModel::setRoute,
                        workoutLogViewModel = workoutLogViewModel,
                        programmeViewModel = programmeViewModel,
                        progressViewModel = progressViewModel,
                        equipmentViewModel = equipmentViewModel,
                    )
                }
            }
        }
    }
}
