package com.scmessenger.android.test

import com.scmessenger.android.ui.roleBasedBottomNavItems
import com.scmessenger.android.ui.startDestinationForRole
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class RoleNavigationPolicyTest {

    @Test
    fun `relay only role hides conversations and contacts`() {
        val routes = roleBasedBottomNavItems(hasIdentity = false).map { it.route }

        assertEquals(listOf("dashboard", "settings"), routes)
        assertFalse(routes.contains("conversations"))
        assertFalse(routes.contains("contacts"))
    }

    @Test
    fun `full role exposes conversations and contacts`() {
        val routes = roleBasedBottomNavItems(hasIdentity = true).map { it.route }

        assertTrue(routes.contains("conversations"))
        assertTrue(routes.contains("contacts"))
        assertEquals(listOf("conversations", "contacts", "dashboard", "settings"), routes)
    }

    @Test
    fun `start destination follows role`() {
        assertEquals("dashboard", startDestinationForRole(hasIdentity = false))
        assertEquals("conversations", startDestinationForRole(hasIdentity = true))
    }
}
