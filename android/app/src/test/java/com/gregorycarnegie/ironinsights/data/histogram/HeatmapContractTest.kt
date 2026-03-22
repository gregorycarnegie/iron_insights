package com.gregorycarnegie.ironinsights.data.histogram

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import kotlin.math.floor
import kotlin.math.roundToInt

class HeatmapContractTest {
    @Test
    fun parseHeatBin_accepts_valid_payload() {
        val bytes = buildHeatmapBytes(
            baseX = 5.0f,
            baseY = 2.0f,
            minX = 100.0f,
            maxX = 115.0f,
            minY = 60.0f,
            maxY = 66.0f,
            width = 3,
            height = 3,
            counts = intArrayOf(
                1, 2, 1,
                2, 6, 2,
                1, 2, 1,
            ),
        )

        val heat = parseHeatBin(bytes)

        assertEquals(5.0f, heat?.baseX ?: error("expected parsed heatmap"), 0.0f)
        assertEquals(2.0f, heat?.baseY ?: error("expected parsed heatmap"), 0.0f)
        assertEquals(100.0f, heat?.minX ?: error("expected parsed heatmap"), 0.0f)
        assertEquals(115.0f, heat?.maxX ?: error("expected parsed heatmap"), 0.0f)
        assertEquals(60.0f, heat?.minY ?: error("expected parsed heatmap"), 0.0f)
        assertEquals(66.0f, heat?.maxY ?: error("expected parsed heatmap"), 0.0f)
        assertEquals(3, heat?.width)
        assertEquals(3, heat?.height)
        assertEquals(listOf(1L, 2L, 1L, 2L, 6L, 2L, 1L, 2L, 1L), heat?.grid)
    }

    @Test
    fun parseHeatBin_rejects_invalid_payload_len() {
        val bytes = buildHeatmapHeader(
            baseX = 5.0f,
            baseY = 2.0f,
            minX = 100.0f,
            maxX = 110.0f,
            minY = 50.0f,
            maxY = 54.0f,
            width = 2,
            height = 2,
        ).apply {
            writeU32(1u)
        }.toByteArray()

        assertNull(parseHeatBin(bytes))
    }

    @Test
    fun parseHeatBin_rejects_unsupported_version() {
        val bytes = buildHeatmapHeader(
            baseX = 5.0f,
            baseY = 2.0f,
            minX = 100.0f,
            maxX = 110.0f,
            minY = 50.0f,
            maxY = 54.0f,
            width = 1,
            height = 1,
            version = 2,
        ).apply {
            writeU32(1u)
        }.toByteArray()

        assertNull(parseHeatBin(bytes))
    }

    @Test
    fun bodyweightConditionedPercentile_uses_nearby_rows() {
        val heat = HeatmapBin(
            minX = 100.0f,
            maxX = 115.0f,
            minY = 60.0f,
            maxY = 66.0f,
            baseX = 5.0f,
            baseY = 2.0f,
            width = 3,
            height = 3,
            grid = listOf(
                1L, 2L, 1L,
                2L, 6L, 2L,
                1L, 2L, 1L,
            ),
        )

        val stats = bodyweightConditionedPercentile(heat, userLift = 106.0f, userBw = 62.5f)

        assertEquals(0.5f, stats?.percentile ?: error("expected stats"), 0.0f)
        assertEquals(9, stats?.rank)
        assertEquals(18L, stats?.totalNearby)
        assertEquals(1, stats?.bwBinIndex)
        assertEquals(1, stats?.liftBinIndex)
        assertEquals(6L, stats?.localCellCount)
        assertEquals(18L, stats?.neighborhoodCount)
        assertEquals(1.0f, stats?.neighborhoodShare ?: error("expected stats"), 0.0f)
    }

    @Test
    fun bodyweightConditionedPercentile_clamps_edges() {
        val heat = HeatmapBin(
            minX = 100.0f,
            maxX = 110.0f,
            minY = 50.0f,
            maxY = 54.0f,
            baseX = 5.0f,
            baseY = 2.0f,
            width = 2,
            height = 2,
            grid = listOf(
                10L, 0L,
                0L, 0L,
            ),
        )

        val stats = bodyweightConditionedPercentile(heat, userLift = 95.0f, userBw = 40.0f)

        assertEquals(0, stats?.bwBinIndex)
        assertEquals(0, stats?.liftBinIndex)
        assertEquals(10L, stats?.totalNearby)
        assertEquals(5, stats?.rank)
        assertEquals(10L, stats?.localCellCount)
        assertEquals(10L, stats?.neighborhoodCount)
        assertEquals(1.0f, stats?.neighborhoodShare ?: error("expected stats"), 0.0f)
    }

    @Test
    fun bodyweightConditionedPercentile_returns_null_for_invalid_heatmap() {
        assertNull(bodyweightConditionedPercentile(null, 100.0f, 60.0f))
        assertNull(
            bodyweightConditionedPercentile(
                HeatmapBin(
                    minX = 0.0f,
                    maxX = 0.0f,
                    minY = 0.0f,
                    maxY = 0.0f,
                    baseX = 0.0f,
                    baseY = 1.0f,
                    width = 0,
                    height = 0,
                    grid = emptyList(),
                ),
                100.0f,
                60.0f,
            ),
        )
    }
}

