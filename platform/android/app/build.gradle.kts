plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

android {
    namespace = "com.Aspenini.MemoryPak"
    compileSdk = 35

    defaultConfig {
        applicationId = "com.Aspenini.MemoryPak"
        // Slint's Android backend currently requires Android 8 / API 26.
        minSdk = 26
        targetSdk = 35
        versionCode = 3
        versionName = "0.3.0"

        ndk {
            abiFilters += listOf("arm64-v8a", "x86_64")
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro",
            )
        }
    }

    sourceSets["main"].res.srcDir("../../../icons/android/res")
}

val rustOutput = layout.projectDirectory.dir("src/main/jniLibs")

tasks.register<Exec>("buildRustDebug") {
    workingDir("../../..")
    commandLine(
        "cargo", "ndk",
        "-t", "arm64-v8a",
        "-t", "x86_64",
        "-o", rustOutput.asFile.absolutePath,
        "build", "-p", "memory_pak_app",
    )
}

tasks.register<Exec>("buildRustRelease") {
    workingDir("../../..")
    commandLine(
        "cargo", "ndk",
        "-t", "arm64-v8a",
        "-t", "x86_64",
        "-o", rustOutput.asFile.absolutePath,
        "build", "--release", "-p", "memory_pak_app",
    )
}

tasks.named("preDebugBuild").configure { dependsOn("buildRustDebug") }
tasks.named("preReleaseBuild").configure { dependsOn("buildRustRelease") }
