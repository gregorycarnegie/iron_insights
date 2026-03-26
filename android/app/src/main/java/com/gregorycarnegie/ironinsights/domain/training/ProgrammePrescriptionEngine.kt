package com.gregorycarnegie.ironinsights.domain.training

import com.gregorycarnegie.ironinsights.data.db.entity.ProgrammeBlock
import kotlin.math.roundToInt

data class ProgrammeLiftBaseline(
    val canonicalLift: String,
    val label: String,
    val e1rmKg: Float?,
)

data class ProgrammePrescriptionPlan(
    val summary: String,
    val weeks: List<ProgrammeWeekPrescription>,
    val missingBaselines: List<String>,
)

data class ProgrammeWeekPrescription(
    val weekNumber: Int,
    val blockName: String,
    val blockType: String,
    val weekInBlock: Int,
    val totalWeeksInBlock: Int,
    val focus: String,
    val sessions: List<ProgrammeSessionPrescription>,
)

data class ProgrammeSessionPrescription(
    val dayLabel: String,
    val title: String,
    val coachingNote: String,
    val warmupNote: String,
    val exercises: List<ExercisePrescription>,
)

data class ExercisePrescription(
    val exerciseName: String,
    val lines: List<SetPrescription>,
    val note: String? = null,
)

data class SetPrescription(
    val label: String,
    val sets: Int,
    val reps: Int,
    val intensityPercent: Float?,
    val targetWeightKg: Float?,
    val targetRpe: Float? = null,
    val restSeconds: Int,
)

private data class BlockWeekProfile(
    val focus: String,
    val primary: SlotPrescription,
    val secondary: SlotPrescription,
    val variation: SlotPrescription,
    val technique: SlotPrescription,
    val topSinglePercent: Float? = null,
    val topSingleRpe: Float? = null,
    val warmupTopPercent: Float,
)

private data class SlotPrescription(
    val sets: Int,
    val reps: Int,
    val percent: Float,
    val restSeconds: Int,
)

object ProgrammePrescriptionEngine {
    fun build(
        blocks: List<ProgrammeBlock>,
        baselines: List<ProgrammeLiftBaseline>,
        roundingIncrementKg: Float,
    ): ProgrammePrescriptionPlan {
        val baselineMap = baselines.associate { it.canonicalLift to it.e1rmKg }
        val missingBaselines = baselines.filter { it.e1rmKg == null }.map { it.label }
        val weeks = mutableListOf<ProgrammeWeekPrescription>()
        var absoluteWeek = 1

        blocks.sortedBy { it.orderIndex }.forEach { block ->
            for (weekIndex in 0 until block.weekCount) {
                val weekProfile = buildWeekProfile(
                    blockType = block.blockType,
                    weekIndex = weekIndex,
                    totalWeeks = block.weekCount,
                )
                weeks += ProgrammeWeekPrescription(
                    weekNumber = absoluteWeek,
                    blockName = block.name,
                    blockType = block.blockType,
                    weekInBlock = weekIndex + 1,
                    totalWeeksInBlock = block.weekCount,
                    focus = weekProfile.focus,
                    sessions = buildSessions(
                        profile = weekProfile,
                        baselines = baselineMap,
                        roundingIncrementKg = roundingIncrementKg,
                    ),
                )
                absoluteWeek++
            }
        }

        return ProgrammePrescriptionPlan(
            summary = "Four sessions per week, auto-progressed from volume to specificity. Main lifts and close variations use your latest estimated maxes when available.",
            weeks = weeks,
            missingBaselines = missingBaselines,
        )
    }

