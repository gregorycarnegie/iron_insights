package com.gregorycarnegie.ironinsights.data.db.seed

import com.gregorycarnegie.ironinsights.data.db.entity.ExerciseDefinition

object DefaultExercises {
    val all: List<ExerciseDefinition> = listOf(
        // Competition lifts
        ExerciseDefinition(name = "Back Squat", canonicalLift = "S", category = "compound", muscleGroup = "legs"),
        ExerciseDefinition(name = "Bench Press", canonicalLift = "B", category = "compound", muscleGroup = "chest"),
        ExerciseDefinition(name = "Deadlift", canonicalLift = "D", category = "compound", muscleGroup = "back"),
        ExerciseDefinition(name = "Total", canonicalLift = "T", category = "compound", muscleGroup = "full body"),

        // Squat variations
        ExerciseDefinition(name = "Front Squat", category = "compound", muscleGroup = "legs"),
        ExerciseDefinition(name = "Pause Squat", category = "compound", muscleGroup = "legs"),
        ExerciseDefinition(name = "Leg Press", category = "compound", muscleGroup = "legs"),

        // Bench variations
        ExerciseDefinition(name = "Close-Grip Bench Press", category = "compound", muscleGroup = "chest"),
        ExerciseDefinition(name = "Incline Bench Press", category = "compound", muscleGroup = "chest"),
        ExerciseDefinition(name = "Dumbbell Bench Press", category = "compound", muscleGroup = "chest"),

        // Deadlift variations
        ExerciseDefinition(name = "Romanian Deadlift", category = "compound", muscleGroup = "back"),
        ExerciseDefinition(name = "Sumo Deadlift", category = "compound", muscleGroup = "back"),
        ExerciseDefinition(name = "Deficit Deadlift", category = "compound", muscleGroup = "back"),

        // Upper body compounds
        ExerciseDefinition(name = "Overhead Press", category = "compound", muscleGroup = "shoulders"),
        ExerciseDefinition(name = "Barbell Row", category = "compound", muscleGroup = "back"),
        ExerciseDefinition(name = "Dumbbell Row", category = "compound", muscleGroup = "back"),
        ExerciseDefinition(name = "Pull-Up", category = "compound", muscleGroup = "back"),
        ExerciseDefinition(name = "Dips", category = "compound", muscleGroup = "chest"),
        ExerciseDefinition(name = "Chin-Up", category = "compound", muscleGroup = "back"),

        // Lower body accessories
        ExerciseDefinition(name = "Lunges", category = "accessory", muscleGroup = "legs"),
        ExerciseDefinition(name = "Leg Curl", category = "isolation", muscleGroup = "legs"),
        ExerciseDefinition(name = "Leg Extension", category = "isolation", muscleGroup = "legs"),
        ExerciseDefinition(name = "Calf Raise", category = "isolation", muscleGroup = "legs"),

        // Upper body accessories
        ExerciseDefinition(name = "Barbell Curl", category = "isolation", muscleGroup = "arms"),
        ExerciseDefinition(name = "Tricep Pushdown", category = "isolation", muscleGroup = "arms"),
        ExerciseDefinition(name = "Lateral Raise", category = "isolation", muscleGroup = "shoulders"),
        ExerciseDefinition(name = "Face Pull", category = "isolation", muscleGroup = "shoulders"),
    )
}
