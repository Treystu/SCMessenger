package com.scmessenger.android.di

import android.content.Context
import com.scmessenger.android.data.MeshRepository
import com.scmessenger.android.data.PreferencesRepository
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
 */
@Module
@InstallIn(SingletonComponent::class)
object AppModule {

    @Provides
    @Singleton
    fun provideMeshRepository(
        @ApplicationContext context: Context
    ): MeshRepository {
        return MeshRepository(context)
    }

    @Provides
    @Singleton
    fun providePreferencesRepository(
        @ApplicationContext context: Context
    ): PreferencesRepository {
        return PreferencesRepository(context)
    }
}
