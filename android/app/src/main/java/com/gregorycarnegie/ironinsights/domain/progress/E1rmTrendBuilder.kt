package com.gregorycarnegie.ironinsights.domain.progress

object E1rmTrendBuilder {
    data class E1rmPoint(
        val epochMs: Long,
        val e1rmKg: Float,
        val weightKg: Float,
        val reps: Int,
    )

    data class E1rmTrend(
        val exerciseId: Long,
        val exerciseName: String,
        val canonicalLift: String?,
        val points: List<E1rmPoint>,
        val currentBestKg: Float?,
        val allTimeBestKg: Float?,
    )

    fun buildTrend(
        exerciseId: Long,
        exerciseName: String,
        canonicalLift: String?,
        sets: List<SetData>,
    ): E1rmTrend {
        val points = sets
            .filter { it.e1rmKg != null }
            .map { E1rmPoint(it.completedAtEpochMs, it.e1rmKg!!, it.weightKg, it.reps) }
            .sortedBy { it.epochMs }
        val allTimeBest = points.maxByOrNull { it.e1rmKg }?.e1rmKg
        val currentBest = points.lastOrNull()?.e1rmKg
        return E1rmTrend(exerciseId, exerciseName, canonicalLift, points, currentBest, allTimeBest)
    }

    data class SetData(
        val completedAtEpochMs: Long,
        val e1rmKg: Float?,
        val weightKg: Float,
        val reps: Int,
    )
}
