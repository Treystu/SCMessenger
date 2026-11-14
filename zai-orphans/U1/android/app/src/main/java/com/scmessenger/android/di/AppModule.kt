package com.scmessenger.android.di

import android.content.Context
import com.scmessenger.android.data.MeshRepository
import com.scmessenger.android.data.PreferencesRepository
import com.scmessenger.android.network.DiagnosticsReporter
import com.scmessenger.android.network.NetworkDiagnostics
import com.scmessenger.android.network.NetworkTypeDetector
import com.scmessenger.android.transport.NetworkDetector
import com.scmessenger.android.utils.CircuitBreaker
import com.scmessenger.android.utils.NetworkFailureMetrics
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

/**
 * Hilt module providing application-level dependencies.
 *
 * This module provides:
 * - MeshRepository: Interface to Rust core via UniFFI
 * - PreferencesRepository: Android preferences storage
 * - Network-related diagnostics and reporting
 */
@Module
@InstallIn(SingletonComponent::class)
object AppModule {

    @Provides
    @Singleton
    fun provideMeshRepository(
        @ApplicationContext context: Context,
        preferencesRepository: PreferencesRepository
    ): MeshRepository {
        return MeshRepository(context, preferencesRepository)
    }

    @Provides
    @Singleton
    fun providePreferencesRepository(
        @ApplicationContext context: Context
    ): PreferencesRepository {
        return PreferencesRepository(context)
    }

    @Provides
    @Singleton
    fun provideIdentityCreationCoordinator(
        meshRepository: MeshRepository,
        preferencesRepository: PreferencesRepository
    ): com.scmessenger.android.data.IdentityCreationCoordinator {
        return com.scmessenger.android.data.IdentityCreationCoordinator(meshRepository, preferencesRepository)
    }

    @Provides
    @Singleton
    fun provideNetworkDiagnostics(
        @ApplicationContext context: Context
    ): NetworkDiagnostics {
        return NetworkDiagnostics(context)
    }

    @Provides
    @Singleton
    fun provideNetworkTypeDetector(
        @ApplicationContext context: Context
    ): NetworkTypeDetector {
        return NetworkTypeDetector(context)
    }

    @Provides
    @Singleton
    fun provideNetworkDetector(
        @ApplicationContext context: Context
    ): NetworkDetector {
        return NetworkDetector(context)
    }

    @Provides
    @Singleton
    fun provideCircuitBreaker(): CircuitBreaker {
        return CircuitBreaker()
    }

    @Provides
    @Singleton
    fun provideNetworkFailureMetrics(): NetworkFailureMetrics {
        return NetworkFailureMetrics()
    }

    @Provides
    @Singleton
    fun provideDiagnosticsReporter(
        @ApplicationContext context: Context,
        networkDiagnostics: NetworkDiagnostics,
        networkTypeDetector: NetworkTypeDetector,
        failureMetrics: NetworkFailureMetrics,
        networkDetector: NetworkDetector,
        circuitBreaker: CircuitBreaker
    ): DiagnosticsReporter {
        return DiagnosticsReporter(
            context,
            networkDiagnostics,
            networkTypeDetector,
            failureMetrics,
            networkDetector,
            circuitBreaker
        )
    }
}
