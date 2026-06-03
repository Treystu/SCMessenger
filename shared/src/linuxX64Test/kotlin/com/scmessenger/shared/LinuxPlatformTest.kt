package com.scmessenger.shared

import kotlin.test.Test
import kotlin.test.assertEquals

class LinuxPlatformTest {

    @Test
    fun testLinuxPlatform() {
        assertEquals("Linux", platformName())
    }
}
