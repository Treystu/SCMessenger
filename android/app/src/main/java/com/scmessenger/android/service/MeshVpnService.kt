package com.scmessenger.android.service

import android.content.Intent
import android.net.VpnService
import android.os.Build
import android.os.ParcelFileDescriptor
import timber.log.Timber
import java.io.FileInputStream
import java.io.FileOutputStream
import java.nio.ByteBuffer

/**
 * Optional VPN service for maximum background persistence.
 * 
 * Creates a dummy VPN tunnel to ensure the app stays alive even under
 * aggressive battery optimization. This is opt-in and togglable from Settings.
 * 
 * The VPN tunnel doesn't route any actual traffic - it's purely a mechanism
 * to maintain the mesh service in the background on devices with aggressive
 * doze/standby modes.
 */
class MeshVpnService : VpnService() {
    
    private var vpnInterface: ParcelFileDescriptor? = null
    private var isRunning = false
    
    override fun onCreate() {
        super.onCreate()
        Timber.d("MeshVpnService created")
    }
    
    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        when (intent?.action) {
            ACTION_START -> startVpn()
            ACTION_STOP -> stopVpn()
        }
        return START_STICKY
    }
    
    private fun startVpn() {
        if (isRunning) {
            Timber.w("VPN already running")
            return
        }
        
        try {
            // Create a dummy VPN interface for persistence (no traffic routing)
            // Do NOT add routes - we only want the VPN to keep the service alive,
            // not to intercept network traffic which would break all connectivity
            val builder = Builder()
                .setSession("SCMessenger VPN")
                .addAddress("10.255.255.1", 32)
            
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                builder.setBlocking(false)
            }
            
            vpnInterface = builder.establish()
            if (vpnInterface == null) {
                Timber.e("Failed to establish VPN interface")
                isRunning = false
                return
            }
            isRunning = true
            
            Timber.i("VPN service started (dummy tunnel for persistence, no routing)")
            
            // Start a thread to keep the interface alive
            // In a real VPN, this would forward packets
            Thread {
                try {
                    val iface = vpnInterface
                    if (iface == null) {
                        Timber.e("VPN interface is null, cannot start packet loop")
                        return@Thread
                    }
                    
                    val input = FileInputStream(iface.fileDescriptor)
                    val output = FileOutputStream(iface.fileDescriptor)
                    val packet = ByteBuffer.allocate(32767)
                    
                    while (isRunning) {
                        // Read any incoming packets (there won't be any)
                        val length = input.channel.read(packet)
                        if (length > 0) {
                            // Ignore - we don't process anything
                            packet.clear()
                        }
                        Thread.sleep(100)
                    }
                } catch (e: Exception) {
                    Timber.e(e, "VPN thread error")
                }
            }.start()
            
        } catch (e: Exception) {
            Timber.e(e, "Failed to start VPN")
            stopSelf()
        }
    }
    
    private fun stopVpn() {
        if (!isRunning) {
            return
        }
        
        try {
            isRunning = false
            vpnInterface?.close()
            vpnInterface = null
            
            Timber.i("VPN service stopped")
        } catch (e: Exception) {
            Timber.e(e, "Failed to stop VPN")
        } finally {
            stopSelf()
        }
    }
    
    override fun onDestroy() {
        super.onDestroy()
        stopVpn()
        Timber.d("MeshVpnService destroyed")
    }
    
    override fun onRevoke() {
        super.onRevoke()
        Timber.w("VPN permission revoked")
        stopVpn()
    }
    
    companion object {
        const val ACTION_START = "com.scmessenger.android.service.VPN_START"
        const val ACTION_STOP = "com.scmessenger.android.service.VPN_STOP"
    }
}
