package com.scmessenger.android.ui.viewmodels

import android.content.Context
import com.scmessenger.android.data.MeshRepository
import com.scmessenger.android.data.PreferencesRepository
import io.mockk.*
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.*
import org.junit.After
import org.junit.Before
import org.junit.Test
import org.junit.Assert.*

/**
 * Unit tests for MeshServiceViewModel.
 */
@OptIn(ExperimentalCoroutinesApi::class)
class MeshServiceViewModelTest {
    
    private lateinit var viewModel: MeshServiceViewModel
    private lateinit var mockContext: Context
    private lateinit var mockMeshRepository: MeshRepository
    private lateinit var mockPreferencesRepository: PreferencesRepository
    
    private val testDispatcher = StandardTestDispatcher()
    
    @Before
    fun setup() {
        Dispatchers.setMain(testDispatcher)
        
        mockContext = mockk(relaxed = true)
        mockMeshRepository = mockk(relaxed = true)
        mockPreferencesRepository = mockk(relaxed = true)
        
        // Setup default flows
        every { mockMeshRepository.serviceState } returns MutableStateFlow(uniffi.api.ServiceState.STOPPED)
        every { mockMeshRepository.serviceStats } returns MutableStateFlow(null)
        every { mockPreferencesRepository.serviceAutoStart } returns MutableStateFlow(false)
        
        viewModel = MeshServiceViewModel(mockContext, mockMeshRepository, mockPreferencesRepository)
    }
    
    @After
    fun tearDown() {
        Dispatchers.resetMain()
    }
    
    @Test
    fun `initial state is STOPPED`() = runTest {
        // When
        val state = viewModel.serviceState.value
        
        // Then
        assertEquals(uniffi.api.ServiceState.STOPPED, state)
    }
    
    @Test
    fun `isRunning is false when state is STOPPED`() = runTest {
        // When
        val isRunning = viewModel.isRunning.value
        
        // Then
        assertFalse(isRunning)
    }
    
    @Test
    fun `isRunning is true when state is RUNNING`() = runTest {
        // Given
        every { mockMeshRepository.serviceState } returns MutableStateFlow(uniffi.api.ServiceState.RUNNING)
        viewModel = MeshServiceViewModel(mockContext, mockMeshRepository, mockPreferencesRepository)
        
        // When
        testDispatcher.scheduler.advanceUntilIdle()
        val isRunning = viewModel.isRunning.value
        
        // Then
        assertTrue(isRunning)
    }
    
    @Test
    fun `setAutoStart updates preference`() = runTest {
        // Given
        coEvery { mockPreferencesRepository.setServiceAutoStart(any()) } just runs
        
        // When
        viewModel.setAutoStart(true)
        testDispatcher.scheduler.advanceUntilIdle()
        
        // Then
        coVerify { mockPreferencesRepository.setServiceAutoStart(true) }
    }
    
    @Test
    fun `getStatsText returns formatted stats`() {
        // Given
        val stats = uniffi.api.ServiceStats(
            peersDiscovered = 5u,
            messagesRelayed = 10u,
            bytesTransferred = 1024u,
            uptimeSecs = 3600u
        )
        every { mockMeshRepository.serviceStats } returns MutableStateFlow(stats)
        viewModel = MeshServiceViewModel(mockContext, mockMeshRepository, mockPreferencesRepository)
        
        // When
        val statsText = viewModel.getStatsText()
        
        // Then
        assertTrue(statsText.contains("Peers Discovered: 5"))
        assertTrue(statsText.contains("Messages Relayed: 10"))
        assertTrue(statsText.contains("Bytes Transferred"))
        assertTrue(statsText.contains("Uptime"))
    }
    
    @Test
    fun `getStatsText handles null stats`() {
        // Given
        every { mockMeshRepository.serviceStats } returns MutableStateFlow(null)
        viewModel = MeshServiceViewModel(mockContext, mockMeshRepository, mockPreferencesRepository)
        
        // When
        val statsText = viewModel.getStatsText()
        
        // Then
        assertEquals("No stats available", statsText)
    }
}
