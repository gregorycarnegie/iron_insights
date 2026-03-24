package com.gregorycarnegie.ironinsights.domain.training

import kotlin.math.roundToInt

object DeloadCalculator {
    data class DeloadPrescription(val weightKg: Float, val sets: Int, val reps: Int)

    fun deloadWeek(
        blockType: String,
        normalWeightKg: Float,
        normalSets: Int,
        normalReps: Int,
    ): DeloadPrescription {
        return when (blockType) {
            "hypertrophy" -> DeloadPrescription(
                weightKg = normalWeightKg * 0.6f,
                sets = (normalSets * 0.5f).roundToInt().coerceAtLeast(1),
                reps = normalReps,
            )
            "strength" -> DeloadPrescription(
                weightKg = normalWeightKg * 0.7f,
                sets = (normalSets * 0.6f).roundToInt().coerceAtLeast(1),
                reps = (normalReps * 0.6f).roundToInt().coerceAtLeast(1),
            )
            "peak" -> DeloadPrescription(
                weightKg = normalWeightKg * 0.5f,
                sets = 2,
                reps = 3,
            )
            else -> DeloadPrescription(
                weightKg = normalWeightKg * 0.5f,
                sets = 2,
                reps = normalReps,
            )
        }
    }
}
