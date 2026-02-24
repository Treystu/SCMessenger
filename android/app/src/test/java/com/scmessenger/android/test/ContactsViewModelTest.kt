package com.scmessenger.android.test

import com.scmessenger.android.data.MeshRepository
import com.scmessenger.android.service.MeshEventBus
import com.scmessenger.android.service.PeerEvent
import com.scmessenger.android.service.TransportType
import com.scmessenger.android.ui.viewmodels.ContactsViewModel
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlinx.coroutines.launch
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class ContactsViewModelTest {

    private lateinit var viewModel: ContactsViewModel
    private lateinit var repository: MeshRepository
    private val testDispatcher = StandardTestDispatcher()

    private fun contact(peerId: String, nickname: String? = null): uniffi.api.Contact {
        return uniffi.api.Contact(
            peerId = peerId,
            nickname = nickname,
            localNickname = null,
            publicKey = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff",
            addedAt = 1u,
            lastSeen = null,
            notes = null
        )
    }

    @Before
    fun setup() {
        Dispatchers.setMain(testDispatcher)
        repository = mockk(relaxed = true)
        every { repository.listContacts() } returns listOf(
            contact("peer-alice", "Alice"),
            contact("peer-bob", "Bob")
        )
        viewModel = ContactsViewModel(repository)
    }

    @After
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun `loadContacts populates contacts state`() = runTest {
        advanceUntilIdle()
        assertEquals(2, viewModel.contacts.value.size)
        assertEquals("Alice", viewModel.contacts.value.first().nickname)
    }

    @Test
    fun `addContact validates key and persists contact`() = runTest {
        every { repository.addContact(any()) } returns Unit
        every { repository.connectToPeer(any(), any()) } returns Unit

        viewModel.addContact(
            peerId = "peer-new",
            publicKey = "a".repeat(64),
            nickname = "New",
            libp2pPeerId = "12D3KooWxyz",
            listeners = listOf("/ip4/1.2.3.4/tcp/4001")
        )
        advanceUntilIdle()

        verify(exactly = 1) { repository.addContact(any()) }
        verify(exactly = 1) { repository.connectToPeer("12D3KooWxyz", any()) }
    }

    @Test
    fun `removeContact forwards removal to repository`() = runTest {
        every { repository.removeContact(any()) } returns Unit

        viewModel.removeContact("peer-alice")
        advanceUntilIdle()

        verify(exactly = 1) { repository.removeContact("peer-alice") }
    }

    @Test
    fun `search filters contacts by nickname`() = runTest {
        val collectJob = launch { viewModel.filteredContacts.collect() }
        advanceUntilIdle()

        viewModel.setSearchQuery("Alice")
        advanceUntilIdle()

        assertEquals(1, viewModel.filteredContacts.value.size)
        assertEquals("peer-alice", viewModel.filteredContacts.value.first().peerId)
        collectJob.cancel()
    }

    @Test
    fun `online status updates nearby peers from peer events`() = runTest {
        every { repository.replayDiscoveredPeerEvents() } returns Unit
        val nearbyJob = launch { viewModel.nearbyPeers.collect() }

        MeshEventBus.emitPeerEvent(PeerEvent.Discovered("peer-online", TransportType.BLE))
        advanceUntilIdle()
        assertTrue(viewModel.nearbyPeers.value.any { it.peerId == "peer-online" && it.isOnline })

        MeshEventBus.emitPeerEvent(PeerEvent.Disconnected("peer-online"))
        advanceUntilIdle()
        assertTrue(viewModel.nearbyPeers.value.none { it.peerId == "peer-online" && it.isOnline })
        nearbyJob.cancel()
    }
}
