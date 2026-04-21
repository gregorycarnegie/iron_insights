package com.gregorycarnegie.ironinsights.domain.calculators

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class CalculatorLogicTest {
    @Test
    fun blended1rm_returns_entered_load_for_single() {
        assertEquals(180f, OneRepMaxCalculator.epley1rm(180f, 1), 0f)
        assertEquals(180f, OneRepMaxCalculator.brzycki1rm(180f, 1), 0f)
        assertEquals(180f, OneRepMaxCalculator.blended1rm(180f, 1), 0f)
        assertEquals(100f, OneRepMaxCalculator.setIntensityPercent(180f, 1), 0f)
    }

    @Test
    fun blended1rm_uses_correct_boundary_behavior() {
        val load = 100f

        assertEquals(
            OneRepMaxCalculator.brzycki1rm(load, 8),
            OneRepMaxCalculator.blended1rm(load, 8),
            0.0001f,
        )
        assertEquals(
            OneRepMaxCalculator.epley1rm(load, 10),
            OneRepMaxCalculator.blended1rm(load, 10),
            0.0001f,
        )

        val brzyckiAtNine = OneRepMaxCalculator.brzycki1rm(load, 9)
        val epleyAtNine = OneRepMaxCalculator.epley1rm(load, 9)
        val blendedAtNine = OneRepMaxCalculator.blended1rm(load, 9)
        assertTrue(blendedAtNine > minOf(brzyckiAtNine, epleyAtNine))
        assertTrue(blendedAtNine < maxOf(brzyckiAtNine, epleyAtNine))
    }

    @Test
    fun formula_range_orders_low_and_high() {
        val epley = OneRepMaxCalculator.epley1rm(140f, 12)
        val brzycki = OneRepMaxCalculator.brzycki1rm(140f, 12)
        val range = OneRepMaxCalculator.formulaRange(loadKg = 140f, reps = 12)

        assertEquals(minOf(epley, brzycki), range.lowerKg, 0.0001f)
        assertEquals(maxOf(epley, brzycki), range.upperKg, 0.0001f)
    }

    @Test
    fun working_weight_for_reps_matches_estimate_inverse_shape() {
        val oneRm = 200f
        val weightForThree = OneRepMaxCalculator.workingWeightForReps(oneRm, 3)
        val weightForTen = OneRepMaxCalculator.workingWeightForReps(oneRm, 10)

        assertEquals(200f, OneRepMaxCalculator.workingWeightForReps(oneRm, 1), 0f)
        assertTrue(weightForTen < oneRm)
        assertTrue(weightForThree < oneRm)
        assertTrue(weightForThree > weightForTen)
    }

    @Test
    fun plate_calculator_returns_exact_standard_load() {
        val result = PlateCalculator.calculate(targetKg = 100f, barKg = 20f)

        assertNull(result.warning)
        assertEquals(100f, result.actualKg, 0.0001f)
        assertEquals(0f, result.remainderKg, 0.0001f)
        assertEquals(2, result.plates.size)
        assertEquals(25f, result.plates[0].plate.weightKg, 0f)
        assertEquals(15f, result.plates[1].plate.weightKg, 0f)
        assertEquals(1, result.plates[0].countPerSide)
        assertEquals(1, result.plates[1].countPerSide)
    }

    @Test
    fun plate_calculator_warns_when_target_is_below_bar() {
        val result = PlateCalculator.calculate(targetKg = 15f, barKg = 20f)

        assertTrue(result.plates.isEmpty())
        assertEquals(20f, result.actualKg, 0.0001f)
        assertEquals(-5f, result.remainderKg, 0.0001f)
        assertNotNull(result.warning)
    }

    @Test
    fun plate_calculator_warns_on_non_exact_targets() {
        val result = PlateCalculator.calculate(targetKg = 101f, barKg = 20f)

        assertEquals(100f, result.actualKg, 0.0001f)
        assertEquals(1f, result.remainderKg, 0.0001f)
        assertNotNull(result.warning)
    }

    @Test
    fun unit_conversion_round_trips_cleanly() {
        val pounds = kgToDisplay(100f, WeightUnit.LB)
        val backToKg = displayToKg(pounds, WeightUnit.LB)

        assertEquals(220.46225f, pounds, 0.0001f)
        assertEquals(100f, backToKg, 0.0001f)
    }
}
