package com.gregorycarnegie.ironinsights.domain.training

import org.junit.Assert.assertEquals
import org.junit.Test

class VolumeCalculatorTest {

    @Test
    fun emptySetList_returnsZeros() {
        val result = VolumeCalculator.compute(emptyList())
        assertEquals(0, result.totalSets)
        assertEquals(0, result.totalReps)
        assertEquals(0f, result.totalVolumeLoad, 0f)
        assertEquals(0f, result.averageIntensity, 0f)
    }

    @Test
    fun warmupSets_areExcluded() {
        val sets = listOf(
            VolumeCalculator.SetData(weightKg = 60f, reps = 10, isWarmup = true),
            VolumeCalculator.SetData(weightKg = 80f, reps = 5, isWarmup = true),
            VolumeCalculator.SetData(weightKg = 100f, reps = 5, isWarmup = false),
        )
        val result = VolumeCalculator.compute(sets)
        assertEquals(1, result.totalSets)
        assertEquals(5, result.totalReps)
        assertEquals(500f, result.totalVolumeLoad, 0.01f)
        assertEquals(100f, result.averageIntensity, 0.01f)
    }

    @Test
    fun basicVolumeComputation() {
        val sets = listOf(
            VolumeCalculator.SetData(weightKg = 100f, reps = 5, isWarmup = false),
            VolumeCalculator.SetData(weightKg = 100f, reps = 5, isWarmup = false),
            VolumeCalculator.SetData(weightKg = 100f, reps = 5, isWarmup = false),
        )
        val result = VolumeCalculator.compute(sets)
        assertEquals(3, result.totalSets)
        assertEquals(15, result.totalReps)
        assertEquals(1500f, result.totalVolumeLoad, 0.01f)
    }

    @Test
    fun averageIntensityCalculation() {
        val sets = listOf(
            VolumeCalculator.SetData(weightKg = 80f, reps = 8, isWarmup = false),
            VolumeCalculator.SetData(weightKg = 90f, reps = 6, isWarmup = false),
            VolumeCalculator.SetData(weightKg = 100f, reps = 4, isWarmup = false),
        )
        val result = VolumeCalculator.compute(sets)
        assertEquals(3, result.totalSets)
        assertEquals(18, result.totalReps)
        assertEquals(90f, result.averageIntensity, 0.01f)
        // 80*8 + 90*6 + 100*4 = 640 + 540 + 400 = 1580
        assertEquals(1580f, result.totalVolumeLoad, 0.01f)
    }
}
