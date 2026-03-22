package com.gregorycarnegie.ironinsights.data.repository

import com.gregorycarnegie.ironinsights.data.model.BodyweightConditionedLookup
import com.gregorycarnegie.ironinsights.data.cache.LatestDatasetVersionCache
import com.gregorycarnegie.ironinsights.data.cache.PublishedPayloadCache
import com.gregorycarnegie.ironinsights.config.EnvironmentConfig
import com.gregorycarnegie.ironinsights.data.model.DatasetLoadSource
import com.gregorycarnegie.ironinsights.data.model.HistogramBin
import com.gregorycarnegie.ironinsights.data.model.HeatmapBin
import com.gregorycarnegie.ironinsights.data.model.LatestJson
import com.gregorycarnegie.ironinsights.data.model.LoadedResource
import com.gregorycarnegie.ironinsights.data.model.PercentileLookup
import com.gregorycarnegie.ironinsights.data.model.RootIndex
import com.gregorycarnegie.ironinsights.data.model.SliceIndex
import com.gregorycarnegie.ironinsights.data.model.SliceIndexEntries
import com.gregorycarnegie.ironinsights.data.model.SliceIndexEntry
import com.gregorycarnegie.ironinsights.data.model.SliceSummary
import com.gregorycarnegie.ironinsights.data.model.TrendPoint
import com.gregorycarnegie.ironinsights.data.model.TrendSeries
import com.gregorycarnegie.ironinsights.data.model.TrendsJson
import org.json.JSONArray
import org.json.JSONException
import org.json.JSONObject
import java.io.IOException
import java.net.HttpURLConnection
import java.net.URL
import java.util.TreeMap

private const val HISTOGRAM_HEADER_SIZE = 22
private const val HISTOGRAM_MAGIC_0 = 'I'.code
private const val HISTOGRAM_MAGIC_1 = 'I'.code
private const val HISTOGRAM_MAGIC_2 = 'H'.code
private const val HISTOGRAM_MAGIC_3 = '1'.code
private const val HEATMAP_HEADER_SIZE = 38
private const val HEATMAP_MAGIC_0 = 'I'.code
private const val HEATMAP_MAGIC_1 = 'I'.code
private const val HEATMAP_MAGIC_2 = 'M'.code
private const val HEATMAP_MAGIC_3 = '1'.code

interface PublishedDataRepository {
    fun readCachedLatestVersion(): String?
    fun fetchLatest(): Result<LoadedResource<LatestJson>>
    fun fetchRootIndex(version: String): Result<LoadedResource<RootIndex>>
    fun fetchSliceIndex(version: String, shardPath: String): Result<LoadedResource<SliceIndex>>
    fun fetchHistogram(version: String, histogramPath: String): Result<LoadedResource<HistogramBin>>
    fun fetchHeatmap(version: String, heatmapPath: String): Result<LoadedResource<HeatmapBin>>
    fun fetchTrends(version: String): Result<LoadedResource<TrendsJson>>
}