    private fun buildSessions(
        profile: BlockWeekProfile,
        baselines: Map<String, Float?>,
        roundingIncrementKg: Float,
    ): List<ProgrammeSessionPrescription> {
        return listOf(
            ProgrammeSessionPrescription(
                dayLabel = "Day 1",
                title = "Squat Volume + Bench Base",
                coachingNote = "Keep every rep technically clean. Do not chase fatigue after the last work set.",
                warmupNote = warmupNote(profile.warmupTopPercent),
                exercises = listOf(
                    liftPrescription(
                        exerciseName = "Back Squat",
                        canonicalLift = "S",
                        work = profile.primary,
                        baselines = baselines,
                        roundingIncrementKg = roundingIncrementKg,
                        topSinglePercent = profile.topSinglePercent,
                        topSingleRpe = profile.topSingleRpe,
                    ),
                    liftPrescription(
                        exerciseName = "Bench Press",
                        canonicalLift = "B",
                        work = profile.secondary,
                        baselines = baselines,
                        roundingIncrementKg = roundingIncrementKg,
                    ),
                    liftPrescription(
                        exerciseName = "Pause Squat",
                        canonicalLift = "S",
                        work = profile.variation,
                        baselines = baselines,
                        roundingIncrementKg = roundingIncrementKg,
                        variationFactor = 0.92f,
                        note = "Pause each rep for a full count on the box or in the hole.",
                    ),
                ),
            ),
            ProgrammeSessionPrescription(
                dayLabel = "Day 2",
                title = "Deadlift Volume + Bench Technique",
                coachingNote = "Treat the deadlift start position like meet day. No grip or setup improvisation.",
                warmupNote = warmupNote(profile.warmupTopPercent),
                exercises = listOf(
                    liftPrescription(
                        exerciseName = "Deadlift",
                        canonicalLift = "D",
                        work = profile.primary,
                        baselines = baselines,
                        roundingIncrementKg = roundingIncrementKg,
                        topSinglePercent = profile.topSinglePercent,
                        topSingleRpe = profile.topSingleRpe,
                    ),
                    liftPrescription(
                        exerciseName = "Bench Press",
                        canonicalLift = "B",
                        work = profile.technique,
                        baselines = baselines,
                        roundingIncrementKg = roundingIncrementKg,
                        note = "Pause every rep and use the same grip you plan to compete with.",
                    ),
                    liftPrescription(
                        exerciseName = "Romanian Deadlift",
                        canonicalLift = "D",
                        work = profile.variation,
                        baselines = baselines,
                        roundingIncrementKg = roundingIncrementKg,
                        variationFactor = 0.78f,
                        note = "Keep the bar close and stop just below the knee if your back position degrades.",
                    ),
                ),
            ),
            ProgrammeSessionPrescription(
                dayLabel = "Day 3",
                title = "Bench Priority + Squat Supplemental",
                coachingNote = "Bench gets the freshest slot today. Keep setup identical across all work sets.",
                warmupNote = warmupNote(profile.warmupTopPercent),
                exercises = listOf(
                    liftPrescription(
                        exerciseName = "Bench Press",
                        canonicalLift = "B",
                        work = profile.primary,
                        baselines = baselines,
                        roundingIncrementKg = roundingIncrementKg,
                        topSinglePercent = profile.topSinglePercent,
                        topSingleRpe = profile.topSingleRpe,
                    ),
                    liftPrescription(
                        exerciseName = "Front Squat",
                        canonicalLift = "S",
                        work = profile.variation,
                        baselines = baselines,
                        roundingIncrementKg = roundingIncrementKg,
                        variationFactor = 0.85f,
                        note = "Stay upright and keep the front rack clean. This is upper-back and quad work, not a max effort.",
                    ),
                    liftPrescription(
                        exerciseName = "Close-Grip Bench Press",
                        canonicalLift = "B",
                        work = profile.variation,
                        baselines = baselines,
                        roundingIncrementKg = roundingIncrementKg,
                        variationFactor = 0.92f,
                        note = "Hands one thumb-length inside normal comp grip.",
                    ),
                ),
            ),
            ProgrammeSessionPrescription(
                dayLabel = "Day 4",
                title = "Specificity Day",
                coachingNote = "Finish the week by practicing competition movement quality under manageable fatigue.",
                warmupNote = warmupNote(profile.warmupTopPercent),
                exercises = listOf(
                    liftPrescription(
                        exerciseName = "Back Squat",
                        canonicalLift = "S",
                        work = profile.technique,
                        baselines = baselines,
                        roundingIncrementKg = roundingIncrementKg,
                        topSinglePercent = if (profile.topSinglePercent != null) profile.topSinglePercent - 0.02f else null,
                        topSingleRpe = profile.topSingleRpe,
                    ),
                    liftPrescription(
                        exerciseName = "Deadlift",
                        canonicalLift = "D",
                        work = profile.secondary,
                        baselines = baselines,
                        roundingIncrementKg = roundingIncrementKg,
                    ),
                    liftPrescription(
                        exerciseName = "Bench Press",
                        canonicalLift = "B",
                        work = profile.secondary,
                        baselines = baselines,
                        roundingIncrementKg = roundingIncrementKg,
                    ),
                ),
            ),
        )
    }

