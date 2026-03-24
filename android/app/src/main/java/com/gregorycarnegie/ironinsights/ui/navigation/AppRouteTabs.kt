package com.gregorycarnegie.ironinsights.ui.navigation

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.FilterChip
import androidx.compose.material3.FilterChipDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.unit.dp
import com.gregorycarnegie.ironinsights.R

@Composable
fun AppRouteTabs(
    selectedRoute: AppRoute,
    onRouteChange: (AppRoute) -> Unit,
    modifier: Modifier = Modifier,
) {
    Surface(
        modifier = modifier.fillMaxWidth(),
        shape = RoundedCornerShape(26.dp),
        color = MaterialTheme.colorScheme.surface.copy(alpha = 0.96f),
        border = BorderStroke(
            width = 1.dp,
            color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.92f),
        ),
    ) {
        Column(
            modifier = Modifier.padding(horizontal = 18.dp, vertical = 16.dp),
            verticalArrangement = Arrangement.spacedBy(14.dp),
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(14.dp),
            ) {
                Icon(
                    painter = painterResource(id = R.drawable.ic_brand_mark),
                    contentDescription = "Iron Insights logo",
                    modifier = Modifier.size(44.dp),
                    tint = androidx.compose.ui.graphics.Color.Unspecified,
                )
                Column(
                    verticalArrangement = Arrangement.spacedBy(2.dp),
                ) {
                    Text(
                        text = "Iron Insights",
                        style = MaterialTheme.typography.titleLarge,
                        color = MaterialTheme.colorScheme.onSurface,
                    )
                    Text(
                        text = "Powerlifting data explorer",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }

            Box(
                modifier = Modifier
                    .width(104.dp)
                    .height(3.dp)
                    .background(
                        brush = Brush.horizontalGradient(
                            colors = listOf(
                                MaterialTheme.colorScheme.primary,
                                MaterialTheme.colorScheme.secondary,
                            ),
                        ),
                        shape = RoundedCornerShape(99.dp),
                    ),
            )

            LazyRow(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(10.dp),
            ) {
                items(
                    AppRoute.analyticsRoutes.toList(),
                    key = { it.name },
                ) { route ->
                    FilterChip(
                        selected = selectedRoute == route,
                        onClick = { onRouteChange(route) },
                        colors = FilterChipDefaults.filterChipColors(
                            selectedContainerColor = MaterialTheme.colorScheme.primary,
                            selectedLabelColor = MaterialTheme.colorScheme.onPrimary,
                            containerColor = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.94f),
                            labelColor = MaterialTheme.colorScheme.onSurfaceVariant,
                        ),
                        label = {
                            Text(route.label)
                        },
                    )
                }
            }
        }
    }
}