class HttpPublishedDataRepository(
    private val environment: EnvironmentConfig,
    private val latestDatasetVersionCache: LatestDatasetVersionCache,
    private val payloadCache: PublishedPayloadCache,
) : PublishedDataRepository {
    override fun readCachedLatestVersion(): String? = latestDatasetVersionCache.read()

    override fun fetchLatest(): Result<LoadedResource<LatestJson>> = runCatching {
        loadJsonWithCache(
            relativePath = "latest.json",
            parser = ::parseLatestJson,
            fallback = {
                latestDatasetVersionCache.read()?.let { cachedVersion ->
                    LoadedResource(
                        value = LatestJson(version = cachedVersion, revision = null),
                        source = DatasetLoadSource.VERSION_CACHE,
                    )
                }
            },
        ).also { latest ->
            latestDatasetVersionCache.write(latest.value.version)
        }
    }

    override fun fetchRootIndex(version: String): Result<LoadedResource<RootIndex>> = runCatching {
        loadJsonWithCache(
            relativePath = "$version/index.json",
            parser = ::parseRootIndex,
        )
    }

    override fun fetchSliceIndex(version: String, shardPath: String): Result<LoadedResource<SliceIndex>> = runCatching {
        val normalizedPath = shardPath.trimStart('/')
        loadJsonWithCache(
            relativePath = "$version/$normalizedPath",
            parser = ::parseSliceIndex,
        )
    }

    override fun fetchHistogram(version: String, histogramPath: String): Result<LoadedResource<HistogramBin>> = runCatching {
        val normalizedPath = histogramPath.trimStart('/')
        loadBinaryWithCache(relativePath = "$version/$normalizedPath") { payload ->
            parseHistogramBin(payload) ?: throw IOException("Failed to parse histogram binary.")
        }
    }

    override fun fetchHeatmap(version: String, heatmapPath: String): Result<LoadedResource<HeatmapBin>> = runCatching {
        val normalizedPath = heatmapPath.trimStart('/')
        loadBinaryWithCache(relativePath = "$version/$normalizedPath") { payload ->
            parseHeatmapBin(payload) ?: throw IOException("Failed to parse heatmap binary.")
        }
    }

    override fun fetchTrends(version: String): Result<LoadedResource<TrendsJson>> = runCatching {
        loadJsonWithCache(
            relativePath = "$version/trends.json",
            parser = ::parseTrendsJson,
        )
    }

    private fun <T> loadJsonWithCache(
        relativePath: String,
        parser: (String) -> T,
        fallback: (() -> LoadedResource<T>?)? = null,
    ): LoadedResource<T> {
        val normalizedPath = relativePath.trimStart('/')
        val requestUrl = environment.dataBaseUrl + normalizedPath

        try {
            val body = loadJsonText(requestUrl)
            val parsed = parser(body)
            payloadCache.writeText(normalizedPath, body)
            return LoadedResource(
                value = parsed,
                source = DatasetLoadSource.NETWORK,
            )
        } catch (networkError: Exception) {
            val cachedBody = payloadCache.readText(normalizedPath)
            if (cachedBody != null) {
                try {
                    return LoadedResource(
                        value = parser(cachedBody),
                        source = DatasetLoadSource.DISK_CACHE,
                    )
                } catch (cacheError: Exception) {
                    payloadCache.delete(normalizedPath)
                    throw combinedLoadError(
                        relativePath = normalizedPath,
                        networkError = networkError,
                        cacheError = cacheError,
                    )
                }
            }

            fallback?.invoke()?.let { return it }
            throw IOException("Failed to load published JSON $normalizedPath.", networkError)
        }
    }

    private fun <T> loadBinaryWithCache(
        relativePath: String,
        parser: (ByteArray) -> T,
    ): LoadedResource<T> {
        val normalizedPath = relativePath.trimStart('/')
        val requestUrl = environment.dataBaseUrl + normalizedPath

        try {
            val payload = loadBinary(requestUrl)
            val parsed = parser(payload)
            payloadCache.writeBytes(normalizedPath, payload)
            return LoadedResource(
                value = parsed,
                source = DatasetLoadSource.NETWORK,
            )
        } catch (networkError: Exception) {
            val cachedPayload = payloadCache.readBytes(normalizedPath)
            if (cachedPayload != null) {
                try {
                    return LoadedResource(
                        value = parser(cachedPayload),
                        source = DatasetLoadSource.DISK_CACHE,
                    )
                } catch (cacheError: Exception) {
                    payloadCache.delete(normalizedPath)
                    throw combinedLoadError(
                        relativePath = normalizedPath,
                        networkError = networkError,
                        cacheError = cacheError,
                    )
                }
            }

            throw IOException("Failed to load published binary $normalizedPath.", networkError)
        }
    }

    private fun loadJsonText(requestUrl: String): String {
        val connection = (URL(requestUrl).openConnection() as HttpURLConnection).apply {
            requestMethod = "GET"
            connectTimeout = 10_000
            readTimeout = 10_000
            setRequestProperty("Accept", "application/json")
        }

        return try {
            val status = connection.responseCode
            if (status !in 200..299) {
                throw IOException("Failed to load published JSON: HTTP $status")
            }

            connection.inputStream.bufferedReader().use { it.readText() }
        } finally {
            connection.disconnect()
        }
    }

    private fun loadBinary(requestUrl: String): ByteArray {
        val connection = (URL(requestUrl).openConnection() as HttpURLConnection).apply {
            requestMethod = "GET"
            connectTimeout = 10_000
            readTimeout = 10_000
        }

        return try {
            val status = connection.responseCode
            if (status !in 200..299) {
                throw IOException("Failed to load published binary: HTTP $status")
            }
            connection.inputStream.use { input ->
                input.readBytes()
            }
        } finally {
            connection.disconnect()
        }
    }

    private fun combinedLoadError(
        relativePath: String,
        networkError: Throwable,
        cacheError: Throwable,
    ): IOException {
        return IOException(
            "Failed to load $relativePath from network and cache.",
            networkError,
        ).apply {
            addSuppressed(cacheError)
        }
    }
}

