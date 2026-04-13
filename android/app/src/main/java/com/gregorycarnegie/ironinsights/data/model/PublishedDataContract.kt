package com.gregorycarnegie.ironinsights.data.model

data class DatasetEndpoint(
    val title: String,
    val relativePath: String,
    val purpose: String,
)

data class PublishedShardKey(
    val sex: String,
    val equip: String,
)

data class PublishedSliceKey(
    val sex: String,
    val equip: String,
    val wc: String,
    val age: String,
    val tested: String,
    val lift: String,
    val metric: String,
    val metricExplicit: Boolean,
)

data class PublishedSlicePaths(
    val meta: String,
    val bin: String,
)

data class PublishedSliceEntry(
    val key: PublishedSliceKey,
    val paths: PublishedSlicePaths,
)

object PublishedDataContract {
    val bootstrapSequence = listOf(
        DatasetEndpoint(
            title = "Latest pointer",
            relativePath = "data/latest.json",
            purpose = "Find the active published dataset version.",
        ),
        DatasetEndpoint(
            title = "Version root index",
            relativePath = "data/<version>/index.json",
            purpose = "Find shard files for the selected sex and equipment bucket.",
        ),
        DatasetEndpoint(
            title = "Shard index",
            relativePath = "data/<version>/index_shards/<sex>/<equip>/index.json",
            purpose = "Resolve slice summaries and binary payload locations.",
        ),
        DatasetEndpoint(
            title = "Combined binary (IIC1)",
            relativePath = "data/<version>/bin/*.bin",
            purpose = "Drive percentile, density, and bodyweight-conditioned views (histogram + heatmap in one fetch).",
        ),
        DatasetEndpoint(
            title = "Trends payload",
            relativePath = "data/<version>/trends_shards/<sex>/<equip>/trends.json",
            purpose = "Power the trend charts and cohort history screens (per sex/equip shard).",
        ),
    )

    val nextMilestones = listOf(
        "Do another device pass on the branded UI and tighten any screen-specific polish gaps.",
        "Add manual cache controls and deeper dataset freshness details around offline fallback.",
        "Decide whether Android should keep the Kotlin contract mirror or call the shared Rust core through JNI.",
        "Configure Android release secrets and Play upload on top of the signed bundle workflow.",
    )
}

object PublishedSliceContract {
    fun parseShardKey(raw: String): PublishedShardKey? {
        val parts = parseKeyParts(raw) ?: return null
        val sex = parts["sex"] ?: return null
        val equip = parts["equip"] ?: return null
        return PublishedShardKey(sex = sex, equip = equip)
    }

    fun parseSliceKey(raw: String): PublishedSliceKey? {
        val parts = parseKeyParts(raw) ?: return null
        val sex = parts["sex"] ?: return null
        val equip = parts["equip"] ?: return null
        val wc = parts["wc"] ?: return null
        val age = parts["age"] ?: return null
        val tested = parts["tested"] ?: return null
        val lift = parts["lift"] ?: return null
        return PublishedSliceKey(
            sex = sex,
            equip = equip,
            wc = wc,
            age = age,
            tested = tested,
            lift = lift,
            metric = parts["metric"] ?: "Kg",
            metricExplicit = parts.containsKey("metric"),
        )
    }

    fun entryFromSliceKey(raw: String): PublishedSliceEntry? {
        val key = parseSliceKey(raw) ?: return null
        val basePath = payloadBasePathFromSliceKey(key) ?: return null
        return PublishedSliceEntry(
            key = key,
            paths = PublishedSlicePaths(
                meta = "meta/$basePath.json",
                bin = "bin/$basePath.bin",
            ),
        )
    }

    fun binPathFromSliceKey(raw: String): String? = entryFromSliceKey(raw)?.paths?.bin

    private fun payloadBasePathFromSliceKey(key: PublishedSliceKey): String? {
        val liftName = when (key.lift) {
            "S" -> "squat"
            "B" -> "bench"
            "D" -> "deadlift"
            "T" -> "total"
            else -> return null
        }

        val testedDir = if (key.tested.equals("yes", ignoreCase = true)) {
            "tested"
        } else {
            slug(key.tested)
        }

        return if (key.metricExplicit) {
            "${slug(key.sex)}/${slug(key.equip)}/${slug(key.wc)}/${slug(key.age)}/$testedDir/${slug(key.metric)}/$liftName"
        } else {
            "${slug(key.sex)}/${slug(key.equip)}/${slug(key.wc)}/${slug(key.age)}/$testedDir/$liftName"
        }
    }

    private fun parseKeyParts(raw: String): Map<String, String>? {
        val parts = LinkedHashMap<String, String>()
        for (part in raw.split('|')) {
            val equalsIndex = part.indexOf('=')
            if (equalsIndex <= 0 || equalsIndex == part.lastIndex) {
                return null
            }
            parts[part.substring(0, equalsIndex)] = part.substring(equalsIndex + 1)
        }
        return parts
    }

    private fun slug(input: String): String {
        val builder = StringBuilder(input.length)
        for (character in input) {
            builder.append(
                when {
                    character in 'A'..'Z' -> character.lowercaseChar()
                    character in 'a'..'z' || character in '0'..'9' || character == '-' -> character
                    else -> '_'
                },
            )
        }
        return builder.toString()
    }
}
