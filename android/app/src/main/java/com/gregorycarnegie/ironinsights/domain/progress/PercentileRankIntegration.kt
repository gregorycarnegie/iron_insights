package com.gregorycarnegie.ironinsights.domain.progress

object PercentileRankIntegration {
    data class PercentileResult(
        val percentile: Float,  // 0-100
        val rank: Long,
        val total: Long,
        val label: String,      // e.g., "82nd percentile (rank 234 of 1,300)"
    )

    fun formatPercentile(percentile: Float, rank: Long, total: Long): PercentileResult {
        val pct = (percentile * 100).let { if (it > 99.9f) 99.9f else it }
        val suffix = ordinalSuffix(pct.toInt())
        val label = "${pct.toInt()}$suffix percentile (rank ${rank.formatWithCommas()} of ${total.formatWithCommas()})"
        return PercentileResult(pct, rank, total, label)
    }

    private fun ordinalSuffix(n: Int): String = when {
        n % 100 in 11..13 -> "th"
        n % 10 == 1 -> "st"
        n % 10 == 2 -> "nd"
        n % 10 == 3 -> "rd"
        else -> "th"
    }

    private fun Long.formatWithCommas(): String = String.format("%,d", this)
}