internal fun parseLatestJson(json: String): LatestJson {
    try {
        val payload = JSONObject(json)
        val version = payload.getString("version")
        val revision = if (payload.isNull("revision")) null else payload.optString("revision")
        return LatestJson(
            version = version,
            revision = revision,
        )
    } catch (error: JSONException) {
        throw IOException("Failed to parse latest dataset pointer.", error)
    }
}

internal fun parseRootIndex(json: String): RootIndex {
    try {
        val payload = JSONObject(json)
        val shardsJson = payload.getJSONObject("shards")
        val shards = TreeMap<String, String>()
        val keys = shardsJson.keys()
        while (keys.hasNext()) {
            val key = keys.next()
            shards[key] = shardsJson.getString(key)
        }
        return RootIndex(shards = shards)
    } catch (error: JSONException) {
        throw IOException("Failed to parse root index.", error)
    }
}

internal fun parseSliceIndex(json: String): SliceIndex {
    try {
        val payload = JSONObject(json)
        val slicesPayload = payload.get("slices")
        val slices = when (slicesPayload) {
            is JSONObject -> {
                val entries = TreeMap<String, SliceIndexEntry>()
                val keys = slicesPayload.keys()
                while (keys.hasNext()) {
                    val key = keys.next()
                    entries[key] = parseSliceIndexEntry(slicesPayload.getJSONObject(key))
                }
                SliceIndexEntries.MapEntries(entries)
            }

            is JSONArray -> {
                val keys = buildList(slicesPayload.length()) {
                    for (index in 0 until slicesPayload.length()) {
                        add(slicesPayload.getString(index))
                    }
                }
                SliceIndexEntries.KeyEntries(keys)
            }

            else -> throw IOException("Unsupported slices payload type in shard index.")
        }
        return SliceIndex(slices = slices)
    } catch (error: JSONException) {
        throw IOException("Failed to parse shard index.", error)
    }
}

private fun parseSliceIndexEntry(json: JSONObject): SliceIndexEntry {
    val summary = if (json.isNull("summary")) {
        null
    } else {
        val summaryJson = json.getJSONObject("summary")
        SliceSummary(
            minKg = summaryJson.getDouble("min_kg").toFloat(),
            maxKg = summaryJson.getDouble("max_kg").toFloat(),
            total = summaryJson.getInt("total"),
        )
    }

    return SliceIndexEntry(
        meta = if (json.isNull("meta")) "" else json.optString("meta", ""),
        hist = json.getString("hist"),
        heat = json.getString("heat"),
        summary = summary,
    )
}

internal fun parseTrendsJson(json: String): TrendsJson {
    try {
        val payload = JSONObject(json)
        val seriesJson = payload.getJSONArray("series")
        val series = buildList(seriesJson.length()) {
            for (index in 0 until seriesJson.length()) {
                add(parseTrendSeries(seriesJson.getJSONObject(index)))
            }
        }
        return TrendsJson(
            bucket = payload.getString("bucket"),
            series = series,
        )
    } catch (error: JSONException) {
        throw IOException("Failed to parse trends payload.", error)
    }
}

private fun parseTrendSeries(json: JSONObject): TrendSeries {
    val pointsJson = json.getJSONArray("points")
    val points = buildList(pointsJson.length()) {
        for (index in 0 until pointsJson.length()) {
            add(parseTrendPoint(pointsJson.getJSONObject(index)))
        }
    }

    return TrendSeries(
        key = json.getString("key"),
        points = points,
    )
}

private fun parseTrendPoint(json: JSONObject): TrendPoint {
    return TrendPoint(
        year = json.getInt("year"),
        total = json.getInt("total"),
        p50 = json.getDouble("p50").toFloat(),
        p90 = json.getDouble("p90").toFloat(),
    )
}

