package com.scmessenger.android.test

import android.content.Context
import android.content.SharedPreferences
import android.net.ConnectivityManager
import com.scmessenger.android.data.MeshRepository
import com.scmessenger.android.transport.SmartTransportRouter
import io.mockk.Awaits
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.just
import io.mockk.mockk
import io.mockk.runs
import io.mockk.slot
import io.mockk.verify
import java.io.File
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Assume
import org.junit.BeforeClass
import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertNotNull
import org.junit.Test
import uniffi.api.ContactManager
import uniffi.api.CoreDelegate
import uniffi.api.HistoryManager
import uniffi.api.IdentityInfo
import uniffi.api.IronCore
import uniffi.api.MeshService
import uniffi.api.MeshServiceConfig
import uniffi.api.MessageDirection
import uniffi.api.MessageRecord
import uniffi.api.MessageStatus
import uniffi.api.ServiceState
import uniffi.api.SwarmBridge

/**
 * Hermetic JVM unit tests locking the Android receipt unification contract.
 *
 * The production receipt callback (CoreDelegate.onReceiptReceived) is only
 * reachable by going through MeshRepository.startMeshService(), so TEST A wires
 * a partial service start with mocked native dependencies and then exercises the
 * callback directly. TEST B drives the closest reachable seam of the send path
 * because sendDeliveryReceiptAsync() is private and tightly coupled to transport
 * internals; the core receipt bytes and their hand-off to the transport layer are
 * asserted via a mocked SwarmBridge.
 */
class ReceiptUnificationTest {

    private val testRoot = File(System.getProperty("user.dir") ?: ".", "build/tmp/receipt-unification-tests")

    init {
        testRoot.mkdirs()
    }

    companion object {
        @JvmStatic
        @BeforeClass
        fun checkNative() {
            val hostLibDir = File("../../target/debug").absoluteFile
            val libName = System.mapLibraryName("scmessenger_core")
            val libFile = File(hostLibDir, libName)
            Assume.assumeTrue(
                "host native core lib not built; skipping (same convention as UniffiIntegrationTest)",
                libFile.exists()
            )
            System.setProperty("jna.library.path", hostLibDir.absolutePath)
        }
    }

    private fun freshFilesDir(): File {
        val dir = File(testRoot, "test-${System.nanoTime()}")
        dir.mkdirs()
        return dir
    }

    private fun fakeContext(filesDir: File): Context {
        return mockk<Context>(relaxed = true) {
            every { this@mockk.filesDir } returns filesDir
            every { getSystemService(Context.CONNECTIVITY_SERVICE) } returns mockk<ConnectivityManager>(relaxed = true)
            every { getSharedPreferences(any(), any()) } returns mockk<SharedPreferences>(relaxed = true)
        }
    }

    private fun setField(target: Any, name: String, value: Any?) {
        val field = target::class.java.getDeclaredField(name)
        field.isAccessible = true
        field.set(target, value)
    }

    @Suppress("UNCHECKED_CAST")
    private fun <T> getField(target: Any, name: String): T? {
        val field = target::class.java.getDeclaredField(name)
        field.isAccessible = true
        return field.get(target) as? T
    }

    private fun writePendingOutbox(filesDir: File, messageId: String, peerId: String) {
        val file = File(filesDir, "pending_outbox.json")
        file.writeText(
            """
            [{
                "queue_id": "q-$messageId",
                "history_record_id": "$messageId",
                "peer_id": "$peerId",
                "route_peer_id": null,
                "listeners": [],
                "envelope_b64": "eA==",
                "created_at": 1,
                "attempt_count": 0,
                "next_attempt_at": 0
            }]
            """.trimIndent()
        )
    }

    private fun callSendDeliveryReceiptAsync(repo: MeshRepository, senderKey: String, messageId: String, senderId: String) {
        val method = MeshRepository::class.java.getDeclaredMethod(
            "sendDeliveryReceiptAsync",
            String::class.java,
            String::class.java,
            String::class.java,
            String::class.java,
            String::class.java,
            String::class.java,
            List::class.java
        )
        method.isAccessible = true
        method.invoke(repo, senderKey, messageId, senderId, null, null, null, emptyList<String>())
    }

    @Suppress("UNCHECKED_CAST")
    private fun pendingReceiptJob(repo: MeshRepository, messageId: String): Job? {
        val map = getField<java.util.concurrent.ConcurrentHashMap<String, Job>>(repo, "pendingReceiptSendJobs")
        return map?.get(messageId)
    }

    private fun cancelRepoScope(repo: MeshRepository) {
        getField<CoroutineScope>(repo, "repoScope")?.cancel()
    }

    @After
    fun cleanup() {
        testRoot.listFiles()?.forEach { it.deleteRecursively() }
    }

