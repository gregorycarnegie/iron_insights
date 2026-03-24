package com.gregorycarnegie.ironinsights.domain.export

object IronInsightsCsvWriter {

    private const val HEADER = "Date,Workout Name,Exercise Name,Set Order,Weight,Reps,RPE,Notes"

    fun write(sessions: List<ExportSession>): String {
        val sb = StringBuilder()
        sb.appendLine(HEADER)
        for (session in sessions) {
            for (exercise in session.exercises) {
                for (set in exercise.sets) {
                    sb.appendLine(
                        buildRow(
                            session.date,
                            session.workoutName,
                            exercise.name,
                            set.setOrder,
                            set.weightKg,
                            set.reps,
                            set.rpe,
                            set.notes,
                        ),
                    )
                }
            }
        }
        return sb.toString()
    }

    private fun buildRow(
        date: String,
        workoutName: String,
        exerciseName: String,
        setOrder: Int,
        weightKg: Float,
        reps: Int,
        rpe: Float?,
        notes: String?,
    ): String {
        return listOf(
            csvField(date),
            csvField(workoutName),
            csvField(exerciseName),
            setOrder.toString(),
            formatWeight(weightKg),
            reps.toString(),
            rpe?.let { formatWeight(it) } ?: "",
            csvField(notes ?: ""),
        ).joinToString(",")
    }

    private fun csvField(value: String): String {
        return if (value.contains(',') || value.contains('"') || value.contains('\n')) {
            "\"${value.replace("\"", "\"\"")}\""
        } else {
            value
        }
    }

    private fun formatWeight(value: Float): String {
        return if (value == value.toLong().toFloat()) {
            value.toLong().toString()
        } else {
            value.toString()
        }
    }

    data class ExportSession(
        val date: String,
        val workoutName: String,
        val exercises: List<ExportExercise>,
    )

    data class ExportExercise(
        val name: String,
        val sets: List<ExportSet>,
    )

    data class ExportSet(
        val setOrder: Int,
        val weightKg: Float,
        val reps: Int,
        val rpe: Float?,
        val notes: String?,
    )
}
