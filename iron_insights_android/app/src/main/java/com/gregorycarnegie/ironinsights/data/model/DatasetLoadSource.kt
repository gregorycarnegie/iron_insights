package com.gregorycarnegie.ironinsights.data.model

enum class DatasetLoadSource {
    NETWORK,
    DISK_CACHE,
    VERSION_CACHE,
}

data class LoadedResource<T>(
    val value: T,
    val source: DatasetLoadSource,
)

data class DatasetLoadSummary(
    val latest: DatasetLoadSource? = null,
    val rootIndex: DatasetLoadSource? = null,
    val shardIndex: DatasetLoadSource? = null,
    val histogram: DatasetLoadSource? = null,
    val heatmap: DatasetLoadSource? = null,
    val trends: DatasetLoadSource? = null,
) {
    fun usesOfflineFallback(): Boolean {
        return listOf(latest, rootIndex, shardIndex, histogram, heatmap, trends).any { source ->
            source == DatasetLoadSource.DISK_CACHE || source == DatasetLoadSource.VERSION_CACHE
        }
    }
}
