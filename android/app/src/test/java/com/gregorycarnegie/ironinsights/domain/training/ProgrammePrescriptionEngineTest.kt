package com.gregorycarnegie.ironinsights.domain.training

import com.gregorycarnegie.ironinsights.data.db.entity.ProgrammeBlock
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class ProgrammePrescriptionEngineTest {

    @Test
    fun `traditional sequence expands into all weeks with four sessions per week`() {
        val blocks = listOf(
            block(id = 1, order = 0, type = "hypertrophy", weeks = 4),
            block(id = 2, order = 1, type = "strength", weeks = 4),
            block(id = 3, order = 2, type = "peak", weeks = 2),
            block(id = 4, order = 3, type = "taper", weeks = 2),
        )

        val plan = ProgrammePrescriptionEngine.build(
            blocks = blocks,
            baselines = standardBaselines(),
            roundingIncrementKg = 2.5f,
        )

        assertEquals(12, plan.weeks.size)
        assertTrue(plan.weeks.all { it.sessions.size == 4 })
        assertEquals(1, plan.weeks.first().weekNumber)
        assertEquals(12, plan.weeks.last().weekNumber)
    }

    @Test
    fun `work sets round to configured increment`() {
        val plan = ProgrammePrescriptionEngine.build(
            blocks = listOf(block(id = 1, order = 0, type = "hypertrophy", weeks = 1)),
            baselines = standardBaselines(),
            roundingIncrementKg = 2.5f,
        )

        val firstSquatWorkSets = plan.weeks
            .first()
            .sessions
            .first()
            .exercises
            .first()
            .lines
            .last()

        assertEquals(142.5f, firstSquatWorkSets.targetWeightKg ?: 0f, 0.01f)
    }

    @Test
    fun `missing lift baseline falls back to percentage only`() {
        val plan = ProgrammePrescriptionEngine.build(
            blocks = listOf(block(id = 1, order = 0, type = "strength", weeks = 1)),
            baselines = listOf(
                ProgrammeLiftBaseline("S", "Squat", null),
                ProgrammeLiftBaseline("B", "Bench", 150f),
                ProgrammeLiftBaseline("D", "Deadlift", 220f),
            ),
            roundingIncrementKg = 2.5f,
        )

        val squatMainWork = plan.weeks
            .first()
            .sessions
            .first()
            .exercises
            .first()
            .lines
            .last()

        assertNull(squatMainWork.targetWeightKg)
        assertNotNull(squatMainWork.intensityPercent)
        assertTrue(plan.missingBaselines.contains("Squat"))
    }

    @Test
    fun `peak weeks include top singles before work sets`() {
        val plan = ProgrammePrescriptionEngine.build(
            blocks = listOf(block(id = 1, order = 0, type = "peak", weeks = 2)),
            baselines = standardBaselines(),
            roundingIncrementKg = 2.5f,
        )

        val squatLines = plan.weeks
            .first()
            .sessions
            .first()
            .exercises
            .first()
            .lines

        assertEquals("Top single", squatLines.first().label)
        assertEquals(1, squatLines.first().reps)
        assertTrue((squatLines.first().targetWeightKg ?: 0f) > (squatLines.last().targetWeightKg ?: 0f))
    }

    private fun standardBaselines(): List<ProgrammeLiftBaseline> {
        return listOf(
            ProgrammeLiftBaseline("S", "Squat", 203f),
            ProgrammeLiftBaseline("B", "Bench", 143f),
            ProgrammeLiftBaseline("D", "Deadlift", 251f),
        )
    }

    private fun block(
        id: Long,
        order: Int,
        type: String,
        weeks: Int,
    ): ProgrammeBlock {
        return ProgrammeBlock(
            id = id,
            programmeId = 1,
            name = type.replaceFirstChar { it.uppercase() },
            blockType = type,
            orderIndex = order,
            weekCount = weeks,
        )
    }
}
