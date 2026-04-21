package com.gregorycarnegie.ironinsights.domain.training

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Test

class ProgrammedWorkoutCodecTest {

    @Test
    fun `session metadata survives encode decode`() {
        val metadata = ProgrammedWorkoutMetadata(
            programmeId = 12,
            programmeName = "Traditional Meet Prep",
            weekNumber = 3,
            sessionIndex = 9,
            dayLabel = "Day 2",
            sessionTitle = "Deadlift Volume + Bench Technique",
        )

        val decoded = ProgrammedWorkoutCodec.decodeSessionMetadata(
            ProgrammedWorkoutCodec.encodeSessionMetadata(metadata),
        )

        assertEquals(metadata, decoded)
    }

    @Test
    fun `exercise prescription survives encode decode`() {
        val exercise = ExercisePrescription(
            exerciseName = "Back Squat",
            note = "Stay braced.",
            lines = listOf(
                SetPrescription(
                    label = "Top single",
                    sets = 1,
                    reps = 1,
                    intensityPercent = 0.9f,
                    targetWeightKg = 180f,
                    targetRpe = 8.0f,
                    restSeconds = 240,
                ),
                SetPrescription(
                    label = "Work sets",
                    sets = 4,
                    reps = 3,
                    intensityPercent = 0.82f,
                    targetWeightKg = 165f,
                    targetRpe = null,
                    restSeconds = 210,
                ),
            ),
        )

        val decoded = ProgrammedWorkoutCodec.decodeExercisePrescription(
            ProgrammedWorkoutCodec.encodeExercisePrescription(exercise),
        )

        assertNotNull(decoded)
        assertEquals(exercise, decoded)
    }
}