    private fun liftPrescription(
        exerciseName: String,
        canonicalLift: String,
        work: SlotPrescription,
        baselines: Map<String, Float?>,
        roundingIncrementKg: Float,
        variationFactor: Float = 1f,
        topSinglePercent: Float? = null,
        topSingleRpe: Float? = null,
        note: String? = null,
    ): ExercisePrescription {
        val lines = mutableListOf<SetPrescription>()
        if (topSinglePercent != null) {
            lines += SetPrescription(
                label = "Top single",
                sets = 1,
                reps = 1,
                intensityPercent = topSinglePercent,
                targetWeightKg = resolveWeight(
                    canonicalLift = canonicalLift,
                    baselines = baselines,
                    intensityPercent = topSinglePercent,
                    roundingIncrementKg = roundingIncrementKg,
                    variationFactor = variationFactor,
                ),
                targetRpe = topSingleRpe,
                restSeconds = 240,
            )
        }
        lines += SetPrescription(
            label = "Work sets",
            sets = work.sets,
            reps = work.reps,
            intensityPercent = work.percent,
            targetWeightKg = resolveWeight(
                canonicalLift = canonicalLift,
                baselines = baselines,
                intensityPercent = work.percent,
                roundingIncrementKg = roundingIncrementKg,
                variationFactor = variationFactor,
            ),
            targetRpe = null,
            restSeconds = work.restSeconds,
        )
        return ExercisePrescription(
            exerciseName = exerciseName,
            lines = lines,
            note = note,
        )
    }

    private fun resolveWeight(
        canonicalLift: String,
        baselines: Map<String, Float?>,
        intensityPercent: Float?,
        roundingIncrementKg: Float,
        variationFactor: Float,
    ): Float? {
        val e1rm = baselines[canonicalLift] ?: return null
        val percent = intensityPercent ?: return null
        return roundToIncrement(
            value = e1rm * percent * variationFactor,
            increment = roundingIncrementKg,
        )
    }

    private fun roundToIncrement(value: Float, increment: Float): Float {
        if (increment <= 0f) return value
        return ((value / increment).roundToInt() * increment).coerceAtLeast(0f)
    }

    private fun warmupNote(firstWorkSetPercent: Float): String {
        val percent = (firstWorkSetPercent * 100f).roundToInt()
        return "Warm up in four jumps: 5 @ 40%, 3 @ 55%, 2 @ 65%, 1 @ 75% before the first $percent% work set."
    }

