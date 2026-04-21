package com.gregorycarnegie.ironinsights.domain.calculators

import kotlin.math.max
import kotlin.math.min

data class RepMaxTarget(
    val reps: Int,
    val label: String,
)

data class TrainingPercentage(
    val percent: Int,
    val label: String,
)

data class FormulaRange(
    val lowerKg: Float,
    val upperKg: Float,
)

object OneRepMaxCalculator {
    const val MIN_REPS: Int = 1
    const val MAX_REPS: Int = 20

    val quickRepPresets: List<Int> = listOf(3, 5, 8, 10)

    val repMaxTargets: List<RepMaxTarget> = listOf(
        RepMaxTarget(1, "max single"),
        RepMaxTarget(2, "heavy double"),
        RepMaxTarget(3, "hard triple"),
        RepMaxTarget(4, "heavy four"),
        RepMaxTarget(5, "classic 5RM"),
        RepMaxTarget(8, "solid volume"),
        RepMaxTarget(10, "high-rep push"),
    )

    val trainingPercentages: List<TrainingPercentage> = listOf(
        TrainingPercentage(60, "light technique"),
        TrainingPercentage(70, "steady volume"),
        TrainingPercentage(75, "comfortable work"),
        TrainingPercentage(80, "hard work sets"),
        TrainingPercentage(85, "top triples"),
        TrainingPercentage(90, "heavy single or double"),
        TrainingPercentage(95, "near-max single"),
    )

    fun epley1rm(
        loadKg: Float,
        reps: Int,
    ): Float {
        return if (reps <= 1) {
            loadKg
        } else {
            loadKg * (1f + reps.toFloat() / 30f)
        }
    }

    fun brzycki1rm(
        loadKg: Float,
        reps: Int,
    ): Float {
        return when {
            reps <= 1 -> loadKg
            reps >= 37 -> 0f
            else -> loadKg * (36f / (37f - reps.toFloat()))
        }
    }

    fun blendFactor(reps: Int): Float {
        return ((reps.toFloat() - 8f) / 2f).coerceIn(0f, 1f)
    }

    fun blended1rm(
        loadKg: Float,
        reps: Int,
    ): Float {
        if (reps <= 1) {
            return loadKg
        }

        val brzycki = brzycki1rm(loadKg, reps)
        val epley = epley1rm(loadKg, reps)
        return when {
            reps < 8 -> brzycki
            reps > 10 -> epley
            else -> brzycki + (epley - brzycki) * blendFactor(reps)
        }
    }

    fun workingWeightForReps(
        oneRmKg: Float,
        reps: Int,
    ): Float {
        if (reps <= 1) {
            return oneRmKg
        }

        val brzycki = oneRmKg * ((37f - reps.toFloat()) / 36f).coerceAtLeast(0f)
        val epley = oneRmKg / (1f + reps.toFloat() / 30f)
        return when {
            reps < 8 -> brzycki
            reps > 10 -> epley
            else -> brzycki + (epley - brzycki) * blendFactor(reps)
        }
    }

    fun formulaRange(
        loadKg: Float,
        reps: Int,
    ): FormulaRange {
        val epley = epley1rm(loadKg, reps)
        val brzycki = brzycki1rm(loadKg, reps)
        return FormulaRange(
            lowerKg = min(epley, brzycki),
            upperKg = max(epley, brzycki),
        )
    }

    fun setIntensityPercent(
        loadKg: Float,
        reps: Int,
    ): Float {
        val estimate = blended1rm(loadKg, reps)
        return if (estimate > 0f) {
            (loadKg / estimate) * 100f
        } else {
            0f
        }
    }

    fun estimateQualityBadge(reps: Int): String {
        return when (reps) {
            1 -> "Actual single entered"
            in 2..5 -> "Strong estimate"
            in 6..10 -> "Good estimate"
            in 11..15 -> "Rougher estimate"
            else -> "Ballpark only"
        }
    }

    fun estimateGuidance(reps: Int): String {
        return when (reps) {
            1 -> "A true single is already your best day number, so the tables below use that single as the base."
            in 2..5 -> "Heavy doubles to fives usually give the cleanest 1RM estimate without needing a true max test."
            in 6..10 -> "This is still useful for planning, but expect a bit more spread between formulas as reps climb."
            in 11..15 -> "Use this as a planning number, not a promise. Higher-rep sets are less precise for predicting a max."
            else -> "Very high-rep sets can drift a lot. Treat this as a rough training estimate rather than a meet-day forecast."
        }
    }
}
