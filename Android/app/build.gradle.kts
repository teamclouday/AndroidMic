import com.google.protobuf.gradle.*

plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.compose.compiler)
    alias(libs.plugins.google.protobuf)
}

android {
    namespace = "io.github.teamclouday.AndroidMic"
    compileSdk = 35

    defaultConfig {
        applicationId = "io.github.teamclouday.AndroidMic"
        minSdk = 23
        targetSdk = 35
        versionCode = 10
        versionName = "2.1.7"
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"

        vectorDrawables.useSupportLibrary = true
        androidResources {
            localeFilters += listOf("en", "fr")
        }

    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
            signingConfig = signingConfigs.getByName("debug")
        }

        debug {
            applicationIdSuffix = ".debug"
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_21
        targetCompatibility = JavaVersion.VERSION_21
    }

    kotlinOptions {
        jvmTarget = "21"
    }
    buildFeatures {
        prefab = true
        compose = true
    }

    lint {
        abortOnError = false
        checkReleaseBuilds = false
    }

//    packaging {
//        resources.excludes.add("google/protobuf/*.proto")
//    }

    sourceSets.getByName("main").resources.srcDir("src/main/proto")
}

protobuf {
    protoc {
        artifact = "com.google.protobuf:protoc:3.25.5"
    }

    generateProtoTasks {
        all().forEach { task ->
            task.builtins {
                id("java") {
                    option("lite")
                }
            }
        }
    }
}

dependencies {
    // AndroidX Core
    implementation(libs.androidx.ktx)
    implementation(libs.androidx.viewmodel.compose)
    implementation(libs.runtime.ktx)
    implementation(libs.runtime.compose)
    implementation(libs.compose.activity)
    implementation(libs.datastore.preferences)


    val composeBom = platform(libs.compose.bom)

    // Compose
    implementation(composeBom)
    implementation(libs.compose.ui)
    implementation(libs.compose.material)
    implementation(libs.compose.material3)
    implementation(libs.compose.material.icons.extended)

    // compose permission
    implementation(libs.accompanist.permissions)

    // Compose Debug
    implementation(libs.compose.ui.preview)
    debugImplementation(libs.androidx.ui.tooling)

    // Streaming
    implementation(libs.protobuf.java.lite)
    implementation(libs.protobuf.gradle.plugin)

    // unit test
    testImplementation(libs.test.junit.ktx)

    // integration test
    androidTestImplementation(composeBom)
    androidTestImplementation(libs.test.junit.ktx)
    androidTestImplementation(libs.kotlinx.coroutines.test)
    androidTestImplementation(libs.androidx.runner)
}