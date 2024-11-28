plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.compose.compiler)
}

android {
    namespace = "com.example.androidMic"
    compileSdk = 35

    defaultConfig {
        applicationId = "com.example.androidMic"
        minSdk = 23
        targetSdk = 35
        versionCode = 9
        versionName = "2.0"
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"

        vectorDrawables.useSupportLibrary = true
        resourceConfigurations.addAll(
            listOf(
                "en",
                "fr"
            )
        )

        externalNativeBuild {
            cmake {
                arguments += "-DANDROID_STL=c++_shared"
            }
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

        create("releaseTesting") {
            initWith(buildTypes["release"])
            applicationIdSuffix = ".testing"
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

    externalNativeBuild {
        cmake {
            path = file("src/main/cpp/CMakeLists.txt")
        }
    }

    lint {
        abortOnError = false
        checkReleaseBuilds = false
    }
}

dependencies {

    // audio lib
    implementation(libs.oboe)

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
    implementation(libs.compose.constraintlayout)

    // compose permission
    implementation(libs.accompanist.permissions)

    // Compose Debug
    implementation(libs.compose.ui.preview)
    debugImplementation(libs.androidx.ui.tooling)

    // unit test
    testImplementation(libs.test.junit.ktx)

    // integration test
    androidTestImplementation(composeBom)
    androidTestImplementation(libs.test.junit.ktx)
    androidTestImplementation(libs.kotlinx.coroutines.test)
    androidTestImplementation(libs.androidx.runner)
}