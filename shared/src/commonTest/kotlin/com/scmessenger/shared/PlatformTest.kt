package com.scmessenger.shared

import kotlin.test.Test
import kotlin.test.assertTrue

class PlatformTest {

    @Test
    fun testPlatformName() {
        assertTrue(platformName().isNotEmpty())
    }
}
