package com.scmessenger.android.ui

import androidx.compose.foundation.layout.*
import androidx.compose.material.icons.filled.Chat
import androidx.compose.material.icons.filled.People
import androidx.compose.material.icons.filled.Router
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
            ConversationsScreen(
                onNavigateToChat = { peerId ->
                    navController.navigate("chat/$peerId")
                }
            )
        }
        
        composable(Screen.Contacts.route) {
            ContactsScreen()
        }
        
        composable(Screen.Dashboard.route) {
            DashboardScreen()
        }

        composable(Screen.Settings.route) {
            SettingsScreen()
        }
        
        composable(
            route = "chat/{peerId}",
            arguments = listOf(androidx.navigation.navArgument("peerId") { type = androidx.navigation.NavType.StringType })
        ) { backStackEntry ->
            val peerId = backStackEntry.arguments?.getString("peerId") ?: return@composable
            ChatScreen(
                conversationId = peerId,
                onNavigateBack = { navController.popBackStack() }
            )
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
    
    // Hide bottom bar on Chat screen
    if (currentRoute?.startsWith("chat/") == true) return

    NavigationBar {
        Screen.bottomNavItems.forEach { screen ->
            NavigationBarItem(
                icon = { Icon(screen.icon, contentDescription = screen.label) },
                label = { Text(screen.label) },
                selected = currentRoute == screen.route,
                onClick = {
                    navController.navigate(screen.route) {
                        popUpTo(navController.graph.startDestinationId) {
                            saveState = true
                        }
                        launchSingleTop = true
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
    object Dashboard: Screen("dashboard", "Network", androidx.compose.material.icons.Icons.Filled.Router)
    object Settings : Screen("settings", "Settings", androidx.compose.material.icons.Icons.Default.Settings)
    
    companion object {
        val bottomNavItems = listOf(Conversations, Contacts, Dashboard, Settings)
    }
}
