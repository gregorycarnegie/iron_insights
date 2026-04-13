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
import android.util.Base64
import org.json.JSONArray
import org.json.JSONException
import org.json.JSONObject
import java.io.IOException
import java.net.HttpURLConnection
import java.net.URL
import java.util.Locale
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
private const val COMBINED_MAGIC_0 = 'I'.code
private const val COMBINED_MAGIC_1 = 'I'.code
private const val COMBINED_MAGIC_2 = 'C'.code
private const val COMBINED_MAGIC_3 = '1'.code
private const val COMBINED_HEADER_SIZE = 10
private const val MAX_CACHED_DATASET_VERSIONS = 2

private enum class PublishedPayloadType(
    val label: String,
) {
    JSON("JSON"),
    BINARY("binary"),
}

private class PublishedHttpException(
    val relativePath: String,
    val payloadType: PublishedPayloadType,
    val status: Int,
) : IOException("Published ${payloadType.label} $relativePath returned HTTP $status.")

private class PublishedPayloadParseException(
    val relativePath: String,
    val payloadType: PublishedPayloadType,
    val source: DatasetLoadSource,
    cause: Throwable,
) : IOException(
        "Published ${payloadType.label} $relativePath from ${source.label()} was malformed.",
        cause,
    )

private data class CachedLatestLookup(
    val latest: LoadedResource<LatestJson>? = null,
    val cacheError: Throwable? = null,
)

interface PublishedDataRepository {
    fun readCachedLatestVersion(): String?
    fun fetchLatest(): Result<LoadedResource<LatestJson>>
    fun fetchRootIndex(version: String): Result<LoadedResource<RootIndex>>
    fun fetchSliceIndex(version: String, shardPath: String): Result<LoadedResource<SliceIndex>>
    fun fetchCombinedBin(
        version: String,
        binPath: String,
        inlineData: String = "",
    ): Result<LoadedResource<Pair<HistogramBin, HeatmapBin?>>>
    fun fetchTrends(version: String, trendsPath: String): Result<LoadedResource<TrendsJson>>
}