internal fun parseHistogramBin(bytes: ByteArray): HistogramBin? {
    if (bytes.size < HISTOGRAM_HEADER_SIZE) {
        return null
    }
    if (
        bytes[0].toInt() != HISTOGRAM_MAGIC_0 ||
        bytes[1].toInt() != HISTOGRAM_MAGIC_1 ||
        bytes[2].toInt() != HISTOGRAM_MAGIC_2 ||
        bytes[3].toInt() != HISTOGRAM_MAGIC_3
    ) {
        return null
    }

    val version = readU16Le(bytes, 4)
    if (version != 1) {
        return null
    }

    val baseBin = readF32Le(bytes, 6)
    val min = readF32Le(bytes, 10)
    val max = readF32Le(bytes, 14)
    val binCount = readU32Le(bytes, 18)
    if (binCount > Int.MAX_VALUE.toLong()) {
        return null
    }

    val payloadLength = bytes.size - HISTOGRAM_HEADER_SIZE
    if (payloadLength.toLong() != binCount * 4L) {
        return null
    }

    val counts = ArrayList<Long>(binCount.toInt())
    var offset = HISTOGRAM_HEADER_SIZE
    repeat(binCount.toInt()) {
        counts += readU32Le(bytes, offset)
        offset += 4
    }

    return HistogramBin(
        min = min,
        max = max,
        baseBin = baseBin,
        counts = counts,
        total = counts.fold(0L, Long::plus),
    )
}

internal fun parseHeatmapBin(bytes: ByteArray): HeatmapBin? {
    if (bytes.size < HEATMAP_HEADER_SIZE) {
        return null
    }
    if (
        bytes[0].toInt() != HEATMAP_MAGIC_0 ||
        bytes[1].toInt() != HEATMAP_MAGIC_1 ||
        bytes[2].toInt() != HEATMAP_MAGIC_2 ||
        bytes[3].toInt() != HEATMAP_MAGIC_3
    ) {
        return null
    }

    val version = readU16Le(bytes, 4)
    if (version != 1) {
        return null
    }

    val baseX = readF32Le(bytes, 6)
    val baseY = readF32Le(bytes, 10)
    val minX = readF32Le(bytes, 14)
    val maxX = readF32Le(bytes, 18)
    val minY = readF32Le(bytes, 22)
    val maxY = readF32Le(bytes, 26)
    val width = readU32Le(bytes, 30)
    val height = readU32Le(bytes, 34)
    if (width > Int.MAX_VALUE.toLong() || height > Int.MAX_VALUE.toLong()) {
        return null
    }

    val payloadLength = bytes.size - HEATMAP_HEADER_SIZE
    val gridSize = width * height
    if (payloadLength.toLong() != gridSize * 4L) {
        return null
    }

    val grid = ArrayList<Long>(gridSize.toInt())
    var offset = HEATMAP_HEADER_SIZE
    repeat(gridSize.toInt()) {
        grid += readU32Le(bytes, offset)
        offset += 4
    }

    return HeatmapBin(
        minX = minX,
        maxX = maxX,
        minY = minY,
        maxY = maxY,
        baseX = baseX,
        baseY = baseY,
        width = width.toInt(),
        height = height.toInt(),
        grid = grid,
    )
}

internal fun percentileForValue(
    histogram: HistogramBin?,
    value: Float,
): PercentileLookup? {
    val histogram = histogram ?: return null
    if (histogram.counts.isEmpty() || histogram.total == 0L || histogram.baseBin <= 0f) {
        return null
    }

    val maxIndex = histogram.counts.lastIndex
    val rawIndex = kotlin.math.floor((value - histogram.min) / histogram.baseBin.toDouble())
    val binIndex = rawIndex.coerceIn(0.0, maxIndex.toDouble()).toInt()
    val below = histogram.counts.take(binIndex).sum()
    val current = histogram.counts[binIndex].toFloat()
    val cdf = below.toFloat() + 0.5f * current
    val percentile = cdf / histogram.total.toFloat()
    val rank = ((1f - percentile) * histogram.total.toFloat()).roundToLong().coerceAtLeast(1L)
    val binLow = histogram.min + binIndex * histogram.baseBin

    return PercentileLookup(
        percentile = percentile,
        rank = rank,
        total = histogram.total,
        binIndex = binIndex,
        binLow = binLow,
        binHigh = binLow + histogram.baseBin,
    )
}

internal fun valueForPercentile(
    histogram: HistogramBin?,
    targetPercentile: Float,
): Float? {
    val histogram = histogram ?: return null
    if (histogram.counts.isEmpty() || histogram.total == 0L || histogram.baseBin <= 0f) {
        return null
    }

    val target = targetPercentile.coerceIn(0f, 1f) * histogram.total.toFloat()
    var below = 0f
    histogram.counts.forEachIndexed { index, count ->
        val countFloat = count.toFloat()
        val cdfMid = below + 0.5f * countFloat
        if (cdfMid >= target) {
            return histogram.min + (index + 0.5f) * histogram.baseBin
        }
        below += countFloat
    }

    return histogram.max
}

