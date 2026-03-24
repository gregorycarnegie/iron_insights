package com.gregorycarnegie.ironinsights.domain.training

import com.gregorycarnegie.ironinsights.data.db.entity.PlannedSet
import com.gregorycarnegie.ironinsights.domain.calculators.OneRepMaxCalculator

object PrescriptionResolver {
    data class ResolvedSet(val weightKg: Float, val reps: Int, val targetRpe: Float?)

    fun resolve(plannedSet: PlannedSet, currentE1rmKg: Float?): ResolvedSet {
        return when (plannedSet.prescriptionType) {
            "percent_1rm" -> {
                val e1rm = currentE1rmKg ?: 0f
                val percent = plannedSet.targetPercent1rm ?: 0f
                ResolvedSet(
                    weightKg = e1rm * percent,
                    reps = plannedSet.targetReps,
                    targetRpe = plannedSet.targetRpe,
                )
            }
            "rpe" -> {
                val e1rm = currentE1rmKg ?: 0f
                val weight = if (e1rm > 0f) {
                    OneRepMaxCalculator.workingWeightForReps(e1rm, plannedSet.targetReps)
                } else {
                    0f
                }
                ResolvedSet(
                    weightKg = weight,
                    reps = plannedSet.targetReps,
                    targetRpe = plannedSet.targetRpe,
                )
            }
            "fixed_weight" -> {
                ResolvedSet(
                    weightKg = 0f,
                    reps = plannedSet.targetReps,
                    targetRpe = plannedSet.targetRpe,
                )
            }
            else -> ResolvedSet(
                weightKg = 0f,
                reps = plannedSet.targetReps,
                targetRpe = plannedSet.targetRpe,
            )
        }
    }
}
