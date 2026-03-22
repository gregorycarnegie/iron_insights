package com.gregorycarnegie.ironinsights

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.Surface
import androidx.compose.ui.Modifier
import androidx.lifecycle.ViewModelProvider
import com.gregorycarnegie.ironinsights.config.AppConfig
import com.gregorycarnegie.ironinsights.data.cache.LatestDatasetVersionCache
import com.gregorycarnegie.ironinsights.data.cache.PublishedPayloadCache
import com.gregorycarnegie.ironinsights.data.repository.HttpPublishedDataRepository
import com.gregorycarnegie.ironinsights.ui.IronInsightsApp
import com.gregorycarnegie.ironinsights.ui.home.HomeViewModel
import com.gregorycarnegie.ironinsights.ui.theme.IronInsightsTheme

class MainActivity : ComponentActivity() {
    private lateinit var homeViewModel: HomeViewModel

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

        setContent {
            val uiState = homeViewModel.uiState
            IronInsightsTheme {
                Surface(modifier = Modifier.fillMaxSize()) {
                    IronInsightsApp(
                        uiState = uiState,
                        onRefresh = homeViewModel::refresh,
                        onFilterChange = homeViewModel::updateFilter,
                        onRouteChange = homeViewModel::setRoute,
                    )
                }
            }
        }
    }
}
