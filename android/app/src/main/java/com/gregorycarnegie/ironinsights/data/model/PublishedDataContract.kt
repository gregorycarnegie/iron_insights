package com.gregorycarnegie.ironinsights.data.model

data class DatasetEndpoint(
    val title: String,
    val relativePath: String,
    val purpose: String,
)

object PublishedDataContract {
    val bootstrapSequence = listOf(
        DatasetEndpoint(
            title = "Latest pointer",
            relativePath = "data/latest.json",
            purpose = "Find the active published dataset version.",
        ),
        DatasetEndpoint(
            title = "Version root index",
            relativePath = "data/<version>/index.json",
            purpose = "Find shard files for the selected sex and equipment bucket.",
        ),
        DatasetEndpoint(
            title = "Shard index",
            relativePath = "data/<version>/index_shards/<sex>/<equip>/index.json",
            purpose = "Resolve slice summaries and binary payload locations.",
        ),
        DatasetEndpoint(
            title = "Histogram and heatmap binaries",
            relativePath = "data/<version>/(hist|heat)/*.bin",
            purpose = "Drive percentile, density, and bodyweight-conditioned views.",
        ),
        DatasetEndpoint(
            title = "Trends payload",
            relativePath = "data/<version>/trends.json",
            purpose = "Power the trend charts and cohort history screens.",
        ),
    )

    val nextMilestones = listOf(
        "Expand the comparison surface with richer cohort variants and summary-only loading paths.",
        "Harden cache invalidation and stale-data controls around versioned payloads.",
        "Extract the shared histogram and heatmap contract out of the web app core.",
        "Configure Android release secrets and Play upload on top of the signed bundle workflow.",
    )
}
