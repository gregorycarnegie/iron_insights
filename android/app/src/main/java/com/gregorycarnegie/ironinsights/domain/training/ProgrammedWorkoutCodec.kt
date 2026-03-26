package com.gregorycarnegie.ironinsights.domain.training

import java.net.URLDecoder
import java.net.URLEncoder
import java.nio.charset.StandardCharsets

data class ProgrammedWorkoutMetadata(
    val programmeId: Long,
    val programmeName: String,
    val weekNumber: Int,
    val sessionIndex: Int,
    val dayLabel: String,
    val sessionTitle: String,
)

data class ProgrammedWorkoutRecommendation(
    val metadata: ProgrammedWorkoutMetadata,
    val session: ProgrammeSessionPrescription,
)

object ProgrammedWorkoutCodec {
    private const val SESSION_PREFIX = "ii_programmed:v1"
    private const val EXERCISE_PREFIX = "ii_guided:v1"

    fun encodeSessionMetadata(metadata: ProgrammedWorkoutMetadata): String {
        return listOf(
            SESSION_PREFIX,
            metadata.programmeId.toString(),
            metadata.weekNumber.toString(),
            metadata.sessionIndex.toString(),
            encode(metadata.programmeName),
            encode(metadata.dayLabel),
            encode(metadata.sessionTitle),
        ).joinToString("|")
    }

    fun decodeSessionMetadata(raw: String?): ProgrammedWorkoutMetadata? {
        val parts = raw?.split("|") ?: return null
        if (parts.size < 7 || parts.firstOrNull() != SESSION_PREFIX) return null
        return ProgrammedWorkoutMetadata(
            programmeId = parts[1].toLongOrNull() ?: return null,
            weekNumber = parts[2].toIntOrNull() ?: return null,
            sessionIndex = parts[3].toIntOrNull() ?: return null,
            programmeName = decode(parts[4]),
            dayLabel = decode(parts[5]),
            sessionTitle = decode(parts[6]),
        )
    }

    fun encodeExercisePrescription(exercise: ExercisePrescription): String {
        val encodedLines = exercise.lines.joinToString("~") { line ->
            listOf(
                encode(line.label),
                line.sets.toString(),
                line.reps.toString(),
                line.intensityPercent?.toString().orEmpty(),
                line.targetWeightKg?.toString().orEmpty(),
                line.targetRpe?.toString().orEmpty(),
                line.restSeconds.toString(),
            ).joinToString("^")
        }
        return listOf(
            EXERCISE_PREFIX,
            encode(exercise.exerciseName),
            encode(exercise.note.orEmpty()),
            encodedLines,
        ).joinToString("|")
    }

    fun decodeExercisePrescription(raw: String?): ExercisePrescription? {
        val parts = raw?.split("|", limit = 4) ?: return null
        if (parts.size < 4 || parts.firstOrNull() != EXERCISE_PREFIX) return null
        val lines = if (parts[3].isBlank()) {
            emptyList()
        } else {
            parts[3].split("~").mapNotNull { decodeSetLine(it) }
        }
        return ExercisePrescription(
            exerciseName = decode(parts[1]),
            note = decode(parts[2]).ifBlank { null },
            lines = lines,
        )
    }

    private fun decodeSetLine(raw: String): SetPrescription? {
        val parts = raw.split("^")
        if (parts.size < 7) return null
        return SetPrescription(
            label = decode(parts[0]),
            sets = parts[1].toIntOrNull() ?: return null,
            reps = parts[2].toIntOrNull() ?: return null,
            intensityPercent = parts[3].toFloatOrNull(),
            targetWeightKg = parts[4].toFloatOrNull(),
            targetRpe = parts[5].toFloatOrNull(),
            restSeconds = parts[6].toIntOrNull() ?: return null,
        )
    }

    private fun encode(value: String): String {
        return URLEncoder.encode(value, StandardCharsets.UTF_8.toString())
    }

    private fun decode(value: String): String {
        return URLDecoder.decode(value, StandardCharsets.UTF_8.toString())
    }
}
