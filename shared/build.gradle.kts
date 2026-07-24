plugins {
    kotlin("multiplatform")
    id("org.jetbrains.compose") version "1.5.11"
}

kotlin {
    jvm()
    linuxX64()

    sourceSets {
        val commonMain by getting {
            dependencies {
                implementation(compose.runtime)
                implementation(compose.foundation)
                implementation(compose.material)
            }
        }
        val jvmMain by getting {
            dependencies {
                implementation(compose.desktop.currentOs)
            }
        }
        val linuxX64Main by getting {
            dependsOn(commonMain)
            dependencies {
                implementation(compose.desktop.currentOs)
                // Wire generated UniFFI Kotlin bindings from desktop_bridge
                compileOnly(files("../desktop_bridge/target/generated-sources/uniffi"))
            }
            // Include generated sources in compilation
            kotlin.srcDir("../desktop_bridge/target/generated-sources/uniffi")
        }
    }
}

compose.desktop {
    application {
        mainClass = "com.scmessenger.shared.MainKt"
    }
}