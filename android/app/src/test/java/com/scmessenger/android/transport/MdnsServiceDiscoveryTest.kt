package com.scmessenger.android.transport

import android.content.Context
import android.net.nsd.NsdManager
import android.net.nsd.NsdServiceInfo
import android.os.Build
import android.os.Looper
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkStatic
import io.mockk.verify
import org.junit.After
import org.junit.Before
import org.junit.Test
import java.net.InetAddress

class MdnsServiceDiscoveryTest {

    @Before
    fun setUp() {
        mockkStatic(Looper::class)
        every { Looper.getMainLooper() } returns mockk(relaxed = true)
    }

    @After
    fun tearDown() {
        unmockkStatic(Looper::class)
    }

    @Test
    fun `onServiceResolved with local peer-id is filtered as self-loopback`() {
        val context = mockk<Context>(relaxed = true)
        val onPeerDiscovered = mockk<(String) -> Unit>(relaxed = true)
        val onDataReceived = mockk<(String, ByteArray) -> Unit>(relaxed = true)
        val onPeerDisconnected: ((String) -> Unit)? = null
        val onLanPeerResolved = mockk<(String, String, Int) -> Unit>(relaxed = true)
        val getLocalPeerId = mockk<(() -> String?)>(relaxed = true)

        every { getLocalPeerId.invoke() } returns "SELF_PEER_ID"

        val discovery = MdnsServiceDiscovery(
            context,
            onPeerDiscovered,
            onDataReceived,
            onPeerDisconnected,
            onLanPeerResolved,
            getLocalPeerId
        )

        val newResolveListenerMethod = MdnsServiceDiscovery::class.java.getDeclaredMethod(
            "newResolveListener",
            String::class.java
        )
        newResolveListenerMethod.isAccessible = true
        val resolveListener = newResolveListenerMethod.invoke(discovery, "_scmessenger._tcp") as NsdManager.ResolveListener

        val serviceInfo = mockk<NsdServiceInfo>(relaxed = true)
        every { serviceInfo.attributes } returns mapOf("peer-id" to "SELF_PEER_ID".toByteArray())
        every { serviceInfo.serviceName } returns "testService"
        every { serviceInfo.port } returns 9001

        val inetAddress = mockk<InetAddress>(relaxed = true)
        every { inetAddress.hostAddress } returns "192.168.0.148"
        every { serviceInfo.host } returns inetAddress

        resolveListener.onServiceResolved(serviceInfo)

        verify(exactly = 0) { onLanPeerResolved(any(), any(), any()) }
    }

    @Test
    fun `onServiceResolved with a different peer-id still resolves normally (no regression)`() {
        val context = mockk<Context>(relaxed = true)
        val onPeerDiscovered = mockk<(String) -> Unit>(relaxed = true)
        val onDataReceived = mockk<(String, ByteArray) -> Unit>(relaxed = true)
        val onPeerDisconnected: ((String) -> Unit)? = null
        val onLanPeerResolved = mockk<(String, String, Int) -> Unit>(relaxed = true)
        val getLocalPeerId = mockk<(() -> String?)>(relaxed = true)

        every { getLocalPeerId.invoke() } returns "SELF_PEER_ID"

        val discovery = MdnsServiceDiscovery(
            context,
            onPeerDiscovered,
            onDataReceived,
            onPeerDisconnected,
            onLanPeerResolved,
            getLocalPeerId
        )

        val newResolveListenerMethod = MdnsServiceDiscovery::class.java.getDeclaredMethod(
            "newResolveListener",
            String::class.java
        )
        newResolveListenerMethod.isAccessible = true
        val resolveListener = newResolveListenerMethod.invoke(discovery, "_scmessenger._tcp") as NsdManager.ResolveListener

        val serviceInfo = mockk<NsdServiceInfo>(relaxed = true)
        every { serviceInfo.attributes } returns mapOf("peer-id" to "REMOTE_PEER_ID".toByteArray())
        every { serviceInfo.serviceName } returns "testService"
        every { serviceInfo.port } returns 9001

        val inetAddress = mockk<InetAddress>(relaxed = true)
        every { inetAddress.hostAddress } returns "192.168.0.148"
        every { serviceInfo.host } returns inetAddress

        resolveListener.onServiceResolved(serviceInfo)

        verify(exactly = 1) { onLanPeerResolved("REMOTE_PEER_ID", "192.168.0.148", 9001) }
    }
}