class HttpPublishedDataRepository(
    private val environment: EnvironmentConfig,
    private val latestDatasetVersionCache: LatestDatasetVersionCache,
    private val payloadCache: PublishedPayloadCache,
) : PublishedDataRepository {
    override fun readCachedLatestVersion(): String? = latestDatasetVersionCache.read()

    override fun fetchLatest(): Result<LoadedResource<LatestJson>> = runCatching {
        val relativePath = "latest.json"
        val requestUrl = environment.dataBaseUrl + relativePath
        val preferredVersion = latestDatasetVersionCache.read()

        try {
            val body = loadJsonText(requestUrl, relativePath)
            val latest = parsePublishedText(
                relativePath = relativePath,
                source = DatasetLoadSource.NETWORK,
                parser = ::parseLatestJson,
                payload = body,
            )
            payloadCache.writeText(relativePath, body)
            pruneVersionedPayloads(relativePath)
            LoadedResource(
                value = latest,
                source = DatasetLoadSource.NETWORK,
            )
        } catch (networkError: Exception) {
            val cachedLatest = loadUsableCachedLatest(relativePath)
            cachedLatest.latest
                ?: resolveLatestFallbackResource(
                    preferredVersion = preferredVersion,
                    cachedVersions = payloadCache.cachedVersions(),
                    hasVersionRootIndex = { version ->
                        payloadCache.hasEntry(versionRootIndexPath(version))
                    },
                )
                ?: throw latestFallbackError(
                    preferredVersion = preferredVersion,
                    networkError = networkError,
                    cacheError = cachedLatest.cacheError,
                )
        }.also { latest ->
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

    override fun fetchCombinedBin(
        version: String,
        binPath: String,
        inlineData: String,
    ): Result<LoadedResource<Pair<HistogramBin, HeatmapBin?>>> = runCatching {
        if (inlineData.isNotEmpty()) {
            val bytes = Base64.decode(inlineData, Base64.DEFAULT)
            val parsed = parseCombinedBin(bytes)
                ?: throw IOException("Failed to parse inline combined binary.")
            return@runCatching LoadedResource(
                value = parsed,
                source = DatasetLoadSource.NETWORK,
            )
        }
        val normalizedPath = binPath.trimStart('/')
        loadBinaryWithCache(relativePath = "$version/$normalizedPath") { payload ->
            parseCombinedBin(payload) ?: throw IOException("Failed to parse combined binary.")
        }
    }

    override fun fetchTrends(version: String, trendsPath: String): Result<LoadedResource<TrendsJson>> = runCatching {
        val normalizedPath = trendsPath.trimStart('/')
        loadJsonWithCache(
            relativePath = "$version/$normalizedPath",
            parser = ::parseTrendsJson,
        )
    }

    private fun loadUsableCachedLatest(relativePath: String): CachedLatestLookup {
        val cachedBody = payloadCache.readText(relativePath) ?: return CachedLatestLookup()

        return try {
            val latest = parsePublishedText(
                relativePath = relativePath,
                source = DatasetLoadSource.DISK_CACHE,
                parser = ::parseLatestJson,
                payload = cachedBody,
            )
            if (!payloadCache.hasEntry(versionRootIndexPath(latest.version))) {
                CachedLatestLookup(cacheError = staleLatestCacheError(latest.version))
            } else {
                CachedLatestLookup(
                    latest = LoadedResource(
                        value = latest,
                        source = DatasetLoadSource.DISK_CACHE,
                    ),
                )
            }
        } catch (cacheError: Exception) {
            payloadCache.delete(relativePath)
            CachedLatestLookup(cacheError = cacheError)
        }
    }

    private fun <T> loadJsonWithCache(
        relativePath: String,
        parser: (String) -> T,
        fallback: (() -> LoadedResource<T>?)? = null,
    ): LoadedResource<T> {
        val normalizedPath = relativePath.trimStart('/')
        val requestUrl = environment.dataBaseUrl + normalizedPath

        try {
            val body = loadJsonText(requestUrl, normalizedPath)
            val parsed = parsePublishedText(
                relativePath = normalizedPath,
                source = DatasetLoadSource.NETWORK,
                parser = parser,
                payload = body,
            )
            payloadCache.writeText(normalizedPath, body)
            pruneVersionedPayloads(normalizedPath)
            return LoadedResource(
                value = parsed,
                source = DatasetLoadSource.NETWORK,
            )
        } catch (networkError: Exception) {
            val cachedBody = payloadCache.readText(normalizedPath)
            if (cachedBody != null) {
                try {
                    return LoadedResource(
                        value = parsePublishedText(
                            relativePath = normalizedPath,
                            source = DatasetLoadSource.DISK_CACHE,
                            parser = parser,
                            payload = cachedBody,
                        ),
                        source = DatasetLoadSource.DISK_CACHE,
                    )
                } catch (cacheError: Exception) {
                    payloadCache.delete(normalizedPath)
                    throw combinedLoadError(
                        relativePath = normalizedPath,
                        payloadType = PublishedPayloadType.JSON,
                        networkError = networkError,
                        cacheError = cacheError,
                    )
                }
            }

            fallback?.invoke()?.let { return it }
            throw primaryLoadError(
                relativePath = normalizedPath,
                payloadType = PublishedPayloadType.JSON,
                networkError = networkError,
            )
        }
    }

    private fun <T> loadBinaryWithCache(
        relativePath: String,
        parser: (ByteArray) -> T,
    ): LoadedResource<T> {
        val normalizedPath = relativePath.trimStart('/')
        val requestUrl = environment.dataBaseUrl + normalizedPath

        try {
            val payload = loadBinary(requestUrl, normalizedPath)
            val parsed = parsePublishedBytes(
                relativePath = normalizedPath,
                source = DatasetLoadSource.NETWORK,
                parser = parser,
                payload = payload,
            )
            payloadCache.writeBytes(normalizedPath, payload)
            pruneVersionedPayloads(normalizedPath)
            return LoadedResource(
                value = parsed,
                source = DatasetLoadSource.NETWORK,
            )
        } catch (networkError: Exception) {
            val cachedPayload = payloadCache.readBytes(normalizedPath)
            if (cachedPayload != null) {
                try {
                    return LoadedResource(
                        value = parsePublishedBytes(
                            relativePath = normalizedPath,
                            source = DatasetLoadSource.DISK_CACHE,
                            parser = parser,
                            payload = cachedPayload,
                        ),
                        source = DatasetLoadSource.DISK_CACHE,
                    )
                } catch (cacheError: Exception) {
                    payloadCache.delete(normalizedPath)
                    throw combinedLoadError(
                        relativePath = normalizedPath,
                        payloadType = PublishedPayloadType.BINARY,
                        networkError = networkError,
                        cacheError = cacheError,
                    )
                }
            }

            throw primaryLoadError(
                relativePath = normalizedPath,
                payloadType = PublishedPayloadType.BINARY,
                networkError = networkError,
            )
        }
    }

    private fun loadJsonText(
        requestUrl: String,
        relativePath: String,
    ): String {
        val connection = (URL(requestUrl).openConnection() as HttpURLConnection).apply {
            requestMethod = "GET"
            connectTimeout = 10_000
            readTimeout = 10_000
            setRequestProperty("Accept", "application/json")
        }

        return try {
            val status = connection.responseCode
            if (status !in 200..299) {
                throw PublishedHttpException(
                    relativePath = relativePath,
                    payloadType = PublishedPayloadType.JSON,
                    status = status,
                )
            }

            connection.inputStream.bufferedReader().use { it.readText() }
        } finally {
            connection.disconnect()
        }
    }

    private fun loadBinary(
        requestUrl: String,
        relativePath: String,
    ): ByteArray {
        val connection = (URL(requestUrl).openConnection() as HttpURLConnection).apply {
            requestMethod = "GET"
            connectTimeout = 10_000
            readTimeout = 10_000
        }

        return try {
            val status = connection.responseCode
            if (status !in 200..299) {
                throw PublishedHttpException(
                    relativePath = relativePath,
                    payloadType = PublishedPayloadType.BINARY,
                    status = status,
                )
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
        payloadType: PublishedPayloadType,
        networkError: Throwable,
        cacheError: Throwable,
    ): IOException {
        return IOException(
            "${primaryLoadError(relativePath, payloadType, networkError).message} Cached copy was also invalid.",
            networkError,
        ).apply {
            addSuppressed(cacheError)
        }
    }

    private fun pruneVersionedPayloads(relativePath: String) {
        if (extractVersionSegment(relativePath) == null) {
            return
        }

        payloadCache.pruneCachedVersions(MAX_CACHED_DATASET_VERSIONS)
    }
}

internal fun resolveLatestFallbackResource(
    preferredVersion: String?,
    cachedVersions: List<String>,
    hasVersionRootIndex: (String) -> Boolean,
): LoadedResource<LatestJson>? {
    val normalizedPreferred = preferredVersion?.trim()?.takeIf { it.isNotEmpty() }
    if (normalizedPreferred != null && hasVersionRootIndex(normalizedPreferred)) {
        return LoadedResource(
            value = LatestJson(version = normalizedPreferred, revision = null),
            source = DatasetLoadSource.VERSION_CACHE,
        )
    }

    val discoveredVersion = cachedVersions
        .asSequence()
        .map { it.trim() }
        .filter { it.isNotEmpty() && it != normalizedPreferred }
        .sortedDescending()
        .firstOrNull { version -> hasVersionRootIndex(version) }
        ?: return null

    return LoadedResource(
        value = LatestJson(version = discoveredVersion, revision = null),
        source = DatasetLoadSource.DISK_CACHE,
    )
}

private fun versionRootIndexPath(version: String): String = "${version.trim()}/index.json"

internal fun extractVersionSegment(relativePath: String): String? {
    val normalizedPath = relativePath.trim().trimStart('/').replace('\\', '/')
    val version = normalizedPath.substringBefore('/', missingDelimiterValue = "")
    return version.takeIf { it.isNotBlank() && it != "latest.json" }
}

private fun staleLatestCacheError(version: String): IOException {
    return IOException(
        "Cached latest pointer references dataset $version, but its cached root index is missing.",
    )
}

private fun latestFallbackError(
    preferredVersion: String?,
    networkError: Throwable,
    cacheError: Throwable?,
): IOException {
    val message = if (preferredVersion.isNullOrBlank()) {
        "Failed to load a usable latest dataset pointer, and no cached dataset version was available."
    } else {
        "Cached dataset version $preferredVersion is stale or incomplete. Reconnect and refresh the published dataset before using the app offline."
    }

    return IOException(message, networkError).apply {
        cacheError?.let(::addSuppressed)
    }
}

private fun <T> parsePublishedText(
    relativePath: String,
    source: DatasetLoadSource,
    parser: (String) -> T,
    payload: String,
): T {
    return try {
        parser(payload)
    } catch (error: Exception) {
        throw PublishedPayloadParseException(
            relativePath = relativePath,
            payloadType = PublishedPayloadType.JSON,
            source = source,
            cause = error,
        )
    }
}

private fun <T> parsePublishedBytes(
    relativePath: String,
    source: DatasetLoadSource,
    parser: (ByteArray) -> T,
    payload: ByteArray,
): T {
    return try {
        parser(payload)
    } catch (error: Exception) {
        throw PublishedPayloadParseException(
            relativePath = relativePath,
            payloadType = PublishedPayloadType.BINARY,
            source = source,
            cause = error,
        )
    }
}

private fun primaryLoadError(
    relativePath: String,
    payloadType: PublishedPayloadType,
    networkError: Throwable,
): IOException {
    val label = payloadType.label.lowercase(Locale.US)
    return when (networkError) {
        is PublishedPayloadParseException -> IOException(
            "Published $label $relativePath was malformed and no valid cache copy was available.",
            networkError,
        )

        is PublishedHttpException -> when (networkError.status) {
            404,
            410,
            -> IOException(
                "Published $label $relativePath was not found. The selected dataset version may be stale or not fully deployed yet.",
                networkError,
            )

            else -> IOException(
                "Failed to load published $label $relativePath: HTTP ${networkError.status}.",
                networkError,
            )
        }

        else -> IOException("Failed to load published $label $relativePath.", networkError)
    }
}

private fun DatasetLoadSource.label(): String {
    return when (this) {
        DatasetLoadSource.NETWORK -> "network"
        DatasetLoadSource.DISK_CACHE -> "disk cache"
        DatasetLoadSource.VERSION_CACHE -> "cached version pointer"
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
        var keys = shardsJson.keys()
        while (keys.hasNext()) {
            val key = keys.next()
            shards[key] = shardsJson.getString(key)
        }
        val trendsShards = TreeMap<String, String>()
        val trendsShardsJson = payload.optJSONObject("trends_shards")
        if (trendsShardsJson != null) {
            keys = trendsShardsJson.keys()
            while (keys.hasNext()) {
                val key = keys.next()
                trendsShards[key] = trendsShardsJson.getString(key)
            }
        }
        return RootIndex(shards = shards, trendsShards = trendsShards)
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
        bin = json.optString("bin", ""),
        inline = json.optString("inline", ""),
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

internal fun parseCombinedBin(bytes: ByteArray): Pair<HistogramBin, HeatmapBin?>? {
    if (bytes.size < COMBINED_HEADER_SIZE) return null
    if (
        bytes[0].toInt() != COMBINED_MAGIC_0 ||
        bytes[1].toInt() != COMBINED_MAGIC_1 ||
        bytes[2].toInt() != COMBINED_MAGIC_2 ||
        bytes[3].toInt() != COMBINED_MAGIC_3
    ) return null

    val version = readU16Le(bytes, 4)
    if (version != 1) return null

    val histLen = readU32Le(bytes, 6).toInt()
    val histEnd = COMBINED_HEADER_SIZE + histLen
    if (histEnd > bytes.size) return null

    val histBytes = bytes.copyOfRange(COMBINED_HEADER_SIZE, histEnd)
    val heatBytes = bytes.copyOfRange(histEnd, bytes.size)

    val histogram = parseHistogramBin(histBytes) ?: return null
    val heatmap = parseHeatmapBin(heatBytes)
    return histogram to heatmap
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