internal fun bodyweightConditionedPercentile(
    heatmap: HeatmapBin?,
    liftValue: Float,
    bodyweightKg: Float,
): BodyweightConditionedLookup? {
    val heatmap = heatmap ?: return null
    if (heatmap.width == 0 || heatmap.height == 0 || heatmap.grid.size != heatmap.width * heatmap.height) {
        return null
    }
    if (heatmap.baseX <= 0f || heatmap.baseY <= 0f) {
        return null
    }

    val totalHeat = heatmap.grid.sum()
    if (totalHeat == 0L) {
        return null
    }

    val liftBinIndex = kotlin.math.floor((liftValue - heatmap.minX) / heatmap.baseX.toDouble())
        .coerceIn(0.0, (heatmap.width - 1).toDouble())
        .toInt()
    val bodyweightBinIndex = kotlin.math.floor((bodyweightKg - heatmap.minY) / heatmap.baseY.toDouble())
        .coerceIn(0.0, (heatmap.height - 1).toDouble())
        .toInt()

    val rowLow = (bodyweightBinIndex - 1).coerceAtLeast(0)
    val rowHigh = (bodyweightBinIndex + 1).coerceAtMost(heatmap.height - 1)

    val nearbyCounts = MutableList(heatmap.width) { 0L }
    for (y in rowLow..rowHigh) {
        for (x in 0 until heatmap.width) {
            val index = y * heatmap.width + x
            nearbyCounts[x] += heatmap.grid[index]
        }
    }

    val totalNearby = nearbyCounts.sum()
    if (totalNearby == 0L) {
        return null
    }

    val below = nearbyCounts.take(liftBinIndex).sum()
    val current = nearbyCounts[liftBinIndex].toFloat()
    val cdf = below.toFloat() + 0.5f * current
    val percentile = cdf / totalNearby.toFloat()
    val rank = ((1f - percentile) * totalNearby.toFloat()).roundToLong().coerceAtLeast(1L)

    val xLow = (liftBinIndex - 1).coerceAtLeast(0)
    val xHigh = (liftBinIndex + 1).coerceAtMost(heatmap.width - 1)
    var neighborhoodCount = 0L
    for (y in rowLow..rowHigh) {
        for (x in xLow..xHigh) {
            neighborhoodCount += heatmap.grid[y * heatmap.width + x]
        }
    }

    return BodyweightConditionedLookup(
        percentile = percentile,
        rank = rank,
        totalNearby = totalNearby,
        bwBinIndex = bodyweightBinIndex,
        bwBinLow = heatmap.minY + bodyweightBinIndex * heatmap.baseY,
        bwBinHigh = heatmap.minY + (bodyweightBinIndex + 1) * heatmap.baseY,
        bwWindowLow = heatmap.minY + rowLow * heatmap.baseY,
        bwWindowHigh = heatmap.minY + (rowHigh + 1) * heatmap.baseY,
        liftBinIndex = liftBinIndex,
        liftBinLow = heatmap.minX + liftBinIndex * heatmap.baseX,
        liftBinHigh = heatmap.minX + (liftBinIndex + 1) * heatmap.baseX,
        localCellCount = heatmap.grid[bodyweightBinIndex * heatmap.width + liftBinIndex],
        neighborhoodCount = neighborhoodCount,
        neighborhoodShare = neighborhoodCount.toFloat() / totalHeat.toFloat(),
    )
}

private fun readU16Le(
    bytes: ByteArray,
    offset: Int,
): Int {
    return (bytes[offset].toInt() and 0xff) or
        ((bytes[offset + 1].toInt() and 0xff) shl 8)
}

private fun readU32Le(
    bytes: ByteArray,
    offset: Int,
): Long {
    return (bytes[offset].toLong() and 0xff) or
        ((bytes[offset + 1].toLong() and 0xff) shl 8) or
        ((bytes[offset + 2].toLong() and 0xff) shl 16) or
        ((bytes[offset + 3].toLong() and 0xff) shl 24)
}

private fun readF32Le(
    bytes: ByteArray,
    offset: Int,
): Float {
    return Float.fromBits(readU32Le(bytes, offset).toInt())
}

private fun Float.roundToLong(): Long = kotlin.math.round(this).toLong()
