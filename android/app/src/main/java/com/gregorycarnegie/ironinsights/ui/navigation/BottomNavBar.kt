package com.gregorycarnegie.ironinsights.ui.navigation

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItemDefaults
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.navigation.NavController
import androidx.navigation.compose.currentBackStackEntryAsState

/**
 * Bottom navigation bar with four primary sections:
 * Log, Lookup (analytics), PRs, Plan.
 *
 * "Lookup" is selected whenever the user is on any analytics sub-route
 * (lookup, compare, trends, calculators).
 */
@Composable
fun BottomNavBar(
    navController: NavController,
    modifier: Modifier = Modifier,
) {
    val navBackStackEntry by navController.currentBackStackEntryAsState()
    val currentRoute = navBackStackEntry?.destination?.route

    val items = listOf(
        AppRoute.LOG,
        AppRoute.LOOKUP,
        AppRoute.PROGRESS,
        AppRoute.PROGRAMMES,
        AppRoute.PROFILE,
    )

    NavigationBar(
        modifier = modifier,
        containerColor = MaterialTheme.colorScheme.surface,
    ) {
        items.forEach { item ->
            val selected = when (item) {
                // Highlight "Lookup" for any analytics sub-route.
                AppRoute.LOOKUP -> AppRoute.fromNavRoute(currentRoute)?.isAnalytics == true
                else -> currentRoute == item.navRoute
            }

            NavigationBarItem(
                selected = selected,
                onClick = {
                    if (!selected) {
                        navController.navigate(item.navRoute) {
                            // Pop up to the start destination to avoid building up
                            // a large back stack of top-level destinations.
                            popUpTo(navController.graph.startDestinationId) {
                                saveState = true
                            }
                            launchSingleTop = true
                            restoreState = true
                        }
                    }
                },
                icon = { /* Icons will be added in a later phase */ },
                label = {
                    Text(text = item.label)
                },
                colors = NavigationBarItemDefaults.colors(
                    selectedIconColor = MaterialTheme.colorScheme.primary,
                    selectedTextColor = MaterialTheme.colorScheme.primary,
                    unselectedIconColor = MaterialTheme.colorScheme.onSurfaceVariant,
                    unselectedTextColor = MaterialTheme.colorScheme.onSurfaceVariant,
                    indicatorColor = MaterialTheme.colorScheme.surface,
                ),
            )
        }
    }
}
