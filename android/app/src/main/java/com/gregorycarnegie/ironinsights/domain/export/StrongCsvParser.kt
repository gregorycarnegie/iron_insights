package com.gregorycarnegie.ironinsights.domain.export

object StrongCsvParser {

    data class ParsedWorkout(
        val date: String,
        val workoutName: String,
        val exerciseName: String,
        val setOrder: Int,
        val weight: Float,
        val reps: Int,
        val rpe: Float?,
        val notes: String?,
    )

    fun parse(csvContent: String): List<ParsedWorkout> {
        val lines = csvContent.lines().filter { it.isNotBlank() }
        if (lines.isEmpty()) return emptyList()

        // Skip the header row
        return lines.drop(1).mapNotNull { line ->
            val fields = parseCsvLine(line)
            if (fields.size < 6) return@mapNotNull null

            ParsedWorkout(
                date = fields[0].trim(),
                workoutName = fields[1].trim(),
                exerciseName = fields[2].trim(),
                setOrder = fields[3].trim().toIntOrNull() ?: return@mapNotNull null,
                weight = fields[4].trim().toFloatOrNull() ?: 0f,
                reps = fields[5].trim().toIntOrNull() ?: 0,
                rpe = fields.getOrNull(6)?.trim()?.toFloatOrNull(),
                notes = fields.getOrNull(7)?.trim()?.takeIf { it.isNotEmpty() },
            )
        }
    }

    private fun parseCsvLine(line: String): List<String> {
        val fields = mutableListOf<String>()
        val current = StringBuilder()
        var inQuotes = false
        var i = 0

        while (i < line.length) {
            val ch = line[i]
            when {
                inQuotes -> {
                    if (ch == '"') {
                        if (i + 1 < line.length && line[i + 1] == '"') {
                            current.append('"')
                            i++ // skip the escaped quote
                        } else {
                            inQuotes = false
                        }
                    } else {
                        current.append(ch)
                    }
                }
                ch == '"' -> inQuotes = true
                ch == ',' -> {
                    fields.add(current.toString())
                    current.clear()
                }
                else -> current.append(ch)
            }
            i++
        }
        fields.add(current.toString())
        return fields
    }
}
