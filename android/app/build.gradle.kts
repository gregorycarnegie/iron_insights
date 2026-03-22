import org.gradle.api.GradleException
import org.gradle.api.Project

fun Project.propertyOrEnv(
    propertyName: String,
    envName: String = propertyName,
): String? {
    return (findProperty(propertyName) as String?)?.takeIf { it.isNotBlank() }
        ?: System.getenv(envName)?.takeIf { it.isNotBlank() }
}

data class ReleaseSigningInputs(
    val storeFilePath: String,
    val storePassword: String,
    val keyAlias: String,
    val keyPassword: String,
)

val releaseVersionCode = project
    .propertyOrEnv("ironInsightsVersionCode", "IRON_INSIGHTS_VERSION_CODE")
    ?.let { raw ->
        raw.toIntOrNull() ?: throw GradleException(
            "ironInsightsVersionCode/IRON_INSIGHTS_VERSION_CODE must be a valid integer.",
        )
    }
    ?: 1

val releaseVersionName = project
    .propertyOrEnv("ironInsightsVersionName", "IRON_INSIGHTS_VERSION_NAME")
    ?: "0.1.0"

val requireReleaseSigning = project
    .propertyOrEnv("ironInsightsRequireReleaseSigning", "IRON_INSIGHTS_REQUIRE_RELEASE_SIGNING")
    ?.toBooleanStrictOrNull()
    ?: false

val releaseSigningValues = mapOf(
    "ironInsightsKeystorePath/IRON_INSIGHTS_KEYSTORE_PATH" to
        project.propertyOrEnv("ironInsightsKeystorePath", "IRON_INSIGHTS_KEYSTORE_PATH"),
    "ironInsightsKeystorePassword/IRON_INSIGHTS_KEYSTORE_PASSWORD" to
        project.propertyOrEnv("ironInsightsKeystorePassword", "IRON_INSIGHTS_KEYSTORE_PASSWORD"),
    "ironInsightsKeyAlias/IRON_INSIGHTS_KEY_ALIAS" to
        project.propertyOrEnv("ironInsightsKeyAlias", "IRON_INSIGHTS_KEY_ALIAS"),
    "ironInsightsKeyPassword/IRON_INSIGHTS_KEY_PASSWORD" to
        project.propertyOrEnv("ironInsightsKeyPassword", "IRON_INSIGHTS_KEY_PASSWORD"),
)

val configuredReleaseSigningCount = releaseSigningValues.values.count { it != null }

if (configuredReleaseSigningCount in 1 until releaseSigningValues.size) {
    val missingKeys = releaseSigningValues
        .filterValues { it == null }
        .keys
        .joinToString(", ")
    throw GradleException(
        "Release signing inputs must be provided as a complete set. Missing: $missingKeys",
    )
}

val releaseSigningInputs = run {
    if (
        configuredReleaseSigningCount == releaseSigningValues.size
    ) {
        ReleaseSigningInputs(
            storeFilePath = releaseSigningValues.getValue(
                "ironInsightsKeystorePath/IRON_INSIGHTS_KEYSTORE_PATH",
            )!!,
            storePassword = releaseSigningValues.getValue(
                "ironInsightsKeystorePassword/IRON_INSIGHTS_KEYSTORE_PASSWORD",
            )!!,
            keyAlias = releaseSigningValues.getValue(
                "ironInsightsKeyAlias/IRON_INSIGHTS_KEY_ALIAS",
            )!!,
            keyPassword = releaseSigningValues.getValue(
                "ironInsightsKeyPassword/IRON_INSIGHTS_KEY_PASSWORD",
            )!!,
        )
    } else {
        null
    }
}

if (requireReleaseSigning && releaseSigningInputs == null) {
    throw GradleException(
        "Release signing is required but no complete release signing input set was provided.",
    )
}

releaseSigningInputs?.let { inputs ->
    if (!project.file(inputs.storeFilePath).exists()) {
        throw GradleException("Release keystore not found at ${inputs.storeFilePath}.")
    }
}

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("org.jetbrains.kotlin.plugin.compose")
}

android {
    namespace = "com.gregorycarnegie.ironinsights"
    compileSdk = 35

    defaultConfig {
        applicationId = "com.gregorycarnegie.ironinsights"
        minSdk = 26
        targetSdk = 35
        versionCode = releaseVersionCode
        versionName = releaseVersionName
        buildConfigField(
            "String",
            "SITE_BASE_URL",
            "\"https://gregorycarnegie.github.io/iron_insights/\"",
        )
    }

    signingConfigs {
        releaseSigningInputs?.let { inputs ->
            create("release") {
                storeFile = file(inputs.storeFilePath)
                storePassword = inputs.storePassword
                keyAlias = inputs.keyAlias
                keyPassword = inputs.keyPassword
            }
        }
    }

    buildTypes {
        getByName("release") {
            isMinifyEnabled = false
            isShrinkResources = false
            releaseSigningInputs?.let {
                signingConfig = signingConfigs.getByName("release")
            }
        }
    }

    buildFeatures {
        compose = true
        buildConfig = true
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = "17"
    }

    packaging {
        resources {
            excludes += "/META-INF/{AL2.0,LGPL2.1}"
        }
    }
}

dependencies {
    val composeBom = platform("androidx.compose:compose-bom:2026.02.01")

    implementation(composeBom)
    androidTestImplementation(composeBom)

    implementation("androidx.activity:activity-compose:1.10.1")
    implementation("androidx.compose.foundation:foundation")
    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.material3:material3")
    implementation("androidx.compose.ui:ui-tooling-preview")
    implementation("androidx.lifecycle:lifecycle-viewmodel:2.9.4")

    testImplementation("junit:junit:4.13.2")

    debugImplementation("androidx.compose.ui:ui-tooling")
}

tasks.register("validateReleaseConfig") {
    group = "verification"
    description = "Validates release metadata and signing inputs without building the app bundle."

    doLast {
        if (releaseVersionCode < 1) {
            throw GradleException("Release versionCode must be greater than zero.")
        }
        if (releaseVersionName.isBlank()) {
            throw GradleException("Release versionName must not be blank.")
        }
        if (requireReleaseSigning && releaseSigningInputs == null) {
            throw GradleException("Release signing inputs are required for this validation.")
        }
    }
}
