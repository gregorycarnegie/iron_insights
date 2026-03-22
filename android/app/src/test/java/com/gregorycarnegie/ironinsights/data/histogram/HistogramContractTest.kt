package com.gregorycarnegie.ironinsights.data.histogram

import com.gregorycarnegie.ironinsights.data.model.TrendPoint
import com.gregorycarnegie.ironinsights.data.model.TrendSeries
import com.gregorycarnegie.ironinsights.ui.home.LookupFilters
import com.gregorycarnegie.ironinsights.ui.home.buildTrendKey
import com.gregorycarnegie.ironinsights.ui.home.buildTrendSeriesPresentation
import com.gregorycarnegie.ironinsights.ui.home.driftSummary
import com.gregorycarnegie.ironinsights.ui.home.growthSummary
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import kotlin.math.floor
import kotlin.math.roundToInt

class HistogramContractTest {
    @Test
    fun parseHistBin_accepts_valid_payload() {
        val bytes = buildHistogramBytes(
            baseBin = 2.5f,
            min = 100.0f,
            max = 107.5f,
            counts = intArrayOf(3, 1, 1),
        )

        val hist = parseHistBin(bytes)

        assertEquals(2.5f, hist?.baseBin ?: error("expected parsed histogram"), 0.0f)
        assertEquals(100.0f, hist?.min ?: error("expected parsed histogram"), 0.0f)
        assertEquals(107.5f, hist?.max ?: error("expected parsed histogram"), 0.0f)
        assertEquals(listOf(3, 1, 1), hist?.counts)
        assertEquals(5, hist?.total)
    }

    @Test
    fun parseHistBin_rejects_invalid_payload_len() {
        val bytes = buildHistogramHeader(
            baseBin = 2.5f,
            min = 100.0f,
            max = 105.0f,
            binCount = 2,
        ).apply {
            writeU32(1u)
        }.toByteArray()

        assertNull(parseHistBin(bytes))
    }

    @Test
    fun parseHistBin_rejects_unsupported_version() {
        val bytes = buildHistogramHeader(
            baseBin = 2.5f,
            min = 100.0f,
            max = 102.5f,
            binCount = 1,
            version = 2,
        ).apply {
            writeU32(1u)
        }.toByteArray()

        assertNull(parseHistBin(bytes))
    }

    @Test
    fun percentileForValue_matches_rust_mid_bin_formula() {
        val hist = HistogramBin(
            min = 100.0f,
            max = 110.0f,
            baseBin = 2.5f,
            counts = listOf(10, 20, 30, 40),
        )

        val stats = percentileForValue(hist, 104.0f)

        assertEquals(0.20f, stats?.percentile ?: error("expected percentile"), 0.0f)
        assertEquals(80, stats?.rank)
        assertEquals(100, stats?.total)
    }

    @Test
    fun percentileForValue_clamps_out_of_range_values() {
        val hist = HistogramBin(
            min = 100.0f,
            max = 110.0f,
            baseBin = 2.5f,
            counts = listOf(10, 20, 30, 40),
        )

        val low = percentileForValue(hist, 80.0f)
        val high = percentileForValue(hist, 200.0f)

        assertTrue(low!!.percentile < high!!.percentile)
        assertEquals(100, low.total)
        assertEquals(100, high.total)
    }

    @Test
    fun buildTrendKey_uses_the_web_series_dimensions() {
        val filters = LookupFilters(
            sex = "M",
            equip = "Raw",
            wc = "90",
            age = "24-34",
            tested = "All",
            lift = "T",
            metric = "Kg",
        )

        assertEquals(
            "sex=M|equip=Raw|tested=All|lift=T|metric=Kg",
            buildTrendKey(filters),
        )
    }

    @Test
    fun buildTrendSeriesPresentation_selects_matching_series_and_builds_summaries() {
        val filters = LookupFilters(
            sex = "M",
            equip = "Raw",
            wc = "All",
            age = "All Ages",
            tested = "All",
            lift = "T",
            metric = "Kg",
        )
        val trends = listOf(
            TrendSeries(
                key = buildTrendKey(filters),
                points = listOf(
                    TrendPoint(year = 2020, total = 120, p50 = 500f, p90 = 700f),
                    TrendPoint(year = 2024, total = 180, p50 = 530f, p90 = 735f),
                ),
            ),
        )

        val presentation = buildTrendSeriesPresentation("year", trends, filters)
            ?: error("expected matching presentation")

        assertEquals("sex=M|equip=Raw|tested=All|lift=T|metric=Kg", presentation.key)
        assertEquals("2020-2024: +60 lifters (+50.0%).", presentation.growthSummary)
        assertEquals("2020-2024: +30.0", presentation.p50DriftSummary)
        assertEquals("2020-2024: +35.0", presentation.p90DriftSummary)
        assertTrue(presentation.note.contains("Men / Raw / All tested statuses / Total / Kg"))
    }

    @Test
    fun buildTrendSeriesPresentation_returns_null_for_short_series() {
        val filters = LookupFilters(
            sex = "F",
            equip = "Raw",
            wc = "All",
            age = "All Ages",
            tested = "Yes",
            lift = "B",
            metric = "Kg",
        )

        val result = buildTrendSeriesPresentation(
            bucket = "year",
            series = listOf(
                TrendSeries(
                    key = buildTrendKey(filters),
                    points = listOf(TrendPoint(year = 2024, total = 40, p50 = 70f, p90 = 95f)),
                ),
            ),
            filters = filters,
        )

        assertNull(result)
    }

