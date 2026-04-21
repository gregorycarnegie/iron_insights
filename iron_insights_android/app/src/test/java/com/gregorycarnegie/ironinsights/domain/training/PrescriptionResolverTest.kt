package com.gregorycarnegie.ironinsights.domain.training

import com.gregorycarnegie.ironinsights.data.db.entity.PlannedSet
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class PrescriptionResolverTest {

    @Test
    fun `percent_1rm resolves weight from e1rm and percentage`() {
        val set = PlannedSet(
            plannedExerciseId = 1,
            setIndex = 0,
            prescriptionType = "percent_1rm",
            targetPercent1rm = 0.8f,
            targetReps = 5,
        )
        val result = PrescriptionResolver.resolve(set, currentE1rmKg = 100f)
        assertEquals(80f, result.weightKg, 0.01f)
        assertEquals(5, result.reps)
    }

    @Test
    fun `percent_1rm with null e1rm produces zero weight`() {
        val set = PlannedSet(
            plannedExerciseId = 1,
            setIndex = 0,
            prescriptionType = "percent_1rm",
            targetPercent1rm = 0.85f,
            targetReps = 3,
        )
        val result = PrescriptionResolver.resolve(set, currentE1rmKg = null)
        assertEquals(0f, result.weightKg, 0.01f)
    }

    @Test
    fun `rpe resolves weight using OneRepMaxCalculator`() {
        val set = PlannedSet(
            plannedExerciseId = 1,
            setIndex = 0,
            prescriptionType = "rpe",
            targetRpe = 8f,
            targetReps = 5,
        )
        val result = PrescriptionResolver.resolve(set, currentE1rmKg = 100f)
        // workingWeightForReps(100, 5) should give a reasonable working weight
        assert(result.weightKg > 0f) { "RPE-resolved weight should be positive" }
        assertEquals(5, result.reps)
        assertEquals(8f, result.targetRpe)
    }

    @Test
    fun `rpe with null e1rm produces zero weight`() {
        val set = PlannedSet(
            plannedExerciseId = 1,
            setIndex = 0,
            prescriptionType = "rpe",
            targetRpe = 7f,
            targetReps = 8,
        )
        val result = PrescriptionResolver.resolve(set, currentE1rmKg = null)
        assertEquals(0f, result.weightKg, 0.01f)
    }

    @Test
    fun `fixed_weight returns zero weight and preserves reps`() {
        val set = PlannedSet(
            plannedExerciseId = 1,
            setIndex = 0,
            prescriptionType = "fixed_weight",
            targetReps = 12,
        )
        val result = PrescriptionResolver.resolve(set, currentE1rmKg = 100f)
        assertEquals(0f, result.weightKg, 0.01f)
        assertEquals(12, result.reps)
    }

    @Test
    fun `unknown prescription type returns zero weight`() {
        val set = PlannedSet(
            plannedExerciseId = 1,
            setIndex = 0,
            prescriptionType = "mystery",
            targetReps = 10,
        )
        val result = PrescriptionResolver.resolve(set, currentE1rmKg = 100f)
        assertEquals(0f, result.weightKg, 0.01f)
        assertEquals(10, result.reps)
        assertNull(result.targetRpe)
    }
}
