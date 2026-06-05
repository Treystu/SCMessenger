# =============================================================================
# SCMessenger ProGuard / R8 rules
#
# Goals:
#   - Preserve all UniFFI (Rust FFI) classes intact.
#   - Preserve Hilt-generated classes.
#   - Preserve Kotlinx Serialization classes (none in use yet, but ready).
#   - Keep Timber call sites working (it already uses androidx.startup in
#     modern versions; no special rules required, but we still keep
#     -keepattributes so reflection-based loggers survive).
#   - Keep Compose runtime classes.
#   - Keep useful crash info: source file + line number table.
#   - Suppress harmless warnings (bouncycastle, javax.annotation) that may
#     appear in transitive dependencies.
# =============================================================================

# -----------------------------------------------------------------------------
# Attributes — line numbers for stack traces, annotations for serialization
# and Hilt-generated code, plus inner-class descriptors.
# -----------------------------------------------------------------------------
-keepattributes SourceFile,LineNumberTable
-keepattributes *Annotation*
-keepattributes InnerClasses,EnclosingMethod
-keepattributes Signature,Exceptions

# Map the source file to an empty value so we still get a "SourceFile" entry
# for crash deobfuscation tools (e.g. retrace) without leaking the original
# path in the APK.
-renamesourcefileattribute SourceFile

# -----------------------------------------------------------------------------
# UniFFI (Rust FFI) — keep every class in the generated package.
# UniFFI generates classes that are invoked by name from JNI; renaming them
# would break the binding at runtime.
# -----------------------------------------------------------------------------
-keep class uniffi.api.** { *; }
-keep interface uniffi.api.** { *; }
-keep class uniffi.** { *; }
-keep interface uniffi.** { *; }

# Keep classes used as FFI callbacks (PlatformBridge implementations,
# JNI bridge entries, etc.).
-keep class * implements uniffi.api.PlatformBridge { *; }
-keep class * implements uniffi.api.CoreDelegate { *; }
-keep class * implements uniffi.api.MeshCoreDelegate { *; }

# -----------------------------------------------------------------------------
# JNA — UniFFI uses JNA to call into the native Rust shared library.
# -----------------------------------------------------------------------------
-keep class com.sun.jna.** { *; }
-keep class * implements com.sun.jna.** { *; }
-keepclassmembers class * {
    native <methods>;
}
-dontwarn java.awt.**
-dontwarn com.sun.jna.**

# -----------------------------------------------------------------------------
# Hilt / Dagger — generated components must not be renamed.
# -----------------------------------------------------------------------------
-keep class dagger.hilt.** { *; }
-keep class * extends dagger.hilt.android.internal.managers.ViewComponentManager$FragmentContextWrapper { *; }
-keep @dagger.hilt.android.lifecycle.HiltViewModel class * extends androidx.lifecycle.ViewModel { *; }
-keep @dagger.hilt.android.AndroidEntryPoint class * { *; }
-keep @dagger.hilt.android.HiltAndroidApp class * { *; }
-keep class **_HiltModules$* { *; }
-keep class **_HiltModules { *; }
-keep class hilt_aggregated_deps.** { *; }
-keep class dagger.hilt.internal.aggregatedroot.codegen.** { *; }
-keep class dagger.hilt.internal.processedrootsentinel.codegen.** { *; }

# -----------------------------------------------------------------------------
# Timber — keep call sites, but the library itself has no special requirements.
# -----------------------------------------------------------------------------
-keep class timber.log.** { *; }
-dontwarn timber.log.**

# -----------------------------------------------------------------------------
# Kotlinx Serialization — none in active use yet, but if/when added the
# generated $$serializer classes must survive shrinking.
# -----------------------------------------------------------------------------
-keepclassmembers @kotlinx.serialization.Serializable class * {
    *** Companion;
}
-keepclasseswithmembers class ** {
    kotlinx.serialization.KSerializer serializer(...);
}
-keep,includedescriptorclasses class com.scmessenger.**$$serializer { *; }
-keepclassmembers class com.scmessenger.** {
    *** Companion;
    kotlinx.serialization.KSerializer serializer(...);
}

# -----------------------------------------------------------------------------
# Compose — default rules cover most of it; preserve a few extra surfaces.
# -----------------------------------------------------------------------------
-keep class androidx.compose.** { *; }
-keep class kotlin.Metadata { *; }
-keep class kotlin.coroutines.Continuation
-keep class kotlin.reflect.** { *; }

# Compose Stable / Immutable annotations.
-keepclassmembers class * {
    @androidx.compose.runtime.Stable *;
    @androidx.compose.runtime.Immutable *;
}

# -----------------------------------------------------------------------------
# DataStore Preferences — uses generated proto serializers.
# -----------------------------------------------------------------------------
-keep class androidx.datastore.** { *; }
-dontwarn androidx.datastore.**

# -----------------------------------------------------------------------------
# Coroutines — keep internal service loader entries.
# -----------------------------------------------------------------------------
-keepnames class kotlinx.coroutines.internal.MainDispatcherFactory {}
-keepnames class kotlinx.coroutines.CoroutineExceptionHandler {}
-keepclassmembernames class kotlinx.** {
    volatile <fields>;
}
-dontwarn kotlinx.coroutines.**

# -----------------------------------------------------------------------------
# Third-party — suppress noisy warnings from transitive libraries.
# -----------------------------------------------------------------------------
-dontwarn org.bouncycastle.**
-dontwarn javax.annotation.**
-dontwarn org.conscrypt.**
-dontwarn org.openjsse.**

# -----------------------------------------------------------------------------
# JNA (UniFFI) callbacks require reflective access to interface methods.
# -----------------------------------------------------------------------------
-keepclassmembers interface * {
    @androidx.annotation.Keep *;
}