    private fun buildWeekProfile(
        blockType: String,
        weekIndex: Int,
        totalWeeks: Int,
    ): BlockWeekProfile {
        val progress = progression(weekIndex, totalWeeks)
        return when (blockType) {
            "hypertrophy" -> BlockWeekProfile(
                focus = "Build muscle and work capacity while keeping all three lifts in rotation.",
                primary = SlotPrescription(sets = 5 - if (progress > 0.66f) 1 else 0, reps = 8 - if (progress > 0.66f) 2 else 0, percent = lerp(0.62f, 0.70f, progress), restSeconds = 180),
                secondary = SlotPrescription(sets = 4, reps = 8, percent = lerp(0.58f, 0.65f, progress), restSeconds = 150),
                variation = SlotPrescription(sets = 3, reps = 10 - if (progress > 0.66f) 2 else 0, percent = lerp(0.50f, 0.60f, progress), restSeconds = 120),
                technique = SlotPrescription(sets = 3, reps = 6, percent = lerp(0.55f, 0.62f, progress), restSeconds = 120),
                warmupTopPercent = lerp(0.62f, 0.70f, progress),
            )
            "accumulation" -> BlockWeekProfile(
                focus = "Hold onto volume but shift the work toward more specific barbell practice.",
                primary = SlotPrescription(sets = 5, reps = 6, percent = lerp(0.68f, 0.77f, progress), restSeconds = 210),
                secondary = SlotPrescription(sets = 4, reps = 6, percent = lerp(0.64f, 0.72f, progress), restSeconds = 180),
                variation = SlotPrescription(sets = 3, reps = 8, percent = lerp(0.56f, 0.64f, progress), restSeconds = 150),
                technique = SlotPrescription(sets = 4, reps = 4, percent = lerp(0.60f, 0.68f, progress), restSeconds = 150),
                warmupTopPercent = lerp(0.68f, 0.77f, progress),
            )
            "strength" -> BlockWeekProfile(
                focus = "Shift toward heavier work while preserving enough total volume to keep progress moving.",
                primary = SlotPrescription(sets = 5, reps = if (progress > 0.5f) 4 else 5, percent = lerp(0.75f, 0.82f, progress), restSeconds = 240),
                secondary = SlotPrescription(sets = 4, reps = 4, percent = lerp(0.70f, 0.78f, progress), restSeconds = 210),
                variation = SlotPrescription(sets = 3, reps = 6, percent = lerp(0.62f, 0.70f, progress), restSeconds = 150),
                technique = SlotPrescription(sets = 3, reps = 3, percent = lerp(0.65f, 0.72f, progress), restSeconds = 150),
                topSinglePercent = lerp(0.85f, 0.88f, progress),
                topSingleRpe = 7.5f,
                warmupTopPercent = lerp(0.75f, 0.82f, progress),
            )
            "intensification" -> BlockWeekProfile(
                focus = "Reduce fluff, practice heavier competition work, and let fatigue-producing volume fall.",
                primary = SlotPrescription(sets = 4, reps = 3, percent = lerp(0.82f, 0.88f, progress), restSeconds = 270),
                secondary = SlotPrescription(sets = 3, reps = 4, percent = lerp(0.76f, 0.82f, progress), restSeconds = 240),
                variation = SlotPrescription(sets = 3, reps = 5, percent = lerp(0.68f, 0.74f, progress), restSeconds = 180),
                technique = SlotPrescription(sets = 3, reps = 2, percent = lerp(0.70f, 0.78f, progress), restSeconds = 180),
                topSinglePercent = lerp(0.88f, 0.92f, progress),
                topSingleRpe = 8.0f,
                warmupTopPercent = lerp(0.82f, 0.88f, progress),
            )
            "realization" -> BlockWeekProfile(
                focus = "Express strength with low-rep work and frequent exposure to heavy singles.",
                primary = SlotPrescription(sets = 3, reps = 2, percent = lerp(0.85f, 0.90f, progress), restSeconds = 300),
                secondary = SlotPrescription(sets = 3, reps = 3, percent = lerp(0.80f, 0.85f, progress), restSeconds = 240),
                variation = SlotPrescription(sets = 2, reps = 4, percent = lerp(0.70f, 0.76f, progress), restSeconds = 180),
                technique = SlotPrescription(sets = 2, reps = 2, percent = lerp(0.72f, 0.80f, progress), restSeconds = 180),
                topSinglePercent = lerp(0.90f, 0.95f, progress),
                topSingleRpe = 8.5f,
                warmupTopPercent = lerp(0.85f, 0.90f, progress),
            )
            "peak" -> BlockWeekProfile(
                focus = "Practice meet-like efforts and keep only the work that directly supports strength expression.",
                primary = SlotPrescription(sets = 3, reps = 1, percent = lerp(0.88f, 0.92f, progress), restSeconds = 300),
                secondary = SlotPrescription(sets = 2, reps = 2, percent = lerp(0.82f, 0.86f, progress), restSeconds = 240),
                variation = SlotPrescription(sets = 2, reps = 3, percent = lerp(0.72f, 0.78f, progress), restSeconds = 180),
                technique = SlotPrescription(sets = 2, reps = 1, percent = lerp(0.78f, 0.82f, progress), restSeconds = 180),
                topSinglePercent = lerp(0.92f, 0.96f, progress),
                topSingleRpe = 8.5f,
                warmupTopPercent = lerp(0.88f, 0.92f, progress),
            )
            "taper" -> BlockWeekProfile(
                focus = "Cut fatigue hard while keeping the nervous system tuned with short, crisp heavy work.",
                primary = SlotPrescription(sets = 2, reps = if (progress > 0.5f) 1 else 2, percent = lerp(0.78f, 0.82f, progress), restSeconds = 240),
                secondary = SlotPrescription(sets = 2, reps = 2, percent = lerp(0.72f, 0.78f, progress), restSeconds = 210),
                variation = SlotPrescription(sets = 1, reps = 3, percent = lerp(0.60f, 0.65f, progress), restSeconds = 150),
                technique = SlotPrescription(sets = 2, reps = 1, percent = lerp(0.70f, 0.75f, progress), restSeconds = 150),
                topSinglePercent = lerp(0.88f, 0.93f, progress),
                topSingleRpe = 7.5f,
                warmupTopPercent = lerp(0.78f, 0.82f, progress),
            )
            "deload" -> BlockWeekProfile(
                focus = "Recover, move cleanly, and leave every session feeling better than you started.",
                primary = SlotPrescription(sets = 2, reps = 5, percent = lerp(0.60f, 0.65f, progress), restSeconds = 150),
                secondary = SlotPrescription(sets = 2, reps = 5, percent = lerp(0.55f, 0.60f, progress), restSeconds = 120),
                variation = SlotPrescription(sets = 2, reps = 6, percent = lerp(0.45f, 0.50f, progress), restSeconds = 90),
                technique = SlotPrescription(sets = 2, reps = 3, percent = lerp(0.50f, 0.55f, progress), restSeconds = 90),
                warmupTopPercent = lerp(0.60f, 0.65f, progress),
            )
            else -> BlockWeekProfile(
                focus = "Train the lifts consistently and add load only when the prescribed work stays clean.",
                primary = SlotPrescription(sets = 4, reps = 6, percent = lerp(0.70f, 0.78f, progress), restSeconds = 180),
                secondary = SlotPrescription(sets = 3, reps = 6, percent = lerp(0.65f, 0.72f, progress), restSeconds = 150),
                variation = SlotPrescription(sets = 3, reps = 8, percent = lerp(0.55f, 0.62f, progress), restSeconds = 120),
                technique = SlotPrescription(sets = 3, reps = 4, percent = lerp(0.60f, 0.66f, progress), restSeconds = 120),
                warmupTopPercent = lerp(0.70f, 0.78f, progress),
            )
        }
    }

    private fun progression(
        weekIndex: Int,
        totalWeeks: Int,
    ): Float {
        if (totalWeeks <= 1) return 1f
        return weekIndex.toFloat() / (totalWeeks - 1).toFloat()
    }

    private fun lerp(
        start: Float,
        end: Float,
        progress: Float,
    ): Float {
        return start + (end - start) * progress.coerceIn(0f, 1f)
    }
}
