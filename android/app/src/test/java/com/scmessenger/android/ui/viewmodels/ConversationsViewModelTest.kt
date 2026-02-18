package com.scmessenger.android.ui.viewmodels

import com.scmessenger.android.data.MeshRepository
import io.mockk.*
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.*
import org.junit.After
import org.junit.Before
import org.junit.Test
import org.junit.Assert.*
import uniffi.api.MessageRecord
import uniffi.api.MessageDirection
import uniffi.api.HistoryStats
import java.util.UUID

@OptIn(ExperimentalCoroutinesApi::class)
class ConversationsViewModelTest {

    private lateinit var viewModel: ConversationsViewModel
    private lateinit var mockMeshRepository: MeshRepository
    private val testDispatcher = StandardTestDispatcher()
    
    // Mock flow for message updates
    private val messageUpdatesFlow = MutableSharedFlow<MessageRecord>(replay = 0)

    @Before
    fun setup() {
        Dispatchers.setMain(testDispatcher)
        mockMeshRepository = mockk(relaxed = true)

        // Mock message updates flow
        every { mockMeshRepository.messageUpdates } returns messageUpdatesFlow
        
        // Mock getRecentMessages to return a list (empty initially, then populated)
        every { mockMeshRepository.getRecentMessages(any(), any()) } returns emptyList()
        
        // Mock getHistoryStats to perform cleanly
        every { mockMeshRepository.getHistoryStats() } returns HistoryStats(0u, 0u, 0u, 0u)
        
        viewModel = ConversationsViewModel(mockMeshRepository)
    }

    @After
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun `viewModel loads messages on initialization`() = runTest {
        // Run any pending coroutines (init block)
        testDispatcher.scheduler.advanceUntilIdle()
        
        // Verify loadMessages was called with any args
        verify { mockMeshRepository.getRecentMessages(any(), any()) }
    }

    @Test
    fun `viewModel reloads messages when a new message update is received`() = runTest {
        // Given viewModel is initialized
        testDispatcher.scheduler.advanceUntilIdle()
        clearMocks(mockMeshRepository, answers = false) // Clear previous calls but keep mocks

        // Setup mock to return a message now
        val testMessage = MessageRecord(
            id = UUID.randomUUID().toString(),
            direction = MessageDirection.SENT,
            peerId = "peer1",
            content = "Hello",
            timestamp = 1000uL,
            delivered = true
        )
        every { mockMeshRepository.getRecentMessages(any(), any()) } returns listOf(testMessage)

        // When a message update is emitted
        messageUpdatesFlow.emit(testMessage)
        testDispatcher.scheduler.advanceUntilIdle()

        // Then verify getRecentMessages is called again (reloading list)
        verify(exactly = 1) { mockMeshRepository.getRecentMessages(any(), any()) }
        
        // And verify state is updated
        assertEquals(1, viewModel.messages.value.size)
        assertEquals("Hello", viewModel.messages.value[0].content)
    }
}
