package com.scmessenger.android.ui.identity

import com.scmessenger.android.data.IdentityCreationCoordinator
import com.scmessenger.android.data.IdentityState
import com.scmessenger.android.data.MeshRepository
import com.scmessenger.android.ui.viewmodels.IdentityViewModel
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class IdentityViewModelTest {

    private lateinit var viewModel: IdentityViewModel
    private lateinit var mockMeshRepository: MeshRepository
    private lateinit var mockCoordinator: IdentityCreationCoordinator
    private val testDispatcher = StandardTestDispatcher()

    @Before
    fun setup() {
        Dispatchers.setMain(testDispatcher)
        mockMeshRepository = mockk(relaxed = true)
        mockCoordinator = mockk(relaxed = true)

        // Mock repository flows
        every { mockMeshRepository.identityInfo } returns MutableStateFlow(null)
        every { mockMeshRepository.serviceState } returns MutableStateFlow(uniffi.api.ServiceState.STARTING)

        // Mock coordinator flows
        every { mockCoordinator.identityState } returns MutableStateFlow(IdentityState.None)
        every { mockCoordinator.error } returns MutableStateFlow(null)
        every { mockCoordinator.progressStage } returns MutableStateFlow(com.scmessenger.android.ui.viewmodels.IdentityProgressStage.Idle)
        every { mockCoordinator.progressSubDetail } returns MutableStateFlow(null)

        viewModel = IdentityViewModel(mockMeshRepository, mockCoordinator)
    }

    @After
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun `createIdentity delegates to coordinator`() = runTest {
        val testNickname = "TestUser"
        
        coEvery { mockCoordinator.createIdentity(testNickname) } returns true

        viewModel.createIdentity(testNickname)
        testDispatcher.scheduler.advanceUntilIdle()

        coVerify(exactly = 1) { mockCoordinator.createIdentity(testNickname) }
        assertEquals("Identity created successfully", viewModel.successMessage.value)
    }

    @Test
    fun `isLoading is true when coordinator state is Restoring`() = runTest {
        val identityStateFlow = MutableStateFlow<IdentityState>(IdentityState.None)
        every { mockCoordinator.identityState } returns identityStateFlow
        
        viewModel = IdentityViewModel(mockMeshRepository, mockCoordinator)
        
        // Subscribe to activate the combine stateIn
        val collectJob = launch(UnconfinedTestDispatcher(testScheduler)) {
            viewModel.isLoading.collect {}
        }
        
        assertFalse(viewModel.isLoading.value)
        
        identityStateFlow.value = IdentityState.Restoring
        testDispatcher.scheduler.advanceUntilIdle()
        
        assertTrue(viewModel.isLoading.value)
        
        collectJob.cancel()
    }
}
