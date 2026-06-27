plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

android {
    namespace = "com.cqrs.wrapper"
    compileSdk = 35

    defaultConfig {
        applicationId = "com.cqrs.wrapper"
        minSdk = 26
        targetSdk = 35
        versionCode = 1
        versionName = "0.1.0"
    }
}
