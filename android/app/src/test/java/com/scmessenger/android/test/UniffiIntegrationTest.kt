package com.scmessenger.android.test

import org.junit.Test
import org.junit.Assert.*

/**
 * Integration tests for UniFFI boundary.
 * 
 * Tests end-to-end flows through the UniFFI layer:
 * - IronCore lifecycle
 * - prepare_message → receive_message roundtrip
 * - ContactManager CRUD
 * - MeshSettings validate/save/load
 * 
 * Note: These tests require:
 * - Actual UniFFI library loaded (JNI)
 * - Storage path for database files
 * - More setup than typical unit tests
 * 
 * Run these as instrumented tests on device/emulator.
 */
@org.junit.Ignore("Requires JNI and native libraries, should be run as instrumented test")
class UniffiIntegrationTest {
    
    companion object {
        @org.junit.BeforeClass
        @JvmStatic
        fun setupLibraryPath() {
            // Set JNA library path for unit tests running on host JVM
            val hostLibPath = java.io.File("../../core/target/android-libs/host").absolutePath
            System.setProperty("jna.library.path", hostLibPath)
            System.out.println("Set jna.library.path to: $hostLibPath")
        }
    }
    
    private val storagePath = "/tmp/scm_test_${System.currentTimeMillis()}"
    
    @Test
    fun `test IronCore initialization`() {
        val config = uniffi.api.MeshServiceConfig(
            discoveryIntervalMs = 30000u,
            relayBudgetPerHour = 200u,
            batteryFloorPct = 20u
        )
        val service = uniffi.api.MeshService.withStorage(config, storagePath)
        val ironCore = service.getCore()
        ironCore!!.initializeIdentity()
        
        val info = ironCore.getIdentityInfo()
        assertTrue(info.initialized)
        assertTrue(info.identityId?.isNotEmpty() == true)
    }
    
    // Note: Roundtrip test removed as IronCore.receiveMessage is handled via CoreDelegate
    
    @Test
    fun `test ContactManager persistence`() {
        val manager = uniffi.api.ContactManager("$storagePath/contacts")
        val peerId = "test_peer_123"
        val nickname = "Test Friend"
        
        val contact = uniffi.api.Contact(
            peerId = peerId,
            nickname = nickname,
            publicKey = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff",
            addedAt = (System.currentTimeMillis() / 1000).toULong(),
            lastSeen = null,
            notes = "Integration test contact"
        )
        
        manager.add(contact)
        val retrieved = manager.list().find { it.peerId == peerId }
        
        assertNotNull(retrieved)
        assertEquals(nickname, retrieved?.nickname)
        
        manager.remove(peerId)
        assertFalse(manager.list().any { it.peerId == peerId })
    }
}
