package com.gregorycarnegie.ironinsights.ui.programmes

// Defaults reflect common resistance-training and powerlifting periodization
// patterns from the literature. They are planning aids, not rigid rules.

data class ProgrammeBlockDraft(
    val name: String,
    val blockType: String,
    val weekCount: Int,
)

data class ProgrammeBlockTemplate(
    val blockType: String,
    val label: String,
    val defaultName: String,
    val weekRange: IntRange,
    val defaultWeeks: Int,
    val goal: String,
    val loadingFocus: String,
    val rationale: String,
)

data class ProgrammePresetTemplate(
    val id: String,
    val title: String,
    val summary: String,
    val blocks: List<ProgrammeBlockDraft>,
)

internal val programmeBlockTemplates = listOf(
    ProgrammeBlockTemplate(
        blockType = "hypertrophy",
        label = "Hypertrophy",
        defaultName = "Hypertrophy",
        weekRange = 4..8,
        defaultWeeks = 5,
        goal = "Build muscle and work capacity before heavier phases.",
        loadingFocus = "Higher volume, moderate loads, more accessory work.",
        rationale = "Useful when you want more tissue tolerance and base volume before pushing intensity.",
    ),
    ProgrammeBlockTemplate(
        blockType = "accumulation",
        label = "Accumulation",
        defaultName = "Accumulation",
        weekRange = 3..6,
        defaultWeeks = 4,
        goal = "Accumulate productive volume with competition-lift practice.",
        loadingFocus = "Moderate intensity with enough volume to drive progress.",
        rationale = "Common first block in block periodization before intensification.",
    ),
    ProgrammeBlockTemplate(
        blockType = "strength",
        label = "Strength",
        defaultName = "Strength",
        weekRange = 3..6,
        defaultWeeks = 4,
        goal = "Move toward heavier work and lower reps.",
        loadingFocus = "Moderate-to-high intensity with volume pulled back from base phases.",
        rationale = "Classic bridge between volume work and peaking.",
    ),
    ProgrammeBlockTemplate(
        blockType = "intensification",
        label = "Intensification",
        defaultName = "Intensification",
        weekRange = 2..5,
        defaultWeeks = 3,
        goal = "Push force output and specificity while trimming fatigue-producing volume.",
        loadingFocus = "Heavier competition-lift work, fewer total hard sets.",
        rationale = "Block-periodization counterpart to a strength phase.",
    ),
    ProgrammeBlockTemplate(
        blockType = "realization",
        label = "Realization",
        defaultName = "Realization",
        weekRange = 1..3,
        defaultWeeks = 2,
        goal = "Express strength with very high specificity and minimal fluff.",
        loadingFocus = "Heavy singles, doubles, and highly specific practice.",
        rationale = "Often the final heavy block before a taper or meet.",
    ),
    ProgrammeBlockTemplate(
        blockType = "peak",
        label = "Peak",
        defaultName = "Peak",
        weekRange = 1..3,
        defaultWeeks = 2,
        goal = "Prepare for top-end strength expression close to competition.",
        loadingFocus = "Very high intensity, low volume, high specificity.",
        rationale = "Best used after a longer base or strength build.",
    ),
    ProgrammeBlockTemplate(
        blockType = "taper",
        label = "Taper",
        defaultName = "Taper",
        weekRange = 1..2,
        defaultWeeks = 1,
        goal = "Drop fatigue while keeping readiness high.",
        loadingFocus = "Large volume reduction with intensity mostly maintained.",
        rationale = "Meet-prep tapers are usually short; this is the final sharpening block.",
    ),
    ProgrammeBlockTemplate(
        blockType = "deload",
        label = "Deload",
        defaultName = "Deload",
        weekRange = 1..2,
        defaultWeeks = 1,
        goal = "Recover between heavier training waves.",
        loadingFocus = "Lower volume and lower stress so the next block starts fresh.",
        rationale = "Use between demanding phases or after a long accumulation of fatigue.",
    ),
)

internal val programmePresetTemplates = listOf(
    ProgrammePresetTemplate(
        id = "traditional_meet_prep",
        title = "Traditional Meet Prep",
        summary = "A classic 12-week build from volume to strength to peak, then taper.",
        blocks = listOf(
            ProgrammeBlockDraft("Hypertrophy", "hypertrophy", 4),
            ProgrammeBlockDraft("Strength", "strength", 4),
            ProgrammeBlockDraft("Peak", "peak", 2),
            ProgrammeBlockDraft("Taper", "taper", 2),
        ),
    ),
    ProgrammePresetTemplate(
        id = "block_peak_cycle",
        title = "Block Peak Cycle",
        summary = "A 10-week accumulation -> intensification -> realization -> taper sequence.",
        blocks = listOf(
            ProgrammeBlockDraft("Accumulation", "accumulation", 4),
            ProgrammeBlockDraft("Intensification", "intensification", 3),
            ProgrammeBlockDraft("Realization", "realization", 2),
            ProgrammeBlockDraft("Taper", "taper", 1),
        ),
    ),
    ProgrammePresetTemplate(
        id = "off_season_strength_build",
        title = "Off-Season Strength Build",
        summary = "A longer 14-week build for capacity, then strength, then a reset week.",
        blocks = listOf(
            ProgrammeBlockDraft("Hypertrophy", "hypertrophy", 6),
            ProgrammeBlockDraft("Accumulation", "accumulation", 4),
            ProgrammeBlockDraft("Strength", "strength", 3),
            ProgrammeBlockDraft("Deload", "deload", 1),
        ),
    ),
)

internal fun programmeBlockTemplate(blockType: String): ProgrammeBlockTemplate {
    return programmeBlockTemplates.firstOrNull { it.blockType == blockType }
        ?: ProgrammeBlockTemplate(
            blockType = blockType,
            label = blockType.replaceFirstChar { it.uppercase() },
            defaultName = blockType.replaceFirstChar { it.uppercase() },
            weekRange = 1..6,
            defaultWeeks = 4,
            goal = "Custom training block.",
            loadingFocus = "Set the duration based on your current training needs.",
            rationale = "Custom block type.",
        )
}

internal fun programmePresetTotalWeeks(preset: ProgrammePresetTemplate): Int {
    return preset.blocks.sumOf { it.weekCount }
}
