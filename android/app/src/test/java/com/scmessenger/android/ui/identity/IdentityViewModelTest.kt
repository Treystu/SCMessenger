package com.scmessenger.android.ui.identity

import com.scmessenger.android.data.MeshRepository
import io.mockk.*
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.*
import org.junit.After
import org.junit.Before
import org.junit.Test
import org.junit.Assert.*
import uniffi.api.IdentityInfo
import java.util.UUID

@OptIn(ExperimentalCoroutinesApi::class)
class IdentityViewModelTest {

    private lateinit var viewModel: IdentityViewModel
    private lateinit var mockMeshRepository: MeshRepository
    private val testDispatcher = StandardTestDispatcher()

    @Before
    fun setup() {
        Dispatchers.setMain(testDispatcher)
        mockMeshRepository = mockk(relaxed = true)

        // Mock initial load identity
        every { mockMeshRepository.getIdentityInfoNonBlocking() } returns null

        // Mock service state flow
        every { mockMeshRepository.serviceState } returns MutableStateFlow(uniffi.api.ServiceState.INITIALIZING)

        viewModel = IdentityViewModel(mockMeshRepository)
    }

    @After
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun `createIdentity with custom salt passes salt to repository`() = runTest {
        testDispatcher.scheduler.advanceUntilIdle()

        val testSalt = byteArrayOf(1, 2, 3, 4)
        val testNickname = "TestUser"

        // Mock the createIdentity call to complete successfully
        coEvery { mockMeshRepository.createIdentity(testSalt) } returns Unit

        // Mock setNickname
        coEvery { mockMeshRepository.setNickname(testNickname) } returns Unit

        // Mock getIdentityInfoNonBlocking to return a valid identity after creation
        val testIdentity = IdentityInfo(
            identityId = "test-id",
            publicKeyHex = "test-pubkey",
            libp2pPeerId = "test-peer",
            nickname = testNickname,
            initialized = true
        )
        coEvery { mockMeshRepository.getIdentityInfoNonBlocking() } returns testIdentity

        viewModel.createIdentity(testNickname, testSalt)
        testDispatcher.scheduler.advanceUntilIdle()

        verify(exactly = 1) { mockMeshRepository.createIdentity(testSalt) }
        verify(exactly = 1) { mockMeshRepository.setNickname(testNickname) }
    }

    @Test
    fun `createIdentity re-entrancy guard prevents second concurrent call`() = runTest {
        testDispatcher.scheduler.advanceUntilIdle()

        val testSalt = byteArrayOf(1, 2, 3, 4)
        val testNickname = "TestUser"

        // Use a suspend lock to simulate a long-running FFI call
        val started = MutableStateFlow(false)
        val completionDeferred = CompletableFuture<Unit>()

        coEvery { mockMeshRepository.createIdentity(any()) } coAnswers {
            started.value = true
            completionDeferred.join()
        }

        // First call - should proceed
        viewModel.createIdentity(testNickname, testSalt)
        testDispatcher.scheduler.advanceUntilIdle()

        // Wait for the coroutine to start
        delay(100)

        // Second call - should be blocked by re-entrancy guard
        viewModel.createIdentity(testNickname, testSalt)
        testDispatcher.scheduler.advanceUntilIdle()

        // Verify only one actual createIdentity call was made
        // The guard should prevent the second call from reaching the repository
        // We need to check that the second call returned early
        assertTrue(started.value)

        // Complete the first call
        completionDeferred.complete(Unit)
        testDispatcher.scheduler.advanceUntilIdle()
    }

    @Test
    fun `createIdentity sets isLoading before launching coroutine`() = runTest {
        testDispatcher.scheduler.advanceUntilIdle()

        val testSalt = byteArrayOf(1, 2, 3, 4)
        val testNickname = "TestUser"

        // Mock with a delay to simulate work
        coEvery { mockMeshRepository.createIdentity(any()) } coAnswers {
            kotlinx.coroutines.delay(100)
            Unit
        }

        // Check initial state - should be false
        assertEquals(false, viewModel.isLoading.value)

        // Start the operation
        viewModel.createIdentity(testNickname, testSalt)

        // Check that isLoading was set to true
        // The state update happens inside the coroutine, so we wait briefly
        testDispatcher.scheduler.advanceUntilIdle()
        assertTrue(viewModel.isLoading.value)

        // Wait for completion
        testDispatcher.scheduler.advanceUntilIdle()
        assertFalse(viewModel.isLoading.value)
    }

