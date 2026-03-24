package com.gregorycarnegie.ironinsights.domain.training

import org.junit.Assert.assertEquals
import org.junit.Test

class DeloadCalculatorTest {

    @Test
    fun `hypertrophy deload uses 60 percent weight, 50 percent sets, same reps`() {
        val result = DeloadCalculator.deloadWeek("hypertrophy", 100f, 4, 10)
        assertEquals(60f, result.weightKg, 0.01f)
        assertEquals(2, result.sets)
        assertEquals(10, result.reps)
    }

    @Test
    fun `strength deload uses 70 percent weight, 60 percent sets, 60 percent reps`() {
        val result = DeloadCalculator.deloadWeek("strength", 100f, 5, 5)
        assertEquals(70f, result.weightKg, 0.01f)
        assertEquals(3, result.sets)
        assertEquals(3, result.reps)
    }

    @Test
    fun `peak deload uses 50 percent weight, 2 sets, 3 reps`() {
        val result = DeloadCalculator.deloadWeek("peak", 140f, 6, 2)
        assertEquals(70f, result.weightKg, 0.01f)
        assertEquals(2, result.sets)
        assertEquals(3, result.reps)
    }

    @Test
    fun `default deload uses 50 percent weight, 2 sets, same reps`() {
        val result = DeloadCalculator.deloadWeek("unknown", 100f, 4, 8)
        assertEquals(50f, result.weightKg, 0.01f)
        assertEquals(2, result.sets)
        assertEquals(8, result.reps)
    }

    @Test
    fun `sets are at least 1 after rounding`() {
        val result = DeloadCalculator.deloadWeek("hypertrophy", 100f, 1, 10)
        assertEquals(1, result.sets)
    }

    @Test
    fun `strength reps are at least 1 after rounding`() {
        val result = DeloadCalculator.deloadWeek("strength", 100f, 2, 1)
        assertEquals(1, result.reps)
    }
}