    @Test
    fun trendSummary_helpers_handle_zero_baselines_and_last_minus_first_drift() {
        val points = listOf(
            TrendPoint(year = 2021, total = 0, p50 = 110f, p90 = 150f),
            TrendPoint(year = 2023, total = 45, p50 = 130f, p90 = 175f),
        )

        assertEquals("2021-2023: +45 lifters (+0.0%).", growthSummary(points))
        assertEquals("2021-2023: +20.0", driftSummary(points) { it.p50 })
        assertEquals("2021-2023: +25.0", driftSummary(points) { it.p90 })
    }

    @Test
    fun percentileForValue_returns_null_for_empty_distribution() {
        assertNull(percentileForValue(HistogramBin(0.0f, 0.0f, 1.0f, emptyList()), 0.0f))
        assertNull(percentileForValue(HistogramBin(0.0f, 3.0f, 1.0f, listOf(0, 0, 0)), 1.0f))
        assertNull(percentileForValue(null, 1.0f))
    }
}

private const val BINARY_FORMAT_VERSION = 1

private data class HistogramBin(
    val min: Float,
    val max: Float,
    val baseBin: Float,
    val counts: List<Int>,
) {
    val total: Int = counts.sum()
}

private data class PercentileStats(
    val percentile: Float,
    val rank: Int,
    val total: Int,
)

private fun parseHistBin(bytes: ByteArray): HistogramBin? {
    if (bytes.size < 22 || !bytes.copyOfRange(0, 4).contentEquals("IIH1".toByteArray())) {
        return null
    }

    val version = bytes.readU16(4)
    if (version != BINARY_FORMAT_VERSION) {
        return null
    }

    val base = bytes.readF32(6)
    val min = bytes.readF32(10)
    val max = bytes.readF32(14)
    val bins = bytes.readU32(18).toInt()
    val payload = bytes.copyOfRange(22, bytes.size)
    if (payload.size != bins * 4) {
        return null
    }

    val counts = buildList(bins) {
        for (index in 0 until bins) {
            add(payload.readU32(index * 4).toInt())
        }
    }

    return HistogramBin(
        min = min,
        max = max,
        baseBin = base,
        counts = counts,
    )
}

private fun percentileForValue(hist: HistogramBin?, value: Float): PercentileStats? {
    val histogram = hist ?: return null
    if (histogram.counts.isEmpty() || histogram.baseBin <= 0.0f) {
        return null
    }

    val total = histogram.total
    if (total == 0) {
        return null
    }

    val binIndex = floor((value - histogram.min) / histogram.baseBin)
        .coerceIn(0.0f, histogram.counts.lastIndex.toFloat())
        .toInt()
    val below = histogram.counts.take(binIndex).sum()
    val current = histogram.counts[binIndex].toFloat()
    val cdf = below.toFloat() + 0.5f * current
    val percentile = cdf / total.toFloat()
    val rank = ((1.0f - percentile) * total.toFloat()).roundToInt().coerceAtLeast(1)

    return PercentileStats(
        percentile = percentile,
        rank = rank,
        total = total,
    )
}

private fun buildHistogramBytes(
    baseBin: Float,
    min: Float,
    max: Float,
    counts: IntArray,
): ByteArray {
    return buildHistogramHeader(
        baseBin = baseBin,
        min = min,
        max = max,
        binCount = counts.size,
    ).apply {
        counts.forEach { writeU32(it.toUInt()) }
    }.toByteArray()
}

private fun buildHistogramHeader(
    baseBin: Float,
    min: Float,
    max: Float,
    binCount: Int,
    version: Int = BINARY_FORMAT_VERSION,
): ByteWriter = ByteWriter().apply {
    writeBytes("IIH1".toByteArray())
    writeU16(version.toUInt())
    writeF32(baseBin)
    writeF32(min)
    writeF32(max)
    writeU32(binCount.toUInt())
}

private class ByteWriter {
    private val bytes = ArrayList<Byte>()

    fun writeBytes(value: ByteArray) {
        value.forEach { bytes.add(it) }
    }

    fun writeU16(value: UInt) {
        bytes.add((value and 0xFFu).toByte())
        bytes.add(((value shr 8) and 0xFFu).toByte())
    }

    fun writeU32(value: UInt) {
        bytes.add((value and 0xFFu).toByte())
        bytes.add(((value shr 8) and 0xFFu).toByte())
        bytes.add(((value shr 16) and 0xFFu).toByte())
        bytes.add(((value shr 24) and 0xFFu).toByte())
    }

    fun writeF32(value: Float) {
        writeU32(value.toRawBits().toUInt())
    }

    fun toByteArray(): ByteArray = ByteArray(bytes.size) { index -> bytes[index] }
}

private fun ByteArray.readU16(offset: Int): Int {
    return readU32(offset).toInt() and 0xFFFF
}

private fun ByteArray.readU32(offset: Int): UInt {
    val b0 = this[offset].toUByte().toUInt()
    val b1 = this[offset + 1].toUByte().toUInt() shl 8
    val b2 = this[offset + 2].toUByte().toUInt() shl 16
    val b3 = this[offset + 3].toUByte().toUInt() shl 24
    return b0 or b1 or b2 or b3
}

private fun ByteArray.readF32(offset: Int): Float {
    return Float.fromBits(readU32(offset).toInt())
}
