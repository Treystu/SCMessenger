package com.scmessenger.android.test

import org.junit.Test
import kotlin.test.assertTrue

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
        val ironCore = uniffi.api.IronCore(storagePath)
        ironCore.initializeIdentity()
        
        val info = ironCore.getIdentityInfo()
        assertTrue(info.initialized)
        assertTrue(info.identityId.isNotEmpty())
    }
    
    @Test
    fun `test message encryption and decryption roundtrip`() {
        val aliceCore = uniffi.api.IronCore("$storagePath/alice")
        val bobCore = uniffi.api.IronCore("$storagePath/bob")
        
        aliceCore.initializeIdentity()
        bobCore.initializeIdentity()
        
        val bobInfo = bobCore.getIdentityInfo()
        val plaintext = "Hello from Alice!"
        
        val encrypted = aliceCore.prepareMessage(bobInfo.publicKeyHex, plaintext)
        val decrypted = bobCore.decryptMessage(encrypted)
        
        kotlin.test.assertEquals(plaintext, String(decrypted))
    }
    
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
        
        kotlin.test.assertNotNull(retrieved)
        kotlin.test.assertEquals(nickname, retrieved?.nickname)
        
        manager.remove(peerId)
        kotlin.test.assertFalse(manager.list().any { it.peerId == peerId })
    }
}
