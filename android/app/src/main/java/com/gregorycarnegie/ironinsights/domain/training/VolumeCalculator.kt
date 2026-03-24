package com.gregorycarnegie.ironinsights.domain.training

object VolumeCalculator {
    data class SessionVolume(
        val totalSets: Int,
        val totalReps: Int,
        val totalVolumeLoad: Float,   // sum of weight * reps for each set
        val averageIntensity: Float,  // average weight across working sets
    )

    fun compute(sets: List<SetData>): SessionVolume {
        val workingSets = sets.filter { !it.isWarmup }
        if (workingSets.isEmpty()) return SessionVolume(0, 0, 0f, 0f)

        val totalSets = workingSets.size
        val totalReps = workingSets.sumOf { it.reps }
        val totalVolumeLoad = workingSets.sumOf { (it.weightKg * it.reps).toDouble() }.toFloat()
        val averageIntensity = workingSets.map { it.weightKg }.average().toFloat()

        return SessionVolume(totalSets, totalReps, totalVolumeLoad, averageIntensity)
    }

    data class SetData(
        val weightKg: Float,
        val reps: Int,
        val isWarmup: Boolean,
    )
}
