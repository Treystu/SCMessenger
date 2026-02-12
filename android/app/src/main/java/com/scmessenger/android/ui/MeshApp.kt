package com.scmessenger.android.ui

import androidx.compose.foundation.layout.*
import androidx.compose.material.icons.filled.Chat
import androidx.compose.material.icons.filled.People
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.navigation.NavHostController
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import com.scmessenger.android.ui.screens.*
import com.scmessenger.android.ui.viewmodels.MainViewModel

/**
 * Root composable for the SCMessenger app.
 * 
 * Sets up the navigation graph and bottom navigation bar.
 */
@Composable
fun MeshApp() {
    val mainViewModel: MainViewModel = hiltViewModel()
    val isReady by mainViewModel.isReady.collectAsState()

    if (!isReady) {
        Box(
            modifier = Modifier.fillMaxSize(),
            contentAlignment = Alignment.Center
        ) {
            Column(horizontalAlignment = Alignment.CenterHorizontally) {
                CircularProgressIndicator()
                Spacer(modifier = Modifier.height(16.dp))
                Text("Initializing Identity...")
            }
        }
    } else {
        val navController = rememberNavController()
        
        Scaffold(
            bottomBar = { MeshBottomBar(navController = navController) }
        ) { paddingValues ->
            MeshNavHost(
                navController = navController,
                modifier = Modifier.padding(paddingValues)
            )
        }
    }
}

/**
 * Navigation host for the app.
 */
@Composable
fun MeshNavHost(
    navController: NavHostController,
    modifier: Modifier = Modifier
) {
    NavHost(
        navController = navController,
        startDestination = Screen.Conversations.route,
        modifier = modifier
    ) {
        composable(Screen.Conversations.route) {
            ConversationsScreen()
        }
        
        composable(Screen.Contacts.route) {
            ContactsScreen()
        }
        
        composable(Screen.Settings.route) {
            SettingsScreen()
        }
    }
}

/**
 * Bottom navigation bar.
 */
@Composable
fun MeshBottomBar(navController: NavHostController) {
    val navBackStackEntry by navController.currentBackStackEntryAsState()
    val currentRoute = navBackStackEntry?.destination?.route
    
    NavigationBar {
        Screen.bottomNavItems.forEach { screen ->
            NavigationBarItem(
                icon = { Icon(screen.icon, contentDescription = screen.label) },
                label = { Text(screen.label) },
                selected = currentRoute == screen.route,
                onClick = {
                    navController.navigate(screen.route) {
                        // Pop up to the start destination of the graph to
                        // avoid building up a large stack of destinations
                        popUpTo(navController.graph.startDestinationId) {
                            saveState = true
                        }
                        // Avoid multiple copies of the same destination
                        launchSingleTop = true
                        // Restore state when reselecting a previously selected item
                        restoreState = true
                    }
                }
            )
        }
    }
}

/**
 * Screen definitions for navigation.
 */
sealed class Screen(val route: String, val label: String, val icon: androidx.compose.ui.graphics.vector.ImageVector) {
    object Conversations : Screen("conversations", "Chats", androidx.compose.material.icons.Icons.Default.Chat)
    object Contacts : Screen("contacts", "Contacts", androidx.compose.material.icons.Icons.Default.People)
    object Settings : Screen("settings", "Settings", androidx.compose.material.icons.Icons.Default.Settings)
    
    companion object {
        val bottomNavItems = listOf(Conversations, Contacts, Settings)
    }
}