private const val HEATMAP_BINARY_FORMAT_VERSION = 1
private const val HEATMAP_MAGIC = "IIM1"

private data class HeatmapBin(
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

private data class BodyweightConditionedStats(
    val percentile: Float,
    val rank: Int,
    val totalNearby: Long,
    val bwBinIndex: Int,
    val liftBinIndex: Int,
    val localCellCount: Long,
    val neighborhoodCount: Long,
    val neighborhoodShare: Float,
)

private fun parseHeatBin(bytes: ByteArray): HeatmapBin? {
    if (bytes.size < 38 || !bytes.copyOfRange(0, 4).contentEquals(HEATMAP_MAGIC.toByteArray())) {
        return null
    }

    val version = bytes.readU16(4)
    if (version != HEATMAP_BINARY_FORMAT_VERSION) {
        return null
    }

    val baseX = bytes.readF32(6)
    val baseY = bytes.readF32(10)
    val minX = bytes.readF32(14)
    val maxX = bytes.readF32(18)
    val minY = bytes.readF32(22)
    val maxY = bytes.readF32(26)
    val width = bytes.readU32(30).toInt()
    val height = bytes.readU32(34).toInt()

    val payload = bytes.copyOfRange(38, bytes.size)
    if (payload.size != width * height * 4) {
        return null
    }

    val grid = buildList(width * height) {
        for (index in 0 until width * height) {
            add(payload.readU32(index * 4).toLong())
        }
    }

    return HeatmapBin(
        minX = minX,
        maxX = maxX,
        minY = minY,
        maxY = maxY,
        baseX = baseX,
        baseY = baseY,
        width = width,
        height = height,
        grid = grid,
    )
}

private fun bodyweightConditionedPercentile(
    heat: HeatmapBin?,
    userLift: Float,
    userBw: Float,
): BodyweightConditionedStats? {
    val heatmap = heat ?: return null
    if (heatmap.width == 0 || heatmap.height == 0 || heatmap.grid.size != heatmap.width * heatmap.height) {
        return null
    }
    if (heatmap.baseX <= 0.0f || heatmap.baseY <= 0.0f) {
        return null
    }

    val totalHeat = heatmap.grid.sum()
    if (totalHeat == 0L) {
        return null
    }

    val liftBinIndex = floor((userLift - heatmap.minX) / heatmap.baseX)
        .coerceIn(0.0f, (heatmap.width - 1).toFloat())
        .toInt()
    val bwBinIndex = floor((userBw - heatmap.minY) / heatmap.baseY)
        .coerceIn(0.0f, (heatmap.height - 1).toFloat())
        .toInt()

    val rowLo = (bwBinIndex - 1).coerceAtLeast(0)
    val rowHi = (bwBinIndex + 1).coerceAtMost(heatmap.height - 1)

    val nearbyCounts = LongArray(heatmap.width)
    for (row in rowLo..rowHi) {
        for (x in 0 until heatmap.width) {
            nearbyCounts[x] += heatmap.grid[row * heatmap.width + x]
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
    val rank = ((1.0f - percentile) * totalNearby.toFloat()).roundToInt().coerceAtLeast(1)

    val xLo = (liftBinIndex - 1).coerceAtLeast(0)
    val xHi = (liftBinIndex + 1).coerceAtMost(heatmap.width - 1)
    var neighborhoodCount = 0L
    for (row in rowLo..rowHi) {
        for (x in xLo..xHi) {
            neighborhoodCount += heatmap.grid[row * heatmap.width + x]
        }
    }

    return BodyweightConditionedStats(
        percentile = percentile,
        rank = rank,
        totalNearby = totalNearby,
        bwBinIndex = bwBinIndex,
        liftBinIndex = liftBinIndex,
        localCellCount = heatmap.grid[bwBinIndex * heatmap.width + liftBinIndex],
        neighborhoodCount = neighborhoodCount,
        neighborhoodShare = neighborhoodCount.toFloat() / heat.totalHeat().toFloat(),
    )
}

private fun HeatmapBin.totalHeat(): Long = grid.sum()

private fun buildHeatmapBytes(
    baseX: Float,
    baseY: Float,
    minX: Float,
    maxX: Float,
    minY: Float,
    maxY: Float,
    width: Int,
    height: Int,
    counts: IntArray,
): ByteArray {
    return buildHeatmapHeader(
        baseX = baseX,
        baseY = baseY,
        minX = minX,
        maxX = maxX,
        minY = minY,
        maxY = maxY,
        width = width,
        height = height,
    ).apply {
        counts.forEach { writeU32(it.toUInt()) }
    }.toByteArray()
}

private fun buildHeatmapHeader(
    baseX: Float,
    baseY: Float,
    minX: Float,
    maxX: Float,
    minY: Float,
    maxY: Float,
    width: Int,
    height: Int,
    version: Int = HEATMAP_BINARY_FORMAT_VERSION,
): HeatmapByteWriter = HeatmapByteWriter().apply {
    writeBytes(HEATMAP_MAGIC.toByteArray())
    writeU16(version.toUInt())
    writeF32(baseX)
    writeF32(baseY)
    writeF32(minX)
    writeF32(maxX)
    writeF32(minY)
    writeF32(maxY)
    writeU32(width.toUInt())
    writeU32(height.toUInt())
}

private class HeatmapByteWriter {
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
