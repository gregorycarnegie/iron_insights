package com.gregorycarnegie.ironinsights.domain.export

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class IronInsightsCsvWriterTest {

    @Test
    fun empty_sessions_produce_header_only() {
        val result = IronInsightsCsvWriter.write(emptyList())
        val lines = result.trim().lines()
        assertEquals(1, lines.size)
        assertEquals("Date,Workout Name,Exercise Name,Set Order,Weight,Reps,RPE,Notes", lines[0])
    }

    @Test
    fun single_session_with_two_sets() {
        val session = IronInsightsCsvWriter.ExportSession(
            date = "2026-03-23 10:00:00",
            workoutName = "Monday Upper",
            exercises = listOf(
                IronInsightsCsvWriter.ExportExercise(
                    name = "Bench Press",
                    sets = listOf(
                        IronInsightsCsvWriter.ExportSet(1, 100f, 5, 8.0f, null),
                        IronInsightsCsvWriter.ExportSet(2, 100f, 5, 8.5f, null),
                    ),
                ),
            ),
        )
        val result = IronInsightsCsvWriter.write(listOf(session))
        val lines = result.trim().lines()
        assertEquals(3, lines.size)
        assertEquals("2026-03-23 10:00:00,Monday Upper,Bench Press,1,100,5,8,", lines[1])
        assertEquals("2026-03-23 10:00:00,Monday Upper,Bench Press,2,100,5,8.5,", lines[2])
    }

    @Test
    fun special_characters_in_notes_are_quoted() {
        val session = IronInsightsCsvWriter.ExportSession(
            date = "2026-03-23",
            workoutName = "Test",
            exercises = listOf(
                IronInsightsCsvWriter.ExportExercise(
                    name = "Squat",
                    sets = listOf(
                        IronInsightsCsvWriter.ExportSet(1, 140f, 3, null, "felt good, smooth"),
                    ),
                ),
            ),
        )
        val result = IronInsightsCsvWriter.write(listOf(session))
        val lines = result.trim().lines()
        assertTrue(lines[1].contains("\"felt good, smooth\""))
    }
}