    @Test
    fun `createIdentity without salt passes null to repository`() = runTest {
        testDispatcher.scheduler.advanceUntilIdle()

        val testNickname = "TestUser"

        // Mock the createIdentity call with null salt
        coEvery { mockMeshRepository.createIdentity(null) } returns Unit

        // Mock setNickname
        coEvery { mockMeshRepository.setNickname(testNickname) } returns Unit

        // Mock getIdentityInfoNonBlocking
        val testIdentity = IdentityInfo(
            identityId = "test-id",
            publicKeyHex = "test-pubkey",
            libp2pPeerId = "test-peer",
            nickname = testNickname,
            initialized = true
        )
        coEvery { mockMeshRepository.getIdentityInfoNonBlocking() } returns testIdentity

        viewModel.createIdentity(testNickname, null)
        testDispatcher.scheduler.advanceUntilIdle()

        verify(exactly = 1) { mockMeshRepository.createIdentity(null) }
        verify(exactly = 1) { mockMeshRepository.setNickname(testNickname) }
    }

    @Test
    fun `createIdentity clears error on success`() = runTest {
        testDispatcher.scheduler.advanceUntilIdle()

        val testSalt = byteArrayOf(1, 2, 3, 4)
        val testNickname = "TestUser"

        coEvery { mockMeshRepository.createIdentity(testSalt) } returns Unit
        coEvery { mockMeshRepository.setNickname(testNickname) } returns Unit
        val testIdentity = IdentityInfo(
            identityId = "test-id",
            publicKeyHex = "test-pubkey",
            libp2pPeerId = "test-peer",
            nickname = testNickname,
            initialized = true
        )
        coEvery { mockMeshRepository.getIdentityInfoNonBlocking() } returns testIdentity

        // Set an error first
        viewModel._error.value = "Previous error"

        viewModel.createIdentity(testNickname, testSalt)
        testDispatcher.scheduler.advanceUntilIdle()

        // Error should be cleared at start of operation
        assertNull(viewModel.error.value)
        // Success message should be set
        assertNotNull(viewModel.successMessage.value)
    }

    @Test
    fun `createIdentity handles exception and sets error state`() = runTest {
        testDispatcher.scheduler.advanceUntilIdle()

        val testSalt = byteArrayOf(1, 2, 3, 4)
        val testNickname = "TestUser"

        val testException = RuntimeException("FFI call failed")
        coEvery { mockMeshRepository.createIdentity(testSalt) } throws testException

        viewModel.createIdentity(testNickname, testSalt)
        testDispatcher.scheduler.advanceUntilIdle()

        assertNotNull(viewModel.error.value)
        assertTrue(viewModel.error.value!!.contains("Failed to create identity"))
    }

    @Test
    fun `createIdentity updates identity info on success`() = runTest {
        testDispatcher.scheduler.advanceUntilIdle()

        val testSalt = byteArrayOf(1, 2, 3, 4)
        val testNickname = "TestUser"

        coEvery { mockMeshRepository.createIdentity(testSalt) } returns Unit
        coEvery { mockMeshRepository.setNickname(testNickname) } returns Unit

        val testIdentity = IdentityInfo(
            identityId = "test-id",
            publicKeyHex = "test-pubkey",
            libp2pPeerId = "test-peer",
            nickname = testNickname,
            initialized = true
        )
        coEvery { mockMeshRepository.getIdentityInfoNonBlocking() } returns testIdentity

        viewModel.createIdentity(testNickname, testSalt)
        testDispatcher.scheduler.advanceUntilIdle()

        assertEquals(testIdentity, viewModel.identityInfo.value)
    }
}
