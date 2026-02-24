package com.scmessenger.android.service

import com.scmessenger.android.service.MeshForegroundService
import org.junit.Assert.assertEquals
import org.junit.Test

class MeshForegroundServiceTest {

    @Test
    fun `null action resolves to Start`() {
        val result = MeshForegroundService.decideCommand(
            action = null,
            serviceRunning = false,
            repositoryRunning = false
        )
        assertEquals(MeshForegroundService.Companion.StartDecision.Start, result)
    }

    @Test
    fun `pause resolves to NoOp when service not running`() {
        val result = MeshForegroundService.decideCommand(
            action = MeshForegroundService.ACTION_PAUSE,
            serviceRunning = false,
            repositoryRunning = false
        )
        assertEquals(MeshForegroundService.Companion.StartDecision.NoOp, result)
    }

    @Test
    fun `pause resolves to Pause when repository running`() {
        val result = MeshForegroundService.decideCommand(
            action = MeshForegroundService.ACTION_PAUSE,
            serviceRunning = false,
            repositoryRunning = true
        )
        assertEquals(MeshForegroundService.Companion.StartDecision.Pause, result)
    }

    @Test
    fun `resume resolves to Resume only when both running`() {
        val result = MeshForegroundService.decideCommand(
            action = MeshForegroundService.ACTION_RESUME,
            serviceRunning = true,
            repositoryRunning = true
        )
        assertEquals(MeshForegroundService.Companion.StartDecision.Resume, result)
    }

    @Test
    fun `resume resolves to Start when state is incomplete`() {
        val result = MeshForegroundService.decideCommand(
            action = MeshForegroundService.ACTION_RESUME,
            serviceRunning = true,
            repositoryRunning = false
        )
        assertEquals(MeshForegroundService.Companion.StartDecision.Start, result)
    }

    @Test
    fun `stop resolves to Stop`() {
        val result = MeshForegroundService.decideCommand(
            action = MeshForegroundService.ACTION_STOP,
            serviceRunning = true,
            repositoryRunning = true
        )
        assertEquals(MeshForegroundService.Companion.StartDecision.Stop, result)
    }

    @Test
    fun `unknown action defaults to Start`() {
        val result = MeshForegroundService.decideCommand(
            action = "unknown",
            serviceRunning = true,
            repositoryRunning = true
        )
        assertEquals(MeshForegroundService.Companion.StartDecision.Start, result)
    }

    @Test
    fun `explicit start resolves to Start`() {
        val result = MeshForegroundService.decideCommand(
            action = MeshForegroundService.ACTION_START,
            serviceRunning = false,
            repositoryRunning = false
        )
        assertEquals(MeshForegroundService.Companion.StartDecision.Start, result)
    }
}