    @Test
    fun `receive path processes delivered receipts and deduplicates`() = runTest {
        val filesDir = freshFilesDir()
        val repo = MeshRepository(fakeContext(filesDir))

        val ironCore = mockk<IronCore>(relaxed = true) {
            every { getIdentityInfo() } returns IdentityInfo(
                identityId = null,
                publicKeyHex = null,
                deviceId = null,
                seniorityTimestamp = null,
                initialized = false,
                nickname = null,
                libp2pPeerId = null
            )
            coEvery { markMessageSent(any()) } just Awaits
        }
        val meshService = mockk<MeshService>(relaxed = true) {
            every { getState() } returns ServiceState.STOPPED
            every { getCore() } returns ironCore
        }
        setField(repo, "meshService", meshService)

        repo.startMeshService(MeshServiceConfig(discoveryIntervalMs = 30000u, batteryFloorPct = 20u))

        val coreDelegate = getField<CoreDelegate>(repo, "coreDelegate")
        assertNotNull(coreDelegate)

        writePendingOutbox(filesDir, "msg-1", "12D3KooWTestPeerIdForReceiptUnification01")

        var delivered = false
        val sentRecord = MessageRecord(
            id = "msg-1",
            direction = MessageDirection.SENT,
            peerId = "peer-1",
            content = "hello",
            timestamp = 1uL,
            senderTimestamp = 1uL,
            delivered = false,
            status = MessageStatus.SENT,
            hidden = false
        )
        // Constant SENT record: the duplicate guard is exercised via the
        // repository's own deliveredReceiptCache, not via record state.
        // NOTE: calls are receiver-qualified -- an unqualified `get(...)`
        // inside every{} binds to MockKMatcherScope.get, not HistoryManager.
        val historyManager = mockk<HistoryManager>(relaxed = true)
        every { historyManager.get("msg-1") } returns sentRecord
        every { historyManager.markDelivered("msg-1") } returns Unit
        val contactManager = mockk<ContactManager>(relaxed = true)
        setField(repo, "historyManager", historyManager)
        setField(repo, "contactManager", contactManager)

        coreDelegate!!.onReceiptReceived("msg-1", "Delivered")

        coVerify(exactly = 1) { historyManager.markDelivered("msg-1") }
        coVerify(exactly = 1) { historyManager.flush() }
        coVerify(exactly = 1) { ironCore.markMessageSent("msg-1") }

        coreDelegate.onReceiptReceived("msg-1", "Delivered")

        coVerify(exactly = 1) { historyManager.markDelivered("msg-1") }
        coVerify(exactly = 1) { historyManager.flush() }
        coVerify(exactly = 1) { ironCore.markMessageSent("msg-1") }

        coreDelegate.onReceiptReceived("msg-garbage", "garbage")

        coVerify(exactly = 0) { historyManager.markDelivered("msg-garbage") }

        cancelRepoScope(repo)
    }

    @Test
    fun `send path passes core receipt bytes unmodified to transport`() = runTest {
        val filesDir = freshFilesDir()
        val repo = MeshRepository(fakeContext(filesDir))

        val peerId = "12D3KooWTestPeeridForReceiptUnification24XyzAbcVwxyz"
        val receiptBytes = byteArrayOf(0x01, 0x02, 0x03, 0x04, 0x05)

        val ironCore = mockk<IronCore>(relaxed = true) {
            every { prepareReceipt(any(), any()) } returns receiptBytes
            every { isPeerBlocked(any(), any()) } returns false
        }
        val meshService = mockk<MeshService>(relaxed = true) {
            every { getState() } returns ServiceState.RUNNING
            every { getCore() } returns ironCore
        }
        setField(repo, "meshService", meshService)
        setField(repo, "ironCore", ironCore)
        setField(repo, "contactManager", mockk<ContactManager>(relaxed = true))

        // Use the real router so the only mock seam is SwarmBridge.sendMessageStatus.
        setField(repo, "smartTransportRouter", SmartTransportRouter())

        val envelopeSlot = slot<ByteArray>()
        val swarmBridge = mockk<SwarmBridge>(relaxed = true) {
            coEvery { sendMessageStatus(any(), capture(envelopeSlot), any(), any()) } returns null
            coEvery { getPeers() } returns listOf(peerId)
        }
        setField(repo, "swarmBridge", swarmBridge)

        callSendDeliveryReceiptAsync(repo, senderKey = "", messageId = "msg-1", senderId = peerId)

        val job = pendingReceiptJob(repo, "msg-1")
        assertNotNull(job)
        job!!.join()

        verify { ironCore.prepareReceipt("", "msg-1") }
        assertArrayEquals(receiptBytes, envelopeSlot.captured)

        cancelRepoScope(repo)
    }
}
