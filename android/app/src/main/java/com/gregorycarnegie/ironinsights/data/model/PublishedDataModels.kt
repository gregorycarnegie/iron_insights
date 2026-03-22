package com.gregorycarnegie.ironinsights.data.model

import java.util.SortedMap

data class LatestJson(
    val version: String,
    val revision: String? = null,
)

data class RootIndex(
    val shards: SortedMap<String, String>,
)

data class SliceIndex(
    val slices: SliceIndexEntries,
)

data class SliceIndexPreview(
    val shardKey: String,
    val shardPath: String,
    val sliceCount: Int,
    val sampleSliceKey: String?,
    val sampleHistPath: String?,
)

data class TrendsPreview(
    val bucket: String,
    val seriesCount: Int,
    val sampleSeriesKey: String?,
    val samplePointCount: Int?,
)

data class HistogramBin(
    val min: Float,
    val max: Float,
    val baseBin: Float,
    val counts: List<Long>,
    val total: Long,
)

data class HeatmapBin(
    val minX: Float,
    val maxX: Float,
    val minY: Float,
    val maxY: Float,
    val baseX: Float,
    val baseY: Float,
    val width: Int,
    val height: Int,
    val grid: List<Long>,
)

data class HistogramLookupPreview(
    val sliceKey: String,
    val cohortLabel: String,
    val liftLabel: String,
    val metric: String,
    val histPath: String,
    val heatPath: String,
    val histogram: HistogramBin,
    val heatmap: HeatmapBin?,
    val p50: Float?,
    val p90: Float?,
)

data class PercentileLookup(
    val percentile: Float,
    val rank: Long,
    val total: Long,
    val binIndex: Int,
    val binLow: Float,
    val binHigh: Float,
)

data class BodyweightConditionedLookup(
    val percentile: Float,
    val rank: Long,
    val totalNearby: Long,
    val bwBinIndex: Int,
    val bwBinLow: Float,
    val bwBinHigh: Float,
    val bwWindowLow: Float,
    val bwWindowHigh: Float,
    val liftBinIndex: Int,
    val liftBinLow: Float,
    val liftBinHigh: Float,
    val localCellCount: Long,
    val neighborhoodCount: Long,
    val neighborhoodShare: Float,
)

sealed interface SliceIndexEntries {
    data class MapEntries(
        val entries: SortedMap<String, SliceIndexEntry>,
    ) : SliceIndexEntries

    data class KeyEntries(
        val keys: List<String>,
    ) : SliceIndexEntries
}

data class SliceIndexEntry(
    val meta: String = "",
    val hist: String,
    val heat: String,
    val summary: SliceSummary? = null,
)

data class SliceSummary(
    val minKg: Float,
    val maxKg: Float,
    val total: Int,
)

data class TrendsJson(
    val bucket: String,
    val series: List<TrendSeries>,
)

data class TrendSeries(
    val key: String,
    val points: List<TrendPoint>,
)

data class TrendPoint(
    val year: Int,
    val total: Int,
    val p50: Float,
    val p90: Float,
)
