package com.gregorycarnegie.ironinsights.ui.navigation

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.FilterChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

@Composable
fun AppRouteTabs(
    selectedRoute: AppRoute,
    onRouteChange: (AppRoute) -> Unit,
    modifier: Modifier = Modifier,
) {
    Surface(
        modifier = modifier.fillMaxWidth(),
        shape = RoundedCornerShape(22.dp),
        color = MaterialTheme.colorScheme.surface.copy(alpha = 0.92f),
    ) {
        LazyRow(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 16.dp, vertical = 14.dp),
            horizontalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            items(AppRoute.entries, key = { it.name }) { route ->
                FilterChip(
                    selected = selectedRoute == route,
                    onClick = { onRouteChange(route) },
                    label = {
                        Text(route.label)
                    },
                )
            }
        }
    }
}
