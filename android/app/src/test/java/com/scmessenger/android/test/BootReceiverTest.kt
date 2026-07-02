package com.scmessenger.android.service

import android.content.Intent
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class BootReceiverTest {

    @Test
    fun `boot completed action is recognized as a boot action`() {
        assertTrue(BootReceiver.isBootAction(Intent.ACTION_BOOT_COMPLETED))
    }

    @Test
    fun `quickboot poweron action is recognized as a boot action`() {
        assertTrue(BootReceiver.isBootAction(BootReceiver.ACTION_QUICKBOOT_POWERON))
    }

    @Test
    fun `unrelated or null action is not a boot action`() {
        assertFalse(BootReceiver.isBootAction(null))
        assertFalse(BootReceiver.isBootAction("android.intent.action.SCREEN_ON"))
    }

    @Test
    fun `auto-starts on boot when the preference is enabled`() {
        assertTrue(
            BootReceiver.shouldAutoStart(
                action = Intent.ACTION_BOOT_COMPLETED,
                autoStartEnabled = true
            )
        )
        assertTrue(
            BootReceiver.shouldAutoStart(
                action = BootReceiver.ACTION_QUICKBOOT_POWERON,
                autoStartEnabled = true
            )
        )
    }

    @Test
    fun `does not auto-start on boot when the preference is disabled`() {
        assertFalse(
            BootReceiver.shouldAutoStart(
                action = Intent.ACTION_BOOT_COMPLETED,
                autoStartEnabled = false
            )
        )
    }

    @Test
    fun `never auto-starts for a non-boot action even if the preference is enabled`() {
        assertFalse(
            BootReceiver.shouldAutoStart(
                action = "android.intent.action.SCREEN_ON",
                autoStartEnabled = true
            )
        )
        assertFalse(
            BootReceiver.shouldAutoStart(action = null, autoStartEnabled = true)
        )
    }
}
