package com.gregorycarnegie.ironinsights.domain.calculators

import java.util.Locale
import kotlin.math.roundToInt

data class PlateSpec(
    val weightKg: Float,
    val name: String,
    val colorHex: String,
)

data class PlateLoad(
    val plate: PlateSpec,
    val countPerSide: Int,
)

data class PlateCalculation(
    val plates: List<PlateLoad>,
    val actualKg: Float,
    val remainderKg: Float,
    val warning: String?,
)

object PlateCalculator {
    val standardPlates: List<PlateSpec> = listOf(
        PlateSpec(25f, "Red 25", "#e63946"),
        PlateSpec(20f, "Blue 20", "#457bca"),
        PlateSpec(15f, "Yellow 15", "#ffd166"),
        PlateSpec(10f, "Green 10", "#5cb85c"),
        PlateSpec(5f, "Black 5", "#444444"),
        PlateSpec(2.5f, "Black 2.5", "#383838"),
        PlateSpec(1.25f, "Black 1.25", "#2a2a2a"),
    )

    fun calculate(
        targetKg: Float,
        barKg: Float,
        plates: List<PlateSpec> = standardPlates,
    ): PlateCalculation {
        val remaining = targetKg - barKg
        if (remaining < 0f) {
            return PlateCalculation(
                plates = emptyList(),
                actualKg = barKg,
                remainderKg = remaining,
                warning = String.format(Locale.US, "Target is less than bar weight (%.1fkg).", barKg),
            )
        }

        val perSide = remaining / 2f
        var leftover = perSide
        val loads = mutableListOf<PlateLoad>()

        for (plate in plates) {
            if (leftover <= 0f) {
                break
            }
            val count = kotlin.math.floor(leftover / plate.weightKg).toInt()
            if (count > 0) {
                loads += PlateLoad(plate = plate, countPerSide = count)
                leftover = roundToThousandth(leftover - count.toFloat() * plate.weightKg)
            }
        }

        val loadedKg = loads.sumOf { (it.countPerSide * 2f * it.plate.weightKg).toDouble() }.toFloat()
        val actualKg = barKg + loadedKg
        val remainderKg = roundToThousandth(targetKg - actualKg)
        val warning = if (remainderKg > 0.001f) {
            String.format(
                Locale.US,
                "Cannot hit %.1fkg exactly with standard plates. Loaded: %.1fkg. Short by %.3fkg.",
                targetKg,
                actualKg,
                remainderKg,
            )
        } else {
            null
        }

        return PlateCalculation(
            plates = loads,
            actualKg = actualKg,
            remainderKg = remainderKg,
            warning = warning,
        )
    }
}

private fun roundToThousandth(value: Float): Float {
    return (value * 1000f).roundToInt() / 1000f
}
